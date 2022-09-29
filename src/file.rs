//! Load and parse a devlog entry file.

use crate::task::Task;
use std::fs::File;
use std::io::Error as IOError;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Represents a devlog entry file.
pub struct LogFile {
    tasks: Vec<Task>,
}

impl LogFile {
    /// Loads and parses the devlog entry file at `path`
    pub fn load(path: &Path) -> Result<LogFile, IOError> {
        let f = File::open(path)?;
        let r = BufReader::new(f);
        let mut tasks = Vec::new();
        let mut start_free_form = false;
        for line in r.lines().flatten() {
            // if the line starts with ``` then assume its
            // a code block, and therefore exempt from devlog
            // formatting rules
            if line.starts_with("```") {
                // just set the inverse of the boolean
                // false -> true | true -> false
                start_free_form = !start_free_form;
                continue;
            } else if start_free_form {
                continue;
            }
            if let Some(task) = Task::from_string(&line) {
                tasks.push(task)
            }
        }
        Ok(LogFile { tasks })
    }

    /// Returns the tasks contained in the devlog entry file.
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{Task, TaskStatus};
    use std::fs::OpenOptions;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_load() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("testlog");
        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&p)
            .unwrap();
        writeln!(f, "+ DONE").unwrap();
        writeln!(f, "- BLOCKED").unwrap();
        writeln!(f, "* INCOMPLETE").unwrap();
        writeln!(f, "COMMENT").unwrap();

        let lf = LogFile::load(&p).unwrap();
        let expected = vec![
            Task::new(TaskStatus::Done, "DONE"),
            Task::new(TaskStatus::Blocked, "BLOCKED"),
            Task::new(TaskStatus::ToDo, "INCOMPLETE"),
        ];
        assert_eq!(lf.tasks(), &expected[..]);
    }
}
