use std::env;
use std::path::Path;
use std::process::Command;

pub fn open_in_editor(path: &Path) -> Result<(), String> {
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    let status = Command::new(&editor)
        .arg(path)
        .status()
        .map_err(|e| format!("Failed to launch editor '{}': {}", editor, e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("Editor exited with status: {}", status))
    }
}
