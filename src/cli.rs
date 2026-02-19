use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ambrogio", about = "Your daily organiser assistant")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Manage your todo list
    Todos {
        #[command(subcommand)]
        action: TodoAction,
    },
    /// Manage projects
    Projects {
        #[command(subcommand)]
        action: ProjectAction,
    },
    /// Pomodoro focus sessions
    Pomodoro {
        #[command(subcommand)]
        action: PomodoroAction,
    },
}

#[derive(Subcommand)]
pub enum TodoAction {
    /// Add a new todo
    Add {
        /// The task description
        description: String,
    },
    /// List open todos
    List,
    /// Mark a todo as complete
    Complete,
}

#[derive(Subcommand)]
pub enum ProjectAction {
    /// List all projects
    List,
    /// Add a new project
    Add {
        /// The project name
        name: String,
    },
    /// Delete a project and all its todos
    Delete,
}

#[derive(Subcommand)]
pub enum PomodoroAction {
    /// Start a 25-minute pomodoro timer
    Start,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn no_args_returns_none_command() {
        let cli = Cli::parse_from(["ambrogio"]);
        assert!(cli.command.is_none());
    }

    #[test]
    fn parses_todos_add() {
        let cli = Cli::parse_from(["ambrogio", "todos", "add", "buy milk"]);
        match cli.command {
            Some(Command::Todos {
                action: TodoAction::Add { description },
            }) => assert_eq!(description, "buy milk"),
            _ => panic!("expected Todos Add"),
        }
    }

    #[test]
    fn parses_todos_list() {
        let cli = Cli::parse_from(["ambrogio", "todos", "list"]);
        assert!(matches!(
            cli.command,
            Some(Command::Todos {
                action: TodoAction::List
            })
        ));
    }

    #[test]
    fn parses_todos_complete() {
        let cli = Cli::parse_from(["ambrogio", "todos", "complete"]);
        assert!(matches!(
            cli.command,
            Some(Command::Todos {
                action: TodoAction::Complete
            })
        ));
    }

    #[test]
    fn parses_projects_list() {
        let cli = Cli::parse_from(["ambrogio", "projects", "list"]);
        assert!(matches!(
            cli.command,
            Some(Command::Projects {
                action: ProjectAction::List
            })
        ));
    }

    #[test]
    fn parses_projects_add() {
        let cli = Cli::parse_from(["ambrogio", "projects", "add", "Work"]);
        match cli.command {
            Some(Command::Projects {
                action: ProjectAction::Add { name },
            }) => assert_eq!(name, "Work"),
            _ => panic!("expected Projects Add"),
        }
    }

    #[test]
    fn parses_projects_delete() {
        let cli = Cli::parse_from(["ambrogio", "projects", "delete"]);
        assert!(matches!(
            cli.command,
            Some(Command::Projects {
                action: ProjectAction::Delete
            })
        ));
    }

    #[test]
    fn parses_pomodoro_start() {
        let cli = Cli::parse_from(["ambrogio", "pomodoro", "start"]);
        assert!(matches!(
            cli.command,
            Some(Command::Pomodoro {
                action: PomodoroAction::Start
            })
        ));
    }
}
