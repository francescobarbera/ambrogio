use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ambrogio", about = "Your daily organiser assistant")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Manage your task list
    #[command(visible_alias = "t")]
    Tasks {
        #[command(subcommand)]
        action: TaskAction,
    },
    /// Manage projects
    #[command(visible_alias = "p")]
    Projects {
        #[command(subcommand)]
        action: ProjectAction,
    },
    /// Pomodoro focus sessions
    #[command(visible_alias = "pom")]
    Pomodoro {
        #[command(subcommand)]
        action: PomodoroAction,
    },
    /// Add a note to a task
    #[command(visible_alias = "n")]
    Note {
        /// The note text
        text: String,
    },
}

#[derive(Subcommand)]
pub enum TaskAction {
    /// Add a new task
    #[command(visible_alias = "a")]
    Add {
        /// The task description
        description: String,
    },
    /// List open tasks
    #[command(visible_alias = "l")]
    List,
    /// Mark a task as complete
    #[command(visible_alias = "c")]
    Complete,
    /// Delete a task
    #[command(visible_alias = "d")]
    Delete,
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
    #[command(visible_alias = "s")]
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
    fn parses_tasks_add() {
        let cli = Cli::parse_from(["ambrogio", "tasks", "add", "buy milk"]);
        match cli.command {
            Some(Command::Tasks {
                action: TaskAction::Add { description },
            }) => assert_eq!(description, "buy milk"),
            _ => panic!("expected Tasks Add"),
        }
    }

    #[test]
    fn parses_tasks_list() {
        let cli = Cli::parse_from(["ambrogio", "tasks", "list"]);
        assert!(matches!(
            cli.command,
            Some(Command::Tasks {
                action: TaskAction::List
            })
        ));
    }

    #[test]
    fn parses_tasks_complete() {
        let cli = Cli::parse_from(["ambrogio", "tasks", "complete"]);
        assert!(matches!(
            cli.command,
            Some(Command::Tasks {
                action: TaskAction::Complete
            })
        ));
    }

    #[test]
    fn parses_tasks_delete() {
        let cli = Cli::parse_from(["ambrogio", "tasks", "delete"]);
        assert!(matches!(
            cli.command,
            Some(Command::Tasks {
                action: TaskAction::Delete
            })
        ));
    }

    #[test]
    fn parses_note() {
        let cli = Cli::parse_from(["ambrogio", "note", "some note text"]);
        match cli.command {
            Some(Command::Note { text }) => assert_eq!(text, "some note text"),
            _ => panic!("expected Note"),
        }
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

    #[test]
    fn alias_t_l_parses_as_tasks_list() {
        let cli = Cli::parse_from(["ambrogio", "t", "l"]);
        assert!(matches!(
            cli.command,
            Some(Command::Tasks {
                action: TaskAction::List
            })
        ));
    }

    #[test]
    fn alias_t_a_parses_as_tasks_add() {
        let cli = Cli::parse_from(["ambrogio", "t", "a", "buy milk"]);
        match cli.command {
            Some(Command::Tasks {
                action: TaskAction::Add { description },
            }) => assert_eq!(description, "buy milk"),
            _ => panic!("expected Tasks Add via alias"),
        }
    }

    #[test]
    fn alias_t_c_parses_as_tasks_complete() {
        let cli = Cli::parse_from(["ambrogio", "t", "c"]);
        assert!(matches!(
            cli.command,
            Some(Command::Tasks {
                action: TaskAction::Complete
            })
        ));
    }

    #[test]
    fn alias_t_d_parses_as_tasks_delete() {
        let cli = Cli::parse_from(["ambrogio", "t", "d"]);
        assert!(matches!(
            cli.command,
            Some(Command::Tasks {
                action: TaskAction::Delete
            })
        ));
    }

    #[test]
    fn alias_n_parses_as_note() {
        let cli = Cli::parse_from(["ambrogio", "n", "a note"]);
        match cli.command {
            Some(Command::Note { text }) => assert_eq!(text, "a note"),
            _ => panic!("expected Note via alias"),
        }
    }

    #[test]
    fn alias_p_parses_as_projects() {
        let cli = Cli::parse_from(["ambrogio", "p", "list"]);
        assert!(matches!(
            cli.command,
            Some(Command::Projects {
                action: ProjectAction::List
            })
        ));
    }

    #[test]
    fn alias_pom_s_parses_as_pomodoro_start() {
        let cli = Cli::parse_from(["ambrogio", "pom", "s"]);
        assert!(matches!(
            cli.command,
            Some(Command::Pomodoro {
                action: PomodoroAction::Start
            })
        ));
    }
}
