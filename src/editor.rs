use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn backup(path: &Path) -> Result<PathBuf> {
    let backup = path.with_extension("nix.nina-backup");
    fs::copy(path, &backup).with_context(|| format!("couldn't backup {}", path.display()))?;
    Ok(backup)
}

pub fn restore(path: &Path, backup: &Path) -> Result<()> {
    fs::copy(backup, path).with_context(|| {
        format!(
            "couldn't restore {} from backup {}",
            path.display(),
            backup.display()
        )
    })?;
    Ok(())
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn add_package(config_nix: &Path, package: &str) -> Result<String> {
    let original = fs::read_to_string(config_nix)
        .with_context(|| format!("couldn't read {}", config_nix.display()))?;
    let edit = prepare_add_package(&original, package)?;
    fs::write(config_nix, &edit.updated)
        .with_context(|| format!("couldn't write {}", config_nix.display()))?;

    Ok(format!("added {} to {}", package, config_nix.display()))
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn remove_package(config_nix: &Path, package: &str) -> Result<String> {
    let original = fs::read_to_string(config_nix)
        .with_context(|| format!("couldn't read {}", config_nix.display()))?;
    let edit = prepare_remove_package(&original, package)?;
    fs::write(config_nix, edit.updated)
        .with_context(|| format!("couldn't write {}", config_nix.display()))?;
    Ok(format!("removed {} from {}", package, config_nix.display()))
}

pub fn read_contents(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("couldn't read {}", path.display()))
}

pub fn write_contents(path: &Path, contents: &str) -> Result<()> {
    fs::write(path, contents).with_context(|| format!("couldn't write {}", path.display()))
}

pub fn prepare_add_package(contents: &str, package: &str) -> Result<EditPreview> {
    let mut lines: Vec<String> = contents.lines().map(ToString::to_string).collect();
    let block = find_system_packages_block(&lines)?;

    if lines[block.content_start..block.content_end]
        .iter()
        .any(|l| l.trim() == package)
    {
        return Ok(EditPreview {
            updated: ensure_trailing_newline(contents),
            diff: "package is already listed, no changes needed ♡".to_string(),
            changed: false,
        });
    }

    let insert_at = lines[block.content_start..block.content_end]
        .iter()
        .position(|line| line.trim() > package)
        .map(|idx| block.content_start + idx)
        .unwrap_or(block.content_end);

    lines.insert(insert_at, format!("{}{}", block.entry_indent, package));
    let updated = format!("{}\n", lines.join("\n"));
    Ok(EditPreview {
        diff: diff_preview(contents, &updated),
        updated,
        changed: true,
    })
}

pub fn prepare_remove_package(contents: &str, package: &str) -> Result<EditPreview> {
    let lines: Vec<&str> = contents.lines().collect();
    let block = find_system_packages_block_from_strs(&lines)?;
    let mut removed = false;
    let mut output = vec![];

    for (idx, line) in contents.lines().enumerate() {
        if idx >= block.content_start && idx < block.content_end && line.trim() == package {
            removed = true;
            continue;
        }
        output.push(line);
    }

    if !removed {
        return Ok(EditPreview {
            updated: ensure_trailing_newline(contents),
            diff: format!("{} wasn't found in systemPackages", package),
            changed: false,
        });
    }

    let updated = format!("{}\n", output.join("\n"));
    Ok(EditPreview {
        diff: diff_preview(contents, &updated),
        updated,
        changed: true,
    })
}

pub fn prepare_add_option_snippet(
    contents: &str,
    option_name: &str,
    snippet: &str,
) -> Result<EditPreview> {
    let normalized = normalize_snippet(snippet);
    if normalized.is_empty() {
        return Err(anyhow!(
            "couldn't build a config snippet for {}",
            option_name
        ));
    }

    let first_line = normalized.lines().next().unwrap_or_default().trim();
    if !first_line.is_empty() && contents.lines().any(|line| line.trim() == first_line) {
        return Ok(EditPreview {
            updated: ensure_trailing_newline(contents),
            diff: format!("{} is already present, no changes needed ♡", option_name),
            changed: false,
        });
    }

    let mut lines = contents
        .lines()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let insert_at = find_option_insert_index(&lines, option_name)?;
    let indent = option_indent(&lines, insert_at);
    let snippet_lines = normalized
        .lines()
        .map(|line| format!("{indent}{line}"))
        .collect::<Vec<_>>();

    for (offset, line) in snippet_lines.into_iter().enumerate() {
        lines.insert(insert_at + offset, line);
    }

    let updated = format!("{}\n", lines.join("\n"));
    Ok(EditPreview {
        diff: diff_preview(contents, &updated),
        updated,
        changed: true,
    })
}

pub fn list_packages(config_nix: &Path) -> Result<Vec<String>> {
    let raw = fs::read_to_string(config_nix)
        .with_context(|| format!("couldn't read {}", config_nix.display()))?;

    let lines: Vec<&str> = raw.lines().collect();
    let block = find_system_packages_block_from_strs(&lines)?;
    Ok(lines[block.content_start..block.content_end]
        .iter()
        .map(|line| line.trim())
        .filter(|trimmed| !trimmed.is_empty() && !trimmed.starts_with('#'))
        .map(ToString::to_string)
        .collect())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SystemPackagesBlock {
    content_start: usize,
    content_end: usize,
    entry_indent: String,
}

#[derive(Debug, Clone)]
pub struct EditPreview {
    pub updated: String,
    pub diff: String,
    pub changed: bool,
}

fn find_system_packages_block(lines: &[String]) -> Result<SystemPackagesBlock> {
    let refs = lines.iter().map(String::as_str).collect::<Vec<_>>();
    find_system_packages_block_from_strs(&refs)
}

fn find_option_insert_index(lines: &[String], option_name: &str) -> Result<usize> {
    let top_level = option_name.split('.').next().unwrap_or(option_name);
    if let Some((idx, _)) = lines.iter().enumerate().rev().find(|(_, line)| {
        let trimmed = line.trim_start();
        trimmed.starts_with(&format!("{top_level}."))
            || trimmed.starts_with(&format!("{top_level} ="))
            || trimmed.starts_with(&format!("{top_level}="))
    }) {
        return Ok(idx + 1);
    }

    lines
        .iter()
        .enumerate()
        .rev()
        .find(|(_, line)| line.trim() == "}")
        .map(|(idx, _)| idx)
        .ok_or_else(|| anyhow!("couldn't find where to insert {}", option_name))
}

fn option_indent(lines: &[String], insert_at: usize) -> String {
    lines
        .get(insert_at.saturating_sub(1))
        .map(|line| {
            line.chars()
                .take_while(|ch| ch.is_whitespace())
                .collect::<String>()
        })
        .filter(|indent| !indent.is_empty())
        .unwrap_or_else(|| "  ".to_string())
}

fn normalize_snippet(snippet: &str) -> String {
    let lines = snippet.lines().map(str::trim_end).collect::<Vec<_>>();
    let start = lines
        .iter()
        .position(|line| !line.trim().is_empty())
        .unwrap_or(lines.len());
    let end = lines
        .iter()
        .rposition(|line| !line.trim().is_empty())
        .map(|idx| idx + 1)
        .unwrap_or(start);
    let trimmed = &lines[start..end];
    if trimmed.is_empty() {
        return String::new();
    }

    let min_indent = trimmed
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|ch| ch.is_whitespace()).count())
        .min()
        .unwrap_or(0);

    trimmed
        .iter()
        .map(|line| line.chars().skip(min_indent).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

fn find_system_packages_block_from_strs(lines: &[&str]) -> Result<SystemPackagesBlock> {
    let start = lines
        .iter()
        .position(|line| line.contains("environment.systemPackages"))
        .ok_or_else(|| anyhow!("couldn't find environment.systemPackages block"))?;

    let mut depth = 0_i32;
    let mut saw_open = false;
    let mut content_start = None;
    let mut content_end = None;

    for (idx, line) in lines.iter().enumerate().skip(start) {
        for ch in line.chars() {
            match ch {
                '[' => {
                    depth += 1;
                    if !saw_open {
                        saw_open = true;
                        content_start = Some(idx + 1);
                    }
                }
                ']' if saw_open => {
                    depth -= 1;
                    if depth == 0 {
                        content_end = Some(idx);
                        break;
                    }
                }
                _ => {}
            }
        }

        if content_end.is_some() {
            break;
        }
    }

    let content_start =
        content_start.ok_or_else(|| anyhow!("couldn't find start of systemPackages list"))?;
    let content_end =
        content_end.ok_or_else(|| anyhow!("couldn't find end of systemPackages list"))?;

    let entry_indent = lines[content_start..content_end]
        .iter()
        .map(|line| {
            line.chars()
                .take_while(|c| c.is_whitespace())
                .collect::<String>()
        })
        .find(|indent| !indent.is_empty())
        .unwrap_or_else(|| {
            let closing_indent = lines[content_end]
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect::<String>();
            format!("{}  ", closing_indent)
        });

    Ok(SystemPackagesBlock {
        content_start,
        content_end,
        entry_indent,
    })
}

fn diff_preview(before: &str, after: &str) -> String {
    let before_lines = before.lines().collect::<Vec<_>>();
    let after_lines = after.lines().collect::<Vec<_>>();
    let max_len = before_lines.len().max(after_lines.len());
    let mut out = vec!["changes to configuration.nix:".to_string(), String::new()];

    for idx in 0..max_len {
        match (before_lines.get(idx), after_lines.get(idx)) {
            (Some(a), Some(b)) if a == b => {}
            (Some(a), Some(b)) => {
                out.push(format!("line {}:", idx + 1));
                out.push(format!("- {}", a));
                out.push(format!("+ {}", b));
                out.push(String::new());
            }
            (Some(a), None) => {
                out.push(format!("line {}:", idx + 1));
                out.push(format!("- {}", a));
                out.push(String::new());
            }
            (None, Some(b)) => {
                out.push(format!("line {}:", idx + 1));
                out.push(format!("+ {}", b));
                out.push(String::new());
            }
            (None, None) => {}
        }
    }

    out.join("\n").trim_end().to_string()
}

fn ensure_trailing_newline(contents: &str) -> String {
    if contents.ends_with('\n') {
        contents.to_string()
    } else {
        format!("{contents}\n")
    }
}

#[cfg(test)]
mod tests {
    use super::{
        diff_preview, find_system_packages_block_from_strs, list_packages,
        prepare_add_option_snippet, prepare_add_package,
    };
    use crate::editor::{add_package, remove_package};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file(name: &str, contents: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("nina-{name}-{nanos}.nix"));
        fs::write(&path, contents).expect("write temp nix file");
        path
    }

    #[test]
    fn find_block_uses_matching_closing_bracket() {
        let lines = vec![
            "{ pkgs, ... }:",
            "{",
            "  environment.systemPackages = with pkgs; [",
            "    (python3.withPackages (ps: with ps; [ requests ]))",
            "    ripgrep",
            "  ];",
            "}",
        ];

        let block = find_system_packages_block_from_strs(&lines).expect("find block");
        assert_eq!(block.content_start, 3);
        assert_eq!(block.content_end, 5);
    }

    #[test]
    fn add_package_inserts_inside_system_packages_block() {
        let path = temp_file(
            "add-package",
            "{ pkgs, ... }:\n{\n  environment.systemPackages = with pkgs; [\n    git\n  ];\n}\n",
        );

        add_package(&path, "pkgs.spotify").expect("add package");
        let updated = fs::read_to_string(&path).expect("read updated file");
        assert!(updated.contains("    pkgs.spotify\n  ];"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn add_package_sorts_alphabetically() {
        let preview = prepare_add_package(
            "{ pkgs, ... }:\n{\n  environment.systemPackages = with pkgs; [\n    git\n    zellij\n  ];\n}\n",
            "pkgs.firefox",
        )
        .expect("prepare add preview");

        assert!(preview
            .updated
            .contains("    git\n    pkgs.firefox\n    zellij"));
    }

    #[test]
    fn diff_preview_shows_changed_line() {
        let diff = diff_preview("hello\nworld\n", "hello\nrust\n");
        assert!(diff.contains("line 2:"));
        assert!(diff.contains("- world"));
        assert!(diff.contains("+ rust"));
    }

    #[test]
    fn add_option_snippet_inserts_before_root_closing_brace() {
        let preview = prepare_add_option_snippet(
            "{ pkgs, ... }:\n{\n  services.openssh.enable = true;\n}\n",
            "services.ollama.enable",
            "services.ollama.enable = true;",
        )
        .expect("prepare add option preview");

        assert!(preview
            .updated
            .contains("services.openssh.enable = true;\n  services.ollama.enable = true;\n}"));
    }

    #[test]
    fn add_option_snippet_is_noop_when_present() {
        let preview = prepare_add_option_snippet(
            "{ pkgs, ... }:\n{\n  services.ollama.enable = true;\n}\n",
            "services.ollama.enable",
            "services.ollama.enable = true;",
        )
        .expect("prepare add option preview");

        assert!(!preview.changed);
    }

    #[test]
    fn remove_package_only_touches_system_packages_block() {
        let path = temp_file(
            "remove-package",
            "{ pkgs, ... }:\n{\n  networking.hostName = \"pkgs.spotify\";\n  environment.systemPackages = with pkgs; [\n    pkgs.spotify\n    git\n  ];\n}\n",
        );

        remove_package(&path, "pkgs.spotify").expect("remove package");
        let updated = fs::read_to_string(&path).expect("read updated file");
        assert!(updated.contains("networking.hostName = \"pkgs.spotify\";"));
        assert!(!updated.contains("\n    pkgs.spotify\n"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn list_packages_reads_only_system_packages_entries() {
        let path = temp_file(
            "list-packages",
            "{ pkgs, ... }:\n{\n  environment.systemPackages = with pkgs; [\n    # keep this\n    git\n    (python3.withPackages (ps: with ps; [ requests ]))\n  ];\n  programs.zsh.enable = true;\n}\n",
        );

        let packages = list_packages(&path).expect("list packages");
        assert_eq!(
            packages,
            vec!["git", "(python3.withPackages (ps: with ps; [ requests ]))"]
        );

        let _ = fs::remove_file(path);
    }
}
