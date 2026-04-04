use anyhow::{Context, Result};
use std::io::{self, IsTerminal, Write};

use crate::packages::NixPackage;

#[derive(Debug, Clone)]
pub struct InstallSelection {
    pub exact: NixPackage,
    pub suggestions: Vec<NixPackage>,
}

#[derive(Debug, Clone)]
pub enum InstallMenuChoice {
    Selected(InstallSelection),
    Cancelled,
    Unavailable,
}

pub async fn choose_package(query: &str, machine: &str) -> Result<InstallMenuChoice> {
    let results = crate::packages::query_packages(query).await?;
    if results.is_empty() {
        return Ok(InstallMenuChoice::Unavailable);
    }

    if !(io::stdin().is_terminal() && io::stdout().is_terminal()) {
        return Ok(InstallMenuChoice::Unavailable);
    }

    println!();
    println!("(˶ᵔ ᵕ ᵔ˶) i found a few matches for '{query}' on {machine}.");
    println!("           let's pick one right here, no fullscreen detour ♡");

    let mut selected = 0usize;
    let mut show_long = false;
    let max_items = results.len().min(6);

    loop {
        render_menu(query, &results, selected, show_long, max_items);
        print!("  choice > ");
        io::stdout()
            .flush()
            .context("failed to flush install picker")?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .context("failed to read install picker input")?;

        let trimmed = input.trim();
        match trimmed {
            "" => {
                return Ok(InstallMenuChoice::Selected(InstallSelection {
                    exact: results[selected].clone(),
                    suggestions: related_suggestions(&results, selected),
                }))
            }
            "j" | "n" => {
                selected = (selected + 1).min(max_items.saturating_sub(1));
            }
            "k" | "p" => {
                selected = selected.saturating_sub(1);
            }
            "d" => {
                show_long = !show_long;
            }
            "q" => return Ok(InstallMenuChoice::Cancelled),
            value => {
                if let Ok(choice) = value.parse::<usize>() {
                    if (1..=max_items).contains(&choice) {
                        let selected = choice - 1;
                        return Ok(InstallMenuChoice::Selected(InstallSelection {
                            exact: results[selected].clone(),
                            suggestions: related_suggestions(&results, selected),
                        }));
                    }
                }
                println!("  use [enter], 1-{max_items}, j/k, d, or q ♡");
            }
        }
    }
}

fn render_menu(
    query: &str,
    results: &[NixPackage],
    selected: usize,
    show_long: bool,
    max_items: usize,
) {
    println!();
    println!("  search   {query}");
    println!("  menu     [enter] install selected  ·  [j/k] browse  ·  [d] details  ·  [q] cancel");
    println!();

    for (index, pkg) in results.iter().take(max_items).enumerate() {
        let marker = if index == selected { "▸" } else { " " };
        let version = pkg.version.as_deref().unwrap_or("unknown");
        println!(
            "  {marker} {}. {}  ·  {}",
            index + 1,
            package_label(pkg),
            version
        );
        println!("       {}", compact_summary(pkg));
    }

    println!();
    println!("  preview");
    for line in preview_lines(&results[selected], show_long) {
        println!("    {line}");
    }
    println!();
}

fn related_suggestions(results: &[NixPackage], selected: usize) -> Vec<NixPackage> {
    results
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != selected)
        .map(|(_, pkg)| pkg.clone())
        .take(5)
        .collect()
}

fn package_label(pkg: &NixPackage) -> String {
    if pkg.attribute.is_empty() {
        pkg.name.clone()
    } else {
        pkg.attribute.clone()
    }
}

fn compact_summary(pkg: &NixPackage) -> String {
    let raw = pkg
        .description
        .as_deref()
        .or(pkg.long_description.as_deref())
        .unwrap_or("no description from nix search yet");
    let single_line = raw
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or(raw);
    truncate(single_line.trim(), 76)
}

fn preview_lines(pkg: &NixPackage, show_long: bool) -> Vec<String> {
    let mut lines = vec![
        format!("attribute   {}", package_label(pkg)),
        format!(
            "version     {}",
            pkg.version.as_deref().unwrap_or("unknown")
        ),
    ];

    if let Some(license) = &pkg.license {
        lines.push(format!("license     {license}"));
    }
    if let Some(homepage) = &pkg.homepage {
        lines.push(format!("homepage    {homepage}"));
    }
    if !pkg.platforms.is_empty() {
        lines.push(format!(
            "platforms   {}",
            truncate(&pkg.platforms.join(", "), 76)
        ));
    }

    lines.push(String::new());
    lines.push(compact_summary(pkg));

    if show_long {
        if let Some(long) = &pkg.long_description {
            for line in long.lines().filter(|line| !line.trim().is_empty()).take(6) {
                lines.push(truncate(line.trim(), 76));
            }
        }
    }

    lines
}

fn truncate(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }

    let clipped = text
        .chars()
        .take(limit.saturating_sub(1))
        .collect::<String>();
    format!("{clipped}…")
}

#[cfg(test)]
mod tests {
    use super::{compact_summary, preview_lines};
    use crate::packages::NixPackage;

    #[test]
    fn compact_summary_prefers_first_non_empty_line() {
        let pkg = NixPackage {
            description: Some("\nfirst line\nsecond line".to_string()),
            ..NixPackage::default()
        };

        assert_eq!(compact_summary(&pkg), "first line");
    }

    #[test]
    fn preview_lines_include_core_metadata() {
        let pkg = NixPackage {
            attribute: "ripgrep".to_string(),
            version: Some("14.1.1".to_string()),
            description: Some("fast search".to_string()),
            ..NixPackage::default()
        };

        let lines = preview_lines(&pkg, false);
        assert!(lines
            .iter()
            .any(|line| line.contains("attribute   ripgrep")));
        assert!(lines.iter().any(|line| line.contains("version     14.1.1")));
        assert!(lines.iter().any(|line| line.contains("fast search")));
    }
}
