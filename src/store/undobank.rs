use core::fmt;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::task::Task;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UndoBank {
    pub undoitems: Vec<UndoItem>,
}

impl UndoBank {
    pub fn empty() -> UndoBank {
        UndoBank { undoitems: vec![] }
    }

    pub fn from_file(path: &PathBuf) -> Result<UndoBank, Box<dyn Error>> {
        let s = std::fs::read_to_string(path)?;
        let bank: UndoBank = serde_yml::from_str(s.as_str())?;
        return Ok(bank);
    }

    pub fn to_file<'a>(&self, file: &PathBuf) -> Result<(), Box<dyn Error>> {
        let len = self.undoitems.len();
        let mut new_self: UndoBank = self.clone();
        if len > 50 {
            new_self.undoitems = new_self.undoitems[(len - 50)..len].to_vec();
        };
        let s = serde_yml::to_string(&new_self)?;
        let mut f = std::fs::File::create(file)?;

        f.write_all(s.as_bytes())?;

        return Ok(());
    }

    pub fn append(&mut self, item: UndoItem) {
        self.undoitems.push(item)
    }

    pub fn pop(&mut self) -> Option<UndoItem> {
        self.undoitems.pop()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum UndoItem {
    Move {
        task: Task,
        from: String,
        to:   String,
    },
    Change {
        from:    Task,
        to:      Task,
        #[serde(default = "default_bank")]
        in_bank: String,
    },

    Add {
        new_task: Task,
    },
    Sequence(Vec<UndoItem>),
}

fn default_bank() -> String {
    return "active".to_string();
}

impl fmt::Display for UndoItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{:?}", self);
    }
}
