mod chat;
mod cli;
mod config;
mod daily;
mod hooks;
mod llm;
mod pomodoro;

use anyhow::Result;
use chrono::Local;
use clap::Parser;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::fs;
use std::io::{self, Write};

use chat::ChatManager;
use cli::{Cli, Command, PomodoroAction, TaskAction};
use config::Config;
use llm::LlmClient;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => run_repl().await,
        Some(Command::Tasks { action }) => run_tasks(action),
        Some(Command::Pomodoro { action }) => run_pomodoro(action).await,
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
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
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

fn today() -> chrono::NaiveDate {
    Local::now().date_naive()
}

fn print_open_tasks(tasks: &[String]) {
    for (i, task) in tasks.iter().enumerate() {
        println!("  {}. {}", i + 1, task);
    }
}

fn read_task_number(count: usize) -> Result<usize> {
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

fn select_today_task(path: &std::path::Path, header: &str) -> Result<Option<(usize, String)>> {
    let open = daily::today_open(path, today())?;
    if open.is_empty() {
        return Ok(None);
    }

    println!("{}", header);
    print_open_tasks(&open);
    let selection = read_task_number(open.len())?;
    Ok(Some((selection, open[selection].clone())))
}

fn run_tasks(action: TaskAction) -> Result<()> {
    let path = config::organiser_path_from_env()?;

    match action {
        TaskAction::Add { description } => {
            daily::add(&path, today(), &description)?;
            println!("Added: {}", description);
        }
        TaskAction::List => {
            let open = daily::today_open(&path, today())?;
            if open.is_empty() {
                println!("No tasks for today.");
            } else {
                print_open_tasks(&open);
            }
        }
        TaskAction::Complete => match select_today_task(&path, "Select a task to complete:")? {
            None => println!("No tasks for today."),
            Some((index, description)) => {
                daily::complete(&path, today(), index)?;
                println!("Completed: {}", description);
            }
        },
    }

    Ok(())
}

async fn run_pomodoro(action: PomodoroAction) -> Result<()> {
    match action {
        PomodoroAction::Start => {
            let path = config::organiser_path_from_env()?;

            let Some((mut selection, mut description)) =
                select_today_task(&path, "Select a task to focus on:")?
            else {
                println!("No tasks for today.");
                return Ok(());
            };

            loop {
                let outcome = pomodoro::run(&description).await?;
                if outcome == pomodoro::Outcome::Cancelled {
                    break;
                }

                daily::add_pomodoro(&path, today(), selection)?;
                hooks::run("pomodoro", "stop")?;

                let break_outcome = pomodoro::run_break().await?;
                if break_outcome == pomodoro::Outcome::Cancelled {
                    break;
                }

                hooks::run("break", "stop")?;

                match select_today_task(&path, "Select a task to focus on:")? {
                    None => {
                        println!("No tasks for today.");
                        break;
                    }
                    Some((next_idx, next_desc)) => {
                        selection = next_idx;
                        description = next_desc;
                    }
                }
            }
        }
    }

    Ok(())
}
