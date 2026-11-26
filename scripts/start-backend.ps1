$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot -Parent
$settings = Get-Content -LiteralPath (Join-Path $repo '.local/settings.json') -Raw | ConvertFrom-Json
$env:DATABASE_URL = "postgres://feedback_app:$($settings.AppPassword)@127.0.0.1:55432/feedback"
$env:ADMIN_REVIEW_TOKEN = $settings.AdminCredential
$env:APP_ORIGIN = 'http://127.0.0.1:8080'
$env:COOKIE_SECURE = 'false'
$env:BIND_ADDR = '127.0.0.1:3000'
Push-Location (Join-Path $repo 'backend')
try { cargo run --locked } finally {
    Pop-Location
    Remove-Item Env:DATABASE_URL,Env:ADMIN_REVIEW_TOKEN -ErrorAction SilentlyContinue
}
