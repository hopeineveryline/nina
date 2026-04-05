mod commands;
mod config;
mod dango;
mod debug;
mod editor;
mod errors;
mod exec;
mod log;
mod machine;
mod options;
mod output;
mod packages;
mod session;
mod tui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::AppContext;

#[derive(Parser, Debug)]
#[command(
    name = "nina",
    version,
    about = "nina, your friendly nix helper ♡",
    long_about = "(˶ᵔ ᵕ ᵔ˶) nina translates nixos workflows into friendly terminal commands.",
    disable_help_subcommand = true
)]
struct Cli {
    #[arg(long, global = true)]
    debug: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    Apply(commands::apply::ApplyArgs),
    Back(commands::back::BackArgs),
    Build(commands::build_cmd::BuildArgs),
    Boot(commands::boot::BootArgs),
    Channel(commands::channel::ChannelArgs),
    History(commands::history::HistoryArgs),
    Go(commands::go::GoArgs),
    Clean(commands::clean::CleanArgs),
    Develop(commands::develop::DevelopArgs),
    Search(commands::search::SearchArgs),
    Install(commands::install::InstallArgs),
    Remove(commands::remove::RemoveArgs),
    Try(commands::try_pkg::TryPkgArgs),
    List(commands::list::ListArgs),
    Fetch(commands::fetch::FetchArgs),
    Flake(commands::flake::FlakeArgs),
    Fmt(commands::fmt::FmtArgs),
    Gen(commands::gen::GenArgs),
    Hash(commands::hash::HashArgs),
    Edit(commands::edit::EditArgs),
    Check(commands::check::CheckArgs),
    Diff(commands::diff::DiffArgs),
    Info(commands::info::InfoArgs),
    #[command(name = "option")]
    OptionCmd(commands::option::OptionArgs),
    Pin(commands::pin::PinArgs),
    #[command(name = "unpin")]
    Unpin(commands::pin::UnpinArgs),
    Pkg(commands::pkg::PkgArgs),
    Profile(commands::profile::ProfileArgs),
    Repl(commands::repl::ReplArgs),
    #[command(name = "run")]
    RunCmd(commands::run_cmd::RunArgs),
    Service(commands::service::ServiceArgs),
    Status(commands::status::StatusArgs),
    Store(commands::store::StoreArgs),
    Update(commands::update::UpdateArgs),
    Upgrade(commands::upgrade::UpgradeArgs),
    Log(commands::log_cmd::LogCmdArgs),
    Doctor(commands::doctor::DoctorArgs),
    #[command(name = "help")]
    Help(commands::help::HelpArgs),
    Hello(commands::hello::HelloArgs),
    Mood(commands::mood::MoodArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let argv: Vec<String> = std::env::args().collect();

    // Capture --debug before parsing (clap swallows it from argv if we don't)
    let debug = argv.contains(&"--debug".to_string());
    if debug {
        debug::set_enabled(true);
        debug::log_state("startup", &format!("nina startup, debug mode active, version {}", env!("CARGO_PKG_VERSION")));
    }

    let cli = Cli::parse_from(&argv);
    let config = config::NinaConfig::load_or_bootstrap()?;
    let ctx = AppContext::new(config);
    let entered = argv.get(1..).map(|args| args.join(" "));
    if debug::is_enabled() {
        debug::log_state("command", entered.as_deref().unwrap_or("interactive prompt"));
    }
    session::run(&ctx, cli, entered.as_deref()).await
}

#[cfg(test)]
mod tests {
    use super::{Cli, Command};
    use clap::{error::ErrorKind, Parser};

    #[test]
    fn parses_option_command() {
        let cli = Cli::try_parse_from(["nina", "option", "services.ollama"])
            .expect("parse option command");
        assert!(matches!(cli.command, Some(Command::OptionCmd(_))));
    }

    #[test]
    fn parses_build_attr_command() {
        let cli = Cli::try_parse_from(["nina", "build", "packages.x86_64-linux.default"])
            .expect("parse build command");
        assert!(matches!(cli.command, Some(Command::Build(_))));
    }

    #[test]
    fn parses_nested_service_logs_follow() {
        let cli = Cli::try_parse_from(["nina", "service", "logs", "ollama", "-f"])
            .expect("parse service logs command");
        assert!(matches!(cli.command, Some(Command::Service(_))));
    }

    #[test]
    fn parses_unpin_command() {
        let cli = Cli::try_parse_from(["nina", "unpin", "nixpkgs"]).expect("parse unpin command");
        assert!(matches!(cli.command, Some(Command::Unpin(_))));
    }

    #[test]
    fn parses_develop_run_with_trailing_args() {
        let cli = Cli::try_parse_from(["nina", "develop", "--run", "cargo", "test"])
            .expect("parse develop run command");
        assert!(matches!(cli.command, Some(Command::Develop(_))));
    }

    macro_rules! command_help_tests {
        ($($name:ident : $command:literal,)+) => {
            $(
                #[test]
                fn $name() {
                    let err = Cli::try_parse_from(["nina", $command, "--help"])
                        .expect_err("help should short-circuit parsing");
                    assert_eq!(err.kind(), ErrorKind::DisplayHelp);
                    let rendered = err.to_string();
                    assert!(rendered.contains("Usage:"), "help output should contain usage");
                }
            )+
        };
    }

    command_help_tests! {
        apply_help_is_available: "apply",
        back_help_is_available: "back",
        build_help_is_available: "build",
        boot_help_is_available: "boot",
        channel_help_is_available: "channel",
        history_help_is_available: "history",
        go_help_is_available: "go",
        clean_help_is_available: "clean",
        develop_help_is_available: "develop",
        search_help_is_available: "search",
        install_help_is_available: "install",
        remove_help_is_available: "remove",
        try_help_is_available: "try",
        list_help_is_available: "list",
        fetch_help_is_available: "fetch",
        flake_help_is_available: "flake",
        fmt_help_is_available: "fmt",
        gen_help_is_available: "gen",
        hash_help_is_available: "hash",
        edit_help_is_available: "edit",
        check_help_is_available: "check",
        diff_help_is_available: "diff",
        info_help_is_available: "info",
        option_help_is_available: "option",
        pin_help_is_available: "pin",
        unpin_help_is_available: "unpin",
        pkg_help_is_available: "pkg",
        profile_help_is_available: "profile",
        repl_help_is_available: "repl",
        run_help_is_available: "run",
        service_help_is_available: "service",
        status_help_is_available: "status",
        store_help_is_available: "store",
        update_help_is_available: "update",
        upgrade_help_is_available: "upgrade",
        log_help_is_available: "log",
        doctor_help_is_available: "doctor",
        help_help_is_available: "help",
        hello_help_is_available: "hello",
        mood_help_is_available: "mood",
    }
}
