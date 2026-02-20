mod chat;
mod cli;
mod config;
mod hooks;
mod llm;
mod pomodoro;
mod todo;

use anyhow::Result;
use chrono::Local;
use clap::Parser;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::fs;
use std::io::{self, Write};

use chat::ChatManager;
use cli::{Cli, Command, PomodoroAction, ProjectAction, TaskAction};
use config::{Config, FileConfig};
use llm::LlmClient;
use todo::TodoStore;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => run_repl().await,
        Some(Command::Tasks { action }) => run_tasks(action),
        Some(Command::Projects { action }) => run_projects(action),
        Some(Command::Pomodoro { action }) => run_pomodoro(action).await,
        Some(Command::Note { text }) => run_note(&text),
    }
}

fn prompt_selection(prompt: &str, items: &[&str]) -> Result<usize> {
    println!("{}", prompt);
    for (i, item) in items.iter().enumerate() {
        println!("  {}. {}", i + 1, item);
    }

    loop {
        print!("Enter number: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().parse::<usize>() {
            Ok(n) if n >= 1 && n <= items.len() => return Ok(n - 1),
            _ => println!("Please enter a number between 1 and {}", items.len()),
        }
    }
}

async fn run_repl() -> Result<()> {
    let config = Config::from_env()?;

    let organiser_content = fs::read_to_string(&config.file_path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to read organiser file '{}': {}",
            config.file_path,
            e
        )
    })?;

    let client = LlmClient::new(config.clone());
    let mut chat = ChatManager::new(client, &organiser_content);

    println!("Ambrogio - Your daily organiser assistant");
    println!("Type 'quit' or 'exit' to leave\n");

    let mut rl = DefaultEditor::new()?;

    loop {
        let readline = rl.readline("you: ");
        match readline {
            Ok(line) => {
                let input = line.trim();

                if input.is_empty() {
                    continue;
                }

                if matches!(input.to_lowercase().as_str(), "quit" | "exit" | "q") {
                    println!("Goodbye!");
                    break;
                }

                let _ = rl.add_history_entry(input);

                match chat.send(input).await {
                    Ok(response) => {
                        println!("\nambrogio: {}\n", response);
                    }
                    Err(e) => {
                        eprintln!("\nError: {}\n", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Goodbye!");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("Goodbye!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

fn run_projects(action: ProjectAction) -> Result<()> {
    let file_config = FileConfig::from_env()?;
    let store = TodoStore::new(file_config.todos_path);

    match action {
        ProjectAction::List => {
            let projects = store.projects()?;
            if projects.is_empty() {
                println!("No projects.");
            } else {
                for (i, project) in projects.iter().enumerate() {
                    println!("  {}. {}", i + 1, project);
                }
            }
        }
        ProjectAction::Add { name } => {
            store.add_project(&name)?;
            println!("Added project: {}", name);
        }
        ProjectAction::Delete => {
            let projects = store.projects()?;
            if projects.is_empty() {
                println!("No projects to delete.");
                return Ok(());
            }

            let items: Vec<&str> = projects.iter().map(|p| p.as_str()).collect();
            let selection = prompt_selection("Select a project to delete:", &items)?;

            print!(
                "Delete '{}' and all its todos? (y/N): ",
                projects[selection]
            );
            io::stdout().flush()?;

            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm)?;

            if confirm.trim().eq_ignore_ascii_case("y") {
                store.delete_project(&projects[selection])?;
                println!("Deleted project: {}", projects[selection]);
            } else {
                println!("Cancelled.");
            }
        }
    }

    Ok(())
}

fn run_tasks(action: TaskAction) -> Result<()> {
    let file_config = FileConfig::from_env()?;
    let store = TodoStore::new(file_config.todos_path);

    match action {
        TaskAction::Add { description } => {
            let projects = store.projects()?;
            if projects.is_empty() {
                println!("No projects. Add a project first with: ambrogio projects add <name>");
                return Ok(());
            }

            let items: Vec<&str> = projects.iter().map(|p| p.as_str()).collect();
            let selection = prompt_selection("Select a project:", &items)?;

            store.add(&projects[selection], &description)?;
            println!("Added to {}: {}", projects[selection], description);
        }
        TaskAction::List => {
            store.print_open_todos()?;
        }
        TaskAction::Complete => {
            let open = store.open_todos()?;
            if open.is_empty() {
                println!("No open tasks to complete.");
                return Ok(());
            }

            print_open_todos_for_selection("Select a task to complete:", &open);
            let selection = read_todo_number(open.len())?;

            store.complete(selection)?;
            println!("Completed: {}", open[selection].description);
        }
        TaskAction::Delete => {
            let open = store.open_todos()?;
            if open.is_empty() {
                println!("No open tasks to delete.");
                return Ok(());
            }

            print_open_todos_for_selection("Select a task to delete:", &open);
            let selection = read_todo_number(open.len())?;

            store.delete(selection)?;
            println!("Deleted: {}", open[selection].description);
        }
    }

    Ok(())
}

fn run_note(text: &str) -> Result<()> {
    let file_config = FileConfig::from_env()?;
    let store = TodoStore::new(file_config.todos_path);
    let open = store.open_todos()?;

    if open.is_empty() {
        println!("No open tasks. Add a task first with: ambrogio tasks add <name>");
        return Ok(());
    }

    print_open_todos_for_selection("Select a task:", &open);
    let selection = read_todo_number(open.len())?;

    store.add_note(selection, text)?;
    println!("Added note to: {}", open[selection].description);

    Ok(())
}

fn print_open_todos_for_selection(header: &str, todos: &[todo::Todo]) {
    println!("{}", header);
    let mut current_project = "";
    for (i, todo) in todos.iter().enumerate() {
        if todo.project != current_project {
            current_project = &todo.project;
            println!("\n  ## {}", current_project);
        }
        println!("  {}. {}", i + 1, todo.description);
    }
}

fn read_todo_number(count: usize) -> Result<usize> {
    loop {
        print!("Enter number: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().parse::<usize>() {
            Ok(n) if n >= 1 && n <= count => return Ok(n - 1),
            _ => println!("Please enter a number between 1 and {}", count),
        }
    }
}

async fn run_pomodoro(action: PomodoroAction) -> Result<()> {
    match action {
        PomodoroAction::Start => {
            let file_config = FileConfig::from_env()?;
            let store = TodoStore::new(file_config.todos_path);
            let open = store.open_todos()?;

            if open.is_empty() {
                println!("No open tasks. Add a task first with: ambrogio tasks add <name>");
                return Ok(());
            }

            print_open_todos_for_selection("Select a task to focus on:", &open);
            let selection = read_todo_number(open.len())?;

            let started_at = Local::now().naive_local();
            let outcome = pomodoro::run(&open[selection].description).await?;
            let cancelled = outcome == pomodoro::Outcome::Cancelled;

            store.add_pomodoro(selection, started_at, cancelled)?;

            if outcome == pomodoro::Outcome::Completed {
                hooks::run("pomodoro", "stop")?;
            }
        }
    }

    Ok(())
}
