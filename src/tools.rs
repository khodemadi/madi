use anyhow::{bail, Result};
use serde_json::Value;
use tokio::process::Command;

#[derive(Debug)]
pub enum ToolCall {
    CreateDirectory { path: String },
    DeleteDirectory { path: String, recursive: bool },
    ListDirectory { path: String },
    ReadFile { path: String },
    SystemInfo,
    DiskUsage { path: String },
    ListProcesses,
    NetworkInfo,
    SystemdStatus { service: String },
    SystemdStart { service: String },
    SystemdStop { service: String },
    SystemdRestart { service: String },
    PackageInstall { package: String },
    PackageRemove { package: String },
    PackageUpdate,
    RunCommand { command: String, args: Vec<String> },
}

impl ToolCall {
    pub fn from_json(name: &str, s: &str) -> Result<Self> {
        let v: Value = serde_json::from_str(s)?;
        let strarg = |k: &str| -> Result<String> {
            v.get(k).and_then(Value::as_str).map(str::to_owned)
                .ok_or_else(|| anyhow::anyhow!("missing argument: {k}"))
        };
        let args = || -> Vec<String> {
            v.get("args").and_then(Value::as_array).map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect()).unwrap_or_default()
        };
        Ok(match name {
            "create_directory" => Self::CreateDirectory { path: strarg("path")? },
            "delete_directory" => Self::DeleteDirectory { path: strarg("path")?, recursive: v.get("recursive").and_then(Value::as_bool).unwrap_or(true) },
            "list_directory" => Self::ListDirectory { path: strarg("path")? },
            "read_file" => Self::ReadFile { path: strarg("path")? },
            "system_info" => Self::SystemInfo,
            "disk_usage" => Self::DiskUsage { path: strarg("path")? },
            "list_processes" => Self::ListProcesses,
            "network_info" => Self::NetworkInfo,
            "systemd_status" => Self::SystemdStatus { service: strarg("service")? },
            "systemd_start" => Self::SystemdStart { service: strarg("service")? },
            "systemd_stop" => Self::SystemdStop { service: strarg("service")? },
            "systemd_restart" => Self::SystemdRestart { service: strarg("service")? },
            "package_install" => Self::PackageInstall { package: strarg("package")? },
            "package_remove" => Self::PackageRemove { package: strarg("package")? },
            "package_update" => Self::PackageUpdate,
            "run_command" => Self::RunCommand { command: strarg("command")?, args: args() },
            _ => bail!("unsupported tool: {name}"),
        })
    }

    pub fn requires_confirmation(&self) -> bool {
        matches!(self,
            Self::DeleteDirectory { .. } | Self::SystemdStart { .. } |
            Self::SystemdStop { .. } | Self::SystemdRestart { .. } |
            Self::PackageInstall { .. } | Self::PackageRemove { .. } |
            Self::PackageUpdate)
    }

    pub fn display(&self) -> String {
        match self {
            Self::CreateDirectory{path} => format!("Create directory: {path}"),
            Self::DeleteDirectory{path,recursive} => format!("Delete directory: {path} (recursive={recursive})"),
            Self::ListDirectory{path} => format!("List directory: {path}"),
            Self::ReadFile{path} => format!("Read file: {path}"),
            Self::SystemInfo => "Read system information".into(),
            Self::DiskUsage{path} => format!("Inspect disk usage: {path}"),
            Self::ListProcesses => "List running processes".into(),
            Self::NetworkInfo => "Read network information".into(),
            Self::SystemdStatus{service} => format!("Check systemd service: {service}"),
            Self::SystemdStart{service} => format!("Start systemd service: {service}"),
            Self::SystemdStop{service} => format!("Stop systemd service: {service}"),
            Self::SystemdRestart{service} => format!("Restart systemd service: {service}"),
            Self::PackageInstall{package} => format!("Install package: {package}"),
            Self::PackageRemove{package} => format!("Remove package: {package}"),
            Self::PackageUpdate => "Update package lists".into(),
            Self::RunCommand{command,args} => format!("Run allowlisted command: {command} {}", args.join(" ")),
        }
    }
}

async fn cmd(program: &str, args: &[String]) -> Result<String> {
    let o = Command::new(program).args(args).output().await?;
    if !o.status.success() { bail!("{} failed: {}", program, String::from_utf8_lossy(&o.stderr)); }
    Ok(String::from_utf8_lossy(&o.stdout).trim().to_string())
}

pub async fn execute(c: &ToolCall) -> Result<String> {
    match c {
        ToolCall::CreateDirectory{path} => { tokio::fs::create_dir_all(path).await?; Ok(format!("Created: {path}")) }
        ToolCall::DeleteDirectory{path,recursive} => { if *recursive {tokio::fs::remove_dir_all(path).await?} else {tokio::fs::remove_dir(path).await?}; Ok(format!("Deleted: {path}")) }
        ToolCall::ListDirectory{path} => {
            let mut r=tokio::fs::read_dir(path).await?; let mut v=Vec::new();
            while let Some(e)=r.next_entry().await? {v.push(e.file_name().to_string_lossy().to_string())} v.sort(); Ok(v.join("\n"))
        }
        ToolCall::ReadFile{path} => {
            let m=tokio::fs::metadata(path).await?; if !m.is_file() {bail!("not a regular file")}; if m.len()>1024*1024 {bail!("file exceeds 1 MiB")}; Ok(tokio::fs::read_to_string(path).await?)
        }
        ToolCall::SystemInfo => cmd("uname",&["-a".into()]).await,
        ToolCall::DiskUsage{path} => cmd("df",&["-h".into(),path.clone()]).await,
        ToolCall::ListProcesses => cmd("ps",&["-eo".into(),"pid,comm,%cpu,%mem".into()]).await,
        ToolCall::NetworkInfo => cmd("ip",&["-brief".into(),"addr".into()]).await,
        ToolCall::SystemdStatus{service} => cmd("systemctl",&["status".into(),service.clone(),"--no-pager".into()]).await,
        ToolCall::SystemdStart{service} => cmd("systemctl",&["start".into(),service.clone()]).await.map(|_|format!("Started: {service}")),
        ToolCall::SystemdStop{service} => cmd("systemctl",&["stop".into(),service.clone()]).await.map(|_|format!("Stopped: {service}")),
        ToolCall::SystemdRestart{service} => cmd("systemctl",&["restart".into(),service.clone()]).await.map(|_|format!("Restarted: {service}")),
        ToolCall::PackageInstall{package} => cmd("apt-get",&["install".into(),"-y".into(),package.clone()]).await,
        ToolCall::PackageRemove{package} => cmd("apt-get",&["remove".into(),"-y".into(),package.clone()]).await,
        ToolCall::PackageUpdate => cmd("apt-get",&["update".into()]).await,
        ToolCall::RunCommand{command,args} => cmd(command,args).await,
    }
}
