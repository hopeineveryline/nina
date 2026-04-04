use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use clap::Args;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::commands::{confirm_action, AppContext};

#[derive(Debug, Clone, Args)]
pub struct EditArgs {
    pub target: Option<String>,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: EditArgs) -> Result<()> {
    let machine = ctx.machine(&args.on)?;
    let file_name = if matches!(args.target.as_deref(), Some("hardware")) {
        "hardware-configuration.nix"
    } else {
        "configuration.nix"
    };

    let target_path = format!("{}/{}", machine.config_dir, file_name);
    if machine.is_local() {
        edit_local(ctx, &machine.config_dir, file_name).await?;
    } else {
        edit_remote(ctx, &machine, &target_path).await?;
    }

    if confirm_action(ctx.config.confirm, "run nina check now?")? {
        crate::commands::check::run(ctx, crate::commands::check::CheckArgs { on: args.on }).await?;
    }

    Ok(())
}

async fn edit_local(ctx: &AppContext, config_dir: &str, file_name: &str) -> Result<()> {
    let path = Path::new(config_dir).join(file_name);
    let cmd = format!("{} {}", ctx.config.editor, path.display());
    crate::exec::run_local(&cmd, |_, _| {}).await?;
    Ok(())
}

async fn edit_remote(
    ctx: &AppContext,
    machine: &crate::machine::Machine,
    remote_path: &str,
) -> Result<()> {
    ctx.output.info(&format!(
        "syncing {} from {} for local editing...",
        remote_path, machine.name
    ));
    let bytes = fetch_remote_file(machine, remote_path).await?;
    let temp_path = temp_edit_path(remote_path)?;
    std::fs::write(&temp_path, bytes.as_bytes())
        .with_context(|| format!("couldn't write temp file {}", temp_path.display()))?;

    let before = std::fs::read(&temp_path)?;
    let cmd = format!("{} {}", ctx.config.editor, temp_path.display());
    crate::exec::run_local(&cmd, |_, _| {}).await?;
    let after = std::fs::read(&temp_path)?;

    if before != after {
        ctx.output
            .step("local copy changed, uploading it back to the remote machine...");
        upload_remote_file(machine, remote_path, &String::from_utf8_lossy(&after)).await?;
        ctx.output.success("remote file updated ♡");
    } else {
        ctx.output
            .face("no changes detected, so i left the remote file alone ♡");
    }

    let _ = std::fs::remove_file(temp_path);
    Ok(())
}

pub(crate) async fn fetch_remote_file(
    machine: &crate::machine::Machine,
    remote_path: &str,
) -> Result<String> {
    let fetch_cmd = format!("base64 < {}", shell_quote(remote_path));
    let fetched = crate::exec::run(machine, &fetch_cmd).await?;
    if !fetched.success() {
        anyhow::bail!("couldn't fetch remote file: {}", fetched.stderr);
    }

    let bytes = STANDARD
        .decode(fetched.stdout.trim())
        .context("couldn't decode remote file payload")?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

pub(crate) async fn upload_remote_file(
    machine: &crate::machine::Machine,
    remote_path: &str,
    contents: &str,
) -> Result<()> {
    let encoded = STANDARD.encode(contents.as_bytes());
    let temp_path = format!(
        "/tmp/nina-upload-{}.tmp",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("time went backwards")?
            .as_nanos()
    );
    let upload_cmd = format!(
        "cat <<'EOF' | base64 -d > {tmp}\n{payload}\nEOF\nsudo mv {tmp} {path}",
        tmp = shell_quote(&temp_path),
        path = shell_quote(remote_path),
        payload = encoded
    );
    let uploaded = crate::exec::run(machine, &upload_cmd).await?;
    if !uploaded.success() {
        anyhow::bail!("couldn't upload updated file: {}", uploaded.stderr);
    }
    Ok(())
}

fn temp_edit_path(remote_path: &str) -> Result<PathBuf> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("time went backwards")?
        .as_secs();
    let file_name = Path::new(remote_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("configuration.nix");
    Ok(std::env::temp_dir().join(format!("nina-{stamp}-{file_name}")))
}

pub(crate) fn shell_quote(path: &str) -> String {
    format!("'{}'", path.replace('\'', "'\\''"))
}
