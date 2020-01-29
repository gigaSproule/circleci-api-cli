use std::fs::read_to_string;
use std::path::PathBuf;

use dirs::home_dir;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use serde::export::fmt::Debug;
use structopt::StructOpt;

use circle_ci_client::CircleCiClient;
use task::Task;

mod circle_ci_client;
mod task;

#[derive(StructOpt)]
struct Cli {
    task: Task,
    #[structopt(short = "p", long = "project")]
    project: Option<String>,
    #[structopt(short = "t", long = "tag")]
    tag: Option<String>,
    #[structopt(short = "b", long = "branch")]
    branch: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    circleci_token: String,
    project: Option<String>,
    tag: Option<String>,
    branch: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    let config = get_config();
    let args: Cli = Cli::from_args();
    let config = merge(&config, &args);
    debug!("Received project {} tag {} and branch {}",
           config.project.as_deref().unwrap_or(""),
           config.tag.as_deref().unwrap_or(""),
           config.branch.as_deref().unwrap_or("")
    );
    let client = CircleCiClient::new(config.circleci_token.to_owned());

    handle_task(&args.task, &config, client);
    Ok(())
}

fn merge(config: &Config, args: &Cli) -> Config {
    debug!("Merging config with args");
    let merged = Config {
        circleci_token: (config.circleci_token).to_owned(),
        project: args.project.to_owned().or(config.project.to_owned()),
        tag: args.tag.to_owned().or(config.tag.to_owned()),
        branch: args.branch.to_owned().or(config.branch.to_owned()),
    };
    debug!("Merged config with args");
    merged
}

fn handle_task(task: &Task, config: &Config, client: CircleCiClient) {
    match task {
        Task::GetAllPipelines => {
            let pipelines = client.get_all_pipelines(&config.project.as_deref().unwrap()).unwrap();
            info!("{:?}", pipelines);
        }
        Task::GetLatestArtifacts => {
            let latest_artifacts = client.get_latest_artifacts(&config.project.as_deref().unwrap(), config.branch.as_ref().unwrap()).unwrap();
            info!("{:?}", latest_artifacts);
        }
        Task::GetMe => {
            let me = client.get_me().unwrap();
            info!("{:?}", me);
        }
        Task::Trigger => {
            let trigger = client.trigger_build_for(&config.project.as_deref().unwrap(), &config.branch, &config.tag).unwrap();
            info!("{:?}", trigger);
        }
        Task::ListAll => {
            let list_all = client.get_all_projects().unwrap();
            info!("{:?}", list_all);
        }
    };
}

fn get_config() -> Config {
    debug!("Getting the config");
    let home = home_dir().expect("No Home :(");
    let config = PathBuf::from(home.join(".circleci-config"));
    let content = read_to_string(&config).expect("Failed to read the content of .circleci-config");
    let config = serde_yaml::from_str(&content).expect("An error occurred during parsing the contents of .circleci-config");
    debug!("Config found {}", serde_yaml::to_string(&config).unwrap());
    config
}
