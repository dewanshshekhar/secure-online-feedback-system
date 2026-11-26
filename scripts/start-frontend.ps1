$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot -Parent
$trunk = Join-Path $repo '.tools/bin/trunk.exe'
if (!(Test-Path $trunk)) { $trunk = (Get-Command trunk -ErrorAction Stop).Source }
Push-Location (Join-Path $repo 'frontend')
$previousNoColor = $env:NO_COLOR
$env:NO_COLOR = 'true'
try {
    & $trunk serve --locked
    if ($LASTEXITCODE -ne 0) { throw 'Frontend server failed' }
} finally { $env:NO_COLOR = $previousNoColor; Pop-Location }
