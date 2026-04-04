use anyhow::{Context, Result};
use serde::de::Deserializer;
use serde::Deserialize;
use serde_json::Value;
use tokio::task::JoinSet;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct NixPackage {
    #[serde(default, alias = "package_attr_name")]
    pub attribute: String,
    #[serde(default, alias = "package_pname", alias = "pname")]
    pub name: String,
    #[serde(default, alias = "package_version")]
    pub version: Option<String>,
    #[serde(default, alias = "package_description")]
    pub description: Option<String>,
    #[serde(default, deserialize_with = "deserialize_license")]
    pub license: Option<String>,
    #[serde(default, deserialize_with = "deserialize_stringish")]
    pub homepage: Option<String>,
    #[serde(default, deserialize_with = "deserialize_platforms")]
    pub platforms: Vec<String>,
    #[serde(default, alias = "package_longDescription")]
    pub long_description: Option<String>,
    #[serde(
        default,
        alias = "package_size",
        alias = "package_download_size",
        alias = "package_installed_size",
        deserialize_with = "deserialize_sizeish"
    )]
    pub size: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PackageResolution {
    pub exact: NixPackage,
    pub suggestions: Vec<NixPackage>,
}

pub async fn query_packages(query: &str) -> Result<Vec<NixPackage>> {
    // Use `nix search` as the primary search method — it queries the local flake's
    // nixpkgs directly and doesn't depend on external search.nixos.org infrastructure.
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let output = tokio::process::Command::new("nix")
        .args(["search", "nixpkgs", trimmed, "--json"])
        .output()
        .await
        .context("nix search failed — is nix installed and nixpkgs available?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("could not find") || stderr.contains("does not provide") {
            anyhow::bail!("nix search couldn't find nixpkgs — try running from your flake directory, or set NINA_NIXPKGS to a flake ref");
        }
        anyhow::bail!("nix search exited with {}: {}", output.status, stderr.trim());
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&raw).context("nix search returned invalid json")?;

    let mut packages = Vec::new();
    if let Some(obj) = parsed.as_object() {
        for (attr, val) in obj {
            let version = val
                .get("version")
                .and_then(|v| v.as_str())
                .map(String::from);
            let description = val
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from);
            let license = val
                .get("license")
                .and_then(|v| v.as_str())
                .map(String::from);
            let homepage = val
                .get("homepage")
                .and_then(|v| v.as_str())
                .map(String::from);
            let long_description = val
                .get("longDescription")
                .and_then(|v| v.as_str())
                .map(String::from);
            let size = val
                .get("size")
                .and_then(|v| v.as_str())
                .map(String::from);

            // Extract the short attribute name (e.g. "ripgrep" from "legacyPackages.x86_64-linux.ripgrep")
            let short_attr = attr
                .rsplit('.')
                .next()
                .unwrap_or(attr)
                .to_string();

            packages.push(NixPackage {
                name: short_attr.clone(),
                attribute: short_attr,
                version,
                description,
                license,
                homepage,
                platforms: Vec::new(),
                long_description,
                size,
            });
        }
    }

    Ok(packages)
}

pub async fn resolve_package(query: &str) -> Result<Option<PackageResolution>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let results = query_packages(trimmed).await?;
    if results.is_empty() {
        return Ok(None);
    }

    let normalized = trimmed.strip_prefix("pkgs.").unwrap_or(trimmed);
    if let Some(exact) = results.iter().find(|pkg| {
        pkg.attribute.eq_ignore_ascii_case(trimmed)
            || pkg.attribute.eq_ignore_ascii_case(normalized)
            || pkg.name.eq_ignore_ascii_case(trimmed)
            || pkg.name.eq_ignore_ascii_case(normalized)
            || pkg
                .attribute
                .eq_ignore_ascii_case(&format!("pkgs.{normalized}"))
    }) {
        let suggestions = fuzzy_suggestions(trimmed, &results, 3);
        return Ok(Some(PackageResolution {
            exact: exact.clone(),
            suggestions,
        }));
    }

    let mut ranked = fuzzy_suggestions(trimmed, &results, 5);
    if ranked.is_empty() {
        ranked = results.clone();
    }
    Ok(Some(PackageResolution {
        exact: ranked[0].clone(),
        suggestions: ranked,
    }))
}

pub async fn resolve_exact_package(query: &str) -> Result<Option<PackageResolution>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let results = query_packages(trimmed).await?;
    if results.is_empty() {
        return Ok(None);
    }

    let normalized = trimmed.strip_prefix("pkgs.").unwrap_or(trimmed);
    let exact = results.iter().find(|pkg| {
        pkg.attribute.eq_ignore_ascii_case(trimmed)
            || pkg.attribute.eq_ignore_ascii_case(normalized)
            || pkg
                .attribute
                .eq_ignore_ascii_case(&format!("pkgs.{normalized}"))
            || pkg.name.eq_ignore_ascii_case(trimmed)
            || pkg.name.eq_ignore_ascii_case(normalized)
    });

    Ok(exact.map(|exact| PackageResolution {
        exact: exact.clone(),
        suggestions: fuzzy_suggestions(trimmed, &results, 5),
    }))
}

pub async fn enrich_packages(packages: Vec<String>) -> Vec<(String, Option<NixPackage>)> {
    let mut tasks = JoinSet::new();
    for package in packages {
        tasks.spawn(async move {
            let resolved = resolve_package(&package).await.ok().flatten();
            (package, resolved.map(|r| r.exact))
        });
    }

    let mut rows = Vec::new();
    while let Some(result) = tasks.join_next().await {
        if let Ok(row) = result {
            rows.push(row);
        }
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows
}

fn fuzzy_suggestions(query: &str, packages: &[NixPackage], limit: usize) -> Vec<NixPackage> {
    let query = query.to_lowercase();
    let mut scored = packages
        .iter()
        .map(|pkg| {
            let hay_attr = pkg.attribute.to_lowercase();
            let hay_name = pkg.name.to_lowercase();
            let mut score = 0_i32;
            if hay_attr == query || hay_name == query {
                score += 100;
            }
            if hay_attr.contains(&query) || hay_name.contains(&query) {
                score += 50;
            }
            score -= (hay_attr.len() as i32 - query.len() as i32).abs();
            (score, pkg.clone())
        })
        .collect::<Vec<_>>();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().take(limit).map(|(_, pkg)| pkg).collect()
}

fn deserialize_stringish<'de, D>(deserializer: D) -> std::result::Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    Ok(match value {
        Some(Value::String(s)) if !s.is_empty() => Some(s),
        Some(Value::Array(items)) => items
            .into_iter()
            .find_map(|v| v.as_str().map(ToString::to_string)),
        _ => None,
    })
}

fn deserialize_platforms<'de, D>(deserializer: D) -> std::result::Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    Ok(match value {
        Some(Value::Array(items)) => items
            .into_iter()
            .filter_map(|v| v.as_str().map(ToString::to_string))
            .collect(),
        Some(Value::String(s)) => vec![s],
        _ => vec![],
    })
}

fn deserialize_license<'de, D>(deserializer: D) -> std::result::Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    Ok(match value {
        Some(Value::String(s)) if !s.is_empty() => Some(s),
        Some(Value::Object(map)) => map
            .get("fullName")
            .and_then(|v| v.as_str())
            .map(ToString::to_string)
            .or_else(|| {
                map.get("shortName")
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string)
            }),
        Some(Value::Array(items)) => items.into_iter().find_map(|item| match item {
            Value::String(s) if !s.is_empty() => Some(s),
            Value::Object(map) => map
                .get("fullName")
                .and_then(|v| v.as_str())
                .map(ToString::to_string)
                .or_else(|| {
                    map.get("shortName")
                        .and_then(|v| v.as_str())
                        .map(ToString::to_string)
                }),
            _ => None,
        }),
        _ => None,
    })
}

fn deserialize_sizeish<'de, D>(deserializer: D) -> std::result::Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    Ok(match value {
        Some(Value::String(s)) if !s.is_empty() => Some(s),
        Some(Value::Number(n)) => n.as_u64().map(format_bytes),
        _ => None,
    })
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}
