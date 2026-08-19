# Madi

AI-native Linux CLI written in Rust.

Madi converts natural language into **registered, validated tools** instead of giving the model direct shell access.

## Examples

```bash
madi "create a folder called projects"
madi "remove the projects directory"
madi "show me what's using my disk"
madi "show running processes"
madi "restart nginx"
madi "install curl"
madi "show network information"
```

## Setup

```bash
export OPENAI_API_KEY="YOUR_API_KEY"
cargo build --release
sudo install -m 0755 target/release/madi /usr/local/bin/madi
```

Optional:

```bash
export MADI_MODEL="gpt-5.6"
```

## Tools

### Filesystem
- create_directory
- delete_directory
- list_directory
- read_file

### System
- system_info
- disk_usage
- list_processes
- network_info

### systemd
- systemd_status
- systemd_start
- systemd_stop
- systemd_restart

### Package management
- package_install
- package_remove
- package_update

### Controlled command execution
- run_command

`run_command` is intentionally allowlisted and does not accept arbitrary shell syntax.

## Safety

Destructive operations require confirmation.

The model cannot directly execute shell commands.

`run_command` only accepts approved Linux commands and rejects shell operators such as `;`, `&&`, `||`, pipes, redirects and command substitution.

System and package operations that modify the machine require confirmation.

Filesystem access rejects `/` as a target.

## Architecture

```text
User
 |
 v
Madi CLI
 |
 v
OpenAI Responses API
 |
 v
Tool Call
 |
 v
Rust validation + safety policy
 |
 v
Confirmation when required
 |
 v
Linux Tool Executor
```

## License

MIT
