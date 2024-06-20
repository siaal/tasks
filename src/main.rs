use std::path::PathBuf;
use std::process::exit;

use clap::{Args, Parser, Subcommand};
use fuzzy_finder::item::Item;
use fuzzy_finder::FuzzyFinder;
use tasks::task::Task;
use tasks::{init_directory, Bank, Config};

#[derive(Debug, Parser)]
#[command(version, about, long_about=None)] // version|about filled in from cargo.toml
struct Cli {
    ///Turn debugging information on
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

// TODO:
// random
// edit

#[derive(Debug, Args)]
struct AddArgs {
    /// Task name
    name:        String,
    /// Longer description of task
    #[arg(short, long)]
    description: Option<String>,
    /// Modifier to time bias takes integer 0+
    #[arg(short, long)]
    #[arg(default_value_t = 100)]
    priority:    u16,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Add a new task
    #[command(short_flag = 'a', alias = "a")]
    Add(AddArgs),
    /// Produce a random task, with a bias for older tasks
    #[command(short_flag = 'r', alias = "r")]
    Random,
    /// Edit an existing task
    #[command(short_flag = 'e', alias = "e")]
    Edit(AddArgs),
    /// Update the last played, without closing it
    #[command(short_flag = 't', alias = "t")]
    Touch {
        #[arg(required = true)]
        /// Filter search with provided terms
        terms: Vec<String>,
    },
    /// Complete a round of the task, without closing it
    #[command(short_flag = 'd', alias = "d")]
    Done {
        #[arg(required = true)]
        /// Filter search with provided terms
        terms: Vec<String>,
    },
    /// Complete and close a task
    #[command(
        short_flag = 'c',
        alias = "f",
        visible_alias = "finish",
        visible_alias = "complete",
        visible_short_flag_alias = 'f'
    )]
    Close {
        /// Filter search with provided terms
        #[arg(required = true)]
        terms: Vec<String>,
    },
    /// List pending tasks
    #[command(short_flag = 'l', alias = "l")]
    List {
        /// Filter search with provided terms
        terms: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    let conf = Config::from_path();
    let (cli, conf) = if conf.verbose {
        let cli = dbg!(cli);
        let conf = dbg!(conf);
        (cli, conf)
    } else {
        (cli, conf)
    };
    let command = cli.command.unwrap_or(Commands::Random);
    let command = dbg!(command);
    init_directory(&conf.task_path).unwrap();
    match &command {
        Commands::List { terms } => run_list(&conf, terms),
        Commands::Add(opts) => run_add(&conf, opts),
        Commands::Touch { terms } => run_touch(&conf, terms),
        Commands::Done { terms } => run_done(&conf, terms),
        Commands::Close { terms } => {
            run_complete(&conf, terms);
        },
        _ => {
            panic!("not implemented");
        },
    }
}

fn run_add(conf: &Config, args: &AddArgs) {
    let active = get_active_file(conf);
    let active = dbg!(active);
    let desc = match &args.description {
        Some(desc) => Some(desc.as_str()),
        None => None,
    };
    let task = Task::new_todo(args.name.clone(), desc, Some(args.priority));
    let mut bank = Bank::from_file(&active).expect("unable to read file");
    bank.append(task);
    let bank = dbg!(bank);
    bank.close().unwrap();
    exit(0);
}

fn run_touch(conf: &Config, terms: &[String]) {
    update_item(conf, terms, Task::touched);
}
fn run_done(conf: &Config, terms: &[String]) {
    update_item(conf, terms, Task::touched)
}

fn run_complete(conf: &Config, terms: &[String]) {
    retire_item(conf, terms);
}

fn update_item<F>(conf: &Config, terms: &[String], f: F)
where
    F: FnOnce(&Task) -> Task,
{
    let active = get_active_file(&conf);
    let mut bank = Bank::from_file(&active).expect("Should be able to load bank");
    match fzf(&bank, terms) {
        None => {
            println!("No task selected. Exiting");
            exit(0);
        },
        Some(task) => {
            let task = f(&task);
            let success = bank.update(task);
            if !success {
                println!("Failed to update task - task not found");
                exit(0);
            }
        },
    }
    bank.close().unwrap();
}

fn retire_item(conf: &Config, terms: &[String]) {
    let active = get_active_file(&conf);
    let closed = get_closed_file(&conf);
    let mut bank = Bank::from_file(&active).expect("Should be able to load bank");
    let mut closed_bank = Bank::from_file(&closed).expect("unable to open closed bank");
    match fzf(&bank, terms) {
        None => {
            println!("No task selected. Exiting");
            exit(0);
        },
        Some(task) => {
            let task = task.completed();
            let success = bank.delete(&task.id());
            if !success {
                println!("Failed to update task - task not found");
                exit(0);
            }
            closed_bank.append(task);
            closed_bank.close().unwrap();
            bank.close().unwrap();
        },
    }
}

fn fzf(bank: &Bank, terms: &[String]) -> Option<Task> {
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
    match items.len() {
        0 => return None,
        1 => return items[0].item.clone(),
        len => {
            let len: i8 = len.try_into().unwrap();
            let fzf = FuzzyFinder::find(items, len.clamp(1, 20));
            match fzf {
                Ok(opt) => return opt,
                Err(err) => panic!("{}", err.to_string()),
            };
        },
    }
}

fn run_list(conf: &Config, terms: &Vec<String>) {
    let active = get_active_file(&conf);
    let terms = terms
        .iter()
        .map(|string| string.as_str())
        .collect::<Vec<&str>>();
    let mut items = Bank::from_file(&active)
        .expect("Unable to read active file")
        .into_iter()
        .filter(|task| task.mass_contains(&terms))
        .peekable();
    if items.peek().is_none() {
        println!(
            "{}",
            if terms.len() == 0 {
                "You have no pending tasks!"
            } else {
                "No tasks match your query!"
            }
        );
    } else {
        for task in items {
            println!("{}", task);
        }
    }
    exit(0);
}

fn get_closed_file(conf: &Config) -> PathBuf {
    conf.task_path.join("complete")
}
fn get_active_file(conf: &Config) -> PathBuf {
    conf.task_path.join("active")
}
