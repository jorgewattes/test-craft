param([Parameter(Mandatory=$true)][string]$Job)
$ErrorActionPreference = 'Stop'
$config = Get-Content -LiteralPath $Job -Raw -Encoding UTF8 | ConvertFrom-Json
if ((Get-FileHash -LiteralPath $config.package -Algorithm SHA256).Hash -ine $config.sha256) { throw 'O pacote baixado falhou na verificacao SHA-256.' }
$script = Join-Path $PSScriptRoot 'install.ps1'
$arguments = @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', ('"{0}"' -f $script), '-Job', ('"{0}"' -f $Job))
$powershell = Join-Path $PSHOME 'powershell.exe'
# Ask for administrator authorization before the application closes. The MSI is
# per-user: install under the original account even if another admin grants consent.
if ($config.installed) {
    $consent = Start-Process -FilePath $powershell -ArgumentList @('-NoProfile', '-NonInteractive', '-Command', 'exit 0') -Verb RunAs -WindowStyle Hidden -Wait -PassThru -ErrorAction Stop
    if ($consent.ExitCode -ne 0) { throw 'Autorizacao administrativa recusada.' }
    Start-Process -FilePath $powershell -ArgumentList $arguments -WindowStyle Hidden -ErrorAction Stop | Out-Null
} else {
    Start-Process -FilePath $powershell -ArgumentList $arguments -Verb RunAs -WindowStyle Hidden -ErrorAction Stop | Out-Null
}
