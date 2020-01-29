use std::collections::HashMap;

use log::info;
use maplit::hashmap;
use reqwest::blocking::Client;
use reqwest::blocking::Response;
use reqwest::Error;
use serde::{Deserialize, Serialize};

const BASE_URL_V1_1: &str = "https://circleci.com/api/v1.1";
const BASE_URL_V2: &str = "https://circleci.com/api/v2";

pub struct CircleCiClient {
    client: Client,
    token: String,
}

impl CircleCiClient {
    pub fn new(token: String) -> CircleCiClient {
        CircleCiClient { client: Client::new(), token }
    }

    pub fn get_me(&self) -> Result<Response, Error> {
        self.get_v1_1("/me", &hashmap! {})
    }

    pub fn get_all_projects(&self) -> Result<Vec<Project>, Error> {
        Ok(self.get_v1_1("/projects", &hashmap! {})?
            .json()?)
    }

    pub fn get_all_pipelines(&self, project: &str) -> Result<PipelineList, Error> {
        Ok(self.get_v2(
            format!("/project/{vcs_type}/{username}/{project}/pipeline",
                    vcs_type = "github",
                    username = "MeinDach",
                    project = project).as_str(), &hashmap!(),
        )?.json()?)
    }

    pub fn get_latest_artifacts(&self, project: &str, branch: &str) -> Result<Vec<Artifact>, Error> {
        Ok(self.get_v1_1(
            format!(
                "/project/{vcs_type}/{username}/{project}/latest/artifacts",
                vcs_type = "github",
                username = "MeinDach",
                project = project
            ).as_str(),
            &hashmap! {"branch" => branch},
        )?.json()?)
    }

    pub fn trigger_build_for(&self, project: &str, branch: &Option<String>, tag: &Option<String>) -> Result<PipelineLight, Error> {
        let mut body = HashMap::new();
        if branch.is_some() {
            body.insert("branch", branch.as_deref().unwrap());
        }
        if tag.is_some() {
            body.insert("tag", tag.as_deref().unwrap());
        }
        Ok(self.post(format!("/project/{}/{}/{}/pipeline", "github", "MeinDach", project).as_str(), body)?
            .json()?)
    }

    fn get_v1_1(&self, url: &str, query_params: &HashMap<&str, &str>) -> Result<Response, Error> {
        let url = self.create_url_v1_1(url, query_params);
        info!("GET {}", url);
        self.client.get(url.as_str())
            .send()
    }

    fn get_v2(&self, url: &str, query_params: &HashMap<&str, &str>) -> Result<Response, Error> {
        let url = self.create_url_v2(url, query_params);
        info!("GET {}", url);
        self.client.get(url.as_str())
            .header("Circle-Token", &self.token)
            .send()
    }

    fn post(&self, url: &str, body: HashMap<&str, &str>) -> Result<Response, Error> {
        let url = self.create_url_v2(url, &hashmap! {});
        info!("POST {} with body {:?}", url, body);
        self.client.post(url.as_str())
            .header("Circle-Token", &self.token)
            .json(&body)
            .send()
    }

    fn create_url_v1_1(&self, path: &str, query_params: &HashMap<&str, &str>) -> String {
        let mut query_params = query_params.to_owned();
        query_params.insert("circle-token", self.token.as_str());
        CircleCiClient::create_url(BASE_URL_V1_1, path, &mut query_params)
    }

    fn create_url_v2(&self, path: &str, query_params: &HashMap<&str, &str>) -> String {
        CircleCiClient::create_url(BASE_URL_V2, path, &mut &query_params)
    }

    fn create_url(base_url: &str, path: &str, query_params: &HashMap<&str, &str>) -> String {
        format!("{}{}?{}", base_url, path,
                query_params.iter()
                    .map(|(key, value)| {
                        format!("{}={}", key, value)
                    })
                    .collect::<Vec<_>>()
                    .join("&")
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    vcs_url: String,
    following: bool,
    username: String,
    #[serde(rename = "reponame")]
    repo_name: String,
    branches: HashMap<String, Branch>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Branch {
    #[serde(default = "empty_string_vec")]
    pusher_logins: Vec<String>,
    last_non_success: Option<Build>,
    last_success: Option<Build>,
    #[serde(default = "empty_build_vec")]
    recent_builds: Vec<Build>,
    #[serde(default = "empty_build_vec")]
    running_builds: Vec<Build>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Build {
    pushed_at: String,
    vcs_revision: String,
    build_num: u32,
    outcome: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Artifact {
    path: String,
    pretty_path: String,
    node_index: u32,
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PipelineLight {
    id: String,
    state: String,
    number: u32,
    created_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PipelineList {
    items: Vec<Pipeline>,
    next_page_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pipeline {
    id: String,
    errors: Vec<PipelineError>,
    project_slug: String,
    updated_at: String,
    number: u32,
    state: String,
    created_at: String,
    trigger: Trigger,
    vcs: VCS,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PipelineError {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Trigger {
    #[serde(rename = "type")]
    trigger_type: String,
    received_at: String,
    actor: Actor,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Actor {
    login: String,
    avatar_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VCS {
    provider_name: String,
    origin_repository_url: String,
    target_repository_url: String,
    revision: String,
    branch: String,
    tag: String,
    commit: Commit,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Commit {
    subject: String,
    body: String,
}

#[derive(Deserialize, Debug)]
struct User {
    login: String,
    id: u32,
}

fn empty_string_vec() -> Vec<String> {
    vec![]
}

fn empty_build_vec() -> Vec<Build> {
    vec![]
}
