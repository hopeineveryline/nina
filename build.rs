use clap::{Arg, ArgAction, Command};
use clap_complete::{
    generate_to,
    shells::{Bash, Fish, Zsh},
};
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"))
        .join("completions");
    fs::create_dir_all(&out_dir).expect("create completions dir");

    let mut cmd = cli();
    generate_to(Bash, &mut cmd, "nina", &out_dir).expect("bash completions");
    let mut cmd = cli();
    generate_to(Zsh, &mut cmd, "nina", &out_dir).expect("zsh completions");
    let mut cmd = cli();
    generate_to(Fish, &mut cmd, "nina", &out_dir).expect("fish completions");
}

fn cli() -> Command {
    Command::new("nina")
        .about("nina, your friendly nix helper ♡")
        .disable_help_subcommand(true)
        .subcommand(
            Command::new("apply")
                .arg(Arg::new("on").long("on").value_name("machine"))
                .arg(Arg::new("dry").long("dry").action(ArgAction::SetTrue))
                .arg(Arg::new("check").long("check").action(ArgAction::SetTrue)),
        )
        .subcommand(Command::new("back").arg(Arg::new("on").long("on").value_name("machine")))
        .subcommand(
            Command::new("history")
                .arg(Arg::new("on").long("on").value_name("machine"))
                .arg(Arg::new("tui").long("tui").action(ArgAction::SetTrue)),
        )
        .subcommand(
            Command::new("go")
                .arg(Arg::new("generation").required(true))
                .arg(Arg::new("on").long("on").value_name("machine")),
        )
        .subcommand(
            Command::new("clean")
                .arg(Arg::new("on").long("on").value_name("machine"))
                .arg(Arg::new("all").long("all").action(ArgAction::SetTrue)),
        )
        .subcommand(
            Command::new("search")
                .arg(Arg::new("query"))
                .arg(Arg::new("on").long("on").value_name("machine")),
        )
        .subcommand(
            Command::new("install")
                .arg(Arg::new("package").required(true))
                .arg(Arg::new("on").long("on").value_name("machine"))
                .arg(
                    Arg::new("no-apply")
                        .long("no-apply")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("remove")
                .arg(Arg::new("package").required(true))
                .arg(Arg::new("on").long("on").value_name("machine"))
                .arg(
                    Arg::new("no-apply")
                        .long("no-apply")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("try")
                .arg(Arg::new("packages").num_args(1..).required(true))
                .arg(Arg::new("on").long("on").value_name("machine")),
        )
        .subcommand(
            Command::new("list")
                .arg(Arg::new("on").long("on").value_name("machine"))
                .arg(Arg::new("grep").long("grep").value_name("needle")),
        )
        .subcommand(
            Command::new("edit")
                .arg(Arg::new("target"))
                .arg(Arg::new("on").long("on").value_name("machine")),
        )
        .subcommand(Command::new("check").arg(Arg::new("on").long("on").value_name("machine")))
        .subcommand(
            Command::new("diff")
                .arg(Arg::new("from"))
                .arg(Arg::new("to"))
                .arg(Arg::new("on").long("on").value_name("machine")),
        )
        .subcommand(
            Command::new("status")
                .arg(Arg::new("on").long("on").value_name("machine"))
                .arg(Arg::new("all").long("all").action(ArgAction::SetTrue)),
        )
        .subcommand(Command::new("update").arg(Arg::new("on").long("on").value_name("machine")))
        .subcommand(
            Command::new("upgrade")
                .arg(Arg::new("on").long("on").value_name("machine"))
                .arg(Arg::new("check").long("check").action(ArgAction::SetTrue)),
        )
        .subcommand(
            Command::new("log")
                .arg(Arg::new("last").long("last").value_name("count"))
                .arg(Arg::new("on").long("on").value_name("machine")),
        )
        .subcommand(
            Command::new("doctor")
                .arg(Arg::new("on").long("on").value_name("machine"))
                .arg(Arg::new("all").long("all").action(ArgAction::SetTrue)),
        )
        .subcommand(Command::new("help").arg(Arg::new("command")))
        .subcommand(Command::new("hello"))
        .subcommand(Command::new("mood"))
}
