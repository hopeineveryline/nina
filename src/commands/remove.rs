use anyhow::Result;
use clap::Args;

use crate::commands::{confirm_action, package_attr_for_config, run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct RemoveArgs {
    pub package: String,
    #[arg(long)]
    pub on: Option<String>,
    #[arg(long)]
    pub no_apply: bool,
}

pub async fn run(ctx: &AppContext, args: RemoveArgs) -> Result<()> {
    let machine = ctx.machine(&args.on)?;
    let config_path = format!("{}/configuration.nix", machine.config_dir);
    let package_attr = package_attr_for_config(&args.package);

    if machine.is_local() {
        let path = std::path::Path::new(&config_path);
        if path.exists() {
            let original = crate::editor::read_contents(path)?;
            let preview = crate::editor::prepare_remove_package(&original, &package_attr)?;
            ctx.output.diff(&preview.diff);
            if !preview.changed {
                ctx.output.warn(&preview.diff);
                return Ok(());
            }
            if !confirm_action(
                ctx.config.confirm,
                "write this change to configuration.nix?",
            )? {
                ctx.output
                    .happy("okay, cancelled before editing your config ♡");
                return Ok(());
            }
            let backup = crate::editor::backup(path)?;
            crate::editor::write_contents(path, &preview.updated)?;
            let msg = format!("removed {} from {}", package_attr, path.display());
            ctx.output.success(&msg);

            if args.no_apply {
                ctx.output
                    .happy("okay, removed from config and skipped rebuild ♡");
                return Ok(());
            }

            if !confirm_action(ctx.config.confirm, "apply now?")? {
                ctx.output
                    .happy("okay, the file is updated. you can run 'nina apply' later ♡");
                return Ok(());
            }

            let cmd = format!(
                "sudo nixos-rebuild switch -I nixos-config={}/configuration.nix",
                machine.config_dir
            );
            if let Err(err) =
                run_machine_command(ctx, &args.on, "patching it in", &cmd, "remove", true).await
            {
                ctx.output
                    .error("the rebuild stumbled — i can restore from backup if you like");
                if confirm_action(ctx.config.confirm, "restore from backup?")? {
                    crate::editor::restore(path, &backup)?;
                    ctx.output
                        .rollback("restored configuration.nix from backup ♡");
                }
                return Err(err);
            }
            return Ok(());
        } else {
            ctx.output
                .warn("configuration.nix not found at configured path");
        }
    } else {
        let original = crate::commands::edit::fetch_remote_file(&machine, &config_path).await?;
        let preview = crate::editor::prepare_remove_package(&original, &package_attr)?;
        ctx.output.diff(&preview.diff);
        if !preview.changed {
            ctx.output.warn(&preview.diff);
            return Ok(());
        }
        if !confirm_action(
            ctx.config.confirm,
            "write this change to the remote configuration?",
        )? {
            ctx.output
                .happy("okay, cancelled before editing your remote config ♡");
            return Ok(());
        }
        let backup_path = format!("{}.nina-backup", config_path);
        crate::exec::run(
            &machine,
            &format!(
                "sudo cp {} {}",
                crate::commands::edit::shell_quote(&config_path),
                crate::commands::edit::shell_quote(&backup_path)
            ),
        )
        .await?;
        crate::commands::edit::upload_remote_file(&machine, &config_path, &preview.updated).await?;
        ctx.output.success("remote configuration updated");

        if args.no_apply {
            ctx.output
                .happy("okay, removed from config and skipped rebuild ♡");
            return Ok(());
        }

        if !confirm_action(ctx.config.confirm, "apply now?")? {
            ctx.output
                .happy("okay, the remote file is updated. you can run 'nina apply' later ♡");
            return Ok(());
        }

        let cmd = format!(
            "sudo nixos-rebuild switch -I nixos-config={}/configuration.nix",
            machine.config_dir
        );
        if let Err(err) =
            run_machine_command(ctx, &args.on, "patching it in", &cmd, "remove", true).await
        {
            ctx.output
                .error("the rebuild stumbled on the remote machine — i can restore from backup if you like");
            if confirm_action(ctx.config.confirm, "restore from remote backup?")? {
                crate::exec::run(
                    &machine,
                    &format!(
                        "sudo cp {} {}",
                        crate::commands::edit::shell_quote(&backup_path),
                        crate::commands::edit::shell_quote(&config_path)
                    ),
                )
                .await?;
                ctx.output
                    .rollback("restored the remote configuration from backup ♡");
            }
            return Err(err);
        }
        return Ok(());
    }

    Ok(())
}
