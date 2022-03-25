use std::env;

async fn get (ci_url: &str, token: &str) -> String {
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

pub async fn get_file_raw(gitlab_base_host: &str, project_id: &str, file_path: &str) -> String {
    let gitlab_token  = env::var("GITLAB_TOKEN").expect("$GITLAB_TOKEN is not set");
    let mut api_url = gitlab_base_host.to_owned();
    api_url.push_str("/api/v4/projects/:projectId/repository/files/:filePath/raw");
    api_url = api_url
        .replacen(":projectId", project_id, 1)
        .replacen(":filePath", file_path, 1);
    get(&api_url, &gitlab_token).await
}