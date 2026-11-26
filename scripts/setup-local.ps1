param([string]$PostgresBin = 'C:\Program Files\PostgreSQL\17\bin')
$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot -Parent
$local = Join-Path $repo '.local'
$data = Join-Path $local 'pgdata'
New-Item -ItemType Directory -Force -Path $local | Out-Null
$identity = [System.Security.Principal.WindowsIdentity]::GetCurrent().Name
& icacls $local /inheritance:r /grant:r "${identity}:(OI)(CI)F" | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Cannot restrict local credential directory permissions' }
function New-Secret { return [Convert]::ToHexString([System.Security.Cryptography.RandomNumberGenerator]::GetBytes(32)).ToLowerInvariant() }
$settingsFile = Join-Path $local 'settings.json'
if (!(Test-Path $settingsFile)) {
    @{ OwnerPassword = (New-Secret); AppPassword = (New-Secret); AdminCredential = (New-Secret) } | ConvertTo-Json | Set-Content -LiteralPath $settingsFile
}
$settings = Get-Content -LiteralPath $settingsFile -Raw | ConvertFrom-Json
$passwordFile = Join-Path $local 'init-password.txt'
if (!(Test-Path (Join-Path $data 'PG_VERSION'))) {
    $settings.OwnerPassword | Set-Content -LiteralPath $passwordFile
    & (Join-Path $PostgresBin 'initdb.exe') -D $data -U feedback_owner --auth=scram-sha-256 --pwfile=$passwordFile --encoding=UTF8 --locale=C
    if ($LASTEXITCODE -ne 0) { throw 'PostgreSQL initialization failed' }
    Remove-Item -LiteralPath $passwordFile
}
& (Join-Path $PostgresBin 'pg_ctl.exe') -D $data status 2>$null | Out-Null
if ($LASTEXITCODE -ne 0) {
    $argsList = "-D `"$data`" -l `"$(Join-Path $local 'postgres.log')`" -o `"-p 55432 -h 127.0.0.1`" -w start"
    $process = Start-Process -FilePath (Join-Path $PostgresBin 'pg_ctl.exe') -ArgumentList $argsList -WindowStyle Hidden -PassThru
    # Wait for pg_ctl itself, not the server process tree it intentionally starts.
    $process.WaitForExit()
    if ($process.ExitCode -ne 0) { throw 'PostgreSQL start failed; see .local/postgres.log' }
}
$previousPassword = $env:PGPASSWORD
try {
    $env:PGPASSWORD = $settings.OwnerPassword
    $connection = @('-h','127.0.0.1','-p','55432','-U','feedback_owner','-v','ON_ERROR_STOP=1')
    # Secrets are random hexadecimal strings, never user-supplied SQL.
    $roleExists = & (Join-Path $PostgresBin 'psql.exe') @connection -d postgres -tAc "SELECT 1 FROM pg_roles WHERE rolname='feedback_app'"
    if ($roleExists -ne '1') {
        & (Join-Path $PostgresBin 'psql.exe') @connection -d postgres -c "CREATE ROLE feedback_app LOGIN PASSWORD '$($settings.AppPassword)'"
        if ($LASTEXITCODE -ne 0) { throw 'Role creation failed' }
    }
    foreach ($database in @('feedback','feedback_test')) {
        $exists = & (Join-Path $PostgresBin 'psql.exe') @connection -d postgres -tAc "SELECT 1 FROM pg_database WHERE datname='$database'"
        if ($exists -ne '1') {
            & (Join-Path $PostgresBin 'createdb.exe') -h 127.0.0.1 -p 55432 -U feedback_owner $database
            if ($LASTEXITCODE -ne 0) { throw 'Database creation failed' }
        }
        & (Join-Path $PostgresBin 'psql.exe') @connection -d $database -f (Join-Path $repo 'backend/migrations/0001_feedback.sql')
        if ($LASTEXITCODE -ne 0) { throw 'Schema installation failed' }
    }
    & (Join-Path $PostgresBin 'psql.exe') @connection -d feedback -c 'REVOKE CREATE ON SCHEMA public FROM PUBLIC; GRANT CONNECT ON DATABASE feedback TO feedback_app; GRANT USAGE ON SCHEMA public TO feedback_app; GRANT SELECT, INSERT ON feedback TO feedback_app;'
    if ($LASTEXITCODE -ne 0) { throw 'Runtime grants failed' }
} finally { $env:PGPASSWORD = $previousPassword }
$settings.AdminCredential | Set-Content -LiteralPath (Join-Path $local 'admin-credential.txt')
Write-Host 'Local PostgreSQL is ready on 127.0.0.1:55432. No feedback has been seeded.'
Write-Host 'Run scripts/start-backend.ps1 and scripts/start-frontend.ps1 in separate terminals.'
Write-Host 'Administrator credential: .local/admin-credential.txt (private and ignored by Git).'
