use core::fmt;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Task {
    task:     TaskType,
    metadata: Metadata,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Metadata {
    id:             String,
    description:    Option<String>,
    name:           String,
    priority:       u16,
    created:        DateTime<Local>,
    last_completed: DateTime<Local>,
    closed_at:      Option<DateTime<Local>>,
}

impl Task {
    pub fn completed(&self) -> Self {
        let now = chrono::Local::now();
        let mut task = self.clone();
        task.metadata.last_completed = now;
        task.metadata.closed_at = Some(now);
        return task;
    }
    pub fn touched(&self) -> Self {
        let now = chrono::Local::now();
        let mut task = self.clone();
        task.metadata.last_completed = now;
        return task;
    }

    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    pub fn id(&self) -> &str {
        &self.metadata.id
    }

    pub fn last_touched(&self) -> &DateTime<Local> {
        &self.metadata.last_completed
    }
    pub fn closed(&self) -> &Option<DateTime<Local>> {
        &self.metadata.closed_at
    }
    pub fn priority(&self) -> &u16 {
        &self.metadata.priority
    }

    pub fn description(&self) -> &Option<String> {
        &self.metadata.description
    }

    pub fn created(&self) -> &DateTime<Local> {
        &self.metadata.created
    }

    pub fn updated_todo<'a>(
        &self,
        desc: Option<&'a str>,
        priority: Option<&u16>,
        name: Option<&'a str>,
    ) -> Task {
        let mut out = self.clone();
        if let Some(desc) = desc {
            out.metadata.description = Some(desc.to_string());
        }
        if let Some(name) = name {
            out.metadata.name = name.to_string();
        }

        if let Some(priority) = priority {
            out.metadata.priority = priority.clone();
        }
        return out;
    }

    pub fn new_todo<'a>(name: String, description: Option<&'a str>, priority: Option<u16>) -> Task {
        let task = TaskType::Todo {};
        let mut meta = Metadata::new();
        meta.name = name.to_string();
        if let Some(desc) = description {
            meta.description = Some(desc.to_string())
        }
        if let Some(priority) = priority {
            meta.priority = priority;
        }
        return Task {
            task,
            metadata: meta,
        };
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum TaskType {
    Todo {},
    Deadline {},
}

impl fmt::Display for TaskType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "{:?}", self);
    }
}

impl TaskType {
    pub fn contains(&self, _string: &str) -> bool {
        match self {
            TaskType::Todo {} => {
                return false;
            },
            TaskType::Deadline {} => {
                return false;
            },
        }
    }
}

static EMPTY_STRING: &'static str = "";

impl Metadata {
    fn new() -> Metadata {
        let now = chrono::Local::now();
        let id: String = now.timestamp_millis().to_string();
        return Metadata {
            id,
            description: None,
            name: EMPTY_STRING.to_string(),
            priority: 100,
            created: now,
            last_completed: now,
            closed_at: None,
        };
    }
    pub fn contains(&self, string: &str) -> bool {
        if self.name.contains(string) {
            return true;
        }
        if let Some(desc) = &self.description {
            if desc.contains(string) {
                return true;
            }
        }
        return self.id.contains(string);
    }
}

impl Task {
    pub fn mass_contains(&self, strings: &[&str]) -> bool {
        for string in strings {
            if !self.contains(string) {
                return false;
            }
        }
        return true;
    }

    pub fn contains(&self, string: &str) -> bool {
        return (self.metadata.contains(string)) || (self.task.contains(string));
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "{:?}", self);
    }
}
