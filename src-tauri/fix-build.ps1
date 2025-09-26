Write-Host "=== Fix Rust/Tauri build cache ===" -ForegroundColor Cyan

# 1. Oprește procesele VS Code pentru a evita lock pe DLL-uri
Get-Process Code -ErrorAction SilentlyContinue | ForEach-Object {
    Write-Host "Oprind VS Code..."
    Stop-Process -Id $_.Id -Force
    Start-Sleep -Seconds 2
}

# 2. Șterge folderul target complet
if (Test-Path ".\target") {
    Write-Host "Șterg target..." -ForegroundColor Yellow
    Remove-Item -Recurse -Force ".\target"
}

# 3. Curăță cu cargo clean
Write-Host "Rulez cargo clean..." -ForegroundColor Yellow
cargo clean

# 4. Actualizează indexul de crate-uri
Write-Host "Rulez cargo update..." -ForegroundColor Yellow
cargo update

# 5. Reconstruiește în mod explicit
Write-Host "Rulez cargo build..." -ForegroundColor Yellow
cargo build

Write-Host "=== Gata! Pornește din nou VS Code și Rust Analyzer va reindexa proiectul. ===" -ForegroundColor Green
