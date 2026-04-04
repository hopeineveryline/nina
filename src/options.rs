use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct NixOption {
    #[serde(default, alias = "option_name")]
    pub name: String,
    #[serde(default, alias = "option_description")]
    pub description: Option<String>,
    #[serde(default, alias = "option_type")]
    pub option_type: Option<String>,
    #[serde(default, alias = "option_default")]
    pub default_value: Option<String>,
    #[serde(default, alias = "option_example")]
    pub example: Option<String>,
    #[serde(default, alias = "option_source")]
    pub source: Option<String>,
}

pub async fn query_options(query: &str) -> Result<Vec<NixOption>> {
    // The search.nixos.org Elasticsearch backend now requires authentication.
    // Options search is unavailable — suggest the web UI instead.
    anyhow::bail!(
        "options search needs search.nixos.org auth — try https://search.nixos.org/options?q={query} for now, or run `nixos-option` for local checks"
    );
}
