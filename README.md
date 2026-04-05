<div align="center">

<img src="demo.gif" alt="nina in action — package search, apply, and history browsing" width="720" />

<br/>

# nina (˶ᵔ ᵕ ᵔ˶)

**nixos management that actually feels good**

[![rust](https://img.shields.io/badge/built%20with-rust-ffd6a5?style=flat-square&logo=rust&logoColor=555)](https://www.rust-lang.org/)
[![nix flake](https://img.shields.io/badge/nix-flake%20ready-caffbf?style=flat-square&logo=nixos&logoColor=555)](https://nixos.org/)
[![tui](https://img.shields.io/badge/interactive-tui-c8b6ff?style=flat-square)](https://github.com/ratatui-org/ratatui)
[![license](https://img.shields.io/badge/license-MIT-ffc8dd?style=flat-square)](./LICENSE)

*a cozy CLI/TUI that wraps nixos-rebuild, generation browsing, inline package search, and diagnostics in one warm interface* 🍡

</div>

---

## why nina? ♡

nixos is powerful — but its default commands are a wall of flags, cryptic errors, and manual recovery steps. nina wraps all of that in something that's actually a joy to use.

| the usual way | with nina |
|---|---|
| `sudo nixos-rebuild switch` | `nina apply` — with a dango mascot sweeping your terminal |
| `nix search nixpkgs ripgrep` | `nina search` — inline search attached to the prompt with preview + one-key actions |
| `nix-env --list-generations` | `nina history` — interactive browser, diffs between generations |
| `vim /etc/nixos/configuration.nix` (no safety net) | `nina edit` — auto-backup, diff preview, rollback if rebuild fails |
| googling a cryptic nix error | nina translates it into plain language with a suggested fix |
| manually ssh-ing into each machine | `nina mood` — health check across all your machines at once |
| `nix-collect-garbage -d` | `nina clean` — guided, shows what'll be freed |

---

## features ✨

**🔍 inline package + option search**
fuzzy-match anything in nixpkgs or NixOS options. browse descriptions, copy snippets, try packages, and install or insert config directly from an inline prompt-attached browser.

**📜 generation history browser**
scroll through every system generation, preview diffs, and jump between them — all without leaving the terminal.

**🛡️ safe config editing**
`nina edit` backs up your `configuration.nix` before touching it, previews the diff, and automatically rolls back if the rebuild fails.

**🩺 diagnostics & health**
`nina doctor` checks your channels, nix daemon, disk space, syntax, and generations — and tells you exactly what to fix.

**🌐 remote machine support**
manage multiple NixOS machines with `--on <machine>`. SSH-based, key-auth, fully unified.

**🎓 teach mode**
not sure what nina is actually running? set `teach: true` in `~/.nina.conf` and she'll print the exact nix commands before executing them.

**🍡 dango mascot + tiny kaomoji bursts**
she walks, sweeps, waves, dances, and cheers. Nina also does little occasional kaomoji reactions for common interactions. purely optional. entirely delightful.

**💬 living prompt + command runway**
run plain `nina` and you get a tiny interactive prompt instead of a dead one-liner. run `nina <command>` and she now pauses just long enough to frame the action, animate the dango, keep you posted with grouped colorful status lines, and then bow out.

---

## commands

```
nina apply        apply your configuration (nixos-rebuild switch)
nina back         roll back to the previous generation
nina boot         list boot entries
nina build        build the current flake or a target attr
nina channel      manage nix channels
nina develop      enter the current dev shell
nina fetch        prefetch a URL and copy its hash
nina flake        manage flake lifecycle commands
nina fmt          format configuration.nix safely
nina gen          quick generation helpers
nina go <n>       switch to a specific generation
nina hash         compute a nix hash for a local path
nina history      browse generations interactively
nina info         system version, kernel, uptime
nina option       fuzzy-search nixos options inline and add snippets
nina pkg          inspect package deps, path, and closure
nina pin          pin a flake input in flake.lock
nina profile      manage user profile packages
nina repl         open nix repl with nixpkgs loaded
nina run          run a flake app or nixpkgs package
nina service      manage systemd services
nina store        inspect and maintain /nix/store
nina unpin        clear a temporary flake pin
nina diff         diff between two generations

nina search       fuzzy-search nixpkgs inline from the prompt
nina install      add a package to configuration.nix
nina remove       remove a package from configuration.nix
nina try          test a package in a shell without installing
nina list         list what's in your configuration.nix

nina edit         edit configuration.nix safely (with backup + rollback)
nina check        validate your config without building
nina status       system info and disk usage
nina doctor       run full diagnostics
nina clean        garbage collect old generations
nina upgrade      update flake inputs and rebuild
nina update       update nix channels
nina log          view operation history

nina mood         quick health check across all your machines
nina hello        greet nina and see your configured machines
```

---

## install

**one-liner (fast, no build):**
```bash
curl -fsSL "https://github.com/hopeineveryline/nina/releases/latest/download/nina-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m | sed 's/x86_64/x64/;s/aarch64/arm64/').tar.gz" -o /tmp/nina.tar.gz && tar xzf /tmp/nina.tar.gz -C /tmp && chmod +x /tmp/nina && mv /tmp/nina ~/bin/ && rm /tmp/nina.tar.gz
```

> ~2MB, no Nix, no Rust toolchain. works on Linux (x64/arm64) and macOS (x64/arm64).

**with nix:**
```bash
nix run github:hopeineveryline/nina
```

or install to your profile:
```bash
nix profile install github:hopeineveryline/nina
```

> ⚠ nix builds from source (~2GB on first run). use the one-liner above for a instant install.

**from a local checkout:**
```bash
nix run .
# or
cargo build && ./target/debug/nina help
```

> requires nix 2.4+ with `nix-command` and `flakes` enabled

on first run, nina writes a starter config to `~/.nina.conf` — no setup needed.

---

## shell completions

building the crate generates completions for bash, zsh, and fish in `./completions/`.

for source installs:
```bash
./install-completions.sh
```

nix installs get completions automatically via `nina.nix`.

---

---

## project structure

```
src/
  commands/    one file per command
  tui/         ratatui history UI + inline search widget
  dango.rs     mascot animations
  editor.rs    safe config editing
  errors.rs    friendly error translation
  config.rs    ~/.nina.conf parsing
  machine.rs   local + remote routing
  exec.rs      command execution
  output.rs    colorized terminal output
  log.rs       jsonl operation log
```

---

<div align="center">

made with ♡ for nixos users who just want things to feel a little warmer

*nina stays warm, lowercase, and encouraging — always*

</div>
