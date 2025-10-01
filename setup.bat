@echo off
echo Kanari Oracle - Setup Script for Windows
echo =======================================
echo.

echo Checking Rust installation...
where rust >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Rust is not installed. Please install Rust from https://rustup.rs/
    echo After installing Rust, run this script again.
    pause
    exit /b 1
)

echo [OK] Rust is installed
echo.

echo Building Kanari Oracle...
cargo build --release
if %errorlevel% neq 0 (
    echo [ERROR] Build failed. Please check the error messages above.
    pause
    exit /b 1
)

echo [OK] Build successful
echo.

echo Creating default configuration...
if not exist "config.json" (
    echo {
    echo   "crypto": {
    echo     "coingecko_api_key": null,
    echo     "binance_api_key": null,
    echo     "binance_secret_key": null,
    echo     "default_vs_currency": "usd",
    echo     "symbols": ["bitcoin", "ethereum", "binancecoin", "cardano", "solana"]
    echo   },
    echo   "stocks": {
    echo     "alpha_vantage_api_key": null,
    echo     "finnhub_api_key": null,
    echo     "symbols": ["AAPL", "GOOGL", "MSFT", "AMZN", "TSLA"]
    echo   },
    echo   "general": {
    echo     "request_timeout": 30,
    echo     "max_retries": 3,
    echo     "retry_delay": 1000,
    echo     "enable_logging": true
    echo   }
    echo } > config.json
    echo [OK] Default configuration created: config.json
) else (
    echo [OK] Configuration file already exists: config.json
)

echo.
echo Setup completed successfully!
echo.
echo Quick Test Commands:
echo ====================
echo 1. List available symbols:
echo    cargo run -- list
echo.
echo 2. Get Apple stock price:
echo    cargo run -- price AAPL --asset-type stock
echo.
echo 3. Start oracle service with 30-second intervals:
echo    cargo run -- start --interval 30
echo.
echo 4. View help:
echo    cargo run -- --help
echo.
echo For better performance, add API keys to config.json
echo See README.md for API key setup instructions.
echo.
pause