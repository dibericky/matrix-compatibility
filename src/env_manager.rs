use std::env;

pub fn get_config_file_path() -> String {
    match env::var("CONFIG_FILE_PATH") {
        Ok(v) => v,
        Err(_) => String::from("config.yml"),
    }
}

pub fn get_gitlab_token() -> String {
    env::var("GITLAB_TOKEN").expect("$GITLAB_TOKEN is not set")
}
