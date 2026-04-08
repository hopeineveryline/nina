use anyhow::{anyhow, Result};
use clap::Args;

use crate::commands::install_menu::InstallMenuChoice;
use crate::commands::{confirm_action, package_attr_for_config, run_machine_command, AppContext};
use crate::packages::{NixPackage, PackageResolution};

enum InstallSelectionState {
    Resolved(PackageResolution),
    Cancelled,
    Unresolved,
}

#[derive(Debug, Clone, Args)]
pub struct InstallArgs {
    pub package: String,
    #[arg(long)]
    pub on: Option<String>,
    #[arg(long)]
    pub no_apply: bool,
}

pub async fn run(ctx: &AppContext, args: InstallArgs) -> Result<()> {
    let machine = ctx.machine(&args.on)?;
    let config_path = format!("{}/configuration.nix", machine.config_dir);
    let resolution = match resolve_install_selection(&args.package, &machine.name).await? {
        InstallSelectionState::Resolved(resolution) => resolution,
        InstallSelectionState::Cancelled => {
            ctx.output.happy("okay, no package picked ♡");
            return Ok(());
        }
        InstallSelectionState::Unresolved => {
            let suggestions = crate::packages::resolve_package(&args.package)
                .await?
                .map(|resolved| {
                    resolved
                        .suggestions
                        .into_iter()
                        .take(3)
                        .map(|pkg| crate::commands::package_attr_for_config(&pkg.attribute))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let extra = if suggestions.is_empty() {
                String::new()
            } else {
                format!(" closest matches: {}", suggestions.join(", "))
            };
            return Err(anyhow!(
                "i couldn't find an exact nixpkgs match for '{}'. try: nina search {}.{}",
                args.package,
                args.package,
                extra
            ));
        }
    };
    let package_attr = package_attr_for_config(if resolution.exact.attribute.is_empty() {
        &args.package
    } else {
        &resolution.exact.attribute
    });
    ctx.output.info(&format!(
        "adding {} to your config on {}...",
        package_attr, machine.name
    ));
    if !resolution.suggestions.is_empty() {
        let suggestions = resolution
            .suggestions
            .iter()
            .take(3)
            .map(|pkg| crate::commands::package_attr_for_config(&pkg.attribute))
            .collect::<Vec<_>>();
        ctx.output
            .step(&format!("closest matches: {}", suggestions.join(", ")));
    }

    if machine.is_local() {
        let path = std::path::Path::new(&config_path);
        if path.exists() {
            let original = crate::editor::read_contents(path)?;
            let preview = crate::editor::prepare_add_package(&original, &package_attr)?;
            ctx.output.diff(&preview.diff);
            if !preview.changed {
                ctx.output.success(&preview.diff);
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
            let msg = format!("added {} to {}", package_attr, path.display());
            ctx.output.success(&msg);

            if args.no_apply {
                ctx.output
                    .happy("all set, i skipped rebuild because --no-apply was set ♡");
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
                run_machine_command(ctx, &args.on, "patching it in", &cmd, "install", true).await
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
        let preview = crate::editor::prepare_add_package(&original, &package_attr)?;
        ctx.output.diff(&preview.diff);
        if !preview.changed {
            ctx.output.success(&preview.diff);
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
                .happy("all set, i skipped rebuild because --no-apply was set ♡");
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
            run_machine_command(ctx, &args.on, "patching it in", &cmd, "install", true).await
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

async fn resolve_install_selection(query: &str, machine: &str) -> Result<InstallSelectionState> {
    if let Some(resolution) = crate::packages::resolve_exact_package(query).await? {
        return Ok(InstallSelectionState::Resolved(resolution));
    }

    match crate::commands::install_menu::choose_package(query, machine).await? {
        InstallMenuChoice::Selected(selection) => Ok(InstallSelectionState::Resolved(
            selection_to_resolution(selection.exact, selection.suggestions),
        )),
        InstallMenuChoice::Cancelled => Ok(InstallSelectionState::Cancelled),
        InstallMenuChoice::Unavailable => Ok(InstallSelectionState::Unresolved),
    }
}

fn selection_to_resolution(exact: NixPackage, suggestions: Vec<NixPackage>) -> PackageResolution {
    PackageResolution { exact, suggestions }
}
