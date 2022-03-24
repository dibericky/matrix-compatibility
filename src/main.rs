use regex::Regex;

use std::str::Split;
use std::env;

use serde_yaml::Value;
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

#[derive(PartialEq, Serialize)]
struct ServiceCompatibility {
    name: String,
    compatibility: Vec<CompatibilityItem>
}


fn get_matrix (pipeline: &str, path_matrix: String) -> Vec<String> {
    let deserialized_map: Value = serde_yaml::from_str(&pipeline).unwrap();
    let path_iterator : Split<char> = path_matrix.split('.');

    let mut current_value: &Value = &deserialized_map;
    for step in path_iterator {
        let rg_w_named = Regex::new(r"(?P<field>[a-zA-Z_]+)\[(?P<index>\d+)\]$").unwrap();
        let matched = rg_w_named.captures(step);
        if matched.is_none() {
            current_value = current_value.get(step).unwrap();
            continue
        }
        let matched = matched.unwrap();
        let index = matched.name("index").unwrap().as_str();
        let index :usize = index.parse().unwrap();
        let field_name = matched.name("field").unwrap().as_str();
        let array = current_value.get(field_name).unwrap();
        let sequence = array.as_sequence().unwrap();
        current_value = sequence.get(index).unwrap();
    }
    
    let mut vec_string : Vec<String> = Vec::new();

    for item in current_value.as_sequence().unwrap().to_vec() {
        vec_string.push(String::from(item.as_str().unwrap()));
    }
   
    vec_string
}

impl Service{
    async fn get_ci(&self, gitlab_base_host: &str) -> String {
        let gitlab_token  = env::var("GITLAB_TOKEN").expect("$GITLAB_TOKEN is not set");

        let mut api_url = gitlab_base_host.to_owned();
        api_url.push_str("/api/v4/projects/:projectId/repository/files/:filePath/raw");
        api_url = api_url
            .replacen(":projectId", &self.ci.project_id, 1)
            .replacen(":filePath", ".gitlab-ci.yml", 1);
        get_pipeline_ci(&api_url, &gitlab_token).await
    }
}

async fn get_service_matrix (service: Service, gitlab_base_host: &str) -> ServiceCompatibility {
    let pipeline = service.get_ci(gitlab_base_host).await;
    let mut compatibility_vec : Vec<CompatibilityItem> = Vec::new();
    for item in service.matrix {
        let result = get_matrix(&pipeline, item.path);
        compatibility_vec.push(CompatibilityItem{
            name: item.name,
            compatible: result
        })
    }
    ServiceCompatibility{
        name: service.name,
        compatibility: compatibility_vec
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = std::fs::File::open("config.yml").unwrap();
    let config : Config = serde_yaml::from_reader(f).unwrap();

    let mut result : Vec<ServiceCompatibility> = Vec::new();

    for service in config.services {
        let service_matrix = get_service_matrix(service, &config.gitlab_base_api_host).await;
        
        result.push(service_matrix)
    }
    let s = serde_yaml::to_string(&result)?;
    println!("{}", s);
    Ok(())
}

async fn get_pipeline_ci (ci_url: &str, token: &str) -> String {
    let client = reqwest::Client::new();
    let response = client
        .get(ci_url)
        .header("Private-Token", token)
        // confirm the request using send()
        .send()
        .await
        // the rest is the same!
        .unwrap()
        .text()
        .await;
    response.unwrap()
}
