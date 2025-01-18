# Tasks

A command line TODO application designed to deliver you a random task with a bias towards forgotten tasks.

`tasks` remembers the last time each task was completed, and will scale each tasks chance of being picked based on how long it's been, recent tasks are *less* likely than older tasks. 

The intention is that `tasks` can help juggle multiple tasks, by keeping you engaged with older tasks before they become forgotten tasks.

Additionally, each task has a `priority` which is another multiplier that increases its chances of being picked.

## Installation

To install run `cargo install --path .` from the project directory.

To configure tasks, a `$HOME/.config/tasks/tasks.toml` file can be created with the settings
`task_path`: string => a directory that the database files are stored in
`debug`: bool => Makes `tasks` run verbosely
`cutoff`: int => minimum amount of time a task must have been waiting in minutes to have been considered

## Usage

`tasks add <Task>` is used to create a new task. Once added, a task will exist until it is (`close`|`finish`|`complete`|`retire`)d, this is not be be confused with `touch`|`done`, which will only update the `last completed` time.

`tasks last` will view the last task that was listed for any reason.

`tasks edit` allows you to edit a task. Task identifiers can be
- `last` to edit the task displayed by `task last`
- `task_id` to edit the task by its unique identifier
- `string` to fuzzy find through a list of results that match the `grep` result for `string`

`tasks done` will mark a task as performed, but not completed. Uses the selection mechanism from `tasks edit`

`tasks close` will mark a task as completed. A completed task will not be suggested. Uses the selection mechanism from `tasks edit`

`tasks list` will list tasks. Can be filtered by tags

`tasks help` will display the help menu. Note that all subcommands have their own helpful help menu!

### Putting it in your `.bashrc`

I like to run tasks automatically by putting `tasks` in my `.bashrc`, so that it provides a random* reminder every time I open the terminal.

## TODO

Currently `tasks` uses regular text files as a "database". This works honestly quite well and doesn't really need to be improved, but the responsible thing to do would be to port to using SQLite.
