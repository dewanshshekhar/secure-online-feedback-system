param([string]$PostgresBin = 'C:\Program Files\PostgreSQL\17\bin')
$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot -Parent
# Stop the two application terminals with Ctrl+C first. This stops only this repo's cluster.
& (Join-Path $PostgresBin 'pg_ctl.exe') -D (Join-Path $repo '.local/pgdata') -m fast -w stop
if ($LASTEXITCODE -ne 0) { throw 'Local PostgreSQL did not stop cleanly' }
