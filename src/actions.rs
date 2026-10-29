//! Custom action execution engine and universal terminal resolver for Desktop Pet mode.

use std::path::Path;
use std::process::Command;
use tracing::{error, info, warn};

/// Checks if a command binary is in PATH or exists at the given path.
pub fn is_command_in_path(cmd: &str) -> bool {
    let p = Path::new(cmd);
    if p.is_file() {
        return true;
    }
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            if Path::new(&dir).join(cmd).is_file() {
                return true;
            }
        }
    }
    false
}

/// Detects the user's default shell from `$SHELL`, falling back to `"sh"`.
pub fn get_default_shell() -> String {
    if let Ok(shell) = std::env::var("SHELL") {
        let clean = shell.trim();
        if !clean.is_empty() {
            return clean.to_string();
        }
    }
    "sh".to_string()
}

/// Discovers an available graphical terminal emulator based on:
/// 1. Optional explicit override in `config.toml` (`terminal_override`)
/// 2. `$TERMINAL` environment variable
/// 3. Prioritized auto-detection across desktop native, modern/Wayland, and classic emulators.
pub fn get_configured_terminal() -> Option<String> {
    // 1. Explicit override in config.toml
    let cfg = crate::config::Config::load();
    if let Some(override_term) = cfg.terminal_override() {
        let clean = override_term.trim();
        if !clean.is_empty() && is_command_in_path(clean) {
            return Some(clean.to_string());
        }
    }

    // 2. $TERMINAL environment variable
    if let Ok(env_term) = std::env::var("TERMINAL") {
        let clean = env_term.trim();
        if !clean.is_empty() && is_command_in_path(clean) {
            return Some(clean.to_string());
        }
    }

    // 3. Prioritized candidate list
    const CANDIDATES: &[&str] = &[
        // Desktop native
        "konsole",
        "gnome-terminal",
        "ptyxis",
        "xfce4-terminal",
        // Modern / Wayland
        "alacritty",
        "ghostty",
        "foot",
        "kitty",
        "wezterm",
        // Classic fallbacks
        "xterm",
        "rxvt",
    ];

    for &candidate in CANDIDATES {
        if is_command_in_path(candidate) {
            return Some(candidate.to_string());
        }
    }

    None
}

/// Formats the command-line arguments for a given terminal emulator, shell, and command string.
pub fn format_terminal_arguments(terminal: &str, shell: &str, cmd_to_run: &str) -> Vec<String> {
    let term_base = Path::new(terminal)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(terminal);

    match term_base {
        "konsole" => vec![
            "-e".to_string(),
            shell.to_string(),
            "-c".to_string(),
            cmd_to_run.to_string(),
        ],
        "gnome-terminal" | "ptyxis" => vec![
            "--".to_string(),
            shell.to_string(),
            "-c".to_string(),
            cmd_to_run.to_string(),
        ],
        "alacritty" | "kitty" | "foot" | "ghostty" | "wezterm" | "xfce4-terminal" => vec![
            "-e".to_string(),
            shell.to_string(),
            "-c".to_string(),
            cmd_to_run.to_string(),
        ],
        "xterm" | "rxvt" => vec![
            "-e".to_string(),
            format!("{} -c '{}'", shell, cmd_to_run),
        ],
        _ => vec![
            "-e".to_string(),
            shell.to_string(),
            "-c".to_string(),
            cmd_to_run.to_string(),
        ],
    }
}

/// Resolves the graphical terminal emulator binary and execution arguments for `cmd_to_run`.
pub fn resolve_terminal_command(cmd_to_run: &str) -> Option<(String, Vec<String>)> {
    let terminal = get_configured_terminal()?;
    let shell = get_default_shell();
    let args = format_terminal_arguments(&terminal, &shell, cmd_to_run);
    Some((terminal, args))
}

/// Notifies about a missing terminal emulator via log warning and Desktop Pet speech bubble.
pub fn notify_terminal_error() {
    let msg = "Couldn't find an installed terminal emulator to run this action!";
    warn!("{}", msg);
    let _ = crate::ipc::send_pet_command(&format!("say {}", msg));
}

/// Launches an interactive command inside an automatically detected graphical terminal emulator.
/// Appends a graceful completion pause to keep the window open until Enter is pressed.
pub fn launch_in_terminal(command_str: &str) -> bool {
    let shell = get_default_shell();
    let is_fish = shell.ends_with("fish");
    let shell_script = if is_fish {
        format!(
            "{}; echo ''; echo 'Done! Press Enter to close...'; read -l dummy",
            command_str
        )
    } else {
        format!(
            "{}; echo ''; echo 'Done! Press Enter to close...'; read dummy",
            command_str
        )
    };

    if let Some((term, args)) = resolve_terminal_command(&shell_script) {
        info!("Spawning terminal '{}' with args: {:?}", term, args);
        match Command::new(&term).args(&args).spawn() {
            Ok(mut child) => {
                info!(
                    "Successfully spawned terminal {} with command: {}",
                    term, command_str
                );
                std::thread::spawn(move || {
                    let _ = child.wait();
                });
                true
            }
            Err(e) => {
                warn!("Failed to spawn terminal '{}': {}", term, e);
                notify_terminal_error();
                false
            }
        }
    } else {
        warn!("Couldn't find an installed terminal emulator to run this action!");
        notify_terminal_error();
        false
    }
}

/// Executes a custom pet action.
/// Intelligently resolves terminal wrappers (e.g. `kitty -e paccache -rk2`),
/// substitutes missing terminal binaries in chained commands, or executes background GUI commands.
pub fn execute_action(command_str: &str) {
    let clean = command_str.trim();
    if clean.is_empty() {
        return;
    }

    info!("Executing custom pet action: {}", clean);

    // 1. Check for terminal wrappers: `<term> -e <cmd>` or `<term> -- <cmd>`
    let term_prefixes = [
        "kitty -e ",
        "kitty -- ",
        "konsole -e ",
        "alacritty -e ",
        "gnome-terminal -- ",
        "ptyxis -- ",
        "foot -e ",
        "ghostty -e ",
        "wezterm -e ",
        "xfce4-terminal -e ",
        "xterm -e ",
        "rxvt -e ",
    ];

    for prefix in term_prefixes {
        if let Some(stripped) = clean.strip_prefix(prefix) {
            let inner = stripped.trim();
            info!(
                "Detected terminal action wrapper '{}', launching inner command in resolved terminal: {}",
                prefix.trim(),
                inner
            );
            launch_in_terminal(inner);
            return;
        }
    }

    // 2. Check if the command starts with a terminal binary that is NOT installed
    let first_word = clean.split_whitespace().next().unwrap_or("");
    let known_terminals = [
        "kitty", "konsole", "alacritty", "gnome-terminal", "ptyxis",
        "foot", "ghostty", "wezterm", "xfce4-terminal", "xterm", "rxvt",
    ];

    if known_terminals.contains(&first_word) && !is_command_in_path(first_word) {
        if let Some(available_term) = get_configured_terminal() {
            let replaced = clean.replacen(first_word, &available_term, 1);
            info!(
                "Substituted uninstalled terminal '{}' with '{}': {}",
                first_word, available_term, replaced
            );
            dispatch_background_shell(&replaced);
            return;
        } else {
            notify_terminal_error();
            return;
        }
    }

    // 3. Check for standalone chained terminal calls like "firefox & kitty"
    if clean.contains("& kitty") && !is_command_in_path("kitty") {
        if let Some(available_term) = get_configured_terminal() {
            let replaced = clean.replace("& kitty", &format!("& {}", available_term));
            info!(
                "Substituted uninstalled terminal 'kitty' with '{}': {}",
                available_term, replaced
            );
            dispatch_background_shell(&replaced);
            return;
        }
    }

    // 4. Default dispatch: spawn via shell in background
    dispatch_background_shell(clean);
}

fn dispatch_background_shell(clean: &str) {
    let shell = get_default_shell();
    match Command::new(&shell)
        .args(["-c", clean])
        .spawn()
    {
        Ok(mut child) => {
            info!("Successfully dispatched action in background via {}: {}", shell, clean);
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        }
        Err(e) => {
            error!("Failed to dispatch action '{}': {}", clean, e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_empty_action() {
        execute_action("");
        execute_action("   ");
    }

    #[test]
    fn test_execute_valid_action() {
        execute_action("true");
    }

    #[test]
    fn test_get_default_shell() {
        let shell = get_default_shell();
        assert!(!shell.is_empty());
    }

    #[test]
    fn test_format_terminal_arguments() {
        let konsole_args = format_terminal_arguments("konsole", "/bin/bash", "ls -la");
        assert_eq!(konsole_args, vec!["-e", "/bin/bash", "-c", "ls -la"]);

        let gnome_args = format_terminal_arguments("gnome-terminal", "/bin/bash", "ls -la");
        assert_eq!(gnome_args, vec!["--", "/bin/bash", "-c", "ls -la"]);

        let ptyxis_args = format_terminal_arguments("ptyxis", "/bin/fish", "htop");
        assert_eq!(ptyxis_args, vec!["--", "/bin/fish", "-c", "htop"]);

        let alacritty_args = format_terminal_arguments("alacritty", "/bin/zsh", "btop");
        assert_eq!(alacritty_args, vec!["-e", "/bin/zsh", "-c", "btop"]);

        let kitty_args = format_terminal_arguments("kitty", "/bin/bash", "paccache -rk2");
        assert_eq!(kitty_args, vec!["-e", "/bin/bash", "-c", "paccache -rk2"]);

        let xterm_args = format_terminal_arguments("xterm", "sh", "echo hi");
        assert_eq!(xterm_args, vec!["-e", "sh -c 'echo hi'"]);
    }

    #[test]
    fn test_resolve_terminal_command() {
        // If at least one terminal exists in PATH (e.g. konsole on Arch), resolution succeeds
        if let Some((term, args)) = resolve_terminal_command("echo hello") {
            assert!(!term.is_empty());
            assert!(!args.is_empty());
            assert!(args.iter().any(|a| a == "echo hello" || a.contains("echo hello")));
        }
    }

    #[test]
    fn test_resolve_pacman_cache_cleanup() {
        let resolved = resolve_terminal_command("paccache -rk2");
        assert!(resolved.is_some());
        let (term, args) = resolved.unwrap();
        assert_eq!(term, "konsole");
        assert_eq!(args[0], "-e");
        assert!(args[1].contains("fish") || args[1].contains("bash") || args[1].contains("sh"));
        assert_eq!(args[2], "-c");
        assert_eq!(args[3], "paccache -rk2");
    }

    #[test]
    fn test_format_terminal_arguments_all() {
        assert_eq!(
            format_terminal_arguments("alacritty", "/bin/fish", "true"),
            vec!["-e", "/bin/fish", "-c", "true"]
        );
        assert_eq!(
            format_terminal_arguments("foot", "/bin/bash", "true"),
            vec!["-e", "/bin/bash", "-c", "true"]
        );
        assert_eq!(
            format_terminal_arguments("ghostty", "/bin/bash", "true"),
            vec!["-e", "/bin/bash", "-c", "true"]
        );
        assert_eq!(
            format_terminal_arguments("wezterm", "/bin/bash", "true"),
            vec!["-e", "/bin/bash", "-c", "true"]
        );
        assert_eq!(
            format_terminal_arguments("gnome-terminal", "/bin/bash", "true"),
            vec!["--", "/bin/bash", "-c", "true"]
        );
        assert_eq!(
            format_terminal_arguments("ptyxis", "/bin/bash", "true"),
            vec!["--", "/bin/bash", "-c", "true"]
        );
        assert_eq!(
            format_terminal_arguments("xterm", "/bin/bash", "true"),
            vec!["-e", "/bin/bash -c 'true'"]
        );
    }

    #[test]
    fn test_execute_action_terminal_wrapper() {
        // Dispatches without panicking
        execute_action("kitty -e true");
    }
}
