# Kanari Oracle - Demo Script
# This script demonstrates various oracle functions

Write-Host "=== Kanari Oracle Demo ===" -ForegroundColor Cyan
Write-Host ""

# Test 1: List available symbols
Write-Host "1. Listing available symbols..." -ForegroundColor Green
cargo run -- list
Write-Host ""

# Test 2: Get individual stock prices
Write-Host "2. Getting individual stock prices..." -ForegroundColor Green
$stocks = @("AAPL", "GOOGL", "TSLA")

foreach ($stock in $stocks) {
    Write-Host "Fetching $stock price..." -ForegroundColor Yellow
    cargo run -- price $stock --asset-type stock
    Write-Host ""
}

# Test 3: Show configuration
Write-Host "3. Current configuration:" -ForegroundColor Green
if (Test-Path "config.json") {
    Get-Content "config.json" | ConvertFrom-Json | ConvertTo-Json -Depth 3
} else {
    Write-Host "No configuration file found. Run 'cargo run -- start' to create default config."
}

Write-Host ""
Write-Host "Demo completed!" -ForegroundColor Cyan
Write-Host "To start the oracle service, run: cargo run -- start" -ForegroundColor Yellow