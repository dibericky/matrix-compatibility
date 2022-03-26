mod config;
mod env_manager;
mod gitlab_api;
mod gitlab_ci;
mod table_builder;

use config::Config;
use std::fs::File;
use std::io::prelude::*;
use table_builder::CompatibilityRow;

fn generate_output(subject: &str, md_table: &str) {
    println!("{}", subject);
    println!("{}", md_table);
    let mut output_file_name = String::from(subject);
    output_file_name.push_str("_output.md");
    let mut output = File::create(output_file_name).unwrap();
    write!(output, "{}", md_table).unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_file_path = env_manager::get_config_file_path();

    let f = File::open(config_file_path).unwrap();
    let config: Config = serde_yaml::from_reader(f).unwrap();

    let compatibility_vec: Vec<CompatibilityRow> = config.config_to_compatibility_vec().await;

    let list_subjects = config.get_list_of_subjects();
    for subject in list_subjects {
        let version_columns = table_builder::get_table_by_subject(subject, &compatibility_vec);
        let md_table = table_builder::as_markdown(version_columns);
        generate_output(subject, &md_table);
    }

    Ok(())
}
