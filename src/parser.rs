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
    /// Must include tags - forces mode to RANDOM
    #[arg(short, long, value_delimiter = ',')]
    pub tags:  Option<Vec<String>>,
    /// Must NOT include tags - forces mode to RANDOM
    #[arg(short, long, value_delimiter = ',')]
    pub ntags: Option<Vec<String>>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Args, Deserialize, Serialize, Clone)]
pub struct AddArgs {
    /// Longer description of task
    #[arg(short, long, alias = "desc")]
    pub description: Option<String>,
    /// Modifier to time bias takes integer 0+
    #[arg(short, long)]
    #[arg(default_value_t = 100)]
    pub priority:    u16,
    /// Task name
    pub name:        Vec<String>,
    /// Tags for the task
    #[arg(short, long, value_delimiter = ',')]
    pub tag:         Vec<String>,
}
// TODO:
// change tag so that you can `--tag foo` and `-tag foo`
// change list and random so that you can filter by/not tag

#[derive(Debug, Args, Deserialize, Serialize, Clone)]
pub struct EditArgs {
    /// New Name
    #[arg(short, long)]
    pub name:        Option<String>,
    /// New Description
    #[arg(short, long, alias = "desc")]
    pub description: Option<String>,
    /// New Priority
    #[arg(short, long)]
    pub priority:    Option<u16>,
    /// Removes any instance of tag from the chosen Todo
    #[arg(short, long, value_delimiter = ',')]
    pub rtag:        Option<Vec<String>>,
    /// Adds all tags to the chosen Todo
    #[arg(short, long, value_delimiter = ',')]
    pub atag:        Option<Vec<String>>,
    /// Sets the tags of the chosen todo
    #[arg(short, long, value_delimiter = ',')]
    pub stag:        Option<Vec<String>>,
    /// Identifier string
    #[arg(required = true)]
    pub identifier:  Vec<String>,
}

#[derive(Debug, Args, Deserialize, Serialize, Clone)]
pub struct ListArgs {
    /// Must include tags
    #[arg(short, long, value_delimiter = ',')]
    pub tags:  Option<Vec<String>>,
    /// Must NOT include tags
    #[arg(short, long, value_delimiter = ',')]
    pub ntags: Option<Vec<String>>,
    /// Filter search with provided terms
    pub terms: Vec<String>,
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
        /// Must include tags
        #[arg(short, long, value_delimiter = ',')]
        tags:  Option<Vec<String>>,
        /// Must NOT include tags
        #[arg(short, long, value_delimiter = ',')]
        ntags: Option<Vec<String>>,
        /// Sets random selection cutoff to 0 so that newly minted tasks can be selected
        #[arg(short, long)]
        force: bool,
        #[arg(default_value_t = 1)]
        n:     u8,
    },
    /// Edit an existing task
    #[command(alias = "e")]
    Edit(EditArgs),
    /// Complete a round of the task, without closing it
    #[command(alias = "d", visible_alias = "touch", alias = "t")]
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
    List(ListArgs),
    /// Reverts the previous entry in the undo list (that changed bank state)
    #[command(alias = "u")]
    Undo,
}
