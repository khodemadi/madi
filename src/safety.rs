use anyhow::{bail, Result};
use std::io::{self, Write};
use std::path::Path;
use crate::tools::ToolCall;

pub fn confirm() -> Result<bool> {
    print!("Continue? [y/N] ");
    io::stdout().flush()?;
    let mut a = String::new();
    io::stdin().read_line(&mut a)?;
    Ok(matches!(a.trim().to_lowercase().as_str(), "y" | "yes"))
}

pub fn validate(call: &ToolCall) -> Result<()> {
    match call {
        ToolCall::CreateDirectory { path }
        | ToolCall::DeleteDirectory { path, .. }
        | ToolCall::ListDirectory { path }
        | ToolCall::ReadFile { path }
        | ToolCall::DiskUsage { path } => validate_path(path),
        ToolCall::SystemInfo | ToolCall::ListProcesses | ToolCall::NetworkInfo
        | ToolCall::SystemdStatus { .. } => Ok(()),
        ToolCall::SystemdStart { service }
        | ToolCall::SystemdStop { service }
        | ToolCall::SystemdRestart { service } => validate_name(service),
        ToolCall::PackageInstall { package }
        | ToolCall::PackageRemove { package } => validate_name(package),
        ToolCall::PackageUpdate => Ok(()),
        ToolCall::RunCommand { command, args } => validate_command(command, args),
    }
}

fn validate_path(raw: &str) -> Result<()> {
    if raw.trim().is_empty() || Path::new(raw) == Path::new("/") {
        bail!("unsafe filesystem path");
    }
    Ok(())
}

fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 128 || !name.chars().all(|c| c.is_ascii_alphanumeric() || ".@_+-".contains(c)) {
        bail!("invalid service/package name");
    }
    Ok(())
}

fn validate_command(command: &str, args: &[String]) -> Result<()> {
    const ALLOWED: &[&str] = &["echo", "uname", "whoami", "id", "pwd", "ls", "cat", "grep", "ip", "df", "free", "ps"];
    if !ALLOWED.contains(&command) {
        bail!("command is not allowlisted: {command}");
    }
    for a in args {
        if a.contains(';') || a.contains("&&") || a.contains("||") || a.contains('|')
            || a.contains('>') || a.contains('<') || a.contains('`') || a.contains("$(") {
            bail!("shell syntax is not allowed");
        }
    }
    Ok(())
}

