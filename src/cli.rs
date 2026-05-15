use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ambrogio", version, about = "Your daily organiser assistant")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Manage today's task list
    #[command(visible_alias = "t")]
    Tasks {
        #[command(subcommand)]
        action: TaskAction,
    },
    /// Pomodoro focus sessions
    #[command(visible_alias = "pom")]
    Pomodoro {
        #[command(subcommand)]
        action: PomodoroAction,
    },
}

#[derive(Subcommand)]
pub enum TaskAction {
    /// Add a new task to today's list
    #[command(visible_alias = "a")]
    Add {
        /// The task description
        description: String,
    },
    /// List today's open tasks
    #[command(visible_alias = "l")]
    List,
    /// Mark one of today's tasks as complete
    #[command(visible_alias = "c")]
    Complete,
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
    fn alias_pom_s_parses_as_pomodoro_start() {
        let cli = Cli::parse_from(["ambrogio", "pom", "s"]);
        assert!(matches!(
            cli.command,
            Some(Command::Pomodoro {
                action: PomodoroAction::Start
            })
        ));
    }

    #[test]
    fn rejects_removed_tasks_delete() {
        assert!(Cli::try_parse_from(["ambrogio", "tasks", "delete"]).is_err());
        assert!(Cli::try_parse_from(["ambrogio", "t", "d"]).is_err());
    }

    #[test]
    fn rejects_removed_projects_command() {
        assert!(Cli::try_parse_from(["ambrogio", "projects", "list"]).is_err());
        assert!(Cli::try_parse_from(["ambrogio", "p", "list"]).is_err());
    }

    #[test]
    fn rejects_removed_note_command() {
        assert!(Cli::try_parse_from(["ambrogio", "note", "some text"]).is_err());
        assert!(Cli::try_parse_from(["ambrogio", "n", "some text"]).is_err());
    }
}
