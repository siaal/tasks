pub mod bank;
pub mod config;
pub mod env;
pub mod task;

use std::error::Error;
use std::ffi::OsString;
use std::path::PathBuf;

pub use config::Config;

pub use crate::bank::Bank;
pub use crate::task::TaskType;
const FILES: [&'static str; 2] = ["active", "complete"];

pub fn init_directory(directory: &PathBuf) -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all(&directory)?;
    let mut dirs: Vec<Box<OsString>> = directory
        .read_dir()
        .expect("reading dir failed")
        .filter_map(|r| match r {
            Err(_) => None,
            Ok(dir) => Some(Box::new(dir.file_name())),
        })
        .collect();
    dirs.sort();
    let bank = Bank::empty();
    let bank = dbg!(bank);

    for file in FILES {
        let file = directory.join(file);
        let name = file.file_name().unwrap();
        let name = Box::new(name.to_os_string());
        let exists = dirs.binary_search(&name).is_ok();
        if !exists {
            bank.to_file(&file)?
        }
    }
    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_directory_test() -> Result<(), Box<dyn Error>> {
        let paths: [&'static str; 2] = ["/tmp/foo", "/tmp/foobar"];
        for path in paths {
            let path = PathBuf::from(path);
            init_directory(&path)?;
        }
        Ok(())
    }
}
