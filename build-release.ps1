param(
    [switch]$Portable,
    [switch]$SkipInstall,
    [switch]$Open
)

$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

Write-Host "=== 偶译析 · 构建 Release 版本 ===" -ForegroundColor Cyan

if (-not $SkipInstall) {
    Write-Host "[1/3] 安装前端依赖..." -ForegroundColor Yellow
    npm install
    if ($LASTEXITCODE -ne 0) { throw "npm install 失败" }
} else {
    Write-Host "[1/3] 跳过依赖安装" -ForegroundColor Yellow
}

Write-Host "[2/3] 构建前端 + 类型检查..." -ForegroundColor Yellow
npm run build
if ($LASTEXITCODE -ne 0) { throw "前端构建失败" }

Write-Host "[3/3] 编译 Rust Release（首次较慢，请耐心等待）..." -ForegroundColor Yellow
if ($Portable) {
    npm run tauri build -- --no-bundle
} else {
    npm run tauri build -- --bundles nsis
}
if ($LASTEXITCODE -ne 0) { throw "Tauri 构建失败" }

$exe = (Get-ChildItem "src-tauri\target\release\*.exe" -ErrorAction SilentlyContinue | Where-Object { $_.Name -notmatch "setup|uninstall" } | Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName
Write-Host "`n=== 构建完成 ===" -ForegroundColor Green
Write-Host "可执行文件: $exe"
if (Test-Path "src-tauri\target\release\bundle\nsis") {
    Write-Host "安装包: src-tauri\target\release\bundle\nsis\*.exe"
}
if ($Open) {
    Start-Process "src-tauri\target\release\bundle\nsis"
}