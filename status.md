# Nina status handoff for website updates

Last updated: 2026-03-30 (after daily-use readiness pass)

This file is the source of truth for another agent updating the website.

Everything described here is now **implemented and committed in git**. This is no longer a split between shipped work and local-only feature branches.

## TL;DR

Nina is now a much broader day-to-day NixOS helper than it was at the start of this session.

The biggest shipped changes are:

1. **Inline package search** replaced the old fullscreen search browser.
2. **Inline NixOS option search** now exists, with snippet copy and direct config insertion.
3. **Prompt-native install recovery** now handles fuzzy `nina install` queries without throwing users into a fullscreen app.
4. **Occasional kaomoji reactions** add small playful feedback to common interactions.
5. **A much larger command surface** is now live for flake workflows, service management, store inspection, package inspection, profile management, and related daily-use tasks.

If another agent is updating the website, it should treat the features below as the current live product.

---

## Live command surface

As of this handoff, `src/main.rs` exposes these top-level commands:

- `apply`
- `back`
- `build`
- `boot`
- `channel`
- `history`
- `go`
- `clean`
- `develop`
- `search`
- `install`
- `remove`
- `try`
- `list`
- `fetch`
- `flake`
- `fmt`
- `gen`
- `hash`
- `edit`
- `check`
- `diff`
- `info`
- `option`
- `pin`
- `unpin`
- `pkg`
- `profile`
- `repl`
- `run`
- `service`
- `status`
- `store`
- `update`
- `upgrade`
- `log`
- `doctor`
- `help`
- `hello`
- `mood`

This is the command list the website should use.

---

## What changed in this session

### 1. Search became inline and prompt-attached

The old fullscreen package search UI was replaced with a shared inline ratatui widget.

Current live behavior for `nina search`:

- type to search nixpkgs inline
- `↑/↓` navigate results
- `i` install selected package
- `t` open a temporary shell for the selected package
- `c` copy the package ref/snippet
- `esc` exit

Website-safe phrasing:

- “Search stays attached to the prompt instead of taking over the whole terminal.”
- “You can browse, preview, install, try, and copy from an inline terminal widget.”

### 2. `nina option` is now a real inline workflow

`nina option` now uses the same inline widget as package search, but targets NixOS options.

Current live behavior:

- queries `search.nixos.org` option data
- previews option metadata inline
- `c` copies a config snippet
- `i` inserts the selected option into `configuration.nix`
- supports local and remote machines via `--on <machine>`
- includes diff preview, confirmation, apply, and rollback behavior

Website-safe phrasing:

- “Search NixOS options inline, copy config snippets, or add them directly to your config.”

### 3. `nina install` got a prompt-native recovery flow

When a package query is not an exact nixpkgs match, Nina now stays in normal stdout/stdin and offers a compact selector.

Current live behavior:

- compact candidate list
- inline preview block
- `j/k` or number keys to browse
- `d` for more detail
- `enter` to select/install
- `q` to cancel cleanly

Website-safe phrasing:

- “Misspelled or fuzzy installs recover inline instead of dumping you into a separate fullscreen search mode.”

### 4. Nina has more playful personality now

Kaomoji bursts were added for meaningful interaction moments.

Current live moments include:

- search results arriving
- empty results
- detail peek
- copy success
- install prompt open
- cancel
- gentle error states

Website-safe phrasing:

- “Nina has subtle playful microinteractions, including small occasional kaomoji reactions.”
- “The dango mascot is still part of the experience, and now the command flows feel warmer too.”

---

## Expanded daily-use command families

These are now part of the real product, not roadmap items.

### Flake and local project workflows

Live:

- `nina build`
- `nina develop`
- `nina flake ...`
- `nina fetch`
- `nina fmt`
- `nina hash`
- `nina pin`
- `nina unpin`
- `nina repl`
- `nina run`

What these enable in practical terms:

- building flake outputs
- entering dev shells and running commands inside them
- common flake lifecycle operations
- prefetching URLs and copying hashes
- safe formatting of nix files
- path hashing
- temporary flake pinning
- quick nix repl access
- running apps/packages directly

### System inspection and lifecycle workflows

Live:

- `nina boot`
- `nina channel ...`
- `nina gen ...`
- `nina info`

What these enable:

- boot entry inspection
- channel management
- generation inspection and deletion helpers
- machine version/kernel/uptime/system metadata

### Package, profile, service, and store workflows

Live:

- `nina pkg ...`
- `nina profile ...`
- `nina service ...`
- `nina store ...`

What these enable:

- package dependency/path/closure/size inspection
- user profile package management
- service status/start/stop/restart/logs/enable/disable
- store info, verification, repair, path resolution, and GC helpers

---

## Product framing the website can safely use now

### Accurate high-level framing

> Nina is a warm Rust CLI/TUI for everyday NixOS management. It wraps rebuilds, rollback, generation browsing, inline package search, inline option search, safe config editing, remote-machine operations, flake workflows, package inspection, service management, store maintenance, and profile tooling in one friendlier terminal interface.

### Good website bullets

- Inline package search that stays attached to the prompt
- Inline NixOS option search with copy/add-to-config flows
- Safe config edits with diff preview, backup, confirm, and rollback
- Prompt-native install recovery for fuzzy package names
- Remote machine support with `--on <machine>`
- Service, store, profile, and package inspection commands
- Flake/dev-shell/day-to-day utility workflows
- Warm personality via dango + tiny occasional kaomoji reactions

### What not to understate anymore

The website should no longer present Nina as just:

- rebuild + rollback helper
- package search toy
- generation browser with some cute UI

That’s incomplete now. Nina is much closer to a broad “daily NixOS companion CLI.”

---

## Technical notes for the next agent

### Major shipped commits from this session

- `ecb2064` — Add kaomoji burst animations and prompt-native install picker
- `f517d6c` — Replace fullscreen search with inline search widget
- `e9ef0f2` — Add attached execution support for interactive commands
- `9590e57` — Add flake and local utility commands
- `26e4b9a` — Add system inspection and generation commands
- `7175b05` — Add package, profile, service, and store commands
- `4264727` — Expose the expanded Nina command surface
- `d05fb1e` — Update docs and demo for the expanded CLI
- `e593b1f` — docs: polish help text consistency for daily-use readiness

### Important architectural facts

- Search/option flows now use a shared inline ratatui widget.
- The old fullscreen search-only TUI modules are gone.
- Interactive commands now have attached execution support for shell-like experiences.
- `AGENTS.md` says **do not use Oracle** for this repo; use local checks plus relevant gstack-style review instead.

---

## Verification run during this work

The expanded command surface and inline-search changes were verified with:

- `lsp_diagnostics` on touched Rust files
- `cargo fmt`
- `cargo check`
- `cargo test`
- `cargo build`

Latest full result during this handoff:

- `cargo check` passed
- `cargo test` passed, 22/22
- `cargo build` passed

---

## Repo state expectation for handoff

After the final feature/docs commit, the repository should have:

- no uncommitted feature work left behind
- no local-only command families that the website has to tiptoe around

If another agent is updating the site, it can treat the current committed tree as the truth.

---

## Daily-use readiness pass (2026-03-30)

A final audit pass was completed before considering Nina ready for daily use.

### What was checked

- **Help text consistency** — All command families now correctly document `[--on <machine>]` where supported; examples show `--on laptop` usage
- **README alignment** — `nina go <n>` now shows the required argument; all commands listed match the committed CLI
- **Error handling tone** — All error paths use Nina's warm voice
- **Command structure** — All 37 commands are wired and consistent between `main.rs`, help text, and README
- **Edge cases** — Empty states, graceful failures, and confirmation flows are warm and clear throughout

### Changes from readiness pass

| File | Change |
|------|--------|
| `README.md` | `nina go` → `nina go <n>` |
| `src/commands/help.rs` | Added `[--on <machine>]` to channel, gen, pkg, profile, service, store help texts; removed errant `...` from flake usage; added `--on laptop` examples |

### Final verification

```
✓ cargo fmt    — no changes needed
✓ cargo check  — compiled cleanly
✓ cargo test   — 22/22 tests passed
✓ cargo build  — built successfully
✓ git push     — all commits pushed to origin/main
```

Nina is **daily-use ready**.
