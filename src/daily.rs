use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use std::fs;
use std::path::Path;

const TODOS_HEADER: &str = "## Todos:";
const POMODORO: &str = "🍅";

/// Returns the open `[ ]` task descriptions in today's `## Todos:` block,
/// in file order. Both dashed (`- [ ] foo`) and dashless (`[ ] foo`) forms
/// are recognised. Returns an empty vec if the file, today's section, or
/// the Todos block is absent.
pub fn today_open(path: &Path, today: NaiveDate) -> Result<Vec<String>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)?;
    Ok(parse_today_open(&content, today))
}

/// Appends `- [ ] description` to today's `## Todos:` block. Creates the
/// day section, the Todos block, or the file itself if any are missing.
/// New day sections are inserted at the top of the file.
pub fn add(path: &Path, today: NaiveDate, description: &str) -> Result<()> {
    let content = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let new_content = insert_task(&content, today, description);
    fs::write(path, new_content)?;
    Ok(())
}

/// Flips `[ ]` → `[x]` on the `index`-th open task in today's Todos block,
/// preserving leading dash/no-dash form and trailing whitespace. Errors with
/// "no tasks for today" if today's section or Todos block is absent, or
/// "index N out of bounds" if the index exceeds the open-task count.
pub fn complete(path: &Path, today: NaiveDate, index: usize) -> Result<()> {
    modify_open_line(path, today, index, complete_line)
}

/// Appends a `🍅` to the `index`-th open task in today's Todos block,
/// before any trailing whitespace. If the line already ends (modulo
/// whitespace) with a `🍅`, the new icon concatenates without a separator.
/// Same error semantics as `complete`.
pub fn add_pomodoro(path: &Path, today: NaiveDate, index: usize) -> Result<()> {
    modify_open_line(path, today, index, append_pomodoro)
}

// --- pure helpers ---

fn format_day(today: NaiveDate) -> String {
    format!("# {}", today.format("%Y-%m-%d"))
}

fn is_day_header(line: &str) -> bool {
    line.starts_with("# ") && !line.starts_with("## ")
}

fn is_section_header(line: &str) -> bool {
    line.starts_with("## ")
}

fn parse_open_task(line: &str) -> Option<&str> {
    line.strip_prefix("- [ ] ")
        .or_else(|| line.strip_prefix("[ ] "))
}

struct TodosRange {
    start: usize,
    end: usize,
}

fn find_todos_range(lines: &[&str], today: NaiveDate) -> Option<TodosRange> {
    let day_header = format_day(today);
    let day_start = lines.iter().position(|l| l.trim_end() == day_header)?;

    let day_end = lines
        .iter()
        .enumerate()
        .skip(day_start + 1)
        .find(|(_, l)| is_day_header(l))
        .map(|(i, _)| i)
        .unwrap_or(lines.len());

    let todos_header = (day_start + 1..day_end).find(|&i| lines[i].trim_end() == TODOS_HEADER)?;

    // day_end already bounds the search at the next day-header, so within
    // (todos_header, day_end) only `##` section headers can appear.
    let block_end = (todos_header + 1..day_end)
        .find(|&i| is_section_header(lines[i]))
        .unwrap_or(day_end);

    Some(TodosRange {
        start: todos_header + 1,
        end: block_end,
    })
}

fn parse_today_open(content: &str, today: NaiveDate) -> Vec<String> {
    let lines: Vec<&str> = content.split('\n').collect();
    let Some(range) = find_todos_range(&lines, today) else {
        return Vec::new();
    };
    lines[range.start..range.end]
        .iter()
        .filter_map(|l| parse_open_task(l).map(|s| s.trim_end().to_string()))
        .collect()
}

fn insert_task(content: &str, today: NaiveDate, description: &str) -> String {
    let new_task = format!("- [ ] {}", description);
    let day_header = format_day(today);
    let lines: Vec<&str> = content.split('\n').collect();

    let Some(day_idx) = lines.iter().position(|l| l.trim_end() == day_header) else {
        if content.is_empty() {
            return format!("{}\n{}\n{}\n", day_header, TODOS_HEADER, new_task);
        }
        return format!(
            "{}\n{}\n{}\n\n{}",
            day_header, TODOS_HEADER, new_task, content
        );
    };

    if let Some(range) = find_todos_range(&lines, today) {
        let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
        new_lines.insert(range.end, new_task);
        return new_lines.join("\n");
    }

    let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    new_lines.insert(day_idx + 1, TODOS_HEADER.to_string());
    new_lines.insert(day_idx + 2, new_task);
    new_lines.join("\n")
}

fn complete_line(line: &str) -> String {
    line.replacen("[ ]", "[x]", 1)
}

fn append_pomodoro(line: &str) -> String {
    let trimmed_len = line.trim_end().len();
    let content_part = &line[..trimmed_len];
    let trailing = &line[trimmed_len..];
    let separator = if content_part.ends_with(POMODORO) {
        ""
    } else {
        " "
    };
    format!("{}{}{}{}", content_part, separator, POMODORO, trailing)
}

fn modify_open_line<F>(path: &Path, today: NaiveDate, index: usize, transform: F) -> Result<()>
where
    F: FnOnce(&str) -> String,
{
    if !path.exists() {
        return Err(anyhow!("no tasks for today"));
    }
    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.split('\n').collect();
    let range = find_todos_range(&lines, today).ok_or_else(|| anyhow!("no tasks for today"))?;
    let target = (range.start..range.end)
        .filter(|&i| parse_open_task(lines[i]).is_some())
        .nth(index)
        .ok_or_else(|| anyhow!("index {} out of bounds", index))?;

    let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    new_lines[target] = transform(&new_lines[target]);
    fs::write(path, new_lines.join("\n"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 5, 15).unwrap()
    }

    fn temp_with(content: &str) -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("organiser.md");
        fs::write(&path, content).unwrap();
        (dir, path)
    }

    fn temp_empty() -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("organiser.md");
        (dir, path)
    }

    fn read(path: &Path) -> String {
        fs::read_to_string(path).unwrap()
    }

    #[test]
    fn today_open_returns_empty_for_missing_file() {
        let (_dir, path) = temp_empty();
        assert!(today_open(&path, today()).unwrap().is_empty());
    }

    #[test]
    fn today_open_returns_empty_when_today_section_absent() {
        let (_dir, path) = temp_with("# 2026-05-14\n## Todos:\n- [ ] old\n");
        assert!(today_open(&path, today()).unwrap().is_empty());
    }

    #[test]
    fn today_open_returns_empty_when_todos_block_absent() {
        let (_dir, path) = temp_with("# 2026-05-15\n## Logs:\nstuff\n");
        assert!(today_open(&path, today()).unwrap().is_empty());
    }

    #[test]
    fn today_open_returns_empty_when_only_completed_tasks() {
        let (_dir, path) = temp_with("# 2026-05-15\n## Todos:\n- [x] done\n- [x] also\n");
        assert!(today_open(&path, today()).unwrap().is_empty());
    }

    #[test]
    fn today_open_reads_both_dashed_and_dashless_forms() {
        let (_dir, path) = temp_with("# 2026-05-15\n## Todos:\n- [ ] dashed\n[ ] dashless\n");
        assert_eq!(
            today_open(&path, today()).unwrap(),
            vec!["dashed".to_string(), "dashless".to_string()]
        );
    }

    #[test]
    fn today_open_ignores_task_lookalike_inside_logs() {
        let (_dir, path) =
            temp_with("# 2026-05-15\n## Todos:\n- [ ] real\n## Logs:\n- [ ] fake\n[ ] also fake\n");
        assert_eq!(
            today_open(&path, today()).unwrap(),
            vec!["real".to_string()]
        );
    }

    #[test]
    fn today_open_returns_only_today_in_multi_day_file() {
        let (_dir, path) = temp_with(
            "# 2026-05-15\n## Todos:\n- [ ] today\n# 2026-05-14\n## Todos:\n- [ ] yesterday\n",
        );
        assert_eq!(
            today_open(&path, today()).unwrap(),
            vec!["today".to_string()]
        );
    }

    #[test]
    fn add_creates_file_with_only_day_and_todos_headers() {
        let (_dir, path) = temp_empty();
        add(&path, today(), "buy milk").unwrap();
        assert_eq!(read(&path), "# 2026-05-15\n## Todos:\n- [ ] buy milk\n");
    }

    #[test]
    fn add_inserts_todos_block_when_today_exists_without_one() {
        let (_dir, path) = temp_with("# 2026-05-15\n## Logs:\nstuff\n");
        add(&path, today(), "buy milk").unwrap();
        assert_eq!(
            read(&path),
            "# 2026-05-15\n## Todos:\n- [ ] buy milk\n## Logs:\nstuff\n"
        );
    }

    #[test]
    fn add_appends_to_existing_todos_block() {
        let (_dir, path) = temp_with("# 2026-05-15\n## Todos:\n- [ ] existing\n## Logs:\nstuff\n");
        add(&path, today(), "new task").unwrap();
        assert_eq!(
            read(&path),
            "# 2026-05-15\n## Todos:\n- [ ] existing\n- [ ] new task\n## Logs:\nstuff\n"
        );
    }

    #[test]
    fn add_prepends_new_day_section_at_top() {
        let (_dir, path) = temp_with("# 2026-05-14\n## Todos:\n- [ ] yesterday\n");
        add(&path, today(), "today task").unwrap();
        assert_eq!(
            read(&path),
            "# 2026-05-15\n## Todos:\n- [ ] today task\n\n# 2026-05-14\n## Todos:\n- [ ] yesterday\n"
        );
    }

    #[test]
    fn add_preserves_no_trailing_newline_state() {
        let (_dir, path) = temp_with("# 2026-05-15\n## Todos:\n- [ ] existing");
        add(&path, today(), "new").unwrap();
        assert_eq!(
            read(&path),
            "# 2026-05-15\n## Todos:\n- [ ] existing\n- [ ] new"
        );
    }

    #[test]
    fn complete_flips_dashed_open_to_done() {
        let (_dir, path) = temp_with("# 2026-05-15\n## Todos:\n- [ ] task\n");
        complete(&path, today(), 0).unwrap();
        assert_eq!(read(&path), "# 2026-05-15\n## Todos:\n- [x] task\n");
    }

    #[test]
    fn complete_preserves_dashless_form_and_trailing_whitespace() {
        let (_dir, path) = temp_with("# 2026-05-15\n## Todos:\n[ ] task  \n");
        complete(&path, today(), 0).unwrap();
        assert_eq!(read(&path), "# 2026-05-15\n## Todos:\n[x] task  \n");
    }

    #[test]
    fn complete_out_of_bounds_returns_err() {
        let (_dir, path) = temp_with("# 2026-05-15\n## Todos:\n- [ ] only one\n");
        let err = complete(&path, today(), 5).unwrap_err();
        assert!(err.to_string().contains("out of bounds"));
    }

    #[test]
    fn complete_returns_err_when_no_today_section() {
        let (_dir, path) = temp_with("# 2026-05-14\n## Todos:\n- [ ] old\n");
        let err = complete(&path, today(), 0).unwrap_err();
        assert!(err.to_string().contains("no tasks for today"));
    }

    #[test]
    fn complete_returns_err_when_today_exists_without_todos_block() {
        let (_dir, path) = temp_with("# 2026-05-15\n## Logs:\nstuff\n");
        let err = complete(&path, today(), 0).unwrap_err();
        assert!(err.to_string().contains("no tasks for today"));
    }

    #[test]
    fn add_pomodoro_first_icon_on_dashed_line() {
        let (_dir, path) = temp_with("# 2026-05-15\n## Todos:\n- [ ] task\n");
        add_pomodoro(&path, today(), 0).unwrap();
        assert_eq!(read(&path), "# 2026-05-15\n## Todos:\n- [ ] task 🍅\n");
    }

    #[test]
    fn add_pomodoro_first_icon_on_dashless_preserves_trailing_whitespace() {
        let (_dir, path) = temp_with("# 2026-05-15\n## Todos:\n[ ] task  \n");
        add_pomodoro(&path, today(), 0).unwrap();
        assert_eq!(read(&path), "# 2026-05-15\n## Todos:\n[ ] task 🍅  \n");
    }

    #[test]
    fn add_pomodoro_second_icon_on_dashed_line_has_no_extra_space() {
        let (_dir, path) = temp_with("# 2026-05-15\n## Todos:\n- [ ] task 🍅\n");
        add_pomodoro(&path, today(), 0).unwrap();
        assert_eq!(read(&path), "# 2026-05-15\n## Todos:\n- [ ] task 🍅🍅\n");
    }

    #[test]
    fn add_pomodoro_third_icon_on_dashed_line() {
        let (_dir, path) = temp_with("# 2026-05-15\n## Todos:\n- [ ] task 🍅🍅\n");
        add_pomodoro(&path, today(), 0).unwrap();
        assert_eq!(read(&path), "# 2026-05-15\n## Todos:\n- [ ] task 🍅🍅🍅\n");
    }

    #[test]
    fn add_pomodoro_second_icon_on_dashless_preserves_trailing_whitespace() {
        let (_dir, path) = temp_with("# 2026-05-15\n## Todos:\n[ ] task 🍅  \n");
        add_pomodoro(&path, today(), 0).unwrap();
        assert_eq!(read(&path), "# 2026-05-15\n## Todos:\n[ ] task 🍅🍅  \n");
    }

    #[test]
    fn add_pomodoro_out_of_bounds_returns_err() {
        let (_dir, path) = temp_with("# 2026-05-15\n## Todos:\n- [ ] only one\n");
        let err = add_pomodoro(&path, today(), 5).unwrap_err();
        assert!(err.to_string().contains("out of bounds"));
    }

    #[test]
    fn add_pomodoro_returns_err_when_no_today_section() {
        let (_dir, path) = temp_with("# 2026-05-14\n## Todos:\n- [ ] old\n");
        let err = add_pomodoro(&path, today(), 0).unwrap_err();
        assert!(err.to_string().contains("no tasks for today"));
    }

    #[test]
    fn add_pomodoro_returns_err_when_today_exists_without_todos_block() {
        let (_dir, path) = temp_with("# 2026-05-15\n## Logs:\nstuff\n");
        let err = add_pomodoro(&path, today(), 0).unwrap_err();
        assert!(err.to_string().contains("no tasks for today"));
    }

    #[test]
    fn parser_terminates_todos_at_eof() {
        let (_dir, path) = temp_with("# 2026-05-15\n## Todos:\n- [ ] one\n- [ ] two");
        assert_eq!(
            today_open(&path, today()).unwrap(),
            vec!["one".to_string(), "two".to_string()]
        );
    }

    #[test]
    fn parser_terminates_todos_at_next_day_header() {
        let (_dir, path) =
            temp_with("# 2026-05-15\n## Todos:\n- [ ] today\n# 2026-05-14\n- [ ] not mine\n");
        assert_eq!(
            today_open(&path, today()).unwrap(),
            vec!["today".to_string()]
        );
    }

    #[test]
    fn parser_terminates_todos_at_next_section_header() {
        let (_dir, path) =
            temp_with("# 2026-05-15\n## Todos:\n- [ ] real\n## Custom:\n- [ ] not mine\n");
        assert_eq!(
            today_open(&path, today()).unwrap(),
            vec!["real".to_string()]
        );
    }
}
