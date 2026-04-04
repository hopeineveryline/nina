use anyhow::Result;
use clap::Args;

use crate::commands::AppContext;
use crate::tui::inline_search::{InlineSearchOutcome, SearchMode, SearchWidget};

#[derive(Debug, Clone, Args)]
pub struct SearchArgs {
    pub query: Option<String>,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: SearchArgs) -> Result<()> {
    let widget = SearchWidget::new(
        SearchMode::Packages,
        args.query.unwrap_or_default(),
        ctx.config.animate,
        ctx.config.dango_pos.clone(),
    );

    match widget.run().await? {
        Some(InlineSearchOutcome::InstallPackage(package)) => {
            crate::commands::install::run(
                ctx,
                crate::commands::install::InstallArgs {
                    package,
                    on: args.on,
                    no_apply: false,
                },
            )
            .await
        }
        Some(InlineSearchOutcome::TryPackage(package)) => {
            crate::commands::try_pkg::run(
                ctx,
                crate::commands::try_pkg::TryPkgArgs {
                    packages: vec![package],
                    on: args.on,
                },
            )
            .await
        }
        Some(InlineSearchOutcome::Copy { label, text }) => {
            crate::commands::copy_to_clipboard(&text)?;
            ctx.output
                .success(&format!("copied '{label}' to clipboard  ♡"));
            Ok(())
        }
        Some(InlineSearchOutcome::AddOption { .. }) => Ok(()),
        None => {
            ctx.output.face("see you  ♡");
            Ok(())
        }
    }
}
