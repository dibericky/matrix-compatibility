use std::cmp::Ordering;
use markdown_table::MarkdownTable;
use serde::Serialize;
use version_compare::{Cmp, compare_to};

#[derive(PartialEq, Serialize)]
struct CompatibilityItem {
    name: String,
    compatible: Vec<String>,
}

#[derive(PartialEq, Serialize, Debug)]
pub struct CompatibilityRow {
    pub(crate) service_name: String,
    pub(crate) compatibility_subject: String,
    pub(crate) version: Option<String>,
}

fn get_table_rows_and_column_by_subject(
    subject: &str,
    rows: &[CompatibilityRow],
) -> (Vec<String>, Vec<String>) {
    let mut versions: Vec<String> = Vec::new();
    let mut services_names: Vec<String> = Vec::new();

    rows.iter()
        .filter(|row| row.compatibility_subject == subject)
        .for_each(|row| {
            if let Some(version) = &row.version {
                if !versions.contains(version) {
                    versions.push(String::from(version));
                }
            }
            if !services_names.contains(&row.service_name) {
                services_names.push(String::from(&row.service_name));
            }
        });
    versions.sort_by(|a, b| if compare_to(a, b, Cmp::Le).unwrap() == true {
        Ordering::Less
    } else {
        Ordering::Greater
    });
    (services_names, versions)
}

pub fn get_table_by_subject(subject: &str, rows: &[CompatibilityRow]) -> Vec<Vec<String>> {
    let (table_rows, table_cols) = get_table_rows_and_column_by_subject(subject, rows);
    let rows_of_subject = rows
        .iter()
        .filter(|row| row.compatibility_subject == subject);

    let mut table: Vec<Vec<String>> = Vec::new();
    let mut first_row: Vec<String> = vec![String::from(subject)];
    for v in table_cols.iter() {
        first_row.push(String::from(v));
    }
    table.push(first_row);
    let results = table_rows
        .iter()
        .map(|service_name| {
            let all_service_version: Vec<String> = rows_of_subject
                .clone()
                .filter(|item| &item.service_name == service_name)
                .filter_map(|item| {
                    if let Some(version) = &item.version {
                        Some(version)
                    } else {
                        None
                    }
                })
                .cloned()
                .collect();
            let list_of_versions = table_cols
                .iter()
                .map(|version| all_service_version.contains(version).to_string())
                .collect::<Vec<String>>();
            let mut versions_for_service: Vec<String> = vec![String::from(service_name)];
            for v in list_of_versions {
                versions_for_service.push(v);
            }
            versions_for_service
        })
        .collect::<Vec<Vec<String>>>();
    for r in results {
        table.push(r);
    }
    table
}

pub fn as_markdown(table: Vec<Vec<String>>) -> String {
    let table = MarkdownTable::new(table);
    table.as_markdown().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_cols_rows() {
        let subject = "mongo";
        let service_1 = String::from("service-1");
        let service_2 = String::from("service-2");
        let rows = vec![
            CompatibilityRow {
                service_name: service_1.to_owned(),
                compatibility_subject: String::from("mongo"),
                version: Some(String::from("1.0.0")),
            },
            CompatibilityRow {
                service_name: service_1.to_owned(),
                compatibility_subject: String::from("kafka"),
                version: Some(String::from("2.0.0")),
            },
            CompatibilityRow {
                service_name: service_2.to_owned(),
                compatibility_subject: String::from("mongo"),
                version: Some(String::from("2.5.0")),
            },
            CompatibilityRow {
                service_name: service_1.to_owned(),
                compatibility_subject: String::from("mongo"),
                version: Some(String::from("1.5.0")),
            },
        ];
        assert_eq!(
            get_table_rows_and_column_by_subject(subject, &rows),
            (
                vec![String::from("service-1"), String::from("service-2")],
                vec![
                    String::from("1.0.0"),
                    String::from("1.5.0"),
                    String::from("2.5.0")
                ],
            )
        );
    }
}
