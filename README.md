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

...

---

## Building

...

---

## Running

...

---