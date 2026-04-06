use anyhow::Result;
use clap::Args;

use crate::commands::{confirm_action, run_machine_command, AppContext};
use crate::tui::inline_search::{InlineSearchOutcome, SearchMode, SearchWidget};

#[derive(Debug, Clone, Args)]
pub struct OptionArgs {
    pub query: Option<String>,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: OptionArgs) -> Result<()> {
    let machine = ctx.machine(&args.on)?;
    let widget = SearchWidget::new(
        SearchMode::Options,
        args.query.unwrap_or_default(),
        ctx.config.animate,
    );

    match widget.run().await? {
        Some(InlineSearchOutcome::Copy { label, text }) => {
            crate::commands::copy_to_clipboard(&text)?;
            ctx.output
                .success(&format!("copied snippet for {label}  ♡"));
            Ok(())
        }
        Some(InlineSearchOutcome::AddOption {
            option_name,
            snippet,
        }) => apply_option(ctx, args.on, &machine, &option_name, &snippet).await,
        Some(InlineSearchOutcome::InstallPackage(_)) | Some(InlineSearchOutcome::TryPackage(_)) => {
            Ok(())
        }
        None => {
            ctx.output.face("see you  ♡");
            Ok(())
        }
    }
}

async fn apply_option(
    ctx: &AppContext,
    on: Option<String>,
    machine: &crate::machine::Machine,
    option_name: &str,
    snippet: &str,
) -> Result<()> {
    let config_path = format!("{}/configuration.nix", machine.config_dir);
    ctx.output.info(&format!(
        "adding {} to your config on {}...",
        option_name, machine.name
    ));

    if machine.is_local() {
        let path = std::path::Path::new(&config_path);
        if path.exists() {
            let original = crate::editor::read_contents(path)?;
            let preview =
                crate::editor::prepare_add_option_snippet(&original, option_name, snippet)?;
            ctx.output.diff(&preview.diff);
            if !preview.changed {
                ctx.output.success(&preview.diff);
                return Ok(());
            }
            if !confirm_action(
                ctx.config.confirm,
                "write this option to configuration.nix?",
            )? {
                ctx.output
                    .warn("okay, cancelled before editing your config ♡");
                return Ok(());
            }
            let backup = crate::editor::backup(path)?;
            crate::editor::write_contents(path, &preview.updated)?;
            ctx.output
                .success(&format!("added {} to {}", option_name, path.display()));

            if !confirm_action(ctx.config.confirm, "apply now?")? {
                ctx.output
                    .face("okay, the file is updated. you can run 'nina apply' later ♡");
                return Ok(());
            }

            let cmd = format!(
                "sudo nixos-rebuild switch -I nixos-config={}/configuration.nix",
                machine.config_dir
            );
            if let Err(err) = run_machine_command(
                ctx,
                &on,
                "patching it in",
                &cmd,
                "option",
                true,
            )
            .await
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
        }

        ctx.output
            .warn("configuration.nix not found at configured path");
        return Ok(());
    }

    let original = crate::commands::edit::fetch_remote_file(machine, &config_path).await?;
    let preview = crate::editor::prepare_add_option_snippet(&original, option_name, snippet)?;
    ctx.output.diff(&preview.diff);
    if !preview.changed {
        ctx.output.success(&preview.diff);
        return Ok(());
    }
    if !confirm_action(
        ctx.config.confirm,
        "write this option to the remote configuration?",
    )? {
        ctx.output
            .warn("okay, cancelled before editing your remote config ♡");
        return Ok(());
    }
    let backup_path = format!("{}.nina-backup", config_path);
    crate::exec::run(
        machine,
        &format!(
            "sudo cp {} {}",
            crate::commands::edit::shell_quote(&config_path),
            crate::commands::edit::shell_quote(&backup_path)
        ),
    )
    .await?;
    crate::commands::edit::upload_remote_file(machine, &config_path, &preview.updated).await?;
    ctx.output.success("remote configuration updated ♡");

    if !confirm_action(ctx.config.confirm, "apply now?")? {
        ctx.output
            .face("okay, the remote file is updated. you can run 'nina apply' later ♡");
        return Ok(());
    }

    let cmd = format!(
        "sudo nixos-rebuild switch -I nixos-config={}/configuration.nix",
        machine.config_dir
    );
    if let Err(err) = run_machine_command(
        ctx,
        &on,
        "patching it in",
        &cmd,
        "option",
        true,
    )
    .await
    {
        ctx.output
            .error("the build failed after editing the remote configuration");
        if confirm_action(ctx.config.confirm, "restore from remote backup?")? {
            crate::exec::run(
                machine,
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

    Ok(())
}
