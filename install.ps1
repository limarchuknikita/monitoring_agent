Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$InstallerVersion = "2026.08.06-final"

function Write-Step {
    param([Parameter(Mandatory)][string]$Message)
    Write-Host "`n==> $Message" -ForegroundColor Cyan
}

function Invoke-Native {
    param(
        [Parameter(Mandatory)][string]$FilePath,
        [string[]]$ArgumentList = @(),
        [int[]]$SuccessExitCodes = @(0)
    )

    & $FilePath @ArgumentList
    $exitCode = $LASTEXITCODE

    if ($SuccessExitCodes -notcontains $exitCode) {
        $renderedArgs = $ArgumentList -join " "
        throw "Command failed with exit code ${exitCode}: `"$FilePath`" $renderedArgs"
    }
}

function Refresh-Path {
    $machinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $cargoBin = Join-Path $HOME ".cargo\bin"

    $parts = @($machinePath, $userPath, $cargoBin) |
        Where-Object { -not [string]::IsNullOrWhiteSpace($_) }

    $env:Path = $parts -join ";"
}

function Assert-Administrator {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = [Security.Principal.WindowsPrincipal]::new($identity)

    if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        throw "Run this script from an elevated PowerShell session (Run as Administrator)."
    }
}

function Get-CommandPath {
    param([Parameter(Mandatory)][string]$Name)

    $command = Get-Command $Name -CommandType Application -ErrorAction SilentlyContinue |
        Select-Object -First 1

    if ($null -eq $command) {
        return $null
    }

    return $command.Source
}

function Ensure-Tool {
    param(
        [Parameter(Mandatory)][string]$CommandName,
        [Parameter(Mandatory)][string]$DisplayName,
        [Parameter(Mandatory)][string]$WingetId,
        [Parameter(Mandatory)][string]$ChocolateyPackage
    )

    Refresh-Path
    if (Get-CommandPath -Name $CommandName) {
        Write-Step "$DisplayName already installed"
        return
    }

    if (Get-CommandPath -Name "winget.exe") {
        Write-Step "Installing $DisplayName via winget"
        Invoke-Native -FilePath "winget.exe" -ArgumentList @(
            "install", "--id", $WingetId, "--exact",
            "--accept-package-agreements", "--accept-source-agreements",
            "--silent", "--disable-interactivity"
        )
    }
    elseif (Get-CommandPath -Name "choco.exe") {
        Write-Step "Installing $DisplayName via Chocolatey"
        Invoke-Native -FilePath "choco.exe" -ArgumentList @(
            "install", $ChocolateyPackage, "-y", "--no-progress"
        )
    }
    else {
        throw "$DisplayName is missing and neither winget nor Chocolatey is available."
    }

    Refresh-Path
    if (-not (Get-CommandPath -Name $CommandName)) {
        throw "$DisplayName was installed, but '$CommandName' is still unavailable. Open a new elevated terminal and rerun this script."
    }
}

function Get-VsWherePath {
    $candidates = @(
        (Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"),
        (Join-Path $env:ProgramFiles "Microsoft Visual Studio\Installer\vswhere.exe")
    ) | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }

    foreach ($candidate in $candidates) {
        if (Test-Path -LiteralPath $candidate -PathType Leaf) {
            return $candidate
        }
    }

    return $null
}

function Get-VisualStudioInstallation {
    param(
        [Parameter(Mandatory)][string]$VsWherePath,
        [switch]$RequireCppTools
    )

    $args = @("-latest", "-products", "*", "-property", "installationPath")
    if ($RequireCppTools) {
        $args = @(
            "-latest", "-products", "*",
            "-requires", "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
            "-property", "installationPath"
        )
    }

    $result = & $VsWherePath @args
    if ($LASTEXITCODE -ne 0) {
        throw "vswhere.exe failed with exit code $LASTEXITCODE."
    }

    return ($result | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | Select-Object -First 1)
}

function Install-Or-Modify-CppBuildTools {
    param([string]$ExistingInstallationPath)

    Write-Step "Installing Visual Studio 2022 C++ Build Tools workload"

    $bootstrapper = Join-Path $env:TEMP "vs_BuildTools.exe"
    Invoke-WebRequest -Uri "https://aka.ms/vs/17/release/vs_BuildTools.exe" -OutFile $bootstrapper -UseBasicParsing

    $arguments = @(
        "--quiet", "--wait", "--norestart", "--nocache",
        "--add", "Microsoft.VisualStudio.Workload.VCTools",
        "--includeRecommended"
    )

    if (-not [string]::IsNullOrWhiteSpace($ExistingInstallationPath)) {
        $arguments += @("--installPath", $ExistingInstallationPath)
    }

    try {
        # Visual Studio bootstrapper can return 3010 when a reboot is required.
        Invoke-Native -FilePath $bootstrapper -ArgumentList $arguments -SuccessExitCodes @(0, 3010)
    }
    finally {
        Remove-Item -LiteralPath $bootstrapper -Force -ErrorAction SilentlyContinue
    }
}

function Ensure-MsvcEnvironment {
    Write-Step "Locating Microsoft C++ Build Tools"

    $vswhere = Get-VsWherePath
    $existingInstallation = $null

    if ($vswhere) {
        $existingInstallation = Get-VisualStudioInstallation -VsWherePath $vswhere
        $cppInstallation = Get-VisualStudioInstallation -VsWherePath $vswhere -RequireCppTools

        if ($cppInstallation) {
            $vsDevCmd = Join-Path $cppInstallation "Common7\Tools\VsDevCmd.bat"
            if (Test-Path -LiteralPath $vsDevCmd -PathType Leaf) {
                return $vsDevCmd
            }
        }
    }

    Install-Or-Modify-CppBuildTools -ExistingInstallationPath $existingInstallation

    $vswhere = Get-VsWherePath
    if (-not $vswhere) {
        throw "Visual Studio installer completed, but vswhere.exe was not found."
    }

    $cppInstallation = Get-VisualStudioInstallation -VsWherePath $vswhere -RequireCppTools
    if (-not $cppInstallation) {
        throw "The Visual C++ workload is still unavailable. Open Visual Studio Installer, select Build Tools 2022, and install 'Desktop development with C++'."
    }

    $vsDevCmd = Join-Path $cppInstallation "Common7\Tools\VsDevCmd.bat"
    if (-not (Test-Path -LiteralPath $vsDevCmd -PathType Leaf)) {
        throw "VsDevCmd.bat was not found at '$vsDevCmd'."
    }

    return $vsDevCmd
}

function Invoke-CargoBuildWithMsvc {
    param(
        [Parameter(Mandatory)][string]$VsDevCmdPath,
        [Parameter(Mandatory)][string]$CargoPath,
        [Parameter(Mandatory)][string]$WorkingDirectory
    )

    Write-Step "Building Rust monitoring service (release)"

    $batchFile = Join-Path $env:TEMP ("monitoring-agent-build-{0}.cmd" -f [Guid]::NewGuid().ToString("N"))
    $batchContent = @"
@echo off
call "$VsDevCmdPath" -arch=amd64 -host_arch=amd64 >nul
if errorlevel 1 exit /b %errorlevel%
where link.exe
if errorlevel 1 exit /b 9009
cd /d "$WorkingDirectory"
"$CargoPath" build --release --target x86_64-pc-windows-msvc
exit /b %errorlevel%
"@

    Set-Content -LiteralPath $batchFile -Value $batchContent -Encoding ASCII

    try {
        Invoke-Native -FilePath $env:ComSpec -ArgumentList @("/d", "/c", $batchFile)
    }
    finally {
        Remove-Item -LiteralPath $batchFile -Force -ErrorAction SilentlyContinue
    }
}

Write-Host "Monitoring Agent installer $InstallerVersion" -ForegroundColor Green

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Push-Location $scriptDir

$oldGoOs = $env:GOOS
$oldGoArch = $env:GOARCH

try {
    Write-Step "Validating environment"
    Assert-Administrator

    Ensure-Tool -CommandName "cargo.exe" -DisplayName "Rust toolchain" -WingetId "Rustlang.Rustup" -ChocolateyPackage "rustup.install"
    Ensure-Tool -CommandName "go.exe" -DisplayName "Go toolchain" -WingetId "GoLang.Go" -ChocolateyPackage "golang"

    Refresh-Path

    $cargo = Get-CommandPath -Name "cargo.exe"
    $go = Get-CommandPath -Name "go.exe"
    if (-not $cargo) { throw "cargo.exe was not found after toolchain setup." }
    if (-not $go) { throw "go.exe was not found after toolchain setup." }

    $cargoToml = Join-Path $scriptDir "Cargo.toml"
    $goMain = Join-Path $scriptDir "activities_project\main.go"
    if (-not (Test-Path -LiteralPath $cargoToml -PathType Leaf)) {
        throw "Cargo.toml was not found in '$scriptDir'."
    }
    if (-not (Test-Path -LiteralPath $goMain -PathType Leaf)) {
        throw "Go source was not found at '$goMain'."
    }

    $binDir = Join-Path $scriptDir "bin"
    New-Item -ItemType Directory -Path $binDir -Force | Out-Null

    $vsDevCmd = Ensure-MsvcEnvironment
    Write-Host "Using MSVC environment: $vsDevCmd" -ForegroundColor DarkGray

    Invoke-CargoBuildWithMsvc -VsDevCmdPath $vsDevCmd -CargoPath $cargo -WorkingDirectory $scriptDir

    Write-Step "Building Go child binary for Windows"
    $env:GOOS = "windows"
    $env:GOARCH = "amd64"
    $goOutput = Join-Path $binDir "child_binary.exe"
    Push-Location (Join-Path $scriptDir "activities_project")
    try {
        Invoke-Native -FilePath $go -ArgumentList @("build", "-o", $goOutput, ".\main.go")
    }
    finally {
        Pop-Location
    }

    Write-Step "Copying Rust service binary"
    $rustOutput = Join-Path $scriptDir "target\x86_64-pc-windows-msvc\release\monitoring_agent.exe"
    $serviceOutput = Join-Path $binDir "monitoring_agent.exe"
    $settingsSource = Join-Path $scriptDir "settings.toml"
    $settingsOutput = Join-Path $binDir "settings.toml"
    if (-not (Test-Path -LiteralPath $rustOutput -PathType Leaf)) {
        throw "Rust build reported success, but '$rustOutput' was not created. Check the package/binary name in Cargo.toml."
    }
    Copy-Item -LiteralPath $rustOutput -Destination $serviceOutput -Force
    if (-not (Test-Path -LiteralPath $settingsSource -PathType Leaf)) {
        throw "settings.toml was not found at '$settingsSource'."
    }
    Copy-Item -LiteralPath $settingsSource -Destination $settingsOutput -Force

    Write-Step "Installing Windows service FlamingoAgent"
    Invoke-Native -FilePath $serviceOutput -ArgumentList @("--install")

    Write-Step "Done"
    Write-Host "Service installation completed. Start it with: sc.exe start FlamingoAgent" -ForegroundColor Green
}
finally {
    $env:GOOS = $oldGoOs
    $env:GOARCH = $oldGoArch
    Pop-Location
}
