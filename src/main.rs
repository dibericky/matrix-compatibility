use regex::Regex;

use std::str::Split;

use futures::executor::block_on;
use serde_yaml::Value;

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

struct MatrixItem {
    path: String,
    name: String
}
struct Service {
    name: String,
    matrix: Vec<MatrixItem>,
}

struct CompatibilityItem {
    name: String,
    compatible: Vec<String>
}
struct ServiceCompatibility {
    name: String,
    compatibility: Vec<CompatibilityItem>
}

async fn get_service_matrix (service: Service) -> ServiceCompatibility {
    let pipeline = get_pipeline_ci().await;

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

fn main() {
    let my_service = Service {
        name: String::from("my-service"),
        matrix: Vec::from([MatrixItem{
            path: String::from("test.parallel.matrix[0].MONGO_VERSION"),
            name: String::from("mongo")
        }]),
    };

    let service_matrix = block_on(get_service_matrix(my_service));
    println!("Service: {}", service_matrix.name);
    for item in service_matrix.compatibility {
       println!("Name: {}, Compatible: {:?}", item.name, item.compatible);
    }
}

async fn get_pipeline_ci () -> String {
    return String::from("    
    install-dependencies:
      before_script:
        - npm version
    test:
      parallel:
        matrix:
          - MONGO_HOST_CI: 'mongo:27017'
            MONGO_VERSION: [\"4.0\", \"4.4\", \"5.0\"]
    
    test-latest:
      variables:
        MONGO_HOST_CI: 'mongo:27017'
    
      services:
        - name: mongo
          alias: mongo
    
    ")
}