use regex::{Captures, Regex};
use serde_yaml::Value;
use std::str::Split;

fn match_if_array_step(step: &str) -> Option<Captures> {
    let regex_array = Regex::new(r"(?P<field>[a-zA-Z_]+)\[(?P<index>\d+)\]$").unwrap();
    regex_array.captures(step)
}

fn get_field_and_index(matched: Option<Captures>) -> (&str, usize) {
    let matched = matched.unwrap();
    let index = matched.name("index").unwrap().as_str();
    let index: usize = index.parse().unwrap();
    let field_name = matched.name("field").unwrap().as_str();
    (field_name, index)
}

fn step_into_array<'a>(current: &'a Value, field_name: &str, index: usize) -> &'a Value {
    let array = current.get(field_name).unwrap();
    let sequence = array.as_sequence().unwrap();
    sequence.get(index).unwrap()
}

fn get_list_of_version_from_matrix(matrix_value: &Value) -> Vec<String> {
    let mut vec_string: Vec<String> = Vec::new();

    for item in matrix_value.as_sequence().unwrap().to_vec() {
        let version = item.as_str().unwrap();
        let version = match Regex::new(r"^(\w+:)?(?P<version>[\d\.]+)$")
            .unwrap()
            .captures(version)
        {
            None => version,
            Some(v) => v.name("version").unwrap().as_str(),
        };
        vec_string.push(String::from(version));
    }
    vec_string
}

pub fn get_matrix(pipeline: &str, path_matrix: &str) -> Vec<String> {
    let deserialized_map: Value = serde_yaml::from_str(&pipeline).unwrap();
    let path_iterator: Split<char> = path_matrix.split('.');

    let mut current_value: &Value = &deserialized_map;
    for step in path_iterator {
        let matched = match_if_array_step(step);
        if matched.is_none() {
            current_value = current_value.get(step).unwrap();
            continue;
        }
        let (field_name, index) = get_field_and_index(matched);
        current_value = step_into_array(current_value, &field_name, index);
    }

    let vec_string: Vec<String> = get_list_of_version_from_matrix(current_value);

    vec_string
}
