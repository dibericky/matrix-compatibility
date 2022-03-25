mod gitlab_api;
mod gitlab_ci;

use std::{collections::{HashMap}};
use markdown_table::MarkdownTable;
use std::fs::File;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Deserialize)]
struct MatrixItem {
    path: String,
    name: String
}

#[derive(PartialEq, Deserialize)]
struct GitlabCi {
    project_id: String
}

#[derive(PartialEq, Deserialize)]
struct Service {
    name: String,
    ci: GitlabCi,
    matrix: Vec<MatrixItem>,
}

#[derive(PartialEq, Deserialize)]
struct Config {
    gitlab_base_api_host: String,
    services: Vec<Service>
}

#[derive(PartialEq, Serialize)]
struct CompatibilityItem {
    name: String,
    compatible: Vec<String>
}

impl Service{
    async fn get_ci(&self, gitlab_base_host: &str) -> String {
        gitlab_api::get_file_raw(&gitlab_base_host, &self.ci.project_id, ".gitlab-ci.yml").await
    }
}

#[derive(PartialEq, Serialize)]
struct CompatibilityRow<'a> {
    service_name: &'a String,
    compatibility_subject: String,
    version: String
}

async fn push_service_compatibility_rows<'a> (compatibility_vec : &mut Vec<CompatibilityRow<'a>>, service: &'a Service, gitlab_base_host: &str) {
    let pipeline = service.get_ci(gitlab_base_host).await;
    for item in &service.matrix {
        let result = gitlab_ci::get_matrix(&pipeline, &item.path);
        for version in result {
            let row: CompatibilityRow<'a> = CompatibilityRow{
                service_name: &service.name,
                compatibility_subject: String::from(&item.name),
                version,
            };
            compatibility_vec.push(row);
        };
    };
}

fn get_table_rows_and_column_by_subject<'a, 'b > (subject: &str, rows: &'a Vec<CompatibilityRow>) -> (Vec<String>, Vec<String>) {
    let mut versions : Vec<String> = Vec::new();
    let mut services_names : Vec<String> = Vec::new();

    rows.iter()
        .filter(|row| row.compatibility_subject == subject)
        .for_each(|row| {
            if !versions.contains(&row.version) {
                versions.push(String::from(&row.version));
            }
            if !services_names.contains(&row.service_name) {
                services_names.push(String::from(row.service_name));
            }
        });
    (services_names, versions)
}

fn get_table_by_subject (subject: &str, rows: &Vec<CompatibilityRow>) -> Vec<Vec<String>> {
    let (table_rows, table_cols) = get_table_rows_and_column_by_subject(subject, rows);
    let rows_of_subject = rows.iter()
        .filter(|row| row.compatibility_subject == subject);

    let mut table : Vec<Vec<String>> = Vec::new();
    let mut first_row : Vec<String> = Vec::new();
    first_row.push(String::from(subject));
    for v in table_cols.iter() {
        first_row.push(String::from(v));
    }
    table.push(first_row);
    let results = table_rows.iter().map(|service_name| {
        let all_service_version : Vec<String> = rows_of_subject
            .clone()
            .filter(|item| item.service_name == service_name)
            .map(|item| &item.version)
            .cloned()
            .collect();
        let list_of_versions = table_cols
            .iter()
            .map(|version| all_service_version.contains(&version).to_string())
            .collect::<Vec<String>>();
        let mut versions_for_service : Vec<String> = Vec::new();
        
        versions_for_service.push(String::from(service_name));
        for v in list_of_versions {
            versions_for_service.push(String::from(v))
        }
        versions_for_service
    })
    .collect::<Vec<Vec<String>>>();
    for r in results {
        table.push(r);
    }
    table
}
type ServiceSupportedVersion = HashMap<String, bool>;

fn get_services_and_versions_by_subject(rows: &Vec<CompatibilityRow>) {
    
}

/**
 {
     "mongo": {
        "crud-service": {"4.0": true, "4.4": true, "5.0", true}
    }
 }
 */

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = std::fs::File::open("config.yml").unwrap();
    let config : Config = serde_yaml::from_reader(f).unwrap();

    let mut compatibility_vec : Vec<CompatibilityRow> = Vec::new();
    for service in &config.services {
        push_service_compatibility_rows(&mut compatibility_vec, service, &config.gitlab_base_api_host).await;        
    }

    let version_columns = get_table_by_subject("mongo", &compatibility_vec);

    let s = serde_yaml::to_string(&compatibility_vec)?;

    let table = MarkdownTable::new(
        version_columns
    );
    println!("{}", table.as_markdown().unwrap());

    let path = "output.md";
    let mut output = File::create(path)?;
    let line = table.as_markdown().unwrap();
    write!(output, "{}", line);

    Ok(())
}

