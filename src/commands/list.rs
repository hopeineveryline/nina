use anyhow::Result;
use clap::Args;

use crate::commands::AppContext;

#[derive(Debug, Clone, Args)]
pub struct ListArgs {
    #[arg(long)]
    pub on: Option<String>,
    #[arg(long)]
    pub grep: Option<String>,
}

pub async fn run(ctx: &AppContext, args: ListArgs) -> Result<()> {
    let machine = ctx.machine(&args.on)?;
    let path = std::path::Path::new(&machine.config_dir).join("configuration.nix");
    let mut packages = if machine.is_local() {
        if !path.exists() {
            ctx.output
                .warn("configuration.nix not found for this machine");
            return Ok(());
        }
        crate::editor::list_packages(&path)?
    } else {
        let remote_path = format!("{}/configuration.nix", machine.config_dir);
        let raw = crate::commands::edit::fetch_remote_file(&machine, &remote_path).await?;
        let temp = std::env::temp_dir().join("nina-remote-list.nix");
        crate::editor::write_contents(&temp, &raw)?;
        let packages = crate::editor::list_packages(&temp)?;
        let _ = std::fs::remove_file(temp);
        packages
    };

    if let Some(needle) = args.grep {
        packages.retain(|p| p.contains(&needle));
    }

    if packages.is_empty() {
        ctx.output
            .face("no packages found in environment.systemPackages yet.");
        return Ok(());
    }

    ctx.output
        .info("here's what i found in environment.systemPackages:");
    let enriched = crate::packages::enrich_packages(packages).await;
    for (package, meta) in enriched {
        if let Some(pkg) = meta {
            let version = pkg.version.unwrap_or_else(|| "unknown".to_string());
            let description = pkg
                .description
                .unwrap_or_else(|| "no description from nix search".to_string());
            ctx.output
                .print(&format!("  - {package:<24} {version:<12} {description}"));
        } else {
            ctx.output.print(&format!("  - {package}"));
        }
    }
    Ok(())
}
