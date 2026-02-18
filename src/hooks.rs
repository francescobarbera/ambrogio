use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

fn default_hooks_dir() -> PathBuf {
    dirs::home_dir()
        .expect("cannot resolve home directory")
        .join(".config/ambrogio/hooks")
}

fn resolve_hook(base: &Path, feature: &str, event: &str) -> PathBuf {
    base.join(feature).join(format!("{event}.sh"))
}

pub fn run(feature: &str, event: &str) -> Result<()> {
    run_with_base(&default_hooks_dir(), feature, event)
}

fn run_with_base(base: &Path, feature: &str, event: &str) -> Result<()> {
    let path = resolve_hook(base, feature, event);

    if !path.exists() {
        return Ok(());
    }

    let output = Command::new("sh").arg(&path).output()?;

    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    if !output.status.success() {
        eprintln!(
            "Warning: hook {}/{}.sh exited with {}",
            feature, event, output.status
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn returns_ok_when_hook_missing() {
        let dir = TempDir::new().unwrap();
        let result = run_with_base(dir.path(), "pomodoro", "stop");
        assert!(result.is_ok());
    }

    #[test]
    fn executes_existing_hook() {
        let dir = TempDir::new().unwrap();
        let hook_dir = dir.path().join("pomodoro");
        fs::create_dir_all(&hook_dir).unwrap();

        let marker = dir.path().join("marker.txt");
        let script = format!("#!/bin/sh\necho ran > {}", marker.display());
        fs::write(hook_dir.join("stop.sh"), script).unwrap();

        run_with_base(dir.path(), "pomodoro", "stop").unwrap();

        assert!(marker.exists(), "hook script did not run");
        assert_eq!(fs::read_to_string(&marker).unwrap().trim(), "ran");
    }

    #[test]
    fn returns_ok_on_hook_failure() {
        let dir = TempDir::new().unwrap();
        let hook_dir = dir.path().join("pomodoro");
        fs::create_dir_all(&hook_dir).unwrap();

        fs::write(hook_dir.join("stop.sh"), "#!/bin/sh\nexit 1").unwrap();

        let result = run_with_base(dir.path(), "pomodoro", "stop");
        assert!(result.is_ok());
    }
}
