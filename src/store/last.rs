use std::error::Error;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Last {
    #[serde(default)]
    pub last: Option<String>,
}

impl Last {
    pub fn from_file(path: &PathBuf) -> Result<Last, Box<dyn Error>> {
        let s = std::fs::read_to_string(path)?;
        let last: Last = serde_yml::from_str(s.as_str())?;
        return Ok(last);
    }

    pub fn to_file<'a>(&self, file: &PathBuf) -> Result<(), Box<dyn Error>> {
        let s = serde_yml::to_string(self)?;
        let mut f = std::fs::File::create(file)?;

        f.write_all(s.as_bytes())?;

        return Ok(());
    }
}

impl Default for Last {
    fn default() -> Self {
        Last { last: None }
    }
}
