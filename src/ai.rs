use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use crate::tools::ToolCall;

const SYSTEM: &str = r#"
You are Madi, an AI Linux assistant.
Return exactly one registered function call. Never return shell commands as prose.
Use filesystem tools for files, systemd tools for services, package tools for Debian/Ubuntu packages, and run_command only for safe read-only commands.
Never invent tools.
"#;

#[derive(Deserialize)]
struct Response { output: Option<Vec<Item>> }

#[derive(Deserialize)]
#[serde(tag="type")]
enum Item {
    #[serde(rename="function_call")]
    FunctionCall { name:String, arguments:String },
    #[serde(other)] Other,
}

pub async fn understand(key:&str, model:&str, input:&str)->Result<ToolCall>{
    let tools = vec![
        f("create_directory","Create a directory.",obj(&[("path","string",true)])),
        f("delete_directory","Delete a directory.",json!({"type":"object","properties":{"path":{"type":"string"},"recursive":{"type":"boolean"}},"required":["path","recursive"],"additionalProperties":false})),
        f("list_directory","List directory entries.",obj(&[("path","string",true)])),
        f("read_file","Read a UTF-8 text file.",obj(&[("path","string",true)])),
        f("system_info","Get Linux system information.",empty()),
        f("disk_usage","Show disk usage.",obj(&[("path","string",true)])),
        f("list_processes","List processes.",empty()),
        f("network_info","Show network interfaces.",empty()),
        f("systemd_status","Check a systemd service.",obj(&[("service","string",true)])),
        f("systemd_start","Start a systemd service.",obj(&[("service","string",true)])),
        f("systemd_stop","Stop a systemd service.",obj(&[("service","string",true)])),
        f("systemd_restart","Restart a systemd service.",obj(&[("service","string",true)])),
        f("package_install","Install a Debian package.",obj(&[("package","string",true)])),
        f("package_remove","Remove a Debian package.",obj(&[("package","string",true)])),
        f("package_update","Update Debian package lists.",empty()),
        f("run_command","Run one allowlisted command without shell interpretation.",json!({"type":"object","properties":{"command":{"type":"string"},"args":{"type":"array","items":{"type":"string"}}},"required":["command","args"],"additionalProperties":false})),
    ];
    let body=json!({"model":model,"instructions":SYSTEM,"input":input,"tools":tools,"tool_choice":"auto"});
    let r=Client::new().post("https://api.openai.com/v1/responses").bearer_auth(key).json(&body).send().await.context("OpenAI request failed")?;
    let status=r.status(); let text=r.text().await?;
    if !status.is_success(){bail!("OpenAI API error ({status}): {text}")};
    let parsed:Response=serde_json::from_str(&text).context("invalid OpenAI response")?;
    for i in parsed.output.unwrap_or_default(){if let Item::FunctionCall{name,arguments}=i{return ToolCall::from_json(&name,&arguments)}}
    bail!("No supported tool call returned")
}

fn f(name:&str,desc:&str,parameters:Value)->Value{json!({"type":"function","name":name,"description":desc,"parameters":parameters})}
fn empty()->Value{json!({"type":"object","properties":{},"additionalProperties":false})}
fn obj(items:&[(&str,&str,bool)])->Value{
    let mut props=serde_json::Map::new(); let mut req=Vec::new();
    for (n,t,r) in items{props.insert((*n).into(),json!({"type":t}));if *r{req.push(*n)}}
    json!({"type":"object","properties":props,"required":req,"additionalProperties":false})
}

