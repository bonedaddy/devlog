//! Open a file using a text editor program (e.g. vim or nano)

use crate::config::Config;
use crate::error::Error;
use crate::hook::{execute_hook, HookType};
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// Opens the specified file in a text editor program.
/// If available, the before-edit and after-edit hooks are invoked.
pub fn open<W: Write>(w: &mut W, config: &Config, path: &Path) -> Result<(), Error> {
    execute_hook(w, config, &HookType::BeforeEdit, &[path.as_os_str()])?;
    open_in_editor(w, config, path)?;
    execute_hook(w, config, &HookType::AfterEdit, &[path.as_os_str()])?;
    Ok(())
}

fn open_in_editor<W: Write>(w: &mut W, config: &Config, path: &Path) -> Result<(), Error> {
    let prog = config.editor_prog();
    let status = Command::new(prog).arg(&path).status()?;

    if status.success() {
        Ok(())
    } else {
        match status.code() {
            Some(code) => writeln!(
                w,
                "Command `{} {}` exited with status {}",
                prog,
                path.to_string_lossy(),
                code
            )
            .map_err(From::from),
            None => writeln!(w, "Process terminated by signal").map_err(From::from),
        }
    }
}
