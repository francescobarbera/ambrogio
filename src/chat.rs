use anyhow::Result;
use chrono::Local;

use crate::llm::{LlmClient, Message};

pub struct ChatManager {
    client: LlmClient,
    system_prompt: String,
    history: Vec<Message>,
}

fn build_system_prompt(today: &str, organiser_content: &str) -> String {
    format!(
        r#"You are Ambrogio, a personal assistant that helps the user understand their schedule and tasks.

You have access to the user's daily organiser. The format is:
- Dates are marked with `# YYYY-MM-DD`
- Scheduled items: `**HH:MM** description`
- Open tasks are marked with [TODO]
- Completed tasks are marked with [DONE]

Today's date is: {today}

---
{organiser_content}
---

Answer questions about the schedule concisely. When listing items, use bullet points.
If asked about "tomorrow", calculate the date based on today.
If asked about "this week", consider the 7 days starting from today."#
    )
}

impl ChatManager {
    pub fn new(client: LlmClient, organiser_content: &str) -> Self {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let system_prompt = build_system_prompt(&today, organiser_content);

        Self {
            client,
            system_prompt,
            history: Vec::new(),
        }
    }

    pub async fn send(&mut self, user_input: &str) -> Result<String> {
        let mut messages = vec![Message::system(&self.system_prompt)];
        messages.extend(self.history.clone());
        messages.push(Message::user(user_input));

        // History is only updated after a successful response to avoid
        // orphaned messages when the API call fails
        let response = self.client.chat(&messages).await?;

        self.history.push(Message::user(user_input));
        self.history.push(Message::assistant(&response));

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_prompt_contains_date() {
        let prompt = build_system_prompt("2026-01-23", "sample content");
        assert!(prompt.contains("Today's date is: 2026-01-23"));
    }

    #[test]
    fn system_prompt_contains_organiser_content() {
        let content = "# 2026-01-23\n**09:00** meeting";
        let prompt = build_system_prompt("2026-01-23", content);
        assert!(prompt.contains(content));
    }

    #[test]
    fn system_prompt_contains_format_instructions() {
        let prompt = build_system_prompt("2026-01-23", "");
        assert!(prompt.contains("[TODO]"));
        assert!(prompt.contains("[DONE]"));
        assert!(prompt.contains("# YYYY-MM-DD"));
        assert!(prompt.contains("**HH:MM**"));
    }

    #[test]
    fn system_prompt_contains_role_description() {
        let prompt = build_system_prompt("2026-01-23", "");
        assert!(prompt.contains("Ambrogio"));
        assert!(prompt.contains("personal assistant"));
    }
}
