mod gitlab_api;
mod gitlab_ci;

use std::collections::{HashMap, HashSet};

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

fn get_services_supported_version_by_subject (subject: &str, rows: &Vec<CompatibilityRow>) -> Vec<String> {
    rows.iter()
        .filter(|row| row.compatibility_subject == subject)
        .map(|row| String::from(&row.version))
        .collect()
}

// fn get_services_and_versions_by_subject(rows: &Vec<CompatibilityRow>) {
//     let mut map : HashMap<&String, (HashSet<&String>, HashSet<&String>)> =  HashMap::new();
//     for row in rows.iter() {
//         if !map.contains_key(&row.compatibility_subject){
//             map.insert(&row.compatibility_subject, (HashSet::new(), HashSet::new()));
//         }
//         let a : (String, String) = (String::from("a"), String::from("b"));
//         let b = String::from("a");
//         map.get(&row.compatibility_subject).unwrap().0.insert(&b);
//     }
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = std::fs::File::open("config.yml").unwrap();
    let config : Config = serde_yaml::from_reader(f).unwrap();

    let mut compatibility_vec : Vec<CompatibilityRow> = Vec::new();
    for service in &config.services {
        push_service_compatibility_rows(&mut compatibility_vec, service, &config.gitlab_base_api_host).await;        
    }

    

    let s = serde_yaml::to_string(&compatibility_vec)?;

    println!("{}", s);

    Ok(())
}

