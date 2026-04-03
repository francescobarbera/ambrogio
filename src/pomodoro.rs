use anyhow::Result;
use std::io::{self, Write};
use std::time::Duration;

const POMODORO_DURATION: Duration = Duration::from_secs(25 * 60);
const BREAK_DURATION: Duration = Duration::from_secs(5 * 60);

#[derive(Debug, PartialEq)]
pub enum Outcome {
    Completed,
    Cancelled,
}

pub fn format_countdown(remaining: Duration) -> String {
    let total_secs = remaining.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

pub async fn run(description: &str) -> Result<Outcome> {
    run_timer(POMODORO_DURATION, "🍅", description).await
}

pub async fn run_break() -> Result<Outcome> {
    run_timer(BREAK_DURATION, "☕", "Break").await
}

async fn run_timer(duration: Duration, emoji: &str, description: &str) -> Result<Outcome> {
    println!("Starting {}: {}", emoji, description);
    println!("Press Ctrl+C to cancel\n");

    let mut remaining = duration;

    loop {
        let countdown = format_countdown(remaining);
        print!(
            "\x1b]0;{} {} - {}\x07\r\x1b[K  {} - {}",
            emoji, countdown, description, countdown, description
        );
        io::stdout().flush()?;

        if remaining.is_zero() {
            break;
        }

        let tick = tokio::time::sleep(Duration::from_secs(1));
        tokio::select! {
            _ = tick => {
                remaining = remaining.saturating_sub(Duration::from_secs(1));
            }
            _ = tokio::signal::ctrl_c() => {
                print!("\x1b]0;\x07");
                println!("\n\nCancelled.");
                return Ok(Outcome::Cancelled);
            }
        }
    }

    print!("\x1b]0;\x07");
    print!("\x07");
    println!("\n\nDone!");

    Ok(Outcome::Completed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_countdown_full_duration() {
        assert_eq!(format_countdown(Duration::from_secs(25 * 60)), "25:00");
    }

    #[test]
    fn format_countdown_partial() {
        assert_eq!(format_countdown(Duration::from_secs(14 * 60 + 31)), "14:31");
    }

    #[test]
    fn format_countdown_zero() {
        assert_eq!(format_countdown(Duration::from_secs(0)), "00:00");
    }

    #[test]
    fn format_countdown_under_one_minute() {
        assert_eq!(format_countdown(Duration::from_secs(5)), "00:05");
    }

    #[test]
    fn pomodoro_duration_is_25_minutes() {
        assert_eq!(POMODORO_DURATION, Duration::from_secs(25 * 60));
    }

    #[test]
    fn break_duration_is_5_minutes() {
        assert_eq!(BREAK_DURATION, Duration::from_secs(5 * 60));
    }
}
