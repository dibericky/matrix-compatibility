mod gitlab_api;
mod gitlab_ci;
mod table_builder;
mod env_manager;

use table_builder::{CompatibilityRow};
use std::fs::File;
use std::io::prelude::*;

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

async fn config_to_compatibility_vec<'a > (config: &'a Config) -> Vec<CompatibilityRow<'a>> {
    let mut compatibility_vec : Vec<CompatibilityRow> = Vec::new();
    for service in &config.services {
        push_service_compatibility_rows(&mut compatibility_vec, service, &config.gitlab_base_api_host).await;        
    }
    compatibility_vec
}

fn get_list_of_subjects (config: &Config) -> Vec<&String> {
    let mut subjects : Vec<&String> = vec![];
    config.services.iter().for_each(|service| {
        service.matrix.iter().for_each(|matrix|{
            if !subjects.contains(&&matrix.name) {
                subjects.push(&matrix.name);
            }
        })
    });
    subjects
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_file_path = env_manager::get_config_file_path();

    let f = File::open(config_file_path).unwrap();
    let config : Config = serde_yaml::from_reader(f).unwrap();

    let compatibility_vec : Vec<CompatibilityRow> = config_to_compatibility_vec(&config).await;

    let list_subjects = get_list_of_subjects(&config);
    for subject in list_subjects {
        let version_columns = table_builder::get_table_by_subject(subject, &compatibility_vec);

        let md_table = table_builder::as_markdown(version_columns);
        println!("{}", subject);
        println!("{}", md_table);
    
        let mut output_file_name = String::from(subject);
        output_file_name.push_str("_output.md");
        let mut output = File::create(output_file_name)?;
        write!(output, "{}", md_table).unwrap();
    }
   
    Ok(())
}
