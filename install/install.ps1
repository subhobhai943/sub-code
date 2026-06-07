$ErrorActionPreference = 'Stop'

$Repo = "subhobhai943/sub-code"
$BinDir = Join-Path $env:USERPROFILE ".subcode\bin"
$ExePath = Join-Path $BinDir "subcode.exe"

Write-Host "Installing SUB CODE for Windows..." -ForegroundColor Cyan

# Detect Architecture
$arch = $env:PROCESSOR_ARCHITECTURE
if ($arch -eq "AMD64" -or $arch -eq "x86_64") {
    $ArchName = "x86_64"
} else {
    Write-Warning "Unsupported architecture: $arch. Defaulting to x86_64."
    $ArchName = "x86_64"
}

$BinaryName = "subcode-windows-${ArchName}.exe"
$DownloadUrl = "https://github.com/$Repo/releases/latest/download/$BinaryName"

# Create target directory
if (-not (Test-Path $BinDir)) {
    Write-Host "Creating directory $BinDir..."
    New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
}

# Download the executable
Write-Host "Downloading $BinaryName from GitHub Releases..."
try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $ExePath -UseBasicParsing
} catch {
    Write-Error "Failed to download $BinaryName. Please check your internet connection and verify the release exists."
    exit 1
}

# Update PATH via Registry (User level)
Write-Host "Updating User PATH environment variable..."
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$BinDir*") {
    if ($UserPath -and (-not $UserPath.EndsWith(";"))) {
        $UserPath += ";"
    }
    $NewPath = $UserPath + $BinDir
    [Environment]::SetEnvironmentVariable("PATH", $NewPath, "User")
    Write-Host "Added $BinDir to your User PATH." -ForegroundColor Green
} else {
    Write-Host "PATH already contains $BinDir."
}

# Add to current session PATH so we can run setup immediately
$env:PATH += ";$BinDir"

# Run setup
Write-Host "Running SUB CODE setup wizard..." -ForegroundColor Cyan
subcode --setup

Write-Host "Installation complete! Please restart your terminal or open a new window to ensure the PATH changes take effect." -ForegroundColor Green
