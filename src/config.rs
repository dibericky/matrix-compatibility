use crate::gitlab_api;
use crate::gitlab_ci;
use crate::table_builder::CompatibilityRow;
use crate::versions::get_major_versions;
use serde::Deserialize;

#[derive(PartialEq, Deserialize, Debug)]
pub struct MatrixItem {
    pub path: String,
    pub name: String,
    pub include: Option<usize>
}

#[derive(PartialEq, Deserialize, Debug)]
pub struct GitlabCi {
    project_id: String,
}

#[derive(PartialEq, Deserialize, Debug)]
pub struct Service {
    pub name: String,
    pub ci: GitlabCi,
    pub matrix: Vec<MatrixItem>,
}

impl Service {
    pub async fn get_ci(&self, gitlab_base_host: &str, git_ref: Option<&str>) -> String {
        gitlab_api::get_file_raw(gitlab_base_host, &self.ci.project_id, ".gitlab-ci.yml", git_ref).await
    }

    pub async fn get_tags(&self, gitlab_base_host: &str) -> Vec<String> {
        let tags = gitlab_api::get_tags(gitlab_base_host, &self.ci.project_id).await;
        tags.iter().map(|tag| tag.name.to_owned()).collect()   
    }
}

#[derive(PartialEq, Deserialize, Debug)]
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
        compatibility_vec: &mut Vec<CompatibilityRow>,
        service: &'a Service,
        gitlab_base_host: &str,
    ) {
        let tags = service.get_tags(gitlab_base_host).await;
        let majors = get_major_versions(&tags);
        let majors = majors[0..2].to_owned();
        for tag_major in majors {
            let service_name_with_version = format!("{}@{}", &service.name, &tag_major);
            let pipeline = service.get_ci(gitlab_base_host, Some(&tag_major)).await;
            for item in &service.matrix {
                if let Some(result) = gitlab_ci::get_matrix(&pipeline, &item.path, item.include, gitlab_base_host).await {
                    for version in result {
                        let row: CompatibilityRow = CompatibilityRow {
                            service_name: service_name_with_version.to_owned(),
                            compatibility_subject: String::from(&item.name),
                            version: Some(version),
                        };
                        compatibility_vec.push(row);
                    }
                } else {
                    let row: CompatibilityRow = CompatibilityRow {
                        service_name: service_name_with_version.to_owned(),
                        compatibility_subject: String::from(&item.name),
                        version: None,
                    };
                    compatibility_vec.push(row);
                }
            }
        }
    }

    pub async fn config_to_compatibility_vec(&self) -> Vec<CompatibilityRow> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_major_versions() {
        let versions : Vec<String> = vec![
            "v3.5.1".to_owned(),
            "v4.1.2".to_owned(), 
            "v5.5.1-rc.0".to_owned(),
            "v5.4.5".to_owned(),
            "v3.9.0".to_owned()
        ];
        let result = get_major_versions(&versions);

        let expected = vec!["v5.4.5", "v4.1.2", "v3.9.0"];
        assert_eq!(
            &result,
            &expected
        )
    }
}