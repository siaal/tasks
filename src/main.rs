// TODO:
// change config so that cutoff accepts strings
// implement undo file
// implement proj files
use std::error::Error;
use std::process::exit;

use clap::Parser;
use tasks::parser::{AddArgs, Cli, Commands, EditArgs, ListArgs};
use tasks::store::{init_store, Store};
use tasks::task::Task;
use tasks::Config;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let mut conf = Config::from_path();
    if !conf.debug && cli.debug {
        conf.debug = cli.debug;
    }
    if conf.debug {
        dbg!(&cli.command);
        println!(
            "The path where tasks are stored: {}",
            &conf.task_path.to_str().ok_or("")?
        );
        dbg!(&conf);
    }
    if cli.force || cli.ntags.is_some() || cli.tags.is_some() {
        conf.cutoff = 0;
        run_random(&conf, 1, cli.tags, cli.ntags);
        exit(0);
    }
    let command = cli.command.unwrap_or(Commands::Random {
        n:     1,
        tags:  None,
        ntags: None,
        force: false,
    });
    if conf.debug {
        dbg!(&command);
    };
    init_store(&conf.task_path)?;
    match &command {
        Commands::List(args) => run_list(&conf, args),
        Commands::Last => run_list(
            &conf,
            &ListArgs {
                tags:  None,
                ntags: None,
                terms: ["last".into()].to_vec(),
            },
        ),
        Commands::Add(opts) => run_add(&conf, opts),
        Commands::Done { terms } => run_touch(&conf, terms),
        Commands::Close { terms } => run_complete(&conf, terms),
        Commands::Random {
            tags,
            ntags,
            n,
            force,
        } => {
            if *force {
                conf.cutoff = 0;
            }
            run_random(&conf, *n, tags.clone(), ntags.clone());
        },
        Commands::Edit(args) => run_edit(&conf, args),
        Commands::Undo => run_undo(&conf),
    };
    Ok(())
}

fn run_undo(conf: &Config) {
    let store = Store::new(conf.task_path.clone());
    match store.undo() {
        Ok(item) => println!("Undone operation:\n{}", item),
        Err(error) => println!("Error in undo: {}", error.to_string()),
    }
}

fn run_random(conf: &Config, n: u8, tags: Option<Vec<String>>, ntags: Option<Vec<String>>) {
    let store = Store::new(conf.task_path.clone());
    let items = store.filter_active(
        &vec![],
        &tags.unwrap_or_default(),
        &ntags.unwrap_or_default(),
    );
    let items = store.select_random_from_list(&items, n, conf.cutoff.clone());

    print_tasks(&items);
    if conf.debug {
        dbg!(&items.len());
    }
}

fn run_add(conf: &Config, args: &AddArgs) {
    let store = Store::new(conf.task_path.clone());
    let desc = match &args.description {
        Some(desc) => Some(desc.as_str()),
        None => None,
    };

    let task = Task::new_todo(
        args.name.join(" "),
        desc,
        Some(args.priority),
        Some(args.tag.to_owned()),
    );
    match store.append(task) {
        Ok(task) => {
            println!("Appended task:");
            print_task(&task);
        },
        Err(err) => println!("Could not add task. {}", err.to_string()),
    }
}

fn run_touch(conf: &Config, terms: &[String]) {
    let store = Store::new(conf.task_path.clone());
    let found = store.fzf(terms);
    match found {
        None => {
            println!("Could not find task!");
            exit(1);
        },
        Some(task) => {
            println!("Editing:");
            print_task(&task);
            let task = update_item(store, task, conf, Task::touched);
            println!("Touched: `{}`", task.name());
        },
    }
}

fn run_complete(conf: &Config, terms: &[String]) {
    let retired = retire_item(conf, terms);
    println!("Now retired:");
    print_task(&retired);
}

fn run_edit(conf: &Config, args: &EditArgs) {
    let store = Store::new(conf.task_path.clone());
    let found = store.fzf(args.identifier.as_ref());
    match found {
        None => {
            println!("Could not find task!");
            exit(1);
        },
        Some(task) => {
            println!("Editing:");
            print_task(&task);
            print!("vvvvvvvvvvvv HAS BECOME vvvvvvvvvvvv\n");
            let out = update_item(store, task, conf, |task| {
                let mut task = task.to_owned();
                if let Some(addtags) = &args.atag {
                    task = task.add_tags(addtags.to_vec());
                }
                if let Some(rmtags) = &args.rtag {
                    task = task.remove_tags(rmtags);
                }
                if let Some(settags) = &args.stag {
                    task = task.set_tags(settags.to_vec());
                }
                task.updated_todo(
                    args.description.as_deref(),
                    args.priority.as_ref(),
                    args.name.as_deref(),
                )
            });
            print_task(&out);
        },
    }
}

fn update_item<F>(store: Store, task: Task, conf: &Config, f: F) -> Task
where
    F: FnOnce(&Task) -> Task,
{
    if conf.debug {
        dbg!(&task);
    }
    let result = store.update_item(task, f);
    if conf.debug {
        dbg!(&result);
    }
    match result {
        Err(error) => {
            println!("Failed to update item, {}", error.to_string());
            exit(1);
        },
        Ok(task) => return task,
    }
}

fn retire_item(conf: &Config, terms: &[String]) -> Task {
    let store = Store::new(conf.task_path.clone());
    let item = store.fzf(terms);
    match item {
        None => {
            println!("No task selected. Exiting");
            exit(0);
        },
        Some(task) => {
            let result = store.retire_item(&task);
            match result {
                Err(err) => {
                    println!("{}", err.to_string());
                    exit(1);
                },
                Ok(task) => {
                    return task;
                },
            }
        },
    }
}

fn run_list(conf: &Config, args: &ListArgs) {
    let store = Store::new(conf.task_path.clone());
    let items: Vec<Task> = store
        .filter_active(
            &args.terms,
            &args.tags.clone().unwrap_or_default(),
            &args.ntags.clone().unwrap_or_default(),
        )
        .into_iter()
        .collect();
    if items.len() == 0 {
        println!(
            "{}",
            if args.terms.len() == 0 && args.tags.is_none() && args.ntags.is_none() {
                "You have no pending tasks!"
            } else {
                "No tasks match your query!"
            }
        );
    } else {
        print_tasks(&items);
    }
    exit(0);
}

fn print_task(task: &Task) {
    println!("{}", task)
}

fn print_tasks(tasks: &[Task]) {
    if tasks.len() == 0 {
        return;
    }
    println!("{}", tasks[0]);
    for task in &tasks[1..] {
        println!("\n{}", task)
    }
}
