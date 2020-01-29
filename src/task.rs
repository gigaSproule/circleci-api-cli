use std::str::FromStr;

pub enum Task {
    GetAllPipelines,
    GetLatestArtifacts,
    GetMe,
    ListAll,
    Trigger,
}

#[derive(Debug)]
pub struct TaskParseStringError {
    pub message: String
}

impl ToString for TaskParseStringError {
    fn to_string(&self) -> String {
        (*self.message).to_owned()
    }
}

impl FromStr for Task {
    type Err = TaskParseStringError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "get_all_pipelines" => Ok(Task::GetAllPipelines),
            "get_latest_artifacts" => Ok(Task::GetLatestArtifacts),
            "get_me" => Ok(Task::GetMe),
            "trigger" => Ok(Task::Trigger),
            "list_all" => Ok(Task::ListAll),
            _ => Err(TaskParseStringError { message: format!("Unknown Task {:?}", s) })
        }
    }
}
