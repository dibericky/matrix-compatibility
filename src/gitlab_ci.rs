use futures::{Future, FutureExt};
use regex::{Captures, Regex};
use serde_yaml::Value;
use std::{str::Split, pin::Pin};

use crate::gitlab_api::get_file_raw;

struct IncludeFromCi {
    project: String,
    file: String,
    reference: String,
    gitlab_base_host: String,
}

impl IncludeFromCi {
    pub fn new(field: &Value, gitlab_base_host: &str) -> IncludeFromCi {
        let project = field.get("project").unwrap().as_str().unwrap();
        let file = field.get("file").unwrap().as_str().unwrap();
        let reference = field.get("ref").unwrap().as_str().unwrap();
        IncludeFromCi {
            project: project.to_owned(),
            file: file.to_owned(),
            reference: reference.to_owned(),
            gitlab_base_host: gitlab_base_host.to_owned(),
        }
    }

    pub async fn fetch(&self) -> String {
        let re_starting_slash = Regex::new(r"^/").unwrap();
        let re_escape_slash = Regex::new(r"/").unwrap();

        let file = re_starting_slash
            .replace_all(&self.file, "")
            .to_string();

        let escaped_slash= "%2F";
        let project = &self.project;
        let project = re_escape_slash
            .replace_all(&project, escaped_slash)
            .to_string();

        let file = re_escape_slash
            .replace_all(&file, escaped_slash)
            .to_string();

        let raw = get_file_raw(
            &self.gitlab_base_host, 
            &project, 
            &file
        ).await;
        raw
    }
}

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

    for item in matrix_value.as_sequence().unwrap() {
        let version = item.as_str().unwrap();
        let captured = Regex::new(r"^(\w+:)?(?P<version>[\d\.]+)$")
            .unwrap()
            .captures(version);
        let version = match captured {
            None => version,
            Some(v) => v.name("version").unwrap().as_str(),
        };
        vec_string.push(String::from(version));
    }
    vec_string
}

fn walk_matrix (map: &Value, path_matrix: &str) -> Vec<String> {
    let path_iterator: Split<char> = path_matrix.split('.');

    let mut current_value: &Value = &map;
    for step in path_iterator {
        let matched = match_if_array_step(step);
        if matched.is_none() {
            current_value = current_value.get(step).unwrap();
            continue;
        }
        let (field_name, index) = get_field_and_index(matched);
        current_value = step_into_array(current_value, field_name, index);
    }

    let vec_string: Vec<String> = get_list_of_version_from_matrix(current_value);

    vec_string
}

pub async fn get_matrix(pipeline: &str, path_matrix: &str, include: Option<usize>, gitlab_base_host: &str) -> Vec<String> {
    let deserialized_map: Value = serde_yaml::from_str(pipeline).unwrap();
    if let Some(index) = include {
        let field = step_into_array(&deserialized_map, "include", index);
        let include = IncludeFromCi::new(field, gitlab_base_host);
        let raw_included = include.fetch().await;
        let deserialized_map: Value = serde_yaml::from_str(&raw_included).unwrap();
        let matrix = walk_matrix(&deserialized_map, path_matrix);
        return matrix;
    }
    walk_matrix(&deserialized_map, path_matrix)
}
