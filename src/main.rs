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

// async fn config_to_compatibility_vec (config: &Config) -> Vec<CompatibilityRow> {
//     let mut compatibility_vec : Vec<CompatibilityRow> = Vec::new();
//     for service in &config.services {
//         push_service_compatibility_rows(&mut compatibility_vec, service, &config.gitlab_base_api_host).await;        
//     }
//     compatibility_vec
// }


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_file_path = env_manager::get_config_file_path();

    let f = File::open(config_file_path).unwrap();
    let config : Config = serde_yaml::from_reader(f).unwrap();

    let mut compatibility_vec : Vec<CompatibilityRow> = Vec::new();
    for service in &config.services {
        push_service_compatibility_rows(&mut compatibility_vec, service, &config.gitlab_base_api_host).await;        
    }

    // TODO: for-each subject
    let version_columns = table_builder::get_table_by_subject("mongo", &compatibility_vec);

    let md_table = table_builder::as_markdown(version_columns);
    println!("{}", md_table);

    let mut output = File::create("output.md")?;
    write!(output, "{}", md_table).unwrap();

    Ok(())
}
