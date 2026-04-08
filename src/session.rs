use std::io::{self, IsTerminal, Write};

use crate::commands::AppContext;
use crate::output::RgbColor;
use crate::{commands, Cli, Command};
use anyhow::{Context, Result};
use clap::Parser;

const PROMPT_SPOTLIGHTS: &[&str] = &[
    "help",
    "mood",
    "search helix",
    "install alejandra --no-apply",
    "status --all",
];

pub async fn run(ctx: &AppContext, cli: Cli, entered: Option<&str>) -> Result<()> {
    match cli.command {
        Some(command) => run_one_shot(ctx, command, entered).await,
        None if io::stdin().is_terminal() && io::stdout().is_terminal() => run_prompt(ctx).await,
        None => {
            ctx.output
                .face("hi! i'm nina~ try 'nina help' to see what i can do ♡");
            Ok(())
        }
    }
}

async fn run_prompt(ctx: &AppContext) -> Result<()> {
    ctx.output.hero(
        "nina's little prompt",
        "say what you want in plain terminal words and i'll stay with you.",
    );
    ctx.output.curious("pick a path and i'll stay nearby.");
    ctx.output.sep();
    for idea in PROMPT_SPOTLIGHTS {
        ctx.output.status_line("idea", idea, RgbColor::LAVENDER);
    }
    ctx.output.tip("type 'quit' when you want the room back.");
    ctx.output.blank();

    loop {
        print!("{}", ctx.output.prompt("nina"));
        io::stdout()
            .flush()
            .context("failed to flush nina prompt")?;

        let mut line = String::new();
        let read = io::stdin()
            .read_line(&mut line)
            .context("failed to read nina prompt input")?;
        if read == 0 {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            ctx.output
                .tip("try a full command like 'search ripgrep' or 'apply --check'.");
            continue;
        }
        if is_exit_phrase(trimmed) {
            break;
        }

        let words = match split_prompt_input(trimmed) {
            Ok(words) => normalize_prompt_words(words),
            Err(message) => {
                ctx.output.warn(&message);
                continue;
            }
        };

        if words.is_empty() {
            continue;
        }

        let cli = match Cli::try_parse_from(
            std::iter::once("nina").chain(words.iter().map(String::as_str)),
        ) {
            Ok(cli) => cli,
            Err(err) => {
                ctx.output.warn("that command didn't parse cleanly.");
                eprintln!("{err}");
                continue;
            }
        };

        if let Some(command) = cli.command {
            if let Err(err) = run_one_shot(ctx, command, Some(trimmed)).await {
                ctx.output
                    .warn(&format!("i'm keeping the prompt open: {err}"));
            }
            ctx.output.blank();
        }
    }

    ctx.output.cozy("okay, i'll be quiet now ♡");
    Ok(())
}

async fn run_one_shot(ctx: &AppContext, command: Command, entered: Option<&str>) -> Result<()> {
    if !io::stdout().is_terminal() {
        return dispatch_command(ctx, command).await;
    }

    let descriptor = descriptor_for(command_name(&command));
    let rendered = entered.unwrap_or(command_name(&command));

    ctx.output.hero("nina is moving", descriptor.headline);
    ctx.output.command_echo(rendered);
    ctx.output
        .status_line(descriptor.tag, descriptor.pulse, descriptor.accent);

    let result = dispatch_command(ctx, command).await;

    match &result {
        Ok(_) => {
            ctx.output
                .status_line("done", descriptor.outro, RgbColor::MINT);
        }
        Err(_) => {
            ctx.output
                .sad("that didn't quite work, so i'm leaving the details where you can see them.");
            ctx.output.status_line(
                "oops",
                "something went wrong, and i'm keeping the trail visible for you.",
                RgbColor::ROSE,
            );
        }
    }

    result
}

async fn dispatch_command(ctx: &AppContext, command: Command) -> Result<()> {
    crate::debug::log_state("dispatch", crate::session::command_name(&command));
    let result = match command {
        Command::Apply(args) => commands::apply::run(ctx, args).await,
        Command::Back(args) => commands::back::run(ctx, args).await,
        Command::Build(args) => commands::build_cmd::run(ctx, args).await,
        Command::Boot(args) => commands::boot::run(ctx, args).await,
        Command::Channel(args) => commands::channel::run(ctx, args).await,
        Command::History(args) => commands::history::run(ctx, args).await,
        Command::Go(args) => commands::go::run(ctx, args).await,
        Command::Clean(args) => commands::clean::run(ctx, args).await,
        Command::Develop(args) => commands::develop::run(ctx, args).await,
        Command::Search(args) => commands::search::run(ctx, args).await,
        Command::Install(args) => commands::install::run(ctx, args).await,
        Command::Remove(args) => commands::remove::run(ctx, args).await,
        Command::Try(args) => commands::try_pkg::run(ctx, args).await,
        Command::List(args) => commands::list::run(ctx, args).await,
        Command::Fetch(args) => commands::fetch::run(ctx, args).await,
        Command::Flake(args) => commands::flake::run(ctx, args).await,
        Command::Fmt(args) => commands::fmt::run(ctx, args).await,
        Command::Gen(args) => commands::gen::run(ctx, args).await,
        Command::Hash(args) => commands::hash::run(ctx, args).await,
        Command::Edit(args) => commands::edit::run(ctx, args).await,
        Command::Check(args) => commands::check::run(ctx, args).await,
        Command::Diff(args) => commands::diff::run(ctx, args).await,
        Command::Info(args) => commands::info::run(ctx, args).await,
        Command::OptionCmd(args) => commands::option::run(ctx, args).await,
        Command::Pin(args) => commands::pin::run(ctx, args).await,
        Command::Unpin(args) => commands::pin::run_unpin(ctx, args).await,
        Command::Pkg(args) => commands::pkg::run(ctx, args).await,
        Command::Profile(args) => commands::profile::run(ctx, args).await,
        Command::Repl(args) => commands::repl::run(ctx, args).await,
        Command::RunCmd(args) => commands::run_cmd::run(ctx, args).await,
        Command::Service(args) => commands::service::run(ctx, args).await,
        Command::Status(args) => commands::status::run(ctx, args).await,
        Command::Store(args) => commands::store::run(ctx, args).await,
        Command::Update(args) => commands::update::run(ctx, args).await,
        Command::Upgrade(args) => commands::upgrade::run(ctx, args).await,
        Command::Log(args) => commands::log_cmd::run(ctx, args).await,
        Command::Doctor(args) => commands::doctor::run(ctx, args).await,
        Command::Help(args) => commands::help::run(ctx, args).await,
        Command::Hello(args) => commands::hello::run(ctx, args).await,
        Command::Mood(args) => commands::mood::run(ctx, args).await,
    };
    crate::debug::log_result("dispatch", result.is_ok());
    result
}

fn split_prompt_input(input: &str) -> std::result::Result<Vec<String>, String> {
    shlex::split(input).ok_or_else(|| {
        "i couldn't understand those quotes. close them up and try once more.".to_string()
    })
}

fn normalize_prompt_words(mut words: Vec<String>) -> Vec<String> {
    if words.first().map(String::as_str) == Some("nina") {
        words.remove(0);
    }
    words
}

fn is_exit_phrase(input: &str) -> bool {
    matches!(input.trim(), "q" | ":q" | "quit" | "exit" | "bye")
}

fn command_name(command: &Command) -> &'static str {
    match command {
        Command::Apply(_) => "apply",
        Command::Back(_) => "back",
        Command::Build(_) => "build",
        Command::Boot(_) => "boot",
        Command::Channel(_) => "channel",
        Command::History(_) => "history",
        Command::Go(_) => "go",
        Command::Clean(_) => "clean",
        Command::Develop(_) => "develop",
        Command::Search(_) => "search",
        Command::Install(_) => "install",
        Command::Remove(_) => "remove",
        Command::Try(_) => "try",
        Command::List(_) => "list",
        Command::Fetch(_) => "fetch",
        Command::Flake(_) => "flake",
        Command::Fmt(_) => "fmt",
        Command::Gen(_) => "gen",
        Command::Hash(_) => "hash",
        Command::Edit(_) => "edit",
        Command::Check(_) => "check",
        Command::Diff(_) => "diff",
        Command::Info(_) => "info",
        Command::OptionCmd(_) => "option",
        Command::Pin(_) => "pin",
        Command::Unpin(_) => "unpin",
        Command::Pkg(_) => "pkg",
        Command::Profile(_) => "profile",
        Command::Repl(_) => "repl",
        Command::RunCmd(_) => "run",
        Command::Service(_) => "service",
        Command::Status(_) => "status",
        Command::Store(_) => "store",
        Command::Update(_) => "update",
        Command::Upgrade(_) => "upgrade",
        Command::Log(_) => "log",
        Command::Doctor(_) => "doctor",
        Command::Help(_) => "help",
        Command::Hello(_) => "hello",
        Command::Mood(_) => "mood",
    }
}

#[derive(Debug, Clone, Copy)]
struct SessionDescriptor {
    tag: &'static str,
    headline: &'static str,
    pulse: &'static str,
    outro: &'static str,
    accent: RgbColor,
}

fn descriptor_for(name: &str) -> SessionDescriptor {
    match name {
        "apply" | "upgrade" => SessionDescriptor {
            tag: "rebuild",
            headline: "getting your system ready.",
            pulse: "i'll keep the important steps grouped together.",
            outro: "the rebuild is settled now ♡",
            accent: RgbColor::MINT,
        },
        "back" | "go" => SessionDescriptor {
            tag: "rewind",
            headline: "looking through your generation history.",
            pulse: "i'm following the generation trail.",
            outro: "you're on the generation you asked for.",
            accent: RgbColor::LAVENDER,
        },
        "clean" | "store" => SessionDescriptor {
            tag: "sweep",
            headline: "cleaning up the older bits without touching what you need.",
            pulse: "i'm looking for unused store paths and extra weight.",
            outro: "things feel a little lighter now.",
            accent: RgbColor::GOLD,
        },
        "search" | "option" | "pkg" | "info" | "status" | "doctor" | "mood" | "list"
        | "history" | "log" | "help" | "boot" => SessionDescriptor {
            tag: "scan",
            headline: "pulling the useful parts forward so they're easy to read.",
            pulse: "i'm gathering the parts worth looking at first.",
            outro: "that should feel easier to read now.",
            accent: RgbColor::PINK,
        },
        "install" | "remove" | "edit" | "fmt" | "pin" | "unpin" | "flake" | "channel"
        | "profile" => SessionDescriptor {
            tag: "shape",
            headline: "lining up the change before anything gets written.",
            pulse: "i'm lining up edits, previews, and the next likely move.",
            outro: "the change went through cleanly.",
            accent: RgbColor::ROSE,
        },
        "build" | "check" | "develop" | "run" | "repl" | "try" | "fetch" | "hash" | "gen" => {
            SessionDescriptor {
                tag: "setup",
                headline: "getting things ready before the busy part starts.",
                pulse: "i'll give you a short lead-in before the heavier output starts.",
                outro: "that finished cleanly.",
                accent: RgbColor::MINT,
            }
        }
        "update" => SessionDescriptor {
            tag: "refresh",
            headline: "refreshing things before anything bigger happens.",
            pulse: "i'm refreshing channels and inputs now.",
            outro: "the channels look fresh again.",
            accent: RgbColor::GOLD,
        },
        "service" => SessionDescriptor {
            tag: "service",
            headline: "checking in on the service before i nudge it.",
            pulse: "i'm making sure the service is ready for the next step.",
            outro: "the service side is wrapped up now ♡",
            accent: RgbColor::LAVENDER,
        },
        "hello" => SessionDescriptor {
            tag: "hi",
            headline: "starting with a little hello.",
            pulse: "i'm finding your machines now.",
            outro: "hello all set ♡",
            accent: RgbColor::PINK,
        },
        _ => SessionDescriptor {
            tag: "nina",
            headline: "getting everything in place before we start.",
            pulse: "keeping you posted while the command takes shape.",
            outro: "all done ♡",
            accent: RgbColor::PINK,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_voice_line(line: &str) {
        if line.is_empty() {
            return;
        }
        assert!(
            !line.contains("  "),
            "descriptor copy should not contain doubled spaces: {line}"
        );
        assert!(
            line.ends_with('.') || line.ends_with('♡'),
            "descriptor copy should end with a period or heart: {line}"
        );
        assert!(
            line.split_whitespace().count() >= 3,
            "descriptor copy should sound complete: {line}"
        );
        let first = line
            .chars()
            .find(|ch| ch.is_alphabetic())
            .expect("descriptor line should contain letters");
        assert!(
            first.is_lowercase(),
            "descriptor copy should preserve nina's lowercase voice: {line}"
        );
    }

    macro_rules! descriptor_accent_tests {
        ($($name:ident : $command:literal => $accent:expr,)+) => {
            $(
                #[test]
                fn $name() {
                    assert_eq!(descriptor_for($command).accent, $accent);
                }
            )+
        };
    }

    macro_rules! descriptor_tag_tests {
        ($($name:ident : $command:literal => $tag:literal,)+) => {
            $(
                #[test]
                fn $name() {
                    assert_eq!(descriptor_for($command).tag, $tag);
                }
            )+
        };
    }

    macro_rules! descriptor_copy_tests {
        (
            $(
                $name:ident : $command:literal => {
                    headline: $headline:literal,
                    pulse: $pulse:literal,
                    outro: $outro:literal,
                },
            )+
        ) => {
            $(
                mod $name {
                    use super::*;

                    #[test]
                    fn headline() {
                        assert_eq!(descriptor_for($command).headline, $headline);
                    }

                    #[test]
                    fn pulse() {
                        assert_eq!(descriptor_for($command).pulse, $pulse);
                    }

                    #[test]
                    fn outro() {
                        assert_eq!(descriptor_for($command).outro, $outro);
                    }

                    #[test]
                    fn copy_quality() {
                        let descriptor = descriptor_for($command);
                        for line in [
                            descriptor.headline,
                            descriptor.pulse,
                            descriptor.outro,
                        ] {
                            assert_voice_line(line);
                        }
                    }
                }
            )+
        };
    }

    descriptor_accent_tests! {
        apply_accent: "apply" => RgbColor::MINT,
        back_accent: "back" => RgbColor::LAVENDER,
        build_accent: "build" => RgbColor::MINT,
        boot_accent: "boot" => RgbColor::PINK,
        channel_accent: "channel" => RgbColor::ROSE,
        history_accent: "history" => RgbColor::PINK,
        go_accent: "go" => RgbColor::LAVENDER,
        clean_accent: "clean" => RgbColor::GOLD,
        develop_accent: "develop" => RgbColor::MINT,
        search_accent: "search" => RgbColor::PINK,
        install_accent: "install" => RgbColor::ROSE,
        remove_accent: "remove" => RgbColor::ROSE,
        try_accent: "try" => RgbColor::MINT,
        list_accent: "list" => RgbColor::PINK,
        fetch_accent: "fetch" => RgbColor::MINT,
        flake_accent: "flake" => RgbColor::ROSE,
        fmt_accent: "fmt" => RgbColor::ROSE,
        gen_accent: "gen" => RgbColor::MINT,
        hash_accent: "hash" => RgbColor::MINT,
        edit_accent: "edit" => RgbColor::ROSE,
        check_accent: "check" => RgbColor::MINT,
        diff_accent: "diff" => RgbColor::PINK,
        info_accent: "info" => RgbColor::PINK,
        option_accent: "option" => RgbColor::PINK,
        pin_accent: "pin" => RgbColor::ROSE,
        unpin_accent: "unpin" => RgbColor::ROSE,
        pkg_accent: "pkg" => RgbColor::PINK,
        profile_accent: "profile" => RgbColor::ROSE,
        repl_accent: "repl" => RgbColor::MINT,
        run_accent: "run" => RgbColor::MINT,
        service_accent: "service" => RgbColor::LAVENDER,
        status_accent: "status" => RgbColor::PINK,
        store_accent: "store" => RgbColor::GOLD,
        update_accent: "update" => RgbColor::GOLD,
        upgrade_accent: "upgrade" => RgbColor::MINT,
        log_accent: "log" => RgbColor::PINK,
        doctor_accent: "doctor" => RgbColor::PINK,
        help_accent: "help" => RgbColor::PINK,
        hello_accent: "hello" => RgbColor::PINK,
        mood_accent: "mood" => RgbColor::PINK,
    }

    descriptor_tag_tests! {
        apply_tag: "apply" => "rebuild",
        back_tag: "back" => "rewind",
        build_tag: "build" => "setup",
        boot_tag: "boot" => "scan",
        channel_tag: "channel" => "shape",
        history_tag: "history" => "scan",
        go_tag: "go" => "rewind",
        clean_tag: "clean" => "sweep",
        develop_tag: "develop" => "setup",
        search_tag: "search" => "scan",
        install_tag: "install" => "shape",
        remove_tag: "remove" => "shape",
        try_tag: "try" => "setup",
        list_tag: "list" => "scan",
        fetch_tag: "fetch" => "setup",
        flake_tag: "flake" => "shape",
        fmt_tag: "fmt" => "shape",
        gen_tag: "gen" => "setup",
        hash_tag: "hash" => "setup",
        edit_tag: "edit" => "shape",
        check_tag: "check" => "setup",
        diff_tag: "diff" => "nina",
        info_tag: "info" => "scan",
        option_tag: "option" => "scan",
        pin_tag: "pin" => "shape",
        unpin_tag: "unpin" => "shape",
        pkg_tag: "pkg" => "scan",
        profile_tag: "profile" => "shape",
        repl_tag: "repl" => "setup",
        run_tag: "run" => "setup",
        service_tag: "service" => "service",
        status_tag: "status" => "scan",
        store_tag: "store" => "sweep",
        update_tag: "update" => "refresh",
        upgrade_tag: "upgrade" => "rebuild",
        log_tag: "log" => "scan",
        doctor_tag: "doctor" => "scan",
        help_tag: "help" => "scan",
        hello_tag: "hello" => "hi",
        mood_tag: "mood" => "scan",
    }

    descriptor_copy_tests! {
        apply: "apply" => {
            headline: "getting your system ready.",
            pulse: "i'll keep the important steps grouped together.",
            outro: "the rebuild is settled now ♡",
        },
        back: "back" => {
            headline: "looking through your generation history.",
            pulse: "i'm following the generation trail.",
            outro: "you're on the generation you asked for.",
        },
        build: "build" => {
            headline: "getting things ready before the busy part starts.",
            pulse: "i'll give you a short lead-in before the heavier output starts.",
            outro: "that finished cleanly.",
        },
        boot: "boot" => {
            headline: "pulling the useful parts forward so they're easy to read.",
            pulse: "i'm gathering the parts worth looking at first.",
            outro: "that should feel easier to read now.",
        },
        channel: "channel" => {
            headline: "lining up the change before anything gets written.",
            pulse: "i'm lining up edits, previews, and the next likely move.",
            outro: "the change went through cleanly.",
        },
        history: "history" => {
            headline: "pulling the useful parts forward so they're easy to read.",
            pulse: "i'm gathering the parts worth looking at first.",
            outro: "that should feel easier to read now.",
        },
        go: "go" => {
            headline: "looking through your generation history.",
            pulse: "i'm following the generation trail.",
            outro: "you're on the generation you asked for.",
        },
        clean: "clean" => {
            headline: "cleaning up the older bits without touching what you need.",
            pulse: "i'm looking for unused store paths and extra weight.",
            outro: "things feel a little lighter now.",
        },
        develop: "develop" => {
            headline: "getting things ready before the busy part starts.",
            pulse: "i'll give you a short lead-in before the heavier output starts.",
            outro: "that finished cleanly.",
        },
        search: "search" => {
            headline: "pulling the useful parts forward so they're easy to read.",
            pulse: "i'm gathering the parts worth looking at first.",
            outro: "that should feel easier to read now.",
        },
        install: "install" => {
            headline: "lining up the change before anything gets written.",
            pulse: "i'm lining up edits, previews, and the next likely move.",
            outro: "the change went through cleanly.",
        },
        remove: "remove" => {
            headline: "lining up the change before anything gets written.",
            pulse: "i'm lining up edits, previews, and the next likely move.",
            outro: "the change went through cleanly.",
        },
        r#try: "try" => {
            headline: "getting things ready before the busy part starts.",
            pulse: "i'll give you a short lead-in before the heavier output starts.",
            outro: "that finished cleanly.",
        },
        list: "list" => {
            headline: "pulling the useful parts forward so they're easy to read.",
            pulse: "i'm gathering the parts worth looking at first.",
            outro: "that should feel easier to read now.",
        },
        fetch: "fetch" => {
            headline: "getting things ready before the busy part starts.",
            pulse: "i'll give you a short lead-in before the heavier output starts.",
            outro: "that finished cleanly.",
        },
        flake: "flake" => {
            headline: "lining up the change before anything gets written.",
            pulse: "i'm lining up edits, previews, and the next likely move.",
            outro: "the change went through cleanly.",
        },
        fmt: "fmt" => {
            headline: "lining up the change before anything gets written.",
            pulse: "i'm lining up edits, previews, and the next likely move.",
            outro: "the change went through cleanly.",
        },
        gen: "gen" => {
            headline: "getting things ready before the busy part starts.",
            pulse: "i'll give you a short lead-in before the heavier output starts.",
            outro: "that finished cleanly.",
        },
        hash: "hash" => {
            headline: "getting things ready before the busy part starts.",
            pulse: "i'll give you a short lead-in before the heavier output starts.",
            outro: "that finished cleanly.",
        },
        edit: "edit" => {
            headline: "lining up the change before anything gets written.",
            pulse: "i'm lining up edits, previews, and the next likely move.",
            outro: "the change went through cleanly.",
        },
        check: "check" => {
            headline: "getting things ready before the busy part starts.",
            pulse: "i'll give you a short lead-in before the heavier output starts.",
            outro: "that finished cleanly.",
        },
        diff: "diff" => {
            headline: "getting everything in place before we start.",
            pulse: "keeping you posted while the command takes shape.",
            outro: "all done ♡",
        },
        info: "info" => {
            headline: "pulling the useful parts forward so they're easy to read.",
            pulse: "i'm gathering the parts worth looking at first.",
            outro: "that should feel easier to read now.",
        },
        option: "option" => {
            headline: "pulling the useful parts forward so they're easy to read.",
            pulse: "i'm gathering the parts worth looking at first.",
            outro: "that should feel easier to read now.",
        },
        pin: "pin" => {
            headline: "lining up the change before anything gets written.",
            pulse: "i'm lining up edits, previews, and the next likely move.",
            outro: "the change went through cleanly.",
        },
        unpin: "unpin" => {
            headline: "lining up the change before anything gets written.",
            pulse: "i'm lining up edits, previews, and the next likely move.",
            outro: "the change went through cleanly.",
        },
        pkg: "pkg" => {
            headline: "pulling the useful parts forward so they're easy to read.",
            pulse: "i'm gathering the parts worth looking at first.",
            outro: "that should feel easier to read now.",
        },
        profile: "profile" => {
            headline: "lining up the change before anything gets written.",
            pulse: "i'm lining up edits, previews, and the next likely move.",
            outro: "the change went through cleanly.",
        },
        repl: "repl" => {
            headline: "getting things ready before the busy part starts.",
            pulse: "i'll give you a short lead-in before the heavier output starts.",
            outro: "that finished cleanly.",
        },
        run: "run" => {
            headline: "getting things ready before the busy part starts.",
            pulse: "i'll give you a short lead-in before the heavier output starts.",
            outro: "that finished cleanly.",
        },
        service: "service" => {
            headline: "checking in on the service before i nudge it.",
            pulse: "i'm making sure the service is ready for the next step.",
            outro: "the service side is wrapped up now ♡",
        },
        status: "status" => {
            headline: "pulling the useful parts forward so they're easy to read.",
            pulse: "i'm gathering the parts worth looking at first.",
            outro: "that should feel easier to read now.",
        },
        store: "store" => {
            headline: "cleaning up the older bits without touching what you need.",
            pulse: "i'm looking for unused store paths and extra weight.",
            outro: "things feel a little lighter now.",
        },
        update: "update" => {
            headline: "refreshing things before anything bigger happens.",
            pulse: "i'm refreshing channels and inputs now.",
            outro: "the channels look fresh again.",
        },
        upgrade: "upgrade" => {
            headline: "getting your system ready.",
            pulse: "i'll keep the important steps grouped together.",
            outro: "the rebuild is settled now ♡",
        },
        log: "log" => {
            headline: "pulling the useful parts forward so they're easy to read.",
            pulse: "i'm gathering the parts worth looking at first.",
            outro: "that should feel easier to read now.",
        },
        doctor: "doctor" => {
            headline: "pulling the useful parts forward so they're easy to read.",
            pulse: "i'm gathering the parts worth looking at first.",
            outro: "that should feel easier to read now.",
        },
        help: "help" => {
            headline: "pulling the useful parts forward so they're easy to read.",
            pulse: "i'm gathering the parts worth looking at first.",
            outro: "that should feel easier to read now.",
        },
        hello: "hello" => {
            headline: "starting with a little hello.",
            pulse: "i'm finding your machines now.",
            outro: "hello all set ♡",
        },
        mood: "mood" => {
            headline: "pulling the useful parts forward so they're easy to read.",
            pulse: "i'm gathering the parts worth looking at first.",
            outro: "that should feel easier to read now.",
        },
    }

    #[test]
    fn split_prompt_input_preserves_quotes() {
        let words = split_prompt_input("develop --run 'cargo test'").expect("split words");
        assert_eq!(words, vec!["develop", "--run", "cargo test"]);
    }

    #[test]
    fn split_prompt_input_rejects_unclosed_quotes() {
        let err = split_prompt_input("search 'broken").expect_err("quote error");
        assert!(err.contains("quotes"));
    }

    #[test]
    fn normalize_prompt_words_strips_leading_binary_name() {
        let words = normalize_prompt_words(vec!["nina".into(), "help".into()]);
        assert_eq!(words, vec!["help"]);
    }

    #[test]
    fn normalize_prompt_words_keeps_regular_command() {
        let words = normalize_prompt_words(vec!["search".into(), "ripgrep".into()]);
        assert_eq!(words, vec!["search", "ripgrep"]);
    }

    #[test]
    fn exit_phrase_accepts_quit_variants() {
        for input in ["q", ":q", "quit", "exit", "bye"] {
            assert!(is_exit_phrase(input), "{input} should exit");
        }
    }

    #[test]
    fn exit_phrase_rejects_normal_commands() {
        for input in ["help", "search", "apply --check"] {
            assert!(!is_exit_phrase(input), "{input} should not exit");
        }
    }

    #[test]
    fn prompt_spotlights_stay_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for spotlight in PROMPT_SPOTLIGHTS {
            assert!(seen.insert(*spotlight), "duplicate spotlight: {spotlight}");
        }
    }

    #[test]
    fn first_prompt_spotlight_stays_help() {
        assert_eq!(PROMPT_SPOTLIGHTS[0], "help");
    }

    #[test]
    fn second_prompt_spotlight_stays_mood() {
        assert_eq!(PROMPT_SPOTLIGHTS[1], "mood");
    }

    #[test]
    fn third_prompt_spotlight_stays_search() {
        assert_eq!(PROMPT_SPOTLIGHTS[2], "search helix");
    }

    #[test]
    fn fourth_prompt_spotlight_stays_install_preview() {
        assert_eq!(PROMPT_SPOTLIGHTS[3], "install alejandra --no-apply");
    }

    #[test]
    fn fifth_prompt_spotlight_stays_status_all() {
        assert_eq!(PROMPT_SPOTLIGHTS[4], "status --all");
    }

    #[test]
    fn prompt_spotlights_parse_as_valid_commands() {
        for spotlight in PROMPT_SPOTLIGHTS {
            let words = normalize_prompt_words(
                split_prompt_input(spotlight).expect("spotlight should split cleanly"),
            );
            let cli = Cli::try_parse_from(
                std::iter::once("nina").chain(words.iter().map(String::as_str)),
            )
            .expect("spotlight should parse");
            assert!(cli.command.is_some(), "spotlight should map to a command");
        }
    }
}
