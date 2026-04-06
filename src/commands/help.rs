use anyhow::Result;
use clap::Args;

use crate::commands::AppContext;

#[derive(Debug, Clone, Args)]
pub struct HelpArgs {
    pub command: Option<String>,
}

pub async fn run(ctx: &AppContext, args: HelpArgs) -> Result<()> {
    if let Some(command) = args.command {
        print_command_help(&command);
        return Ok(());
    }

    ctx.output.face("nina command guide ♡");
    ctx.output.print_plain("  apply     rebuild + switch");
    ctx.output.print_plain("  back      rollback one generation");
    ctx.output.print_plain("  boot      list bootloader entries");
    ctx.output.print_plain("  build     build the current flake or attr");
    ctx.output.print_plain("  channel   manage nix channels");
    ctx.output.print_plain("  clean     prune old generations + garbage collect");
    ctx.output.print_plain("  develop   enter the current dev shell");
    ctx.output.print_plain("  diff      compare generations");
    ctx.output.print_plain("  doctor    run diagnostics");
    ctx.output.print_plain("  edit      edit configuration.nix or hardware config");
    ctx.output.print_plain("  fetch     prefetch a URL and copy its hash");
    ctx.output.print_plain("  flake     init, update, check, show, or lock a flake");
    ctx.output.print_plain("  fmt       format configuration.nix safely");
    ctx.output.print_plain("  gen       quick generation helpers");
    ctx.output.print_plain("  go <n>    switch to a specific generation");
    ctx.output.print_plain("  hash      compute a nix hash for a local path");
    ctx.output.print_plain("  hello     meet nina");
    ctx.output.print_plain("  history   browse generations interactively");
    ctx.output.print_plain("  info      system version + uptime info");
    ctx.output.print_plain("  install   add a package to configuration.nix");
    ctx.output.print_plain("  list      list configured system packages");
    ctx.output.print_plain("  log       show ~/.nina.log history");
    ctx.output.print_plain("  mood      friendly status summary");
    ctx.output.print_plain("  option    search nixos options inline + add snippets");
    ctx.output.print_plain("  pin       pin a flake input to a commit");
    ctx.output.print_plain("  pkg       inspect package deps, path, and closure");
    ctx.output.print_plain("  profile   manage user profile packages");
    ctx.output.print_plain("  remove    remove a package from configuration.nix");
    ctx.output.print_plain("  repl      open nix repl with nixpkgs loaded");
    ctx.output.print_plain("  run       run a flake app or nixpkgs package");
    ctx.output.print_plain("  search    search nixpkgs inline from the prompt");
    ctx.output.print_plain("  service   manage systemd services");
    ctx.output.print_plain("  status    health + generation summary");
    ctx.output.print_plain("  store     inspect and maintain /nix/store");
    ctx.output.print_plain("  try       open a temporary nix shell");
    ctx.output.print_plain("  unpin     clear a temporary flake pin");
    ctx.output.print_plain("  update    refresh nix channels");
    ctx.output.print_plain("  upgrade   update + apply");
    ctx.output.print_plain("  help      explain a command in plain language");
    ctx.output.blank();
    ctx.output
        .tip("use: nina help <command> for flags, examples, and what nina does under the hood.");
    Ok(())
}

fn print_command_help(command: &str) {
    match command {
        "apply" => print_help(
            "nina apply [--dry] [--check] [--on <machine>]",
            "Builds your current configuration.nix and either dry-runs it, checks it, or switches to it.",
            "Under the hood: runs nixos-rebuild against the configured machine path.",
            &["nina apply", "nina apply --check --on laptop"],
        ),
        "back" => print_help(
            "nina back [--on <machine>]",
            "Rolls back exactly one system generation.",
            "Under the hood: runs nixos-rebuild switch --rollback.",
            &["nina back", "nina back --on homelab"],
        ),
        "boot" => print_help(
            "nina boot [--on <machine>]",
            "Shows the boot entry list so you can verify what the machine will boot next.",
            "Under the hood: runs bootctl list on the selected machine.",
            &["nina boot", "nina boot --on laptop"],
        ),
        "build" => print_help(
            "nina build [attr-or-url]",
            "Builds the current flake output, a specific attribute, or a remote flake target.",
            "Under the hood: runs nix build, optionally against .#attr or the URL you pass.",
            &["nina build", "nina build packages.x86_64-linux.default"],
        ),
        "channel" => print_help(
            "nina channel <list|add|remove|update> [--on <machine>]",
            "Manages classic nix channels when a machine is not using flakes for everything.",
            "Under the hood: wraps nix-channel with Nina's confirmation and machine routing.",
            &["nina channel list", "nina channel add https://nixos.org/channels/nixos-25.05 --on laptop"],
        ),
        "clean" => print_help(
            "nina clean [--all] [--on <machine>]",
            "Shows how many generations you have, asks for confirmation, then prunes and garbage-collects.",
            "Under the hood: deletes older generations according to config or runs nix-collect-garbage -d for --all.",
            &["nina clean", "nina clean --all --on builder"],
        ),
        "develop" => print_help(
            "nina develop [--run <cmd>] [--show] [--on <machine>]",
            "Enters the current flake's default dev shell, runs one command inside it, or previews what it provides.",
            "Under the hood: runs nix develop or nix flake show from your current directory.",
            &["nina develop", "nina develop --run cargo test", "nina develop --show"],
        ),
        "diff" => print_help(
            "nina diff [from] [to] [--on <machine>]",
            "Compares generations so you can see what changed between them.",
            "Under the hood: defaults to current vs previous, or uses the specific generation numbers you pass.",
            &["nina diff", "nina diff 41 42 --on desktop"],
        ),
        "doctor" => print_help(
            "nina doctor [--all] [--on <machine>]",
            "Runs health checks for nix daemon, syntax, disk, generations, channel setup, and more.",
            "Under the hood: executes a battery of targeted commands and prints repair hints for failures.",
            &["nina doctor", "nina doctor --all"],
        ),
        "edit" => print_help(
            "nina edit [configuration|hardware] [--on <machine>]",
            "Opens your config in $EDITOR locally, or downloads/uploads a remote copy for editing.",
            "Under the hood: local edit is direct, remote edit round-trips the file and offers a check afterward.",
            &["nina edit", "nina edit hardware --on server-a"],
        ),
        "fetch" => print_help(
            "nina fetch <url>",
            "Downloads a URL into the nix store, prints the hash, and copies it to your clipboard.",
            "Under the hood: runs nix-prefetch-url --print-path and then inspects the fetched store path.",
            &["nina fetch https://example.com/package-1.0.tar.gz"],
        ),
        "flake" => print_help(
            "nina flake <init|update|check|show|lock|clone>",
            "Wraps the everyday flake lifecycle so you do not have to memorize nix flake subcommands.",
            "Under the hood: runs nix flake init/update/check/show/lock/clone from your current directory.",
            &["nina flake init", "nina flake update nixpkgs", "nina flake show"],
        ),
        "fmt" => print_help(
            "nina fmt [--all] [--check] [--on <machine>]",
            "Formats configuration.nix, previews the changes, and only writes after confirmation.",
            "Under the hood: runs nixpkgs-fmt against one or more .nix files and compares the result before writing.",
            &["nina fmt", "nina fmt --all", "nina fmt --check --on laptop"],
        ),
        "gen" => print_help(
            "nina gen <list|current|delete> [--on <machine>]",
            "Provides quick generation commands alongside Nina's full history browser.",
            "Under the hood: wraps nix-env generation commands for the system profile.",
            &["nina gen list", "nina gen current", "nina gen delete old --on laptop"],
        ),
        "go" => print_help(
            "nina go <generation> [--on <machine>]",
            "Switches directly to a known generation number after verifying it exists.",
            "Under the hood: uses nix-env --switch-generation plus switch-to-configuration.",
            &["nina go 42", "nina go 17 --on desktop"],
        ),
        "hash" => print_help(
            "nina hash <path>",
            "Computes a nix hash for a local path or file and copies it to your clipboard.",
            "Under the hood: runs nix hash path in your current directory.",
            &["nina hash ./my-tarball.tar.gz"],
        ),
        "hello" => print_help(
            "nina hello",
            "Introduces Nina and shows the machines she knows about.",
            "Under the hood: reads your config and prints local/remote machine labels.",
            &["nina hello"],
        ),
        "history" => print_help(
            "nina history [--on <machine>]",
            "Opens the generation browser where you can inspect, diff, and switch generations.",
            "Under the hood: reads nix-env --list-generations and feeds an interactive ratatui view.",
            &["nina history", "nina history --on server-a"],
        ),
        "info" => print_help(
            "nina info [--on <machine>]",
            "Prints a machine summary with nixos version, kernel, state version, architecture, and uptime.",
            "Under the hood: combines nixos-version, uname, uptime, and nix --version on the selected machine.",
            &["nina info", "nina info --on homelab"],
        ),
        "install" => print_help(
            "nina install <package> [--no-apply] [--on <machine>]",
            "Adds an exact nixpkgs package to configuration.nix, previews the diff, and optionally rebuilds.",
            "Under the hood: resolves the package, edits environment.systemPackages, backs up, and can restore on failed apply.",
            &["nina install firefox", "nina install pkgs.neovim --no-apply"],
        ),
        "list" => print_help(
            "nina list [--grep <text>] [--on <machine>]",
            "Lists packages currently declared in environment.systemPackages, with metadata when available.",
            "Under the hood: parses configuration.nix locally or remotely and enriches entries from nix search.",
            &["nina list", "nina list --grep firefox"],
        ),
        "log" => print_help(
            "nina log [--last <count>] [--on <machine>]",
            "Shows recent Nina operations from ~/.nina.log.",
            "Under the hood: reads JSONL log entries and prints machine, command, outcome, and generation transitions.",
            &["nina log", "nina log --last 20 --on server-a"],
        ),
        "mood" => print_help(
            "nina mood",
            "Summarizes the overall vibe of your configured machines.",
            "Under the hood: reuses status data and rewrites it in Nina's friendlier tone.",
            &["nina mood"],
        ),
        "option" => print_help(
            "nina option [query]",
            "Searches NixOS options in an inline browser where you can copy snippets or add them straight to configuration.nix.",
            "Under the hood: renders the same inline ratatui widget used by package search, but currently points you to search.nixos.org when the remote options index needs auth.",
            &["nina option ollama", "nina option services.tailscale"],
        ),
        "pin" => print_help(
            "nina pin <input> <commit> | nina pin <input> --stable",
            "Temporarily pins a flake input in flake.lock without editing the lockfile by hand.",
            "Under the hood: uses nix flake lock --override-input so your flake.nix can keep its usual source URL.",
            &["nina pin nixpkgs 06278c77b5d1", "nina pin nixpkgs --stable"],
        ),
        "pkg" => print_help(
            "nina pkg <why|deps|size|path|closure> [--on <machine>]",
            "Inspects a package's store path, dependencies, size, and why it appears in the current system closure.",
            "Under the hood: wraps nix why-depends, nix-store, nix path-info, and nix eval helpers.",
            &["nina pkg why python3", "nina pkg size firefox --on laptop"],
        ),
        "profile" => print_help(
            "nina profile <list|install|remove|upgrade> [--on <machine>]",
            "Manages your user profile packages separately from system packages in configuration.nix.",
            "Under the hood: wraps nix profile commands on the selected machine.",
            &["nina profile list", "nina profile install ripgrep --on laptop"],
        ),
        "remove" => print_help(
            "nina remove <package> [--no-apply] [--on <machine>]",
            "Removes a package from configuration.nix with a preview before writing.",
            "Under the hood: edits environment.systemPackages, backs up, and can restore if rebuild fails.",
            &["nina remove firefox", "nina remove pkgs.neovim --on laptop"],
        ),
        "repl" => print_help(
            "nina repl [--pure]",
            "Opens nix repl with nixpkgs preloaded, unless you ask for a pure empty repl.",
            "Under the hood: runs nix repl directly, or with --expr 'import <nixpkgs> {}'.",
            &["nina repl", "nina repl --pure"],
        ),
        "run" => print_help(
            "nina run [pkg-or-url]",
            "Runs the current flake app, a nixpkgs package, or a remote flake app without installing it.",
            "Under the hood: runs nix run, adding --no-write-lock-file for remote flake URLs.",
            &["nina run", "nina run firefox", "nina run github:owner/app"],
        ),
        "search" => print_help(
            "nina search [query] [--on <machine>]",
            "Opens Nina's inline package browser so you can search, preview, install, try, or copy without leaving the prompt.",
            "Under the hood: runs `nix search` against nixpkgs, falls back when that path is unavailable, and renders results inside an inline ratatui viewport instead of a fullscreen dashboard.",
            &["nina search firefox", "nina search --on server-a"],
        ),
        "service" => print_help(
            "nina service <list|status|start|stop|restart|logs|enable|disable> [--on <machine>]",
            "Wraps systemd service management with Nina's machine routing and safer confirmations.",
            "Under the hood: uses systemctl and journalctl on the selected machine.",
            &["nina service list", "nina service logs ollama -f", "nina service restart tailscaled --on laptop"],
        ),
        "status" => print_help(
            "nina status [--all] [--on <machine>]",
            "Shows current generation, generation count, channel, dirty state, uptime, and store size.",
            "Under the hood: queries the system profile, config timestamps, nix-channel, and /nix/store usage.",
            &["nina status", "nina status --all"],
        ),
        "store" => print_help(
            "nina store <gc|verify|repair|info|path> [--on <machine>]",
            "Inspects store usage, verifies integrity, repairs corruption, and resolves package paths.",
            "Under the hood: wraps nix store, nix eval, and a few store accounting commands.",
            &["nina store info", "nina store verify --on laptop", "nina store path firefox"],
        ),
        "try" => print_help(
            "nina try <pkg>... [--on <machine>]",
            "Opens a temporary nix shell without changing your system config.",
            "Under the hood: runs nix shell against the requested package refs.",
            &["nina try ripgrep", "nina try nil alejandra"],
        ),
        "unpin" => print_help(
            "nina unpin <input>",
            "Clears a temporary flake lock override so the input follows your flake.nix source again.",
            "Under the hood: runs nix flake update --update-input for the named input.",
            &["nina unpin nixpkgs"],
        ),
        "update" => print_help(
            "nina update [--on <machine>]",
            "Refreshes nix channels without applying a new generation yet.",
            "Under the hood: runs nix-channel --update on the selected machine.",
            &["nina update", "nina update --on server-a"],
        ),
        "upgrade" => print_help(
            "nina upgrade [--check] [--on <machine>]",
            "Runs channel update and then applies, or checks, the new system state.",
            "Under the hood: combines nina update with nina apply.",
            &["nina upgrade", "nina upgrade --check --on laptop"],
        ),
        _ => {
            println!("(˶ᵔ ᵕ ᵔ˶) i don't have a custom guide for that yet, but clap --help always works ♡");
        }
    }
}

fn print_help(usage: &str, what_it_does: &str, under_the_hood: &str, examples: &[&str]) {
    println!("usage: {usage}\n");
    println!("what it does:");
    println!("  {what_it_does}\n");
    println!("under the hood:");
    println!("  {under_the_hood}\n");
    println!("examples:");
    for example in examples {
        println!("  {example}");
    }
}
