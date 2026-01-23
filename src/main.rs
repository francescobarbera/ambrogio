mod chat;
mod config;
mod llm;

use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::fs;

use chat::ChatManager;
use config::Config;
use llm::LlmClient;

#[tokio::main]
async fn main() -> Result<()> {
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
                    println!("Arrivederci!");
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
                println!("Arrivederci!");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("Arrivederci!");
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
