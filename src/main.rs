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
use cli::{Cli, Command, PomodoroAction, TodoAction};
use config::{Config, FileConfig};
use llm::LlmClient;
use todo::TodoStore;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => run_repl().await,
        Some(Command::Todos { action }) => run_todos(action),
        Some(Command::Pomodoro { action }) => run_pomodoro(action).await,
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

fn run_todos(action: TodoAction) -> Result<()> {
    let file_config = FileConfig::from_env()?;
    let store = TodoStore::new(file_config.todos_path);

    match action {
        TodoAction::Add { description } => {
            store.add(&description)?;
            println!("Added: {}", description);
        }
        TodoAction::List => {
            store.print_open_todos()?;
        }
        TodoAction::Complete => {
            let open = store.open_todos()?;
            if open.is_empty() {
                println!("No open todos to complete.");
                return Ok(());
            }

            let items: Vec<&str> = open.iter().map(|t| t.description.as_str()).collect();
            let selection = prompt_selection("Select a todo to complete:", &items)?;

            store.complete(selection)?;
            println!("Completed: {}", open[selection].description);
        }
    }

    Ok(())
}

async fn run_pomodoro(action: PomodoroAction) -> Result<()> {
    match action {
        PomodoroAction::Start => {
            let file_config = FileConfig::from_env()?;
            let store = TodoStore::new(file_config.todos_path);
            let open = store.open_todos()?;

            if open.is_empty() {
                println!("No open todos. Add a todo first.");
                return Ok(());
            }

            let items: Vec<&str> = open.iter().map(|t| t.description.as_str()).collect();
            let selection = prompt_selection("Select a todo to focus on:", &items)?;

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
