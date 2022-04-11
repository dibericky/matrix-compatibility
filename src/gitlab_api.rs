use crate::env_manager::{Env};
use serde::Deserialize;
use serde_json;

mod gitlab_http_client {
    pub async fn get(ci_url: &str, token: &str) -> String {
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
}

#[derive(Deserialize)]
pub struct GitlabCommitApi {
    pub name: String
}

mod gitlab_api_urls {
    pub fn get_file_raw(gitlab_base_host: &str, project_id: &str, file_path: &str) -> String {
        let mut api_url = gitlab_base_host.to_owned();
        api_url.push_str("/api/v4/projects/:projectId/repository/files/:filePath/raw");
        api_url
            .replacen(":projectId", project_id, 1)
            .replacen(":filePath", file_path, 1)
    }

    pub fn get_tags(gitlab_base_host: &str, project_id: &str) -> String {
        let mut api_url = gitlab_base_host.to_owned();
        api_url.push_str("/api/v4/projects/:projectId/repository/tags");
        api_url
            .replacen(":projectId", project_id, 1)
    }
}

pub async fn get_file_raw(gitlab_base_host: &str, project_id: &str, file_path: &str) -> String {
    let gitlab_token = Env::get().gitlab_token;
    let api_url = gitlab_api_urls::get_file_raw(gitlab_base_host, project_id, file_path);
    gitlab_http_client::get(&api_url, &gitlab_token).await
}

pub async fn get_tags(gitlab_base_host: &str, project_id: &str) -> Vec<GitlabCommitApi> {
    let gitlab_token = Env::get().gitlab_token;
    let api_url = gitlab_api_urls::get_tags(gitlab_base_host, project_id);
    let response = gitlab_http_client::get(&api_url, &gitlab_token).await;
    let tags: Vec<GitlabCommitApi> = serde_json::from_str(&response).unwrap_or_default();
    tags
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_urls_get_file_raw() {
        let host = "http://my-url.com";
        let project_id = "foo%2Fbar";
        let file_path = "my-file.json";
        let result = gitlab_api_urls::get_file_raw(host, project_id, file_path);
        assert_eq!(
            &result,
            "http://my-url.com/api/v4/projects/foo%2Fbar/repository/files/my-file.json/raw"
        )
    }
}
