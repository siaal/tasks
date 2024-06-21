use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize};

const PATH: [&'static str; 2] = ["~/.config/tasks/tasks.toml", "~/.config/tasks.toml"];

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    #[serde(default = "default_path", deserialize_with = "deserialize_path")]
    pub task_path: PathBuf,
    #[serde(default = "default_verbose")]
    pub verbose:   bool,
    #[serde(default = "default_cutoff")]
    pub cutoff:    u64,
}

fn default_cutoff() -> u64 {
    return 60 * 60 * 24; // 1 day
}

fn expand_path(path: &str) -> PathBuf {
    let path = shellexpand::tilde(path);
    let p = Path::new(&*path).to_path_buf();
    return p;
}

fn deserialize_path<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    return Ok(expand_path(&s));
}

fn default_path() -> PathBuf {
    return expand_path("~/.tasks");
}

fn default_verbose() -> bool {
    return false;
}

impl Default for Config {
    fn default() -> Self {
        return Config {
            task_path: default_path(),
            verbose:   default_verbose(),
            cutoff:    default_cutoff(),
        };
    }
}

impl Config {
    pub fn from_path() -> Config {
        for path in PATH {
            match Config::from_file(&path) {
                Ok(conf) => return conf,
                Err(_error) => {
                    let e = _error.to_string();
                    println!("Error in {path} - {e}");
                    continue;
                },
            }
        }
        println!("using fallback");
        return Config::default();
    }
    pub fn from_string(string: &str) -> Result<Config, Box<dyn Error>> {
        let conf: Config = toml::from_str::<Config>(string)?;
        return Ok(conf);
    }

    pub fn from_file(file: &str) -> Result<Config, Box<dyn Error>> {
        let file = &*shellexpand::full(file)?;
        let content = fs::read_to_string(file)?;
        let conf = Config::from_string(&content)?;
        return Ok(conf);
    }

    pub fn to_string<'a>(&self) -> Result<String, Box<dyn Error>> {
        let s = toml::to_string(self)?;
        return Ok(s);
    }

    pub fn to_file(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let path = &*shellexpand::full(path)?;
        let s = self.to_string()?;
        fs::write(path, s)?;
        return Ok(());
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string() {
        let s = "";
        assert_eq!(s, s)
    }
}
