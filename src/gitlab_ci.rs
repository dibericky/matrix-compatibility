use regex::Regex;
use std::str::Split;
use serde_yaml::Value;

pub fn get_matrix (pipeline: &str, path_matrix: &str) -> Vec<String> {
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
        let version = item.as_str().unwrap();
        let version = match Regex::new(r"^(\w+:)?(?P<version>[\d\.]+)$")
        .unwrap()
        .captures(version) {
            None => version,
            Some(v) => v.name("version").unwrap().as_str()
        };
        vec_string.push(String::from(version));
    }
   
    vec_string
}
