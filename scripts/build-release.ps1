param([string]$Repository = '')
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
if ($Repository) {
    if ($Repository -notmatch '^[A-Za-z0-9][A-Za-z0-9_.-]*/[A-Za-z0-9][A-Za-z0-9_.-]*$') { throw 'Use conta/repositorio.' }
    @{ repository = $Repository } | ConvertTo-Json | Set-Content -LiteralPath release-config.json -Encoding UTF8
}
$config = Get-Content -LiteralPath release-config.json -Raw
[IO.File]::WriteAllText((Join-Path $PWD 'release-config.json'), $config.TrimStart([char]0xFEFF), (New-Object Text.UTF8Encoding $false))
$manifest = Get-Content -LiteralPath Cargo.toml -Raw
$version = [regex]::Match($manifest, '(?m)^version = "([0-9]+\.[0-9]+\.[0-9]+)"').Groups[1].Value
if (-not $version) { throw 'Versao invalida no Cargo.toml.' }
if ($env:GITHUB_REF_TYPE -eq 'tag' -and $env:GITHUB_REF_NAME -ne "v$version") { throw 'A tag precisa corresponder a versao do Cargo.toml.' }
cargo test --locked
if ($LASTEXITCODE -ne 0) { throw 'Os testes falharam.' }
cargo build --release --locked
if ($LASTEXITCODE -ne 0) { throw 'A compilacao falhou.' }
$packageName = "caderno-$version-windows-x64"
$stage = Join-Path $PWD "dist/$packageName"
New-Item -ItemType Directory -Force -Path $stage | Out-Null
Copy-Item -LiteralPath target/release/caderno.exe -Destination (Join-Path $stage 'caderno.exe') -Force
foreach ($file in @('README.md', 'LICENSE', 'CREDITS.md', 'release-config.json')) {
    Copy-Item -LiteralPath $file -Destination $stage -Force
}
$zip = Join-Path $PWD "dist/$packageName.zip"
$files = @('caderno.exe', 'README.md', 'LICENSE', 'CREDITS.md', 'release-config.json') | ForEach-Object { Join-Path $stage $_ }
Compress-Archive -LiteralPath $files -DestinationPath $zip -Force
$hash = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
[IO.File]::WriteAllText("$zip.sha256", "$hash  $packageName.zip" + [Environment]::NewLine, (New-Object Text.UTF8Encoding $false))
Write-Host "Release pronta: $zip"
