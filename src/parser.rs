use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
#[command(version, about, long_about=None)] // version|about filled in from cargo.toml
pub struct Cli {
    ///Turn debugging information on
    #[arg(short, long)]
    pub debug: bool,

    /// Runs command as tasks random -f 1
    #[arg(short, long)]
    pub force: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Args, Deserialize, Serialize, Clone)]
pub struct AddArgs {
    /// Longer description of task
    #[arg(short, long)]
    pub description: Option<String>,
    /// Modifier to time bias takes integer 0+
    #[arg(short, long)]
    #[arg(default_value_t = 100)]
    pub priority:    u16,
    /// Task name
    pub name:        Vec<String>,
}

#[derive(Debug, Args, Deserialize, Serialize, Clone)]
pub struct EditArgs {
    /// New Name
    #[arg(short, long)]
    pub name:        Option<String>,
    /// New Description
    #[arg(short, long)]
    pub description: Option<String>,
    /// New Priority
    #[arg(short, long)]
    pub priority:    Option<u16>,
    /// Identifier string
    #[arg(required = true)]
    pub identifier:  Vec<String>,
}

#[derive(Debug, Subcommand, Deserialize, Serialize, Clone)]
pub enum Commands {
    /// Add a new task
    #[command(alias = "a")]
    Add(AddArgs),
    /// Prints the last viewed task
    Last,
    /// Produce a random task, with a bias for older tasks
    #[command(alias = "r")]
    Random {
        /// Sets random selection cutoff to 0 so that newly minted tasks can be selected
        #[arg(short, long)]
        force: bool,
        #[arg(default_value_t = 1)]
        n:     u8,
    },
    /// Edit an existing task
    #[command(alias = "e")]
    Edit(EditArgs),
    /// Update the last played, without closing it
    #[command(alias = "t")]
    Touch {
        #[arg(required = true)]
        /// Filter search with provided terms
        terms: Vec<String>,
    },
    /// Complete a round of the task, without closing it
    #[command(alias = "d")]
    Done {
        #[arg(required = true)]
        /// Filter search with provided terms
        terms: Vec<String>,
    },
    /// Complete and close a task
    #[command(
        alias = "f",
        visible_alias = "finish",
        visible_alias = "complete",
        visible_alias = "retire"
    )]
    Close {
        /// Filter search with provided terms
        #[arg(required = true)]
        terms: Vec<String>,
    },
    /// List pending tasks
    #[command(alias = "l")]
    List {
        /// Filter search with provided terms
        terms: Vec<String>,
    },
    /// Reverts the previous entry in the undo list (that changed bank state)
    #[command(alias = "u")]
    Undo,
}
