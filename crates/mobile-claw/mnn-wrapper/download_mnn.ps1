param(
    [string]$OutputDir = ".\lib",
    [string]$Version = "2.8.0"
)

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "MNN Pre-built Library Downloader" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

$baseDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$outputPath = Join-Path $baseDir $OutputDir

if (-not (Test-Path $outputPath)) {
    New-Item -ItemType Directory -Path $outputPath -Force | Out-Null
}

$downloadUrl = "https://github.com/alibaba/MNN/releases/download/$Version/MNN-Windows-x64.zip"
$zipFile = Join-Path $outputPath "MNN-Windows-x64.zip"

Write-Host "Downloading MNN $Version for Windows x64..." -ForegroundColor Yellow
Write-Host "URL: $downloadUrl" -ForegroundColor Gray

try {
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri $downloadUrl -OutFile $zipFile -UseBasicParsing
    Write-Host "Download completed!" -ForegroundColor Green
} catch {
    Write-Host "Failed to download from GitHub releases." -ForegroundColor Red
    Write-Host "Error: $_" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please download MNN manually from:" -ForegroundColor Yellow
    Write-Host "https://github.com/alibaba/MNN/releases" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "And extract the following files to $outputPath`:" -ForegroundColor Yellow
    Write-Host "  - MNN.dll" -ForegroundColor Gray
    Write-Host "  - MNN.lib" -ForegroundColor Gray
    exit 1
}

Write-Host "Extracting MNN library..." -ForegroundColor Yellow
try {
    Expand-Archive -Path $zipFile -DestinationPath $outputPath -Force
    Remove-Item $zipFile -Force
    Write-Host "Extraction completed!" -ForegroundColor Green
} catch {
    Write-Host "Failed to extract: $_" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "MNN library installed successfully!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "Library location: $outputPath" -ForegroundColor Cyan
Write-Host ""
Write-Host "To build with MNN support, run:" -ForegroundColor Yellow
Write-Host "  `$env:MNN_LIB_DIR = '$outputPath'" -ForegroundColor Gray
Write-Host "  cargo build --features mnn" -ForegroundColor Gray
Write-Host ""
