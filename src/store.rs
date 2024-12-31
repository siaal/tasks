use std::error::Error;
use std::ffi::OsString;
use std::path::PathBuf;

use fuzzy_finder::item::Item;
use fuzzy_finder::FuzzyFinder;
use rand::Rng;
use undobank::UndoItem;

mod bank;
mod last;
mod undobank;
use std::cell::{RefCell, RefMut};

use bank::Bank;
use last::Last;
use undobank::UndoBank;

use crate::task::Task;
pub use crate::task::TaskType;

pub struct Store {
    directory: PathBuf,
    active:    RefCell<Option<Bank>>,
    closed:    RefCell<Option<Bank>>,
    undo:      RefCell<Option<UndoBank>>,
    last:      RefCell<Option<Last>>,
}

const ACTIVE: &'static str = "active";
const CLOSED: &'static str = "closed";
const UNDO: &'static str = "undo";
const LAST: &'static str = "last";

pub fn init_store(directory: &PathBuf) -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all(&directory)?;
    let mut dirs: Vec<Box<OsString>> = directory
        .read_dir()
        .expect("reading dir failed")
        .filter_map(|r| match r {
            Ok(dir) => Some(Box::new(dir.file_name())),
            Err(_) => None,
        })
        .collect();
    dirs.sort();
    let files = [ACTIVE, CLOSED, UNDO, LAST];

    let bank = Bank::empty();
    for (i, file) in files.iter().enumerate() {
        let file = directory.join(file);
        let name = file.file_name().unwrap();
        let name = Box::new(name.to_os_string());
        let exists = dirs.binary_search(&name).is_ok();

        if !exists {
            match i {
                0 | 1 => bank.to_file(&file)?,
                2 => UndoBank::empty().to_file(&file)?,
                3 => Last::default().to_file(&file)?,
                _ => panic!("`i` beyond 3 not implemented"),
            }
        }
    }
    return Ok(());
}

impl Store {
    pub fn new(dir: PathBuf) -> Store {
        Store {
            directory: dir.clone(),
            active:    RefCell::from(None),
            closed:    RefCell::from(None),
            undo:      RefCell::from(None),
            last:      RefCell::from(None),
        }
    }

    fn load_active(&self) -> RefMut<Bank> {
        let mut active = self.active.borrow_mut();
        if let None = *active {
            let bank = Bank::from_file(&self.directory.join(ACTIVE)).unwrap();
            *active = Some(bank);
        }
        return std::cell::RefMut::map(active, |opt| opt.as_mut().unwrap());
    }

    fn unload_active(&self) {
        if let Some(active) = self.active.take() {
            active.to_file(&self.directory.join(ACTIVE)).unwrap();
        }
    }

    fn load_closed(&self) -> RefMut<Bank> {
        let mut closed = self.closed.borrow_mut();
        if let None = *closed {
            let bank = Bank::from_file(&self.directory.join(CLOSED)).unwrap();
            *closed = Some(bank);
        }
        return std::cell::RefMut::map(closed, |opt| {
            opt.as_mut().expect("Expected value but found None")
        });
    }
    fn unload_closed(&self) {
        let closed = self.closed.take();
        if let Some(closed) = closed {
            closed.to_file(&self.directory.join(CLOSED)).unwrap();
        }
    }

    fn load_undo(&self) -> RefMut<UndoBank> {
        let mut undo = self.undo.borrow_mut();
        if let None = *undo {
            let bank = UndoBank::from_file(&self.directory.join(UNDO)).unwrap();
            *undo = Some(bank);
        }
        return std::cell::RefMut::map(undo, |opt| {
            opt.as_mut().expect("Expected value but found None")
        });
    }

    fn unload_undo(&self) {
        let undo = self.undo.take();
        if let Some(undo) = undo {
            undo.to_file(&self.directory.join(UNDO)).unwrap();
        }
    }

    fn load_last(&self) -> RefMut<Last> {
        let mut last = self.last.borrow_mut();
        if let None = *last {
            let bank = Last::from_file(&self.directory.join(LAST)).unwrap();
            *last = Some(bank);
        }
        return std::cell::RefMut::map(last, |opt| {
            opt.as_mut().expect("Expected value but found None")
        });
    }
    fn unload_last(&self) {
        let last = self.last.take();
        if let Some(last) = last {
            last.to_file(&self.directory.join(LAST)).unwrap();
        }
    }

    pub fn undo(&self) -> Result<UndoItem, Box<dyn Error>> {
        let mut last = self.load_last();
        last.last = None;

        let mut undo = self.load_undo();

        let item = undo.pop();
        match item {
            Some(item) => {
                self.undo_item(item.clone())?;
                return Ok(item);
            },
            None => {
                return Err("No more undo items.".into());
            },
        }
    }

    fn undo_item(&self, item: UndoItem) -> Result<(), Box<dyn Error>> {
        match item {
            UndoItem::Add { new_task } => self.delete_item(ACTIVE, new_task)?,
            UndoItem::Move { task, from, to } => self.move_item(task, &to, &from)?,
            UndoItem::Change {
                from,
                to: _,
                in_bank,
            } => self.force_update_item(in_bank.as_str(), from)?,
            UndoItem::Sequence(vec) => {
                for item in vec.into_iter().rev() {
                    self.undo_item(item)?;
                }
            },
        }
        return Ok(());
    }

    pub fn move_item(
        &self,
        task: Task,
        from_bank: &str,
        to_bank: &str,
    ) -> Result<(), Box<dyn Error>> {
        self.delete_item(from_bank, task.clone())?;
        self.add_item(task, to_bank)?;
        return Ok(());
    }

    pub fn add_item(&self, task: Task, bank: &str) -> Result<(), Box<dyn Error>> {
        let mut bank = self.get_bank(bank)?;
        bank.append(task);
        return Ok(());
    }

    pub fn delete_item(&self, bank: &str, task: Task) -> Result<(), Box<dyn Error>> {
        let mut bank = self.get_bank(bank)?;
        bank.delete(task.id().into());
        return Ok(());
    }

    pub fn force_update_item(&self, bank: &str, updated: Task) -> Result<(), Box<dyn Error>> {
        let mut bank = self.get_bank(bank)?;
        bank.update(updated);
        return Ok(());
    }

    pub fn get_bank(&self, bank_name: &str) -> Result<RefMut<Bank>, Box<dyn Error>> {
        match bank_name {
            ACTIVE => Ok(self.load_active()),
            CLOSED => Ok(self.load_closed()),
            _ => Err(format!("Invalid bank name: {}", bank_name).into()),
        }
    }

    pub fn retire_item(&self, task: &Task) -> Result<Task, Box<dyn Error>> {
        let mut last = self.load_last();
        let mut active = self.load_active();
        let mut closed = self.load_closed();
        let mut undo = self.load_undo();

        last.last = None;

        let completed = task.completed();

        let ok = active.delete(task.id());
        if !ok {
            return Err("Could not find task in active list".into());
        }
        closed.append(completed.clone());
        undo.append(UndoItem::Sequence(vec![
            UndoItem::Change {
                in_bank: ACTIVE.to_string(),
                from:    task.clone(),
                to:      completed.clone(),
            },
            UndoItem::Move {
                task: completed.clone(),
                from: ACTIVE.to_string(),
                to:   CLOSED.to_string(),
            },
        ]));
        return Ok(completed);
    }

    pub fn append(&self, task: Task) -> Result<Task, Box<dyn Error>> {
        let mut active = self.load_active();
        let mut last = self.load_last();
        let mut undo = self.load_undo();

        active.append(task.clone());
        last.last = Some(task.id().to_string());

        undo.append(UndoItem::Add {
            new_task: task.clone(),
        });
        return Ok(task);
    }

    pub fn fzf(&self, terms: &[String]) -> Option<Task> {
        if let Some(task) = self.keyword_check(terms) {
            return Some(task);
        }
        let active = self.load_active();
        match fzf_inner(&active, &terms) {
            Some(task) => {
                let mut last = self.load_last();
                last.last = Some(task.id().to_string());
                return Some(task);
            },
            None => return None,
        }
    }

    pub fn filter_active(&self, terms: &[String], tags: &[String], ntags: &[String]) -> Vec<Task> {
        if let Some(task) = self.keyword_check(terms) {
            return vec![task];
        }
        let active = self.load_active();
        let terms = terms
            .iter()
            .map(|string| string.as_str())
            .collect::<Vec<&str>>();
        active
            .iter()
            .filter(|task| task.mass_contains(&terms))
            .filter(|task| tags.iter().all(|tag| task.is_tagged(tag)))
            .filter(|task| !ntags.iter().any(|ntag| task.is_tagged(ntag)))
            .cloned()
            .collect()
    }

    pub fn update_item<F>(&self, task: Task, f: F) -> Result<Task, Box<dyn Error>>
    where
        F: FnOnce(&Task) -> Task,
    {
        let mut undo = self.load_undo();
        let mut active = self.load_active();

        let transformed = f(&task);
        active.update(transformed.clone());
        undo.append(UndoItem::Change {
            from:    task.clone(),
            to:      transformed.clone(),
            in_bank: ACTIVE.to_string(),
        });
        return Ok(transformed);
    }

    pub fn select_random_from_list(&self, list: &[Task], n: u8, cutoff: u64) -> Vec<Task> {
        let mut last = self.load_last();
        let now = chrono::Local::now();
        let mappings: Vec<(u64, usize)> = list
            .iter()
            .enumerate()
            .filter_map(|(idx, task)| {
                let duration_passed: u64 = now
                    .signed_duration_since(task.last_touched())
                    .num_seconds()
                    .try_into()
                    .unwrap();
                if duration_passed < cutoff {
                    return None;
                }
                let priority = task.priority();
                let weight = duration_passed * (*priority as u64);
                return Some((idx, weight));
            })
            .scan(0 as u64, |counter, (idx, duration_passed)| {
                *counter += duration_passed;
                Some((counter.clone(), idx))
            })
            .collect();
        if mappings.len() == 0 {
            return vec![];
        }
        let mut chosen: Vec<usize> = vec![];
        let mut rng = rand::thread_rng();
        let max_mapping = mappings[mappings.len() - 1].0;
        for _ in 0..n {
            loop {
                let chosen_passed: u64 = rng.gen_range((0 as u64)..=max_mapping);

                let loc =
                    match mappings.binary_search_by_key(&chosen_passed, |(passed, _idx)| *passed) {
                        Ok(loc) => loc,
                        Err(loc) => loc,
                    };

                if chosen.contains(&loc) {
                    continue;
                }
                chosen.push(loc);
                break;
            }
        }
        let chosen: Vec<Task> = chosen.iter().map(|i| list[*i].clone()).collect();
        if chosen.len() == 1 {
            last.last = Some(chosen[0].id().to_string());
        } else {
            last.last = None;
        }

        return chosen;
    }

    pub fn select_random(&self, n: u8, cutoff: u64) -> Vec<Task> {
        let active = self.load_active();
        return self.select_random_from_list(&active.tasks, n, cutoff);
    }

    fn keyword_check(&self, terms: &[String]) -> Option<Task> {
        if terms.len() != 1 {
            return None;
        }
        let term: &str = terms[0].as_ref();
        if term.to_lowercase() == "last" {
            let last = self.load_last();
            let active = self.load_active();
            match &last.last {
                None => return None,
                Some(id) => {
                    let found = active.find(&id);
                    let found = found.to_owned().cloned();
                    return found;
                },
            }
        }
        return None;
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        self.unload_active();
        self.unload_undo();
        self.unload_closed();
        self.unload_last();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}

// Assumes you've already checked for `last`
fn fzf_inner(bank: &Bank, terms: &[String]) -> Option<Task> {
    let terms = terms
        .iter()
        .map(|string| string.as_str())
        .collect::<Vec<&str>>();
    let items: Vec<Item<Task>> = bank
        .iter()
        .filter(|task| task.mass_contains(&terms))
        .cloned()
        .map(|task| Item::new(task.name().to_string(), task))
        .collect();
    let item = match items.len() {
        0 => None,
        1 => items[0].item.clone(),
        len => {
            let len: i8 = len.try_into().unwrap();
            let fzf = FuzzyFinder::find(items, len.clamp(1, 20));
            match fzf {
                Ok(opt) => opt,
                Err(err) => panic!("{}", err.to_string()),
            }
        },
    };

    return item;
}
