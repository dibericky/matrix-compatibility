mod gitlab_api;
mod gitlab_ci;
mod table_builder;

use table_builder::{CompatibilityRow};
use markdown_table::MarkdownTable;
use std::fs::File;
use std::io::prelude::*;
use std::env;

use serde::{Deserialize};

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

impl Service{
    async fn get_ci(&self, gitlab_base_host: &str) -> String {
        gitlab_api::get_file_raw(&gitlab_base_host, &self.ci.project_id, ".gitlab-ci.yml").await
    }
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


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_file_path = match env::var("CONFIG_FILE_PATH") {
        Ok(v) => v,
        Err(_) => String::from("config.yml")
    };

    let f = std::fs::File::open(config_file_path).unwrap();
    let config : Config = serde_yaml::from_reader(f).unwrap();

    let mut compatibility_vec : Vec<CompatibilityRow> = Vec::new();
    for service in &config.services {
        push_service_compatibility_rows(&mut compatibility_vec, service, &config.gitlab_base_api_host).await;        
    }

    // TODO: for-each subject
    let version_columns = table_builder::get_table_by_subject("mongo", &compatibility_vec);

    let table = MarkdownTable::new(
        version_columns
    );
    println!("{}", table.as_markdown().unwrap());

    let path = "output.md";
    let mut output = File::create(path)?;
    let line = table.as_markdown().unwrap();
    write!(output, "{}", line).unwrap();

    Ok(())
}
