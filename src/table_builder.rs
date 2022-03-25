use serde::{Serialize};

#[derive(PartialEq, Serialize)]
struct CompatibilityItem {
    name: String,
    compatible: Vec<String>
}

#[derive(PartialEq, Serialize)]
pub struct CompatibilityRow<'a> {
    pub(crate) service_name: &'a String,
    pub(crate) compatibility_subject: String,
    pub(crate) version: String
}

fn get_table_rows_and_column_by_subject<'a, 'b > (subject: &str, rows: &'a Vec<CompatibilityRow>) -> (Vec<String>, Vec<String>) {
    let mut versions : Vec<String> = Vec::new();
    let mut services_names : Vec<String> = Vec::new();

    rows.iter()
        .filter(|row| row.compatibility_subject == subject)
        .for_each(|row| {
            if !versions.contains(&row.version) {
                versions.push(String::from(&row.version));
            }
            if !services_names.contains(&row.service_name) {
                services_names.push(String::from(row.service_name));
            }
        });
    (services_names, versions)
}

pub fn get_table_by_subject (subject: &str, rows: &Vec<CompatibilityRow>) -> Vec<Vec<String>> {
    let (table_rows, table_cols) = get_table_rows_and_column_by_subject(subject, rows);
    let rows_of_subject = rows.iter()
        .filter(|row| row.compatibility_subject == subject);

    let mut table : Vec<Vec<String>> = Vec::new();
    let mut first_row : Vec<String> = Vec::new();
    first_row.push(String::from(subject));
    for v in table_cols.iter() {
        first_row.push(String::from(v));
    }
    table.push(first_row);
    let results = table_rows.iter().map(|service_name| {
        let all_service_version : Vec<String> = rows_of_subject
            .clone()
            .filter(|item| item.service_name == service_name)
            .map(|item| &item.version)
            .cloned()
            .collect();
        let list_of_versions = table_cols
            .iter()
            .map(|version| all_service_version.contains(&version).to_string())
            .collect::<Vec<String>>();
        let mut versions_for_service : Vec<String> = Vec::new();
        
        versions_for_service.push(String::from(service_name));
        for v in list_of_versions {
            versions_for_service.push(String::from(v))
        }
        versions_for_service
    })
    .collect::<Vec<Vec<String>>>();
    for r in results {
        table.push(r);
    }
    table
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
            CompatibilityRow{
                service_name: &service_1,
                compatibility_subject: String::from("mongo"),
                version: String::from("1.0.0"),
            },
            CompatibilityRow{
                service_name: &service_1,
                compatibility_subject: String::from("kafka"),
                version: String::from("2.0.0"),
            },
            CompatibilityRow{
                service_name: &service_2,
                compatibility_subject: String::from("mongo"),
                version: String::from("2.5.0"),
            },
            CompatibilityRow{
                service_name: &service_1,
                compatibility_subject: String::from("mongo"),
                version: String::from("1.5.0"),
            },
        ];
        assert_eq!(get_table_rows_and_column_by_subject(subject, &rows), (
            vec![String::from("service-1"), String::from("service-2")],
            vec![String::from("1.0.0"), String::from("2.5.0"), String::from("1.5.0")],
        )
        );
    }
}