# FlamingoAgent

FlamingoAgent is a cross-platform monitoring agent written in Rust, designed to run as a background service on Windows. It collects system metrics and launches a child binary with elevated privileges to log these metrics.

---

## Features

- Collects system metrics every 5 seconds, including current UTC time and process memory usage (RSS).
- Launches a child binary with administrator privileges, passing the collected metrics as string arguments.
- Sets ACL permissions on the child binary’s log file to restrict access to Administrators and SYSTEM.
- Cross-platform architecture using conditional compilation and cross-platform libraries.

---

## Installation

Windows service installation is automated by `install.ps1`.

1. Open PowerShell as Administrator.
2. From the repository root, run:

```powershell
./install.ps1
```

The script will:
- Ensure Rust and Go toolchains are available.
- Build the Rust service in release mode.
- Build the Go child binary.
- Copy binaries and `settings.toml` into `bin`.
- Install the Windows service as `FlamingoAgent`.

Start the installed service:

```powershell
sc.exe start FlamingoAgent
```

Stop it:

```powershell
sc.exe stop FlamingoAgent
```

---

## Building

Manual build from repository root:

```powershell
cargo build --release --target x86_64-pc-windows-msvc
go build -o .\bin\child_binary.exe .\activities_project\main.go
```

If you want a local non-service run, copy the service binary to `bin` and keep `settings.toml` next to it.

---

## Running

### Install mode

```powershell
.\bin\monitoring_agent.exe --install
```

### Foreground/daemon mode (without installing service)

```powershell
.\bin\monitoring_agent.exe
```

In this mode the agent:
- Collects UTC time and RSS every 5 seconds.
- Launches the child binary with metrics.
- Applies ACL restrictions to the log file (`Administrators` and `SYSTEM`).

### Child binary standalone behavior

The child accepts a metrics string argument and logs it every 5 seconds by default:

```powershell
.\bin\child_binary.exe "2026-01-01 00:00:00 | 12.34 MB"
```

For one-shot logging (used by the Rust service internally):

```powershell
.\bin\child_binary.exe "2026-01-01 00:00:00 | 12.34 MB" --once
```

### Quick verification

1. Start service: `sc.exe start FlamingoAgent`
2. Check service state: `sc.exe query FlamingoAgent`
3. Inspect logs: `Get-Content .\bin\logs\agent.log -Tail 20`
4. Verify ACL: `icacls .\bin\logs\agent.log`

---