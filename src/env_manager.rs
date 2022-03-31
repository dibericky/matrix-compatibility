use std::env;
use dotenv::dotenv;

fn get_config_file_path() -> String {
    match env::var("CONFIG_FILE_PATH") {
        Ok(v) => v,
        Err(_) => String::from("config.yml"),
    }
}

fn get_env(name: &str) -> String {
    env::var(name).expect(&format!("{} env should be set", name))
}

#[derive(Debug)]
pub struct Env {
    pub gitlab_token: String,
    pub config_file_path: String
}

impl Env {
    pub fn get() -> Env {
        dotenv().ok();

        Env{
            config_file_path: get_config_file_path(),
            gitlab_token: get_env("GITLAB_TOKEN")
        }
    }
}