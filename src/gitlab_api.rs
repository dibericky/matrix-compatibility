use crate::env_manager;

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

mod gitlab_api_urls {
    pub fn get_file_raw(gitlab_base_host: &str, project_id: &str, file_path: &str) -> String {
        let mut api_url = gitlab_base_host.to_owned();
        api_url.push_str("/api/v4/projects/:projectId/repository/files/:filePath/raw");
        api_url
            .replacen(":projectId", project_id, 1)
            .replacen(":filePath", file_path, 1)
    }
}

pub async fn get_file_raw(gitlab_base_host: &str, project_id: &str, file_path: &str) -> String {
    let gitlab_token = env_manager::get_gitlab_token();
    let api_url = gitlab_api_urls::get_file_raw(gitlab_base_host, project_id, file_path);
    gitlab_http_client::get(&api_url, &gitlab_token).await
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
