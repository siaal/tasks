use std::error::Error;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::task::Task;

#[derive(Debug, Deserialize, Serialize)]
pub struct Bank {
    #[serde(skip)]
    bank_path: PathBuf,
    pub tasks: Vec<Task>,
}

impl Bank {
    pub fn close(&self) -> Result<(), Box<dyn Error>> {
        self.to_file(&self.bank_path)
    }

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

    pub fn empty() -> Bank {
        return Bank {
            bank_path: "".into(),
            tasks:     vec![],
        };
    }

    pub fn from_file(path: &PathBuf) -> Result<Bank, Box<dyn Error>> {
        let s = std::fs::read_to_string(path)?;
        let mut bank: Bank = serde_yml::from_str(s.as_str())?;
        bank.bank_path = path.clone();
        return Ok(bank);
    }

    pub fn to_file<'a>(&self, file: &PathBuf) -> Result<(), Box<dyn Error>> {
        let s = serde_yml::to_string(dbg!(self))?;
        let f = std::fs::File::create(file)?;

        let s = dbg!(s);
        let mut f = dbg!(f);
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
