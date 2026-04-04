pub mod apply;
pub mod back;
pub mod boot;
pub mod build_cmd;
pub mod channel;
pub mod check;
pub mod clean;
pub mod develop;
pub mod diff;
pub mod doctor;
pub mod edit;
pub mod fetch;
pub mod flake;
pub mod fmt;
pub mod gen;
pub mod go;
pub mod hash;
pub mod hello;
pub mod help;
pub mod history;
pub mod info;
pub mod install;
pub mod install_menu;
pub mod list;
pub mod log_cmd;
pub mod mood;
pub mod option;
pub mod pin;
pub mod pkg;
pub mod profile;
pub mod remove;
pub mod repl;
pub mod run_cmd;
pub mod search;
pub mod service;
pub mod status;
pub mod store;
pub mod try_pkg;
pub mod update;
pub mod upgrade;

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use crossterm::terminal;
use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::time::Instant;

use crate::config::NinaConfig;
use crate::errors::translate_nix_error;
use crate::log::{append, OperationLogEntry};
use crate::machine::{resolve_machine, Machine};
use crate::output::Output;

#[derive(Debug, Clone)]
pub struct AppContext {
    pub config: NinaConfig,
    pub output: Output,
}

impl AppContext {
    pub fn new(config: NinaConfig) -> Self {
        let output = Output::new(config.color, config.teach);
        Self { config, output }
    }

    pub fn machine(&self, on: &Option<String>) -> Result<Machine> {
        resolve_machine(&self.config, on.as_deref())
    }
}

pub async fn run_machine_command(
    ctx: &AppContext,
    on: &Option<String>,
    start_message: &str,
    command_line: &str,
    command_name: &str,
    destructive: bool,
) -> Result<()> {
    let machine = ctx.machine(on)?;
    maybe_warn_about_dev_shell(ctx, &machine, command_name, command_line);
    ctx.output
        .info(&format!("{} on {}...", start_message, machine.name));
    ctx.output.step(&format!("{}", command_line));

    if destructive && !confirm_if_needed(ctx)? {
        ctx.output.warn("okay, cancelled with no changes ♡");
        return Ok(());
    }

    let dango = if ctx.config.animate && io::stdout().is_terminal() {
        terminal::size().ok().map(|(width, height)| {
            let pos = crate::dango::position_from_pref(&ctx.config.dango_pos, width, height);
            crate::dango::DangoPlayer::start(animation_for(command_name), pos)
        })
    } else {
        None
    };

    let gen_before = current_generation(&machine).await.ok().flatten();
    let started = Instant::now();
    let result = crate::exec::run_with_stream(&machine, command_line, |is_stderr, line| {
        if is_stderr {
            eprintln!("{}", line);
        } else {
            ctx.output.step(line);
        }
    })
    .await;
    let elapsed = started.elapsed().as_millis() as u64;
    let gen_after = current_generation(&machine).await.ok().flatten();

    if let Some(player) = dango {
        player.stop().await;
    }

    match result {
        Ok(output) => {
            if output.success() {
                play_reaction(ctx, crate::dango::DangoAnimation::Happy);
                ctx.output.success("done! ♡");
                if let (Some(before), Some(after)) = (gen_before, gen_after) {
                    if before != after {
                        ctx.output
                            .step(&format!("generation moved from {} → {}", before, after));
                    }
                }
                ctx.output.teach_command(command_line);
                append_log(
                    &machine.name,
                    command_name,
                    "success",
                    elapsed,
                    gen_before,
                    gen_after,
                )?;
                return Ok(());
            }

            let friendly = translate_nix_error(&output.stderr);
            play_reaction(ctx, crate::dango::DangoAnimation::Sad);
            ctx.output.error(&friendly.summary);
            if !friendly.detail.trim().is_empty() {
                eprintln!("{}", friendly.detail);
            }
            ctx.output.warn(&friendly.suggestion);
            append_log(
                &machine.name,
                command_name,
                "failure",
                elapsed,
                gen_before,
                gen_after,
            )?;
            Err(anyhow!("command failed"))
        }
        Err(err) => {
            append_log(
                &machine.name,
                command_name,
                "failure",
                elapsed,
                gen_before,
                gen_after,
            )
            .context("failed to write operation log")?;
            Err(err)
        }
    }
}

pub async fn run_attached_machine_command(
    ctx: &AppContext,
    on: &Option<String>,
    start_message: &str,
    command_line: &str,
    command_name: &str,
    destructive: bool,
) -> Result<()> {
    let machine = ctx.machine(on)?;
    maybe_warn_about_dev_shell(ctx, &machine, command_name, command_line);
    ctx.output
        .info(&format!("{} on {}...", start_message, machine.name));
    ctx.output.step(command_line);

    if destructive && !confirm_if_needed(ctx)? {
        ctx.output.warn("okay, cancelled with no changes ♡");
        return Ok(());
    }

    let gen_before = current_generation(&machine).await.ok().flatten();
    let started = Instant::now();
    let status = crate::exec::run_attached(&machine, command_line).await?;
    let elapsed = started.elapsed().as_millis() as u64;
    let gen_after = current_generation(&machine).await.ok().flatten();

    if status == 0 {
        ctx.output.success("done! ♡");
        if let (Some(before), Some(after)) = (gen_before, gen_after) {
            if before != after {
                ctx.output
                    .step(&format!("generation moved from {} → {}", before, after));
            }
        }
        ctx.output.teach_command(command_line);
        append_log(
            &machine.name,
            command_name,
            "success",
            elapsed,
            gen_before,
            gen_after,
        )?;
        return Ok(());
    }

    append_log(
        &machine.name,
        command_name,
        "failure",
        elapsed,
        gen_before,
        gen_after,
    )?;
    Err(anyhow!("command failed"))
}

fn animation_for(command_name: &str) -> crate::dango::DangoAnimation {
    match command_name {
        "clean" => crate::dango::DangoAnimation::Sweep,
        "back" => crate::dango::DangoAnimation::WalkBack,
        "try" => crate::dango::DangoAnimation::Wave,
        _ => crate::dango::DangoAnimation::Idle,
    }
}

fn play_reaction(ctx: &AppContext, animation: crate::dango::DangoAnimation) {
    if !(ctx.config.animate && io::stdout().is_terminal()) {
        return;
    }

    if let Ok((width, height)) = terminal::size() {
        let pos = crate::dango::position_from_pref(&ctx.config.dango_pos, width, height);
        let _ = crate::dango::DangoPlayer::play_once(animation, pos);
    }
}

pub fn confirm_action(confirm_enabled: bool, prompt: &str) -> Result<bool> {
    if !confirm_enabled {
        return Ok(true);
    }

    print!("  {} (y/n) > ", prompt);
    io::stdout().flush().context("failed to flush prompt")?;

    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .context("failed to read confirmation")?;

    Ok(matches!(answer.trim(), "y" | "Y" | "yes" | "YES"))
}

pub fn package_attr_for_config(package: &str) -> String {
    if package.starts_with("pkgs.") || package.contains('.') {
        package.to_string()
    } else {
        format!("pkgs.{package}")
    }
}

pub fn package_shell_ref(package: &str) -> String {
    package.strip_prefix("pkgs.").unwrap_or(package).to_string()
}

pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub fn current_dir_command(command: &str) -> Result<String> {
    let cwd = std::env::current_dir().context("couldn't determine the current directory")?;
    let cwd = cwd.display().to_string();
    Ok(format!("cd {} && {}", shell_quote(&cwd), command))
}

pub fn current_dir_command_for(machine: &Machine, command: &str) -> Result<String> {
    if machine.is_local() {
        current_dir_command(command)
    } else {
        Ok(command.to_string())
    }
}

pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new().context("clipboard isn't available")?;
    clipboard
        .set_text(text.to_string())
        .context("couldn't copy to clipboard")
}

fn append_log(
    machine: &str,
    command: &str,
    outcome: &str,
    duration_ms: u64,
    gen_before: Option<u32>,
    gen_after: Option<u32>,
) -> Result<()> {
    let entry = OperationLogEntry {
        ts: Utc::now(),
        machine: machine.to_string(),
        command: command.to_string(),
        outcome: outcome.to_string(),
        gen_before,
        gen_after,
        duration_ms,
    };
    append(&entry)
}

pub(crate) async fn current_generation(machine: &Machine) -> Result<Option<u32>> {
    let output = crate::exec::run(
        machine,
        "nix-env --list-generations -p /nix/var/nix/profiles/system | awk '/\\(current\\)/ {print $1}'",
    )
    .await?;
    if !output.success() {
        return Ok(None);
    }
    Ok(output.stdout.trim().parse::<u32>().ok())
}

fn confirm_if_needed(ctx: &AppContext) -> Result<bool> {
    confirm_action(ctx.config.confirm, "are you sure?")
}

fn maybe_warn_about_dev_shell(
    ctx: &AppContext,
    machine: &Machine,
    command_name: &str,
    command_line: &str,
) {
    if !machine.is_local() || std::env::var_os("IN_NIX_SHELL").is_some() {
        return;
    }
    if !Path::new("flake.nix").exists() {
        return;
    }
    let mentions_tool = command_name == "apply"
        || command_name == "build"
        || ["cargo", "rustc", "python"]
            .iter()
            .any(|needle| command_line.contains(needle));
    if mentions_tool {
        ctx.output
            .face("this project has a dev shell — run nina develop if you hit build errors ♡");
    }
}

#[cfg(test)]
mod tests {
    use super::current_dir_command_for;
    use crate::machine::{Machine, MachineKind};

    #[test]
    fn current_dir_command_for_remote_skips_local_cd() {
        let machine = Machine {
            name: "remote".to_string(),
            kind: MachineKind::Remote {
                host: "example.com".to_string(),
                port: 22,
                user: Some("admin".to_string()),
            },
            config_dir: "/etc/nixos".to_string(),
            ssh_key: None,
        };

        assert_eq!(
            current_dir_command_for(&machine, "nix develop").expect("build command"),
            "nix develop"
        );
    }
}
