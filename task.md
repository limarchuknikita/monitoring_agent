📌 Windows Rust Service with Child Process
Task Requirements:
Rust Service (Background Agent)

## Runs as a background service.
* Collects two system metrics every 5 seconds: Current UTC time and Process memory usage (RSS) or any equivalent metric available via Windows API
* Launches a child binary with administrator privileges, passing the metrics as string arguments to it
* Sets ACL permissions on the child binary’s log file so that only Administrators and SYSTEM can read or modify it.
* The code should be cross-platform, using conditional compilation (cfg) and cross-platform libraries wherever possible to simplify future adaptation for other OS.
* Handles errors gracefully, the service must not crash if metric collection or process spawning fails.


## Child Binary (C/C++ or Go)
* Accepts one metrics as a string arguments from the command line
* Every 5 seconds, logs the metrics both to stdout and a log file.
* The log file must respect the ACL restrictions applied by the Rust service.
* The code should also be written in a cross-platform manner, using conditional compilation or standard cross-platform APIs where applicable.


## Install Mode:
* The Rust service supports a --install command-line flag.
* When launched with this flag, it installs itself as a Windows Service named FlamingoAgent, configured for automatic startup.
* When launched without the --install flag, it runs as a background daemon — collecting metrics, starting the child binary, and setting ACL permissions.

## Build & Installation Script:
* Provide a build script (install.bat or install.ps1) that:
* Builds the Rust service (cargo build --release).
* Builds the child binary preferably in C++ (or Go).
* Runs the Rust service in --install mode to register it as a Windows Service.


## Evaluation Criteria:
* System metrics are collected and logged every 5 seconds.
* Child process is launched with the provided argument and elevated privileges.


ACL permissions are correctly applied to the child process log file (access restricted to Administrators and SYSTEM).
Code is as cross-platform as possible (both Rust and the child binary).
Rust service installs correctly as a Windows Service via --install.
Proper error handling and stable daemon behavior.
Functional build and installation script allowing full deployment on a clean Windows environment.


✅Deliverables:
Submit a repository or archive containing:
agent-rust/ — Rust service with Install Mode, ACL setup, and cross-platform architecture.
logger-child/ — Child binary (C/C++ or Go) with cross-platform implementation.
install.bat or install.ps1 — Build and installation script.
README.md — Brief instructions on building, installing, and testing the solution.
