use anyhow::Result;
use clap::Args;

use crate::commands::AppContext;

#[derive(Debug, Clone, Args)]
pub struct GcArgs {
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: GcArgs) -> Result<()> {
    crate::commands::clean::run(
        ctx,
        crate::commands::clean::CleanArgs {
            on: args.on,
            all: true,
        },
    )
    .await
}
