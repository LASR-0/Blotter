//! Command-line flags. They work on first launch and when forwarded from a
//! second launch to the running instance, which is how Wayland users bind
//! quick capture in their compositor (`blotter --capture`).

use serde::Serialize;

/// What a launch asks the app to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CliAction {
    /// Bring the main window forward.
    #[default]
    Show,
    /// Open the quick-capture popup.
    Capture,
    /// Open the AI chat.
    Chat,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LaunchArgs {
    pub action: CliAction,
    /// Start in the tray without showing a window (used by autostart).
    pub hidden: bool,
    /// Flags we didn't recognise, reported once on stderr.
    pub unknown: Vec<String>,
}

/// The subcommand that runs the MCP server instead of the app.
pub const MCP_SUBCOMMAND: &str = "mcp";

/// Whether this launch is `blotter mcp` (the subcommand must come first).
pub fn is_mcp<I, S>(argv: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    argv.into_iter()
        .nth(1)
        .is_some_and(|a| a.as_ref() == MCP_SUBCOMMAND)
}

/// Parses `argv` (including the program name at index 0). The first action
/// flag wins; repeated or conflicting ones after it are ignored.
pub fn parse<I, S>(argv: I) -> LaunchArgs
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut out = LaunchArgs::default();
    let mut action = None;
    for arg in argv.into_iter().skip(1) {
        match arg.as_ref() {
            "--capture" => {
                action.get_or_insert(CliAction::Capture);
            }
            "--chat" => {
                action.get_or_insert(CliAction::Chat);
            }
            "--hidden" | "--minimized" => out.hidden = true,
            other => out.unknown.push(other.to_owned()),
        }
    }
    out.action = action.unwrap_or_default();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_flags_means_show() {
        assert_eq!(parse(["blotter"]), LaunchArgs::default());
    }

    #[test]
    fn action_flags() {
        assert_eq!(parse(["blotter", "--capture"]).action, CliAction::Capture);
        assert_eq!(parse(["blotter", "--chat"]).action, CliAction::Chat);
        assert_eq!(
            parse(["blotter", "--chat", "--capture"]).action,
            CliAction::Chat,
            "first action wins"
        );
    }

    #[test]
    fn mcp_subcommand_only_in_first_place() {
        assert!(is_mcp(["blotter", "mcp"]));
        assert!(!is_mcp(["blotter"]));
        assert!(!is_mcp(["blotter", "--hidden", "mcp"]));
    }

    #[test]
    fn hidden_and_unknown() {
        let a = parse(["blotter", "--hidden", "--frobnicate"]);
        assert!(a.hidden);
        assert_eq!(a.action, CliAction::Show);
        assert_eq!(a.unknown, ["--frobnicate"]);
        assert!(parse(["blotter", "--minimized"]).hidden);
    }
}
