$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
# Use the same Windows PowerShell host as the application, including when CI runs pwsh.
$windowsPowerShell = Join-Path $env:SystemRoot 'System32/WindowsPowerShell/v1.0/powershell.exe'
if (-not (Test-Path -LiteralPath $windowsPowerShell)) { throw 'Windows PowerShell is required for updater checks.' }
$testRoot = Join-Path $PWD ('checks/updater-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $testRoot | Out-Null
$payload = Join-Path $testRoot 'payload'
New-Item -ItemType Directory -Path $payload | Out-Null
$executable = Join-Path $testRoot 'test-craft.exe'
[IO.File]::WriteAllText($executable, 'previous executable')
[IO.File]::WriteAllText((Join-Path $payload 'test-craft.exe'), 'new executable')
$data = Join-Path $testRoot 'data'
New-Item -ItemType Directory -Path $data | Out-Null
[IO.File]::WriteAllText((Join-Path $data 'banco.sqlite3'), 'preserve database')
$package = Join-Path $testRoot 'package.zip'
Compress-Archive -LiteralPath (Join-Path $payload 'test-craft.exe') -DestinationPath $package
$result = Join-Path $testRoot 'result.txt'
$job = Join-Path $testRoot 'job.json'
$config = @{ package=$package; sha256=(Get-FileHash $package -Algorithm SHA256).Hash; executable=$executable; installed=$false; parentId=2147483647; result=$result }
$config | ConvertTo-Json | Set-Content -LiteralPath $job -Encoding UTF8
& $windowsPowerShell -NoProfile -ExecutionPolicy Bypass -File src/update-install.ps1 -Job $job -Quiet
if ($LASTEXITCODE -ne 0) { throw 'Portable update failed.' }
if ([IO.File]::ReadAllText($executable) -cne 'new executable') { throw 'Executable not replaced.' }
if ([IO.File]::ReadAllText($executable + '.previous') -cne 'previous executable') { throw 'Backup not preserved.' }
if ([IO.File]::ReadAllText((Join-Path $data 'banco.sqlite3')) -cne 'preserve database') { throw 'Database changed.' }
if (-not ([IO.File]::ReadAllText($result).StartsWith('Atualizacao instalada'))) { throw 'Success not recorded.' }
# A modified package must never replace the current executable.
$config.sha256 = '0' * 64
$config | ConvertTo-Json | Set-Content -LiteralPath $job -Encoding UTF8
& $windowsPowerShell -NoProfile -ExecutionPolicy Bypass -File src/update-install.ps1 -Job $job -Quiet
if ($LASTEXITCODE -eq 0) { throw 'Corrupt update accepted.' }
if ([IO.File]::ReadAllText($executable) -cne 'new executable') { throw 'Corrupt update changed executable.' }
if (-not ([IO.File]::ReadAllText($result).StartsWith('Falha na atualizacao'))) { throw 'Failure not recorded.' }
# Check both embedded scripts parse, without triggering UAC or installing an MSI.
foreach ($file in @('src/update-launch.ps1', 'src/update-install.ps1')) {
    $parseErrors=$null; $parseTokens=$null
    [Management.Automation.Language.Parser]::ParseFile((Join-Path $PWD $file), [ref]$parseTokens, [ref]$parseErrors) | Out-Null
    if ($parseErrors.Count) { throw ($parseErrors | Out-String) }
}
Write-Host 'Updater checks passed: replacement, backup, data preservation, checksum rejection, result reporting and script syntax.'
