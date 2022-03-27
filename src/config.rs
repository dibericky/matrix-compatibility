use crate::gitlab_api;
use crate::gitlab_ci;
use crate::table_builder::CompatibilityRow;
use serde::Deserialize;

#[derive(PartialEq, Deserialize)]
pub struct MatrixItem {
    pub path: String,
    pub name: String,
}

#[derive(PartialEq, Deserialize)]
pub struct GitlabCi {
    project_id: String,
}

#[derive(PartialEq, Deserialize)]
pub struct Service {
    pub name: String,
    pub ci: GitlabCi,
    pub matrix: Vec<MatrixItem>,
}

impl Service {
    pub async fn get_ci(&self, gitlab_base_host: &str) -> String {
        gitlab_api::get_file_raw(gitlab_base_host, &self.ci.project_id, ".gitlab-ci.yml").await
    }
}

#[derive(PartialEq, Deserialize)]
pub struct Config {
    pub gitlab_base_api_host: String,
    pub services: Vec<Service>,
}

impl Config {
    pub fn get_list_of_subjects(&self) -> Vec<&String> {
        let mut subjects: Vec<&String> = vec![];
        self.services.iter().for_each(|service| {
            service.matrix.iter().for_each(|matrix| {
                if !subjects.contains(&&matrix.name) {
                    subjects.push(&matrix.name);
                }
            })
        });
        subjects
    }

    async fn push_service_compatibility_rows<'a>(
        compatibility_vec: &mut Vec<CompatibilityRow<'a>>,
        service: &'a Service,
        gitlab_base_host: &str,
    ) {
        let pipeline = service.get_ci(gitlab_base_host).await;
        for item in &service.matrix {
            let result = gitlab_ci::get_matrix(&pipeline, &item.path);
            for version in result {
                let row: CompatibilityRow<'a> = CompatibilityRow {
                    service_name: &service.name,
                    compatibility_subject: String::from(&item.name),
                    version,
                };
                compatibility_vec.push(row);
            }
        }
    }

    pub async fn config_to_compatibility_vec(&self) -> Vec<CompatibilityRow<'_>> {
        let mut compatibility_vec: Vec<CompatibilityRow> = Vec::new();
        for service in &self.services {
            Config::push_service_compatibility_rows(
                &mut compatibility_vec,
                service,
                &self.gitlab_base_api_host,
            )
            .await;
        }
        compatibility_vec
    }
}
