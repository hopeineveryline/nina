use anyhow::{Context, Result};
use serde::de::Deserializer;
use serde::Deserialize;
use serde_json::Value;
use std::env;
use tokio::task::JoinSet;
use tokio::time::{timeout, Duration};

const PACKAGE_QUERY_TIMEOUT_SECS: u64 = 20;

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
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    match query_packages_with_nix_search(trimmed).await {
        Ok(packages) => Ok(packages),
        Err(primary_err) => match query_packages_with_nix_env(trimmed).await {
            Ok(packages) => Ok(packages),
            Err(fallback_err) => Err(primary_err.context(format!(
                "fallback nix-env search also failed: {fallback_err}"
            ))),
        },
    }
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

async fn query_packages_with_nix_search(query: &str) -> Result<Vec<NixPackage>> {
    let nixpkgs_ref = env::var("NINA_NIXPKGS").unwrap_or_else(|_| "nixpkgs".to_string());
    let attempts = [
        vec![
            "search".to_string(),
            nixpkgs_ref.clone(),
            query.to_string(),
            "--json".to_string(),
        ],
        vec![
            "--extra-experimental-features".to_string(),
            "nix-command flakes".to_string(),
            "search".to_string(),
            nixpkgs_ref,
            query.to_string(),
            "--json".to_string(),
        ],
    ];

    let mut last_err = None;
    for args in attempts {
        let output = run_command_with_timeout("nix", &args, "running nix search").await?;

        if output.status.success() {
            let raw = String::from_utf8_lossy(&output.stdout);
            let parsed: Value =
                serde_json::from_str(&raw).context("nix search returned invalid json")?;
            return Ok(packages_from_value(&parsed));
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        last_err = Some(search_error(output.status, stderr.trim()));
    }

    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("nix search failed for an unknown reason")))
}

async fn query_packages_with_nix_env(query: &str) -> Result<Vec<NixPackage>> {
    let args = vec![
        "-qaP".to_string(),
        "--json".to_string(),
        format!("*{query}*"),
    ];
    let output = run_command_with_timeout("nix-env", &args, "running nix-env search").await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "nix-env search exited with {}: {}",
            output.status,
            stderr.trim()
        );
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    let parsed: Value =
        serde_json::from_str(&raw).context("nix-env search returned invalid json")?;
    Ok(packages_from_value(&parsed))
}

async fn run_command_with_timeout(
    program: &str,
    args: &[String],
    activity: &str,
) -> Result<std::process::Output> {
    timeout(
        Duration::from_secs(PACKAGE_QUERY_TIMEOUT_SECS),
        tokio::process::Command::new(program)
            .kill_on_drop(true)
            .args(args.iter().map(String::as_str))
            .output(),
    )
    .await
    .with_context(|| format!("{activity} timed out after {PACKAGE_QUERY_TIMEOUT_SECS}s"))?
    .with_context(|| format!("{activity} failed — is {program} available on this machine?"))
}

fn search_error(status: std::process::ExitStatus, stderr: &str) -> anyhow::Error {
    if stderr.contains("could not find") || stderr.contains("does not provide") {
        return anyhow::anyhow!(
            "nix search couldn't find nixpkgs — try running from your flake directory, or set NINA_NIXPKGS to a flake ref"
        );
    }

    anyhow::anyhow!("nix search exited with {status}: {stderr}")
}

fn packages_from_value(parsed: &Value) -> Vec<NixPackage> {
    let mut packages = Vec::new();
    if let Some(obj) = parsed.as_object() {
        for (attr, value) in obj {
            packages.push(package_from_value(attr, value));
        }
    }
    packages
}

fn package_from_value(attr: &str, value: &Value) -> NixPackage {
    let meta = value.get("meta").unwrap_or(value);
    let short_attr = attr.rsplit('.').next().unwrap_or(attr).to_string();

    NixPackage {
        attribute: short_attr.clone(),
        name: string_field(value, meta, &["pname", "name"]).unwrap_or_else(|| short_attr.clone()),
        version: string_field(value, meta, &["version"]),
        description: string_field(value, meta, &["description"]),
        license: string_field(value, meta, &["license"]),
        homepage: string_field(value, meta, &["homepage"]),
        platforms: platforms_field(value, meta),
        long_description: string_field(value, meta, &["longDescription"]),
        size: string_field(
            value,
            meta,
            &[
                "size",
                "downloadSize",
                "download-size",
                "installedSize",
                "installed-size",
            ],
        ),
    }
}

fn string_field(value: &Value, meta: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(key).or_else(|| meta.get(key)))
        .and_then(stringish)
}

fn platforms_field(value: &Value, meta: &Value) -> Vec<String> {
    value
        .get("platforms")
        .or_else(|| meta.get("platforms"))
        .map(platforms_from_value)
        .unwrap_or_default()
}

fn stringish(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Array(items) => items.iter().find_map(stringish),
        Value::Object(map) => ["fullName", "spdxId", "shortName", "url", "value", "name"]
            .iter()
            .find_map(|key| map.get(*key).and_then(stringish)),
        Value::Bool(v) => Some(v.to_string()),
        Value::Number(v) => Some(v.to_string()),
        _ => None,
    }
}

fn platforms_from_value(value: &Value) -> Vec<String> {
    match value {
        Value::Array(items) => items.iter().filter_map(stringish).collect(),
        _ => stringish(value).into_iter().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn package_from_nix_search_shape_prefers_short_attr() {
        let value = serde_json::json!({
            "version": "14.1.0",
            "description": "fast search",
            "license": "MIT",
            "homepage": "https://github.com/BurntSushi/ripgrep"
        });

        let pkg = package_from_value("legacyPackages.x86_64-linux.ripgrep", &value);
        assert_eq!(pkg.attribute, "ripgrep");
        assert_eq!(pkg.name, "ripgrep");
        assert_eq!(pkg.version.as_deref(), Some("14.1.0"));
        assert_eq!(pkg.description.as_deref(), Some("fast search"));
    }

    #[test]
    fn package_from_nix_env_shape_reads_meta_fields() {
        let value = serde_json::json!({
            "name": "ripgrep-14.1.0",
            "meta": {
                "description": "fast search",
                "homepage": ["https://github.com/BurntSushi/ripgrep"],
                "license": { "fullName": "MIT" },
                "platforms": ["x86_64-linux", "aarch64-linux"],
                "longDescription": "longer package summary"
            },
            "version": "14.1.0"
        });

        let pkg = package_from_value("ripgrep", &value);
        assert_eq!(pkg.attribute, "ripgrep");
        assert_eq!(pkg.name, "ripgrep-14.1.0");
        assert_eq!(
            pkg.homepage.as_deref(),
            Some("https://github.com/BurntSushi/ripgrep")
        );
        assert_eq!(pkg.license.as_deref(), Some("MIT"));
        assert_eq!(pkg.platforms, vec!["x86_64-linux", "aarch64-linux"]);
        assert_eq!(
            pkg.long_description.as_deref(),
            Some("longer package summary")
        );
    }

    #[tokio::test]
    async fn live_nix_package_lookup_smoke_skips_without_nix() -> Result<()> {
        if Command::new("nix").arg("--version").output().is_err() {
            eprintln!("skipping live nix package lookup smoke test because nix is unavailable");
            return Ok(());
        }

        let results = query_packages("ripgrep").await?;
        assert!(
            !results.is_empty(),
            "expected ripgrep search to return at least one package"
        );
        assert!(
            results
                .iter()
                .any(|pkg| pkg.attribute == "ripgrep" || pkg.name.contains("ripgrep")),
            "expected ripgrep to appear in package results: {results:?}"
        );

        let exact = resolve_exact_package("ripgrep").await?;
        assert!(
            exact.is_some(),
            "expected an exact ripgrep package resolution"
        );
        assert_eq!(exact.unwrap().exact.attribute, "ripgrep");
        Ok(())
    }
}
