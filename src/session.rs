use std::io::{self, IsTerminal, Write};
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::terminal;
use tokio::time::sleep;

use crate::commands::AppContext;
use crate::dango::{DangoAnimation, DangoPlayer};
use crate::output::RgbColor;
use crate::{commands, Cli, Command};

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
    ctx.output
        .curious("pick a path and i'll keep the terminal warm.");
    ctx.output.sep();
    for idea in PROMPT_SPOTLIGHTS {
        ctx.output.status_line("idea", idea, RgbColor::LAVENDER);
    }
    ctx.output.tip("type 'quit' when you want the room back.");
    ctx.output.blank();
    pulse(ctx, DangoAnimation::Dance, 700);

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

    ctx.output.cozy("okay, i'll drift off now ♡");
    pulse(ctx, DangoAnimation::Wave, 500);
    Ok(())
}

async fn run_one_shot(ctx: &AppContext, command: Command, entered: Option<&str>) -> Result<()> {
    if !io::stdout().is_terminal() {
        return dispatch_command(ctx, command).await;
    }

    let descriptor = descriptor_for(command_name(&command));
    let rendered = entered.unwrap_or(command_name(&command));

    ctx.output.hero("nina is moving", descriptor.headline);
    ctx.output.excited(descriptor.headline);
    ctx.output.command_echo(rendered);
    ctx.output
        .status_line(descriptor.tag, descriptor.pulse, descriptor.accent);
    ctx.output.tip(descriptor.followup);
    pulse(ctx, descriptor.animation, descriptor.pulse_ms);

    let started = std::time::Instant::now();
    let result = dispatch_command(ctx, command).await;
    let elapsed = format_elapsed(started.elapsed());

    match &result {
        Ok(_) => {
            ctx.output
                .status_line("done", descriptor.outro, RgbColor::MINT);
            ctx.output.kv_succ("elapsed", &elapsed);
            linger(ctx, DangoAnimation::Happy).await;
        }
        Err(_) => {
            ctx.output
                .sad("that one stumbled, so i'm leaving the trail where it is.");
            ctx.output.status_line(
                "oops",
                "something bit back; i'm leaving the trail visible.",
                RgbColor::ROSE,
            );
            ctx.output.kv_err("elapsed", &elapsed);
            linger(ctx, DangoAnimation::Sad).await;
        }
    }

    result
}

async fn dispatch_command(ctx: &AppContext, command: Command) -> Result<()> {
    match command {
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
    }
}

fn pulse(ctx: &AppContext, animation: DangoAnimation, millis: u64) {
    if !ctx.config.animate || !io::stdout().is_terminal() {
        return;
    }

    let Some(pos) = dango_pos(&ctx.config.dango_pos) else {
        return;
    };

    let player = DangoPlayer::start(animation, pos);
    // Run the animation on a dedicated thread so it completes fully before
    // dispatch_command starts — prevents animation frames from interleaving with command output.
    let handle = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("tokio runtime for animation");
        rt.block_on(async {
            sleep(Duration::from_millis(millis)).await;
            player.stop().await;
        });
    });
    let _ = handle.join();
}

async fn linger(ctx: &AppContext, animation: DangoAnimation) {
    if !ctx.config.animate || !io::stdout().is_terminal() {
        sleep(Duration::from_millis(480)).await;
        return;
    }

    let Some(pos) = dango_pos(&ctx.config.dango_pos) else {
        sleep(Duration::from_millis(480)).await;
        return;
    };

    let player = DangoPlayer::play_once(animation, pos);
    sleep(Duration::from_millis(160)).await;
    player.stop().await;
}

fn dango_pos(pref: &str) -> Option<(u16, u16)> {
    terminal::size()
        .ok()
        .map(|(width, height)| crate::dango::position_from_pref(pref, width, height))
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

fn format_elapsed(elapsed: Duration) -> String {
    if elapsed.as_secs() > 0 {
        format!("{:.1}s", elapsed.as_secs_f64())
    } else {
        format!("{}ms", elapsed.as_millis())
    }
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
    followup: &'static str,
    outro: &'static str,
    accent: RgbColor,
    animation: DangoAnimation,
    pulse_ms: u64,
}

fn descriptor_for(name: &str) -> SessionDescriptor {
    match name {
        "apply" | "upgrade" => SessionDescriptor {
            tag: "rebuild",
            headline: "warming the rails before a full system move.",
            pulse: "checking the shape of your nix world first.",
            followup: "i'll narrate the big steps instead of dumping a silent wall.",
            outro: "the rebuild is all settled in ♡",
            accent: RgbColor::MINT,
            animation: DangoAnimation::Dance,
            pulse_ms: 420,
        },
        "back" | "go" => SessionDescriptor {
            tag: "time-hop",
            headline: "rewinding gently so the generations don't feel sharp.",
            pulse: "tracking the generation trail.",
            followup: "if the generation shifts, i'll point out the jump.",
            outro: "the system landed on the generation you asked for.",
            accent: RgbColor::LAVENDER,
            animation: DangoAnimation::WalkBack,
            pulse_ms: 360,
        },
        "clean" | "store" => SessionDescriptor {
            tag: "sweep",
            headline: "dusting the nix corners without losing your footing.",
            pulse: "looking for old weight and stray store bits.",
            followup: "expect a practical cleanup story, not just raw bytes.",
            outro: "the shelves feel lighter now.",
            accent: RgbColor::GOLD,
            animation: DangoAnimation::Sweep,
            pulse_ms: 360,
        },
        "search" | "option" | "pkg" | "info" | "status" | "doctor" | "mood" | "list"
        | "history" | "log" | "help" | "boot" => SessionDescriptor {
            tag: "scan",
            headline: "turning the searchlight on and keeping the output readable.",
            pulse: "collecting the parts worth your eyes first.",
            followup: "i'll keep the signal colorful and grouped.",
            outro: "the answer should feel easier to hold now.",
            accent: RgbColor::PINK,
            animation: DangoAnimation::Wave,
            pulse_ms: 300,
        },
        "install" | "remove" | "edit" | "fmt" | "pin" | "unpin" | "flake" | "channel"
        | "profile" => SessionDescriptor {
            tag: "shape",
            headline: "reshaping config with a little ceremony so it feels alive.",
            pulse: "lining up edits, previews, and the next likely move.",
            followup: "i'll show the shape of the change before i disappear.",
            outro: "the config moved without losing its softness.",
            accent: RgbColor::ROSE,
            animation: DangoAnimation::Happy,
            pulse_ms: 320,
        },
        "build" | "check" | "develop" | "run" | "repl" | "try" | "fetch" | "hash" | "gen" => {
            SessionDescriptor {
                tag: "forge",
                headline: "setting up the shell and keeping the terminal warm.",
                pulse: "bringing tools and command context into the same room.",
                followup: "you'll get a short runway before the heavy output starts.",
                outro: "the run finished with a cleaner landing.",
                accent: RgbColor::MINT,
                animation: DangoAnimation::Spin,
                pulse_ms: 280,
            }
        }
        "update" => SessionDescriptor {
            tag: "refresh",
            headline: "shaking fresh metadata loose before anything bigger happens.",
            pulse: "refreshing channels and inputs without going blank on you.",
            followup: "i'll tell you when the network-y part is the reason we're waiting.",
            outro: "the channels look fresh again.",
            accent: RgbColor::GOLD,
            animation: DangoAnimation::Idle,
            pulse_ms: 280,
        },
        "service" => SessionDescriptor {
            tag: "daemon",
            headline: "walking over to systemd with a little extra bedside manner.",
            pulse: "checking on the service before giving it a nudge.",
            followup: "you'll see the service story as it changes.",
            outro: "the service bit is all wrapped up ♡",
            accent: RgbColor::LAVENDER,
            animation: DangoAnimation::Wave,
            pulse_ms: 300,
        },
        "hello" => SessionDescriptor {
            tag: "hi",
            headline: "leaning in with a proper introduction instead of a dead splash line.",
            pulse: "finding your machines and giving the stage a little light.",
            followup: "this one is meant to feel playful.",
            outro: "the hello landed sweet ♡",
            accent: RgbColor::PINK,
            animation: DangoAnimation::Dance,
            pulse_ms: 320,
        },
        _ => SessionDescriptor {
            tag: "nina",
            headline: "waking up the prompt so it feels less like a cold subprocess.",
            pulse: "keeping you posted while the command takes shape.",
            followup: "i'll stay visible for a beat before i leave.",
            outro: "all done, i'll leave the trail here ♡",
            accent: RgbColor::PINK,
            animation: DangoAnimation::Idle,
            pulse_ms: 250,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_voice_line(line: &str) {
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

    macro_rules! descriptor_animation_tests {
        ($($name:ident : $command:literal => $animation:expr,)+) => {
            $(
                #[test]
                fn $name() {
                    assert_eq!(descriptor_for($command).animation, $animation);
                }
            )+
        };
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
                    followup: $followup:literal,
                    outro: $outro:literal,
                    pulse_ms: $pulse_ms:expr,
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
                    fn followup() {
                        assert_eq!(descriptor_for($command).followup, $followup);
                    }

                    #[test]
                    fn outro() {
                        assert_eq!(descriptor_for($command).outro, $outro);
                    }

                    #[test]
                    fn pulse_ms() {
                        assert_eq!(descriptor_for($command).pulse_ms, $pulse_ms);
                    }

                    #[test]
                    fn copy_quality() {
                        let descriptor = descriptor_for($command);
                        for line in [
                            descriptor.headline,
                            descriptor.pulse,
                            descriptor.followup,
                            descriptor.outro,
                        ] {
                            assert_voice_line(line);
                        }
                    }
                }
            )+
        };
    }

    descriptor_animation_tests! {
        apply_animation: "apply" => DangoAnimation::Dance,
        back_animation: "back" => DangoAnimation::WalkBack,
        build_animation: "build" => DangoAnimation::Spin,
        boot_animation: "boot" => DangoAnimation::Wave,
        channel_animation: "channel" => DangoAnimation::Happy,
        history_animation: "history" => DangoAnimation::Wave,
        go_animation: "go" => DangoAnimation::WalkBack,
        clean_animation: "clean" => DangoAnimation::Sweep,
        develop_animation: "develop" => DangoAnimation::Spin,
        search_animation: "search" => DangoAnimation::Wave,
        install_animation: "install" => DangoAnimation::Happy,
        remove_animation: "remove" => DangoAnimation::Happy,
        try_animation: "try" => DangoAnimation::Spin,
        list_animation: "list" => DangoAnimation::Wave,
        fetch_animation: "fetch" => DangoAnimation::Spin,
        flake_animation: "flake" => DangoAnimation::Happy,
        fmt_animation: "fmt" => DangoAnimation::Happy,
        gen_animation: "gen" => DangoAnimation::Spin,
        hash_animation: "hash" => DangoAnimation::Spin,
        edit_animation: "edit" => DangoAnimation::Happy,
        check_animation: "check" => DangoAnimation::Spin,
        diff_animation: "diff" => DangoAnimation::Idle,
        info_animation: "info" => DangoAnimation::Wave,
        option_animation: "option" => DangoAnimation::Wave,
        pin_animation: "pin" => DangoAnimation::Happy,
        unpin_animation: "unpin" => DangoAnimation::Happy,
        pkg_animation: "pkg" => DangoAnimation::Wave,
        profile_animation: "profile" => DangoAnimation::Happy,
        repl_animation: "repl" => DangoAnimation::Spin,
        run_animation: "run" => DangoAnimation::Spin,
        service_animation: "service" => DangoAnimation::Wave,
        status_animation: "status" => DangoAnimation::Wave,
        store_animation: "store" => DangoAnimation::Sweep,
        update_animation: "update" => DangoAnimation::Idle,
        upgrade_animation: "upgrade" => DangoAnimation::Dance,
        log_animation: "log" => DangoAnimation::Wave,
        doctor_animation: "doctor" => DangoAnimation::Wave,
        help_animation: "help" => DangoAnimation::Wave,
        hello_animation: "hello" => DangoAnimation::Dance,
        mood_animation: "mood" => DangoAnimation::Wave,
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
        back_tag: "back" => "time-hop",
        build_tag: "build" => "forge",
        boot_tag: "boot" => "scan",
        channel_tag: "channel" => "shape",
        history_tag: "history" => "scan",
        go_tag: "go" => "time-hop",
        clean_tag: "clean" => "sweep",
        develop_tag: "develop" => "forge",
        search_tag: "search" => "scan",
        install_tag: "install" => "shape",
        remove_tag: "remove" => "shape",
        try_tag: "try" => "forge",
        list_tag: "list" => "scan",
        fetch_tag: "fetch" => "forge",
        flake_tag: "flake" => "shape",
        fmt_tag: "fmt" => "shape",
        gen_tag: "gen" => "forge",
        hash_tag: "hash" => "forge",
        edit_tag: "edit" => "shape",
        check_tag: "check" => "forge",
        diff_tag: "diff" => "nina",
        info_tag: "info" => "scan",
        option_tag: "option" => "scan",
        pin_tag: "pin" => "shape",
        unpin_tag: "unpin" => "shape",
        pkg_tag: "pkg" => "scan",
        profile_tag: "profile" => "shape",
        repl_tag: "repl" => "forge",
        run_tag: "run" => "forge",
        service_tag: "service" => "daemon",
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
            headline: "warming the rails before a full system move.",
            pulse: "checking the shape of your nix world first.",
            followup: "i'll narrate the big steps instead of dumping a silent wall.",
            outro: "the rebuild is all settled in ♡",
            pulse_ms: 420,
        },
        back: "back" => {
            headline: "rewinding gently so the generations don't feel sharp.",
            pulse: "tracking the generation trail.",
            followup: "if the generation shifts, i'll point out the jump.",
            outro: "the system landed on the generation you asked for.",
            pulse_ms: 360,
        },
        build: "build" => {
            headline: "setting up the shell and keeping the terminal warm.",
            pulse: "bringing tools and command context into the same room.",
            followup: "you'll get a short runway before the heavy output starts.",
            outro: "the run finished with a cleaner landing.",
            pulse_ms: 280,
        },
        boot: "boot" => {
            headline: "turning the searchlight on and keeping the output readable.",
            pulse: "collecting the parts worth your eyes first.",
            followup: "i'll keep the signal colorful and grouped.",
            outro: "the answer should feel easier to hold now.",
            pulse_ms: 300,
        },
        channel: "channel" => {
            headline: "reshaping config with a little ceremony so it feels alive.",
            pulse: "lining up edits, previews, and the next likely move.",
            followup: "i'll show the shape of the change before i disappear.",
            outro: "the config moved without losing its softness.",
            pulse_ms: 320,
        },
        history: "history" => {
            headline: "turning the searchlight on and keeping the output readable.",
            pulse: "collecting the parts worth your eyes first.",
            followup: "i'll keep the signal colorful and grouped.",
            outro: "the answer should feel easier to hold now.",
            pulse_ms: 300,
        },
        go: "go" => {
            headline: "rewinding gently so the generations don't feel sharp.",
            pulse: "tracking the generation trail.",
            followup: "if the generation shifts, i'll point out the jump.",
            outro: "the system landed on the generation you asked for.",
            pulse_ms: 360,
        },
        clean: "clean" => {
            headline: "dusting the nix corners without losing your footing.",
            pulse: "looking for old weight and stray store bits.",
            followup: "expect a practical cleanup story, not just raw bytes.",
            outro: "the shelves feel lighter now.",
            pulse_ms: 360,
        },
        develop: "develop" => {
            headline: "setting up the shell and keeping the terminal warm.",
            pulse: "bringing tools and command context into the same room.",
            followup: "you'll get a short runway before the heavy output starts.",
            outro: "the run finished with a cleaner landing.",
            pulse_ms: 280,
        },
        search: "search" => {
            headline: "turning the searchlight on and keeping the output readable.",
            pulse: "collecting the parts worth your eyes first.",
            followup: "i'll keep the signal colorful and grouped.",
            outro: "the answer should feel easier to hold now.",
            pulse_ms: 300,
        },
        install: "install" => {
            headline: "reshaping config with a little ceremony so it feels alive.",
            pulse: "lining up edits, previews, and the next likely move.",
            followup: "i'll show the shape of the change before i disappear.",
            outro: "the config moved without losing its softness.",
            pulse_ms: 320,
        },
        remove: "remove" => {
            headline: "reshaping config with a little ceremony so it feels alive.",
            pulse: "lining up edits, previews, and the next likely move.",
            followup: "i'll show the shape of the change before i disappear.",
            outro: "the config moved without losing its softness.",
            pulse_ms: 320,
        },
        r#try: "try" => {
            headline: "setting up the shell and keeping the terminal warm.",
            pulse: "bringing tools and command context into the same room.",
            followup: "you'll get a short runway before the heavy output starts.",
            outro: "the run finished with a cleaner landing.",
            pulse_ms: 280,
        },
        list: "list" => {
            headline: "turning the searchlight on and keeping the output readable.",
            pulse: "collecting the parts worth your eyes first.",
            followup: "i'll keep the signal colorful and grouped.",
            outro: "the answer should feel easier to hold now.",
            pulse_ms: 300,
        },
        fetch: "fetch" => {
            headline: "setting up the shell and keeping the terminal warm.",
            pulse: "bringing tools and command context into the same room.",
            followup: "you'll get a short runway before the heavy output starts.",
            outro: "the run finished with a cleaner landing.",
            pulse_ms: 280,
        },
        flake: "flake" => {
            headline: "reshaping config with a little ceremony so it feels alive.",
            pulse: "lining up edits, previews, and the next likely move.",
            followup: "i'll show the shape of the change before i disappear.",
            outro: "the config moved without losing its softness.",
            pulse_ms: 320,
        },
        fmt: "fmt" => {
            headline: "reshaping config with a little ceremony so it feels alive.",
            pulse: "lining up edits, previews, and the next likely move.",
            followup: "i'll show the shape of the change before i disappear.",
            outro: "the config moved without losing its softness.",
            pulse_ms: 320,
        },
        gen: "gen" => {
            headline: "setting up the shell and keeping the terminal warm.",
            pulse: "bringing tools and command context into the same room.",
            followup: "you'll get a short runway before the heavy output starts.",
            outro: "the run finished with a cleaner landing.",
            pulse_ms: 280,
        },
        hash: "hash" => {
            headline: "setting up the shell and keeping the terminal warm.",
            pulse: "bringing tools and command context into the same room.",
            followup: "you'll get a short runway before the heavy output starts.",
            outro: "the run finished with a cleaner landing.",
            pulse_ms: 280,
        },
        edit: "edit" => {
            headline: "reshaping config with a little ceremony so it feels alive.",
            pulse: "lining up edits, previews, and the next likely move.",
            followup: "i'll show the shape of the change before i disappear.",
            outro: "the config moved without losing its softness.",
            pulse_ms: 320,
        },
        check: "check" => {
            headline: "setting up the shell and keeping the terminal warm.",
            pulse: "bringing tools and command context into the same room.",
            followup: "you'll get a short runway before the heavy output starts.",
            outro: "the run finished with a cleaner landing.",
            pulse_ms: 280,
        },
        diff: "diff" => {
            headline: "waking up the prompt so it feels less like a cold subprocess.",
            pulse: "keeping you posted while the command takes shape.",
            followup: "i'll stay visible for a beat before i leave.",
            outro: "all done, i'll leave the trail here ♡",
            pulse_ms: 250,
        },
        info: "info" => {
            headline: "turning the searchlight on and keeping the output readable.",
            pulse: "collecting the parts worth your eyes first.",
            followup: "i'll keep the signal colorful and grouped.",
            outro: "the answer should feel easier to hold now.",
            pulse_ms: 300,
        },
        option: "option" => {
            headline: "turning the searchlight on and keeping the output readable.",
            pulse: "collecting the parts worth your eyes first.",
            followup: "i'll keep the signal colorful and grouped.",
            outro: "the answer should feel easier to hold now.",
            pulse_ms: 300,
        },
        pin: "pin" => {
            headline: "reshaping config with a little ceremony so it feels alive.",
            pulse: "lining up edits, previews, and the next likely move.",
            followup: "i'll show the shape of the change before i disappear.",
            outro: "the config moved without losing its softness.",
            pulse_ms: 320,
        },
        unpin: "unpin" => {
            headline: "reshaping config with a little ceremony so it feels alive.",
            pulse: "lining up edits, previews, and the next likely move.",
            followup: "i'll show the shape of the change before i disappear.",
            outro: "the config moved without losing its softness.",
            pulse_ms: 320,
        },
        pkg: "pkg" => {
            headline: "turning the searchlight on and keeping the output readable.",
            pulse: "collecting the parts worth your eyes first.",
            followup: "i'll keep the signal colorful and grouped.",
            outro: "the answer should feel easier to hold now.",
            pulse_ms: 300,
        },
        profile: "profile" => {
            headline: "reshaping config with a little ceremony so it feels alive.",
            pulse: "lining up edits, previews, and the next likely move.",
            followup: "i'll show the shape of the change before i disappear.",
            outro: "the config moved without losing its softness.",
            pulse_ms: 320,
        },
        repl: "repl" => {
            headline: "setting up the shell and keeping the terminal warm.",
            pulse: "bringing tools and command context into the same room.",
            followup: "you'll get a short runway before the heavy output starts.",
            outro: "the run finished with a cleaner landing.",
            pulse_ms: 280,
        },
        run: "run" => {
            headline: "setting up the shell and keeping the terminal warm.",
            pulse: "bringing tools and command context into the same room.",
            followup: "you'll get a short runway before the heavy output starts.",
            outro: "the run finished with a cleaner landing.",
            pulse_ms: 280,
        },
        service: "service" => {
            headline: "walking over to systemd with a little extra bedside manner.",
            pulse: "checking on the service before giving it a nudge.",
            followup: "you'll see the service story as it changes.",
            outro: "the service bit is all wrapped up ♡",
            pulse_ms: 300,
        },
        status: "status" => {
            headline: "turning the searchlight on and keeping the output readable.",
            pulse: "collecting the parts worth your eyes first.",
            followup: "i'll keep the signal colorful and grouped.",
            outro: "the answer should feel easier to hold now.",
            pulse_ms: 300,
        },
        store: "store" => {
            headline: "dusting the nix corners without losing your footing.",
            pulse: "looking for old weight and stray store bits.",
            followup: "expect a practical cleanup story, not just raw bytes.",
            outro: "the shelves feel lighter now.",
            pulse_ms: 360,
        },
        update: "update" => {
            headline: "shaking fresh metadata loose before anything bigger happens.",
            pulse: "refreshing channels and inputs without going blank on you.",
            followup: "i'll tell you when the network-y part is the reason we're waiting.",
            outro: "the channels look fresh again.",
            pulse_ms: 280,
        },
        upgrade: "upgrade" => {
            headline: "warming the rails before a full system move.",
            pulse: "checking the shape of your nix world first.",
            followup: "i'll narrate the big steps instead of dumping a silent wall.",
            outro: "the rebuild is all settled in ♡",
            pulse_ms: 420,
        },
        log: "log" => {
            headline: "turning the searchlight on and keeping the output readable.",
            pulse: "collecting the parts worth your eyes first.",
            followup: "i'll keep the signal colorful and grouped.",
            outro: "the answer should feel easier to hold now.",
            pulse_ms: 300,
        },
        doctor: "doctor" => {
            headline: "turning the searchlight on and keeping the output readable.",
            pulse: "collecting the parts worth your eyes first.",
            followup: "i'll keep the signal colorful and grouped.",
            outro: "the answer should feel easier to hold now.",
            pulse_ms: 300,
        },
        help: "help" => {
            headline: "turning the searchlight on and keeping the output readable.",
            pulse: "collecting the parts worth your eyes first.",
            followup: "i'll keep the signal colorful and grouped.",
            outro: "the answer should feel easier to hold now.",
            pulse_ms: 300,
        },
        hello: "hello" => {
            headline: "leaning in with a proper introduction instead of a dead splash line.",
            pulse: "finding your machines and giving the stage a little light.",
            followup: "this one is meant to feel playful.",
            outro: "the hello landed sweet ♡",
            pulse_ms: 320,
        },
        mood: "mood" => {
            headline: "turning the searchlight on and keeping the output readable.",
            pulse: "collecting the parts worth your eyes first.",
            followup: "i'll keep the signal colorful and grouped.",
            outro: "the answer should feel easier to hold now.",
            pulse_ms: 300,
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
    fn format_elapsed_prefers_millis_for_short_runs() {
        assert_eq!(format_elapsed(Duration::from_millis(245)), "245ms");
    }

    #[test]
    fn format_elapsed_uses_seconds_for_longer_runs() {
        assert_eq!(format_elapsed(Duration::from_millis(1500)), "1.5s");
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
