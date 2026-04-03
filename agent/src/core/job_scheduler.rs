use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Job {
    pub id: String,
    pub status: JobStatus,
}

#[derive(Debug, Clone)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

pub struct JobScheduler {
    jobs: HashMap<String, Job>,
}

impl JobScheduler {
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
        }
    }
    
    pub fn schedule(&mut self, job: Job) -> String {
        let id = Uuid::new_v4().to_string();
        self.jobs.insert(id.clone(), job);
        id
    }
    
    pub fn get_status(&self, id: &str) -> Option<JobStatus> {
        self.jobs.get(id).map(|j| j.status.clone())
    }
}
