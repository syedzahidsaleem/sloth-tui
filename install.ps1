# Sloth installer script for Windows
# Usage:
#   irm https://raw.githubusercontent.com/syedzahidsaleem/sloth-tui/main/install.ps1 | iex

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$Repo = "syedzahidsaleem/sloth-tui"
$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\sloth-tui"

Write-Host "Installing Sloth..." -ForegroundColor Cyan

$Arch = if ([IntPtr]::Size -eq 8) { "x86_64" } else { "x86" }
if ($Arch -ne "x86_64") {
    Write-Error "Unsupported architecture: $Arch on Windows. Sloth requires 64-bit Windows."
    exit 1
}

$ArchiveName = "sloth-tui-windows-x86_64.zip"
$DownloadUrl = "https://github.com/$Repo/releases/latest/download/$ArchiveName"

$TmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $TmpDir -Force | Out-Null

try {
    $ZipPath = Join-Path $TmpDir $ArchiveName
    Write-Host "Downloading $ArchiveName from $DownloadUrl..." -ForegroundColor Gray
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath -UseBasicParsing

    Expand-Archive -Path $ZipPath -DestinationPath $TmpDir -Force

    $BinSrc = Join-Path $TmpDir "sloth-tui.exe"
    if (-not (Test-Path $BinSrc)) {
        Write-Error "Binary 'sloth-tui.exe' not found in downloaded archive!"
        exit 1
    }

    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }

    $Dest = Join-Path $InstallDir "sloth-tui.exe"
    Copy-Item -Path $BinSrc -Destination $Dest -Force

    # Ensure $InstallDir is in User PATH
    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($UserPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
        Write-Host "Added $InstallDir to User PATH." -ForegroundColor Green
        Write-Host "Please restart your terminal session to pick up the updated PATH." -ForegroundColor Yellow
    }

    Write-Host "Sloth successfully installed to: $Dest" -ForegroundColor Green
}
finally {
    Remove-Item -Path $TmpDir -Recurse -Force -ErrorAction SilentlyContinue
}

# Check for mpv prerequisite
if (-not (Get-Command mpv -ErrorAction SilentlyContinue)) {
    Write-Host ""
    Write-Host "Notice: 'mpv' was not detected in your PATH." -ForegroundColor Yellow
    Write-Host "Sloth uses mpv for video streaming playback." -ForegroundColor Yellow
    Write-Host "You can install mpv easily on Windows via:" -ForegroundColor Yellow
    Write-Host "  winget install shinchiro.mpv" -ForegroundColor Cyan
}

Write-Host ""
Write-Host "Run 'sloth-tui' to start streaming!" -ForegroundColor Cyan
