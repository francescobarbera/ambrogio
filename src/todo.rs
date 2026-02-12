use anyhow::Result;
use chrono::NaiveDateTime;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
pub struct Todo {
    pub description: String,
    pub done: bool,
}

fn parse_line(line: &str) -> Option<Todo> {
    let trimmed = line.trim();
    if let Some(desc) = trimmed.strip_prefix("- [ ] ") {
        Some(Todo {
            description: desc.to_string(),
            done: false,
        })
    } else {
        trimmed.strip_prefix("- [x] ").map(|desc| Todo {
            description: desc.to_string(),
            done: true,
        })
    }
}

fn find_open_todo_line(lines: &[&str], open_index: usize) -> Result<usize> {
    let mut open_count = 0;
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("- [ ] ") {
            if open_count == open_index {
                return Ok(i);
            }
            open_count += 1;
        }
    }
    anyhow::bail!("Todo index {} out of bounds", open_index)
}

fn write_lines(path: &Path, lines: &[String], trailing_newline: bool) -> Result<()> {
    let mut output = lines.join("\n");
    if trailing_newline {
        output.push('\n');
    }
    fs::write(path, output)?;
    Ok(())
}

pub struct TodoStore {
    path: PathBuf,
}

impl TodoStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn add(&self, description: &str) -> Result<()> {
        let entry = format!("- [ ] {}\n", description);

        if self.path.exists() {
            let content = fs::read_to_string(&self.path)?;
            let separator = if content.ends_with('\n') || content.is_empty() {
                ""
            } else {
                "\n"
            };
            fs::write(&self.path, format!("{}{}{}", content, separator, entry))?;
        } else {
            if let Some(parent) = self.path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&self.path, entry)?;
        }

        Ok(())
    }

    pub fn load_all(&self) -> Result<Vec<Todo>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.path)?;
        Ok(content.lines().filter_map(parse_line).collect())
    }

    pub fn open_todos(&self) -> Result<Vec<Todo>> {
        Ok(self.load_all()?.into_iter().filter(|t| !t.done).collect())
    }

    pub fn add_pomodoro(
        &self,
        open_index: usize,
        started_at: NaiveDateTime,
        cancelled: bool,
    ) -> Result<()> {
        let content = fs::read_to_string(&self.path)?;
        let lines: Vec<&str> = content.lines().collect();
        let target = find_open_todo_line(&lines, open_index)?;

        let mut insert_at = target + 1;
        while insert_at < lines.len() && lines[insert_at].starts_with("  ") {
            insert_at += 1;
        }

        let status = if cancelled { " cancelled" } else { "" };
        let pomodoro_line = format!("  - 🍅 {}{}", started_at.format("%Y-%m-%d %H:%M"), status);

        let mut new_lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
        new_lines.insert(insert_at, pomodoro_line);

        write_lines(&self.path, &new_lines, content.ends_with('\n'))
    }

    pub fn complete(&self, index: usize) -> Result<()> {
        let content = fs::read_to_string(&self.path)?;
        let lines: Vec<&str> = content.lines().collect();
        let target = find_open_todo_line(&lines, index)?;

        let mut new_lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
        new_lines[target] = new_lines[target].replacen("- [ ] ", "- [x] ", 1);

        write_lines(&self.path, &new_lines, content.ends_with('\n'))
    }

    pub fn print_open_todos(&self) -> Result<()> {
        let todos = self.open_todos()?;

        if todos.is_empty() {
            println!("No open todos.");
        } else {
            for (i, todo) in todos.iter().enumerate() {
                println!("  {}. {}", i + 1, todo.description);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn parses_open_todo() {
        let todo = parse_line("- [ ] buy milk").unwrap();
        assert_eq!(todo.description, "buy milk");
        assert!(!todo.done);
    }

    #[test]
    fn parses_done_todo() {
        let todo = parse_line("- [x] buy milk").unwrap();
        assert_eq!(todo.description, "buy milk");
        assert!(todo.done);
    }

    #[test]
    fn ignores_non_todo_lines() {
        assert!(parse_line("# heading").is_none());
        assert!(parse_line("some text").is_none());
        assert!(parse_line("").is_none());
        assert!(parse_line("- regular list item").is_none());
    }

    #[test]
    fn add_creates_file_if_missing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("todos.md");
        let store = TodoStore::new(path.clone());

        store.add("buy milk").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "- [ ] buy milk\n");
    }

    #[test]
    fn add_appends_to_existing_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("todos.md");
        fs::write(&path, "- [ ] first task\n").unwrap();

        let store = TodoStore::new(path.clone());
        store.add("second task").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "- [ ] first task\n- [ ] second task\n");
    }

    #[test]
    fn load_all_returns_empty_for_missing_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("todos.md");
        let store = TodoStore::new(path);

        let todos = store.load_all().unwrap();
        assert!(todos.is_empty());
    }

    #[test]
    fn load_all_parses_mixed_content() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("todos.md");
        fs::write(&path, "- [ ] open task\n- [x] done task\nrandom line\n").unwrap();

        let store = TodoStore::new(path);
        let todos = store.load_all().unwrap();

        assert_eq!(todos.len(), 2);
        assert!(!todos[0].done);
        assert!(todos[1].done);
    }

    #[test]
    fn open_todos_filters_done() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("todos.md");
        fs::write(&path, "- [ ] open\n- [x] done\n- [ ] also open\n").unwrap();

        let store = TodoStore::new(path);
        let open = store.open_todos().unwrap();

        assert_eq!(open.len(), 2);
        assert_eq!(open[0].description, "open");
        assert_eq!(open[1].description, "also open");
    }

    #[test]
    fn complete_marks_correct_todo() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("todos.md");
        fs::write(&path, "- [ ] first\n- [ ] second\n- [ ] third\n").unwrap();

        let store = TodoStore::new(path.clone());
        store.complete(1).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "- [ ] first\n- [x] second\n- [ ] third\n");
    }

    #[test]
    fn complete_errors_on_out_of_bounds() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("todos.md");
        fs::write(&path, "- [ ] only one\n").unwrap();

        let store = TodoStore::new(path);
        let result = store.complete(5);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("out of bounds"));
    }

    #[test]
    fn complete_preserves_pomodoro_sub_items() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("todos.md");
        fs::write(&path, "- [ ] task\n  - 🍅 2026-02-12 10:00\n- [ ] other\n").unwrap();

        let store = TodoStore::new(path.clone());
        store.complete(0).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(
            content,
            "- [x] task\n  - 🍅 2026-02-12 10:00\n- [ ] other\n"
        );
    }

    #[test]
    fn complete_skips_done_todos_when_counting() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("todos.md");
        fs::write(&path, "- [x] done\n- [ ] first open\n- [ ] second open\n").unwrap();

        let store = TodoStore::new(path.clone());
        store.complete(1).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "- [x] done\n- [ ] first open\n- [x] second open\n");
    }

    fn datetime(year: i32, month: u32, day: u32, hour: u32, min: u32) -> NaiveDateTime {
        chrono::NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(hour, min, 0)
            .unwrap()
    }

    #[test]
    fn add_pomodoro_inserts_under_correct_todo() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("todos.md");
        fs::write(&path, "- [ ] first\n- [ ] second\n").unwrap();

        let store = TodoStore::new(path.clone());
        store
            .add_pomodoro(0, datetime(2026, 2, 12, 10, 0), false)
            .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(
            content,
            "- [ ] first\n  - 🍅 2026-02-12 10:00\n- [ ] second\n"
        );
    }

    #[test]
    fn add_pomodoro_cancelled() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("todos.md");
        fs::write(&path, "- [ ] task\n").unwrap();

        let store = TodoStore::new(path.clone());
        store
            .add_pomodoro(0, datetime(2026, 2, 12, 14, 30), true)
            .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "- [ ] task\n  - 🍅 2026-02-12 14:30 cancelled\n");
    }

    #[test]
    fn add_pomodoro_appends_after_existing_pomodoros() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("todos.md");
        fs::write(&path, "- [ ] task\n  - 🍅 2026-02-12 10:00\n- [ ] other\n").unwrap();

        let store = TodoStore::new(path.clone());
        store
            .add_pomodoro(0, datetime(2026, 2, 12, 11, 0), false)
            .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(
            content,
            "- [ ] task\n  - 🍅 2026-02-12 10:00\n  - 🍅 2026-02-12 11:00\n- [ ] other\n"
        );
    }

    #[test]
    fn add_pomodoro_skips_done_todos_when_counting() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("todos.md");
        fs::write(&path, "- [x] done\n- [ ] first open\n- [ ] second open\n").unwrap();

        let store = TodoStore::new(path.clone());
        store
            .add_pomodoro(1, datetime(2026, 2, 12, 9, 0), false)
            .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(
            content,
            "- [x] done\n- [ ] first open\n- [ ] second open\n  - 🍅 2026-02-12 09:00\n"
        );
    }

    #[test]
    fn add_pomodoro_errors_on_out_of_bounds() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("todos.md");
        fs::write(&path, "- [ ] only one\n").unwrap();

        let store = TodoStore::new(path);
        let result = store.add_pomodoro(5, datetime(2026, 2, 12, 10, 0), false);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("out of bounds"));
    }

    #[test]
    fn load_all_ignores_pomodoro_lines() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("todos.md");
        fs::write(&path, "- [ ] task\n  - 🍅 2026-02-12 10:00\n- [x] done\n").unwrap();

        let store = TodoStore::new(path);
        let todos = store.load_all().unwrap();

        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].description, "task");
        assert_eq!(todos[1].description, "done");
    }
}
