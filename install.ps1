Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Write-Step {
	param([string]$Message)
	Write-Host "`n==> $Message" -ForegroundColor Cyan
}

function Test-CommandExists {
	param([string]$Name)
	if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
		throw "Required command '$Name' was not found in PATH."
	}
}

function Install-WithWinget {
	param(
		[string]$Id,
		[string]$DisplayName
	)
	Write-Step "Installing $DisplayName via winget"
	winget install --id $Id --exact --accept-package-agreements --accept-source-agreements --silent
}

function Install-WithChocolatey {
	param(
		[string]$Package,
		[string]$DisplayName
	)
	Write-Step "Installing $DisplayName via choco"
	choco install $Package -y
}

function Ensure-CommandInstalled {
	param(
		[string]$CommandName,
		[string]$DisplayName,
		[string]$WingetId,
		[string]$ChocolateyPackage
	)

	if (Get-Command $CommandName -ErrorAction SilentlyContinue) {
		Write-Step "$DisplayName already installed"
		return
	}

	if (Get-Command winget -ErrorAction SilentlyContinue) {
		Install-WithWinget -Id $WingetId -DisplayName $DisplayName
	}
	elseif (Get-Command choco -ErrorAction SilentlyContinue) {
		Install-WithChocolatey -Package $ChocolateyPackage -DisplayName $DisplayName
	}
	else {
		throw "$DisplayName is missing and no package manager (winget/choco) is available. Install it manually, then rerun this script."
	}

	$env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")

	if (-not (Get-Command $CommandName -ErrorAction SilentlyContinue)) {
		throw "$DisplayName installation command completed but '$CommandName' is still unavailable in PATH. Open a new terminal and rerun the script."
	}
}

function Assert-Admin {
	$currentIdentity = [Security.Principal.WindowsIdentity]::GetCurrent()
	$principal = New-Object Security.Principal.WindowsPrincipal($currentIdentity)
	$isAdmin = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
	if (-not $isAdmin) {
		throw "Run this script in an elevated PowerShell session (Run as Administrator)."
	}
}

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Push-Location $scriptDir

try {
	Write-Step "Validating environment"
	Assert-Admin
	Ensure-CommandInstalled -CommandName "cargo" -DisplayName "Rust toolchain" -WingetId "Rustlang.Rustup" -ChocolateyPackage "rustup.install"
	Ensure-CommandInstalled -CommandName "go" -DisplayName "Go toolchain" -WingetId "GoLang.Go" -ChocolateyPackage "golang"

	if (-not (Test-Path "./bin")) {
		New-Item -ItemType Directory -Path "./bin" | Out-Null
	}

	Write-Step "Building Rust monitoring service (release)"
	cargo build --release

	Write-Step "Building Go child binary for Windows"
	$env:GOOS = "windows"
	$env:GOARCH = "amd64"
	go build -o "./bin/child_binary.exe" "./activities_project/main.go"

	Write-Step "Copying Rust service binary"
	Copy-Item "./target/release/monitoring_agent.exe" "./bin/monitoring_agent.exe" -Force

	Write-Step "Installing Windows service monitoring_agent"
	& "./bin/monitoring_agent.exe" --install

	Write-Step "Done"
	Write-Host "Service install command executed. To start service manually run: sc start monitoring_agent" -ForegroundColor Green
}
finally {
	Pop-Location
}
