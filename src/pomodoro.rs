use anyhow::Result;
use std::io::{self, Write};
use std::time::Duration;

const POMODORO_DURATION: Duration = Duration::from_secs(25 * 60);

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
    println!("Starting pomodoro: {}", description);
    println!("Press Ctrl+C to cancel\n");

    let mut remaining = POMODORO_DURATION;

    loop {
        let countdown = format_countdown(remaining);
        print!(
            "\x1b]0;🍅 {} - {}\x07\r\x1b[K  {} - {}",
            countdown, description, countdown, description
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
                println!("\n\nPomodoro cancelled.");
                return Ok(Outcome::Cancelled);
            }
        }
    }

    print!("\x1b]0;\x07");
    print!("\x07");
    println!("\n\nPomodoro complete!");

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
}
