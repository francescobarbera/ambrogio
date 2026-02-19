use anyhow::Result;
use chrono::NaiveDateTime;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
pub struct Todo {
    pub description: String,
    pub done: bool,
    pub project: String,
}

fn parse_todo_line(line: &str) -> Option<(String, bool)> {
    let trimmed = line.trim();
    if let Some(desc) = trimmed.strip_prefix("- [ ] ") {
        Some((desc.to_string(), false))
    } else {
        trimmed
            .strip_prefix("- [x] ")
            .map(|desc| (desc.to_string(), true))
    }
}

fn parse_project_header(line: &str) -> Option<String> {
    line.strip_prefix("## ").map(|name| name.trim().to_string())
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

fn find_section_end(lines: &[&str], header_index: usize) -> usize {
    for (i, line) in lines.iter().enumerate().skip(header_index + 1) {
        if line.starts_with("## ") {
            return i;
        }
    }
    lines.len()
}

pub struct TodoStore {
    path: PathBuf,
}

impl TodoStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn projects(&self) -> Result<Vec<String>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.path)?;
        Ok(content.lines().filter_map(parse_project_header).collect())
    }

    pub fn add_project(&self, name: &str) -> Result<()> {
        let header = format!("## {}\n", name);

        if self.path.exists() {
            let content = fs::read_to_string(&self.path)?;

            if content
                .lines()
                .any(|l| parse_project_header(l).as_deref() == Some(name))
            {
                anyhow::bail!("Project '{}' already exists", name);
            }

            let separator = if content.ends_with('\n') || content.is_empty() {
                ""
            } else {
                "\n"
            };
            fs::write(&self.path, format!("{}{}{}", content, separator, header))?;
        } else {
            if let Some(parent) = self.path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&self.path, header)?;
        }

        Ok(())
    }

    pub fn delete_project(&self, name: &str) -> Result<()> {
        let content = fs::read_to_string(&self.path)?;
        let lines: Vec<&str> = content.lines().collect();

        let header_index = lines
            .iter()
            .position(|l| parse_project_header(l).as_deref() == Some(name))
            .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", name))?;

        let section_end = find_section_end(&lines, header_index);

        let mut new_lines: Vec<String> = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            if i < header_index || i >= section_end {
                new_lines.push(line.to_string());
            }
        }

        write_lines(&self.path, &new_lines, content.ends_with('\n'))
    }

    pub fn add(&self, project: &str, description: &str) -> Result<()> {
        let content = fs::read_to_string(&self.path)?;
        let lines: Vec<&str> = content.lines().collect();

        let header_index = lines
            .iter()
            .position(|l| parse_project_header(l).as_deref() == Some(project))
            .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", project))?;

        let section_end = find_section_end(&lines, header_index);

        let mut new_lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
        new_lines.insert(section_end, format!("- [ ] {}", description));

        write_lines(&self.path, &new_lines, content.ends_with('\n'))
    }

    pub fn load_all(&self) -> Result<Vec<Todo>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.path)?;
        let mut current_project = String::new();
        let mut todos = Vec::new();

        for line in content.lines() {
            if let Some(project) = parse_project_header(line) {
                current_project = project;
            } else if let Some((description, done)) = parse_todo_line(line) {
                if !current_project.is_empty() {
                    todos.push(Todo {
                        description,
                        done,
                        project: current_project.clone(),
                    });
                }
            }
        }

        Ok(todos)
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
            return Ok(());
        }

        let mut current_project = String::new();
        for (i, todo) in todos.iter().enumerate() {
            if todo.project != current_project {
                current_project = todo.project.clone();
                println!("\n  ## {}", current_project);
            }
            println!("  {}. {}", i + 1, todo.description);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn store_with_content(dir: &TempDir, content: &str) -> (TodoStore, PathBuf) {
        let path = dir.path().join("todos.md");
        fs::write(&path, content).unwrap();
        (TodoStore::new(path.clone()), path)
    }

    #[test]
    fn parses_open_todo_line() {
        let (desc, done) = parse_todo_line("- [ ] buy milk").unwrap();
        assert_eq!(desc, "buy milk");
        assert!(!done);
    }

    #[test]
    fn parses_done_todo_line() {
        let (desc, done) = parse_todo_line("- [x] buy milk").unwrap();
        assert_eq!(desc, "buy milk");
        assert!(done);
    }

    #[test]
    fn ignores_non_todo_lines() {
        assert!(parse_todo_line("# heading").is_none());
        assert!(parse_todo_line("## Project").is_none());
        assert!(parse_todo_line("some text").is_none());
        assert!(parse_todo_line("").is_none());
    }

    #[test]
    fn parses_project_header_line() {
        assert_eq!(parse_project_header("## Work"), Some("Work".to_string()));
        assert_eq!(
            parse_project_header("## My Project"),
            Some("My Project".to_string())
        );
    }

    #[test]
    fn ignores_non_header_lines() {
        assert!(parse_project_header("# heading").is_none());
        assert!(parse_project_header("- [ ] task").is_none());
        assert!(parse_project_header("").is_none());
    }

    #[test]
    fn projects_returns_empty_for_missing_file() {
        let dir = TempDir::new().unwrap();
        let store = TodoStore::new(dir.path().join("todos.md"));
        assert!(store.projects().unwrap().is_empty());
    }

    #[test]
    fn projects_lists_all_project_names() {
        let dir = TempDir::new().unwrap();
        let (store, _) = store_with_content(&dir, "## Work\n- [ ] task\n## Personal\n- [ ] task\n");
        assert_eq!(store.projects().unwrap(), vec!["Work", "Personal"]);
    }

    #[test]
    fn add_project_creates_file_with_header() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("todos.md");
        let store = TodoStore::new(path.clone());

        store.add_project("Work").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "## Work\n");
    }

    #[test]
    fn add_project_appends_to_existing_file() {
        let dir = TempDir::new().unwrap();
        let (store, path) = store_with_content(&dir, "## Work\n- [ ] task\n");

        store.add_project("Personal").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "## Work\n- [ ] task\n## Personal\n");
    }

    #[test]
    fn add_project_rejects_duplicate() {
        let dir = TempDir::new().unwrap();
        let (store, _) = store_with_content(&dir, "## Work\n");

        let result = store.add_project("Work");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn delete_project_removes_section() {
        let dir = TempDir::new().unwrap();
        let (store, path) =
            store_with_content(&dir, "## Work\n- [ ] task 1\n## Personal\n- [ ] task 2\n");

        store.delete_project("Work").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "## Personal\n- [ ] task 2\n");
    }

    #[test]
    fn delete_project_removes_last_section() {
        let dir = TempDir::new().unwrap();
        let (store, path) =
            store_with_content(&dir, "## Work\n- [ ] task 1\n## Personal\n- [ ] task 2\n");

        store.delete_project("Personal").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "## Work\n- [ ] task 1\n");
    }

    #[test]
    fn delete_project_errors_on_unknown_project() {
        let dir = TempDir::new().unwrap();
        let (store, _) = store_with_content(&dir, "## Work\n");

        let result = store.delete_project("Unknown");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn add_todo_under_correct_project() {
        let dir = TempDir::new().unwrap();
        let (store, path) = store_with_content(&dir, "## Work\n- [ ] existing\n## Personal\n");

        store.add("Work", "new task").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(
            content,
            "## Work\n- [ ] existing\n- [ ] new task\n## Personal\n"
        );
    }

    #[test]
    fn add_todo_to_last_project() {
        let dir = TempDir::new().unwrap();
        let (store, path) = store_with_content(&dir, "## Work\n## Personal\n");

        store.add("Personal", "buy milk").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "## Work\n## Personal\n- [ ] buy milk\n");
    }

    #[test]
    fn add_todo_errors_on_unknown_project() {
        let dir = TempDir::new().unwrap();
        let (store, _) = store_with_content(&dir, "## Work\n");

        let result = store.add("Unknown", "task");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn load_all_returns_todos_with_projects() {
        let dir = TempDir::new().unwrap();
        let (store, _) = store_with_content(
            &dir,
            "## Work\n- [ ] task 1\n- [x] task 2\n## Personal\n- [ ] task 3\n",
        );

        let todos = store.load_all().unwrap();

        assert_eq!(todos.len(), 3);
        assert_eq!(todos[0].project, "Work");
        assert_eq!(todos[0].description, "task 1");
        assert!(!todos[0].done);
        assert_eq!(todos[1].project, "Work");
        assert!(todos[1].done);
        assert_eq!(todos[2].project, "Personal");
        assert_eq!(todos[2].description, "task 3");
    }

    #[test]
    fn load_all_ignores_todos_without_project() {
        let dir = TempDir::new().unwrap();
        let (store, _) = store_with_content(&dir, "- [ ] orphan\n## Work\n- [ ] task\n");

        let todos = store.load_all().unwrap();
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].project, "Work");
    }

    #[test]
    fn load_all_ignores_pomodoro_lines() {
        let dir = TempDir::new().unwrap();
        let (store, _) = store_with_content(
            &dir,
            "## Work\n- [ ] task\n  - 🍅 2026-02-12 10:00\n- [x] done\n",
        );

        let todos = store.load_all().unwrap();
        assert_eq!(todos.len(), 2);
    }

    #[test]
    fn load_all_returns_empty_for_missing_file() {
        let dir = TempDir::new().unwrap();
        let store = TodoStore::new(dir.path().join("todos.md"));
        assert!(store.load_all().unwrap().is_empty());
    }

    #[test]
    fn open_todos_filters_done() {
        let dir = TempDir::new().unwrap();
        let (store, _) =
            store_with_content(&dir, "## Work\n- [ ] open\n- [x] done\n- [ ] also open\n");

        let open = store.open_todos().unwrap();
        assert_eq!(open.len(), 2);
        assert_eq!(open[0].description, "open");
        assert_eq!(open[1].description, "also open");
    }

    #[test]
    fn complete_marks_correct_todo_globally() {
        let dir = TempDir::new().unwrap();
        let (store, path) = store_with_content(
            &dir,
            "## Work\n- [ ] first\n## Personal\n- [ ] second\n- [ ] third\n",
        );

        store.complete(1).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(
            content,
            "## Work\n- [ ] first\n## Personal\n- [x] second\n- [ ] third\n"
        );
    }

    #[test]
    fn complete_errors_on_out_of_bounds() {
        let dir = TempDir::new().unwrap();
        let (store, _) = store_with_content(&dir, "## Work\n- [ ] only one\n");

        let result = store.complete(5);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("out of bounds"));
    }

    #[test]
    fn complete_preserves_pomodoro_sub_items() {
        let dir = TempDir::new().unwrap();
        let (store, path) = store_with_content(
            &dir,
            "## Work\n- [ ] task\n  - 🍅 2026-02-12 10:00\n- [ ] other\n",
        );

        store.complete(0).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(
            content,
            "## Work\n- [x] task\n  - 🍅 2026-02-12 10:00\n- [ ] other\n"
        );
    }

    #[test]
    fn complete_skips_done_todos_when_counting() {
        let dir = TempDir::new().unwrap();
        let (store, path) = store_with_content(
            &dir,
            "## Work\n- [x] done\n- [ ] first open\n- [ ] second open\n",
        );

        store.complete(1).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(
            content,
            "## Work\n- [x] done\n- [ ] first open\n- [x] second open\n"
        );
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
        let (store, path) = store_with_content(&dir, "## Work\n- [ ] first\n- [ ] second\n");

        store
            .add_pomodoro(0, datetime(2026, 2, 12, 10, 0), false)
            .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(
            content,
            "## Work\n- [ ] first\n  - 🍅 2026-02-12 10:00\n- [ ] second\n"
        );
    }

    #[test]
    fn add_pomodoro_cancelled() {
        let dir = TempDir::new().unwrap();
        let (store, path) = store_with_content(&dir, "## Work\n- [ ] task\n");

        store
            .add_pomodoro(0, datetime(2026, 2, 12, 14, 30), true)
            .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(
            content,
            "## Work\n- [ ] task\n  - 🍅 2026-02-12 14:30 cancelled\n"
        );
    }

    #[test]
    fn add_pomodoro_appends_after_existing_pomodoros() {
        let dir = TempDir::new().unwrap();
        let (store, path) = store_with_content(
            &dir,
            "## Work\n- [ ] task\n  - 🍅 2026-02-12 10:00\n- [ ] other\n",
        );

        store
            .add_pomodoro(0, datetime(2026, 2, 12, 11, 0), false)
            .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(
            content,
            "## Work\n- [ ] task\n  - 🍅 2026-02-12 10:00\n  - 🍅 2026-02-12 11:00\n- [ ] other\n"
        );
    }

    #[test]
    fn add_pomodoro_across_projects() {
        let dir = TempDir::new().unwrap();
        let (store, path) =
            store_with_content(&dir, "## Work\n- [ ] task 1\n## Personal\n- [ ] task 2\n");

        store
            .add_pomodoro(1, datetime(2026, 2, 12, 9, 0), false)
            .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(
            content,
            "## Work\n- [ ] task 1\n## Personal\n- [ ] task 2\n  - 🍅 2026-02-12 09:00\n"
        );
    }

    #[test]
    fn add_pomodoro_errors_on_out_of_bounds() {
        let dir = TempDir::new().unwrap();
        let (store, _) = store_with_content(&dir, "## Work\n- [ ] only one\n");

        let result = store.add_pomodoro(5, datetime(2026, 2, 12, 10, 0), false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("out of bounds"));
    }

    #[test]
    fn delete_project_with_pomodoros() {
        let dir = TempDir::new().unwrap();
        let (store, path) = store_with_content(
            &dir,
            "## Work\n- [ ] task\n  - 🍅 2026-02-12 10:00\n## Personal\n- [ ] task\n",
        );

        store.delete_project("Work").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "## Personal\n- [ ] task\n");
    }
}
