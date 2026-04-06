use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NinaConfig {
    #[serde(default = "default_editor")]
    pub editor: String,
    #[serde(default = "default_generations")]
    pub generations: u32,
    #[serde(default = "default_true")]
    pub confirm: bool,
    #[serde(default = "default_true")]
    pub color: bool,
    #[serde(default = "default_true")]
    pub teach: bool,
    #[serde(default = "default_true")]
    pub animate: bool,
    #[serde(default)]
    pub machines: Vec<MachineConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineConfig {
    pub name: String,
    pub config: String,
    #[serde(default)]
    pub local: bool,
    #[serde(default)]
    pub default: bool,
    pub host: Option<String>,
    pub user: Option<String>,
    pub ssh_key: Option<String>,
    #[serde(default = "default_ssh_port")]
    pub port: u16,
}

fn default_ssh_port() -> u16 {
    22
}

fn default_editor() -> String {
    env::var("EDITOR").unwrap_or_else(|_| "nano".to_string())
}

fn default_generations() -> u32 {
    5
}

fn default_true() -> bool {
    true
}

impl Default for NinaConfig {
    fn default() -> Self {
        let hostname = env::var("HOSTNAME").unwrap_or_else(|_| "local".to_string());
        Self {
            editor: default_editor(),
            generations: default_generations(),
            confirm: default_true(),
            color: default_true(),
            teach: default_true(),
            animate: default_true(),
            machines: vec![MachineConfig {
                name: hostname,
                config: "/etc/nixos".to_string(),
                local: true,
                default: true,
                host: None,
                user: None,
                ssh_key: None,
                port: default_ssh_port(),
            }],
        }
    }
}

impl NinaConfig {
    pub fn load_or_bootstrap() -> Result<Self> {
        let path = config_path()?;
        if path.exists() {
            let raw = fs::read_to_string(&path)
                .with_context(|| format!("couldn't read config at {}", path.display()))?;
            let parsed: NinaConfig = toml::from_str(&raw)
                .with_context(|| format!("couldn't parse config at {}", path.display()))?;
            return Ok(parsed.with_fallbacks());
        }

        let cfg = if io::stdin().is_terminal() && io::stdout().is_terminal() {
            bootstrap_wizard()?
        } else {
            NinaConfig::default()
        };
        cfg.write(&path)?;
        println!("(˶ᵔ ᵕ ᵔ˶) hi! i'm nina~");
        println!("           i wrote your config to {} ♡", path.display());
        Ok(cfg)
    }

    fn with_fallbacks(mut self) -> Self {
        if self.machines.is_empty() {
            self.machines = NinaConfig::default().machines;
        }
        self
    }

    pub fn write(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("couldn't create {}", parent.display()))?;
        }
        let raw = toml::to_string_pretty(self).context("couldn't serialize nina config")?;
        fs::write(path, raw).with_context(|| format!("couldn't write {}", path.display()))?;
        Ok(())
    }
}

pub fn config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("couldn't find home directory")?;
    Ok(home.join(".nina.conf"))
}

fn bootstrap_wizard() -> Result<NinaConfig> {
    let hostname = env::var("HOSTNAME").unwrap_or_else(|_| "local".to_string());
    println!("(˶ᵔ ᵕ ᵔ˶) hi! let's set nina up together.");

    let editor = prompt("editor command", &default_editor())?;
    let machine_name = prompt("default machine name", &hostname)?;
    let config_dir = prompt("configuration.nix directory", "/etc/nixos")?;
    let local = prompt_bool("is this machine local?", true)?;

    let (host, port, user, ssh_key) = if local {
        (None, default_ssh_port(), None, None)
    } else {
        let host = Some(prompt("remote host", "nixos.local")?);
        let port_str = prompt("ssh port", "22")?;
        let port = port_str.parse::<u16>().unwrap_or(22);
        let user = Some(prompt(
            "remote user",
            &env::var("USER").unwrap_or_else(|_| "admin".to_string()),
        )?);
        let ssh_key = Some(prompt("ssh private key path", "~/.ssh/id_ed25519")?);
        (host, port, user, ssh_key)
    };

    Ok(NinaConfig {
        editor,
        generations: default_generations(),
        confirm: default_true(),
            color: default_true(),
            teach: default_true(),
            animate: default_true(),
            machines: vec![MachineConfig {
            name: machine_name,
            config: config_dir,
            local,
            default: true,
            host,
            user,
            ssh_key,
            port,
        }],
    })
}

fn prompt(label: &str, default: &str) -> Result<String> {
    print!("  {} [{}]: ", label, default);
    io::stdout().flush().context("failed to flush prompt")?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("failed to read config input")?;
    let trimmed = input.trim();
    Ok(if trimmed.is_empty() {
        default.to_string()
    } else {
        trimmed.to_string()
    })
}

fn prompt_bool(label: &str, default: bool) -> Result<bool> {
    let default_text = if default { "Y/n" } else { "y/N" };
    let raw = prompt(label, default_text)?;
    Ok(match raw.as_str() {
        "Y/n" | "y" | "Y" | "yes" | "YES" => true,
        "y/N" | "n" | "N" | "no" | "NO" => false,
        _ => default,
    })
}

pub fn expand_tilde(path: &str) -> Result<PathBuf> {
    if let Some(stripped) = path.strip_prefix("~/") {
        let home = dirs::home_dir().context("couldn't find home directory")?;
        return Ok(home.join(stripped));
    }
    Ok(PathBuf::from(path))
}
