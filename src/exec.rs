use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use russh::client;
use russh::keys::key;
use russh::keys::load_secret_key;
use russh::{ChannelMsg, Disconnect};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

use crate::machine::{Machine, MachineKind};

#[derive(Debug, Clone)]
pub struct ExecResult {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

impl ExecResult {
    pub fn success(&self) -> bool {
        self.status == 0
    }
}

pub async fn run(machine: &Machine, command_line: &str) -> Result<ExecResult> {
    run_with_stream(machine, command_line, |_, _| {}).await
}

pub async fn run_attached(machine: &Machine, command_line: &str) -> Result<i32> {
    match &machine.kind {
        MachineKind::Local => run_local_attached(command_line).await,
        MachineKind::Remote { host, port, user } => {
            run_remote_attached(
                host,
                *port,
                user.as_deref(),
                machine.ssh_key.as_deref(),
                command_line,
            )
            .await
        }
    }
}

pub async fn run_with_stream<F>(
    machine: &Machine,
    command_line: &str,
    mut on_line: F,
) -> Result<ExecResult>
where
    F: FnMut(bool, &str),
{
    match &machine.kind {
        MachineKind::Local => run_local(command_line, &mut on_line).await,
        MachineKind::Remote { host, port, user } => {
            run_remote(
                host,
                *port,
                user.as_deref(),
                machine.ssh_key.as_deref(),
                command_line,
                &mut on_line,
            )
            .await
        }
    }
}

pub async fn run_local<F>(command_line: &str, mut on_line: F) -> Result<ExecResult>
where
    F: FnMut(bool, &str),
{
    let mut child = Command::new("sh")
        .arg("-lc")
        .arg(command_line)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to run local command: {command_line}"))?;

    let stdout = child.stdout.take().context("missing stdout pipe")?;
    let stderr = child.stderr.take().context("missing stderr pipe")?;
    let (tx, mut rx) = mpsc::unbounded_channel::<(bool, String)>();

    let stdout_tx = tx.clone();
    tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = stdout_tx.send((false, line));
        }
    });

    let stderr_tx = tx.clone();
    tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = stderr_tx.send((true, line));
        }
    });
    drop(tx);

    let mut stdout_buf = String::new();
    let mut stderr_buf = String::new();
    while let Some((is_stderr, line)) = rx.recv().await {
        on_line(is_stderr, &line);
        if is_stderr {
            stderr_buf.push_str(&line);
            stderr_buf.push('\n');
        } else {
            stdout_buf.push_str(&line);
            stdout_buf.push('\n');
        }
    }
    let status = child.wait().await?.code().unwrap_or(1);

    Ok(ExecResult {
        status,
        stdout: stdout_buf,
        stderr: stderr_buf,
    })
}

pub async fn run_local_attached(command_line: &str) -> Result<i32> {
    let status = Command::new("sh")
        .arg("-lc")
        .arg(command_line)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .await
        .with_context(|| format!("failed to run local command: {command_line}"))?;
    Ok(status.code().unwrap_or(1))
}

pub async fn run_remote<F>(
    host: &str,
    port: u16,
    user: Option<&str>,
    ssh_key: Option<&str>,
    command_line: &str,
    mut on_line: F,
) -> Result<ExecResult>
where
    F: FnMut(bool, &str),
{
    let user = user.unwrap_or("admin");
    if ssh_key.is_none() {
        return run_remote_via_ssh(host, port, user, command_line, on_line).await;
    }
    let key_path = ssh_key.ok_or_else(|| {
        anyhow!(
            "remote execution needs ssh_key set in ~/.nina.conf for now, so i know which key to use for {user}@{host}"
        )
    })?;

    let key_pair = load_secret_key(Path::new(key_path), None)
        .with_context(|| format!("couldn't load ssh key at {key_path}"))?;

    let config = Arc::new(client::Config {
        inactivity_timeout: Some(Duration::from_secs(15)),
        ..<_>::default()
    });

    let mut session = client::connect(config, (host, port), NinaSshClient {})
        .await
        .with_context(|| format!("couldn't connect to {user}@{host} over ssh"))?;

    let authenticated = session
        .authenticate_publickey(user, Arc::new(key_pair))
        .await
        .with_context(|| format!("ssh authentication failed for {user}@{host}"))?;

    if !authenticated {
        bail!("ssh authentication was rejected for {user}@{host}");
    }

    let mut channel = session
        .channel_open_session()
        .await
        .with_context(|| format!("couldn't open ssh session for {user}@{host}"))?;
    channel
        .exec(true, command_line)
        .await
        .with_context(|| format!("remote command failed to start on {user}@{host}"))?;

    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut status = None;
    let mut stdout_partial = String::new();
    let mut stderr_partial = String::new();

    while let Some(msg) = channel.wait().await {
        match msg {
            ChannelMsg::Data { data } => {
                let text = String::from_utf8_lossy(&data);
                push_stream_chunk(&mut stdout, &mut stdout_partial, &text, false, &mut on_line);
            }
            ChannelMsg::ExtendedData { data, .. } => {
                let text = String::from_utf8_lossy(&data);
                push_stream_chunk(&mut stderr, &mut stderr_partial, &text, true, &mut on_line);
            }
            ChannelMsg::ExitStatus { exit_status } => status = Some(exit_status as i32),
            _ => {}
        }
    }

    flush_partial(&mut stdout, &mut stdout_partial, false, &mut on_line);
    flush_partial(&mut stderr, &mut stderr_partial, true, &mut on_line);

    session
        .disconnect(Disconnect::ByApplication, "nina done", "English")
        .await
        .ok();

    Ok(ExecResult {
        status: status.unwrap_or(1),
        stdout,
        stderr,
    })
}

async fn run_remote_via_ssh<F>(
    host: &str,
    port: u16,
    user: &str,
    command_line: &str,
    on_line: F,
) -> Result<ExecResult>
where
    F: FnMut(bool, &str),
{
    let remote = escaped(&format!("sh -lc {}", escaped(command_line)));
    let port_flag = if port == 22 {
        String::new()
    } else {
        format!(" -p {port}")
    };
    let ssh_cmd = format!("ssh -o BatchMode=yes -tt{port_flag} {user}@{host} {remote}");
    run_local(&ssh_cmd, on_line)
        .await
        .with_context(|| format!("ssh-agent fallback failed for {user}@{host}"))
}

async fn run_remote_attached(
    host: &str,
    port: u16,
    user: Option<&str>,
    ssh_key: Option<&str>,
    command_line: &str,
) -> Result<i32> {
    let user = user.unwrap_or("admin");
    let remote = escaped(&format!("sh -lc {}", escaped(command_line)));
    let key_flag = ssh_key
        .map(|key| format!(" -i {}", escaped(key)))
        .unwrap_or_default();
    let port_flag = if port == 22 {
        String::new()
    } else {
        format!(" -p {port}")
    };
    let ssh_cmd = format!(
        "ssh -o BatchMode=yes -tt{key_flag}{port_flag} {user}@{host} {remote}",
        key_flag = key_flag,
        port_flag = port_flag,
        user = user,
        host = host,
        remote = remote
    );
    run_local_attached(&ssh_cmd)
        .await
        .with_context(|| format!("ssh-agent fallback failed for {user}@{host}"))
}

fn escaped(command_line: &str) -> String {
    format!("'{}'", command_line.replace('\'', "'\\''"))
}

fn push_stream_chunk<F>(
    aggregate: &mut String,
    partial: &mut String,
    chunk: &str,
    is_stderr: bool,
    on_line: &mut F,
) where
    F: FnMut(bool, &str),
{
    aggregate.push_str(chunk);
    partial.push_str(chunk);
    while let Some(idx) = partial.find('\n') {
        let line = partial[..idx].to_string();
        on_line(is_stderr, &line);
        partial.drain(..=idx);
    }
}

fn flush_partial<F>(aggregate: &mut String, partial: &mut String, is_stderr: bool, on_line: &mut F)
where
    F: FnMut(bool, &str),
{
    if !partial.is_empty() {
        on_line(is_stderr, partial);
        aggregate.push('\n');
        partial.clear();
    }
}

struct NinaSshClient;

#[async_trait]
impl client::Handler for NinaSshClient {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}
