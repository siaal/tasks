use std::error::Error;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::task::Task;

#[derive(Debug, Deserialize, Serialize)]
pub struct Bank {
    pub tasks: Vec<Task>,
}

impl Bank {
    pub fn update(&mut self, updated_task: Task) -> bool {
        for (i, task) in self.tasks.iter().enumerate() {
            if task.id() == updated_task.id() {
                self.tasks[i] = updated_task;
                return true;
            }
        }
        return false;
    }

    pub fn append(&mut self, task: Task) {
        self.tasks.push(task)
    }

    pub fn delete<'a>(&mut self, task_id: &'a str) -> bool {
        for (i, task) in self.tasks.iter().enumerate() {
            let id = &task.id();
            if *id == task_id {
                self.tasks.swap_remove(i);
                return true;
            }
        }
        return false;
    }

    pub fn find(&self, task_id: &str) -> Option<&Task> {
        for e in self.tasks.iter() {
            if e.id() == task_id {
                return Some(&e);
            }
        }
        return None;
    }

    pub fn empty() -> Bank {
        return Bank { tasks: vec![] };
    }

    pub fn from_file(path: &PathBuf) -> Result<Bank, Box<dyn Error>> {
        let s = std::fs::read_to_string(path)?;
        let bank: Bank = serde_yml::from_str(s.as_str())?;
        return Ok(bank);
    }

    pub fn to_file<'a>(&self, file: &PathBuf) -> Result<(), Box<dyn Error>> {
        let s = serde_yml::to_string(self)?;
        let mut f = std::fs::File::create(file)?;

        f.write_all(s.as_bytes())?;

        return Ok(());
    }

    pub fn iter(&self) -> impl Iterator<Item = &Task> {
        return self.tasks.iter();
    }
}

impl IntoIterator for Bank {
    type Item = Task;
    type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.tasks.into_iter()
    }
}
