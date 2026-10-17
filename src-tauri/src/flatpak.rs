//! Running inside a Flatpak (roadmap Phase 13). The sandbox changes three
//! things, and outside one none of this applies:
//!
//! - Programs on the host, such as Claude Code, are reached through
//!   `flatpak-spawn --host`. That needs `--talk-name=org.freedesktop.Flatpak`,
//!   which the manifest leaves out; the user opts in with `flatpak override`.
//! - Anything outside the sandbox that starts Blotter (an MCP client, the
//!   login entry) goes through `flatpak run`.
//! - The login entry is written here, since the autostart plugin would
//!   point it at `/app/bin/blotter`, which only exists inside the sandbox.

use std::io;
use std::path::PathBuf;

/// The Flatpak app id when running in one.
pub fn app_id() -> Option<String> {
    std::env::var("FLATPAK_ID").ok().filter(|id| !id.is_empty())
}

/// How a program outside the sandbox starts Blotter, before Blotter's own
/// arguments: `flatpak run --command=blotter <id>`, or this executable.
pub fn self_command() -> io::Result<(String, Vec<String>)> {
    Ok(match app_id() {
        Some(id) => self_command_in(&id),
        None => (std::env::current_exe()?.display().to_string(), Vec::new()),
    })
}

fn self_command_in(id: &str) -> (String, Vec<String>) {
    (
        "flatpak".to_owned(),
        vec![
            "run".to_owned(),
            "--command=blotter".to_owned(),
            id.to_owned(),
        ],
    )
}

/// The login entry in the host's `~/.config/autostart` (the manifest
/// grants `xdg-config/autostart:create`).
fn autostart_file(id: &str) -> io::Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no home folder"))?;
    Ok(home
        .join(".config")
        .join("autostart")
        .join(format!("{id}.desktop")))
}

fn autostart_entry(id: &str, args: &[&str]) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName=Blotter\nExec=flatpak run {id} {}\n\
         X-Flatpak={id}\nStartupNotify=false\nTerminal=false\n",
        args.join(" ")
    )
}

pub fn autostart_enabled(id: &str) -> io::Result<bool> {
    Ok(autostart_file(id)?.is_file())
}

pub fn set_autostart(id: &str, enabled: bool, args: &[&str]) -> io::Result<()> {
    let file = autostart_file(id)?;
    if enabled {
        if let Some(dir) = file.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&file, autostart_entry(id, args))
    } else {
        match std::fs::remove_file(&file) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outside_programs_start_blotter_through_flatpak_run() {
        let (cmd, args) = self_command_in("io.github.lasr0.Blotter");
        assert_eq!(cmd, "flatpak");
        assert_eq!(
            args,
            ["run", "--command=blotter", "io.github.lasr0.Blotter"]
        );
    }

    #[test]
    fn login_entry_runs_the_flatpak_hidden() {
        let entry = autostart_entry("io.github.lasr0.Blotter", &["--hidden"]);
        assert!(entry.contains("Exec=flatpak run io.github.lasr0.Blotter --hidden\n"));
        assert!(entry.contains("X-Flatpak=io.github.lasr0.Blotter\n"));
    }
}
