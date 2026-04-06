use anyhow::{anyhow, Result};

use crate::config::{expand_tilde, MachineConfig, NinaConfig};

#[derive(Debug, Clone)]
pub enum MachineKind {
    Local,
    Remote {
        host: String,
        port: u16,
        user: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct Machine {
    pub name: String,
    pub config_dir: String,
    pub ssh_key: Option<String>,
    pub kind: MachineKind,
}

impl Machine {
    pub fn from_config(raw: &MachineConfig) -> Self {
        let kind = if raw.local {
            MachineKind::Local
        } else {
            MachineKind::Remote {
                host: raw.host.clone().unwrap_or_else(|| raw.name.clone()),
                port: raw.port,
                user: raw.user.clone(),
            }
        };

        Self {
            name: raw.name.clone(),
            config_dir: raw.config.clone(),
            ssh_key: raw
                .ssh_key
                .as_deref()
                .and_then(|p| expand_tilde(p).ok())
                .map(|p| p.display().to_string()),
            kind,
        }
    }

    pub fn is_local(&self) -> bool {
        matches!(self.kind, MachineKind::Local)
    }

    pub fn endpoint_label(&self) -> String {
        match &self.kind {
            MachineKind::Local => "local".to_string(),
            MachineKind::Remote { host, port, user } => {
                let addr = if *port == 22 {
                    host.clone()
                } else {
                    format!("{}:{}", host, port)
                };
                if let Some(user) = user {
                    format!("ssh: {}@{}", user, addr)
                } else {
                    format!("ssh: {}", addr)
                }
            }
        }
    }
}

pub fn resolve_machine(config: &NinaConfig, requested: Option<&str>) -> Result<Machine> {
    let selected = match requested {
        Some(name) => config.machines.iter().find(|m| m.name == name),
        None => config
            .machines
            .iter()
            .find(|m| m.default)
            .or_else(|| config.machines.first()),
    };

    selected
        .map(Machine::from_config)
        .ok_or_else(|| anyhow!("i couldn't find that machine in ~/.nina.conf"))
}

#[cfg(test)]
mod tests {
    use super::{resolve_machine, MachineKind};
    use crate::config::{MachineConfig, NinaConfig};

    fn config_with_machines() -> NinaConfig {
        NinaConfig {
            editor: "hx".to_string(),
            generations: 5,
            confirm: true,
            color: true,
            teach: true,
            animate: true,
            machines: vec![
                MachineConfig {
                    name: "desktop".to_string(),
                    config: "/etc/nixos".to_string(),
                    local: true,
                    default: true,
                    host: None,
                    user: None,
                    ssh_key: None,
                    port: 22,
                },
                MachineConfig {
                    name: "server".to_string(),
                    config: "/srv/nixos".to_string(),
                    local: false,
                    default: false,
                    host: Some("server.local".to_string()),
                    user: Some("admin".to_string()),
                    ssh_key: Some("~/.ssh/id_ed25519".to_string()),
                    port: 22,
                },
            ],
        }
    }

    #[test]
    fn resolve_machine_uses_default_when_unspecified() {
        let machine =
            resolve_machine(&config_with_machines(), None).expect("resolve default machine");
        assert_eq!(machine.name, "desktop");
        assert!(matches!(machine.kind, MachineKind::Local));
    }

    #[test]
    fn resolve_machine_uses_named_remote_machine() {
        let machine = resolve_machine(&config_with_machines(), Some("server"))
            .expect("resolve named machine");
        assert_eq!(machine.name, "server");
        assert!(matches!(machine.kind, MachineKind::Remote { .. }));
        assert_eq!(
            machine.ssh_key.as_deref(),
            dirs::home_dir()
                .map(|p| p.join(".ssh/id_ed25519").display().to_string())
                .as_deref()
        );
    }
}
