param([string]$Repository = '', [switch]$SkipMsiValidation)
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
$changelog = Get-Content -LiteralPath CHANGELOG.md -Raw -Encoding UTF8
$notes = [regex]::Match($changelog, '(?ms)^# Test Craft ' + [regex]::Escape($version) + '\r?\n(.*?)(?=^# Test Craft |\z)')
if (-not $notes.Success) { throw 'Faltam as notas desta versao no CHANGELOG.md.' }
cargo test --locked
if ($LASTEXITCODE -ne 0) { throw 'Os testes falharam.' }
# Apply the CRT setting to every target dependency, including bundled SQLite.
# An explicit target keeps these flags away from host build scripts/proc macros.
$releaseTarget = 'x86_64-pc-windows-msvc'
$previousRustFlags = $env:RUSTFLAGS
try {
    $env:RUSTFLAGS = "$previousRustFlags -C target-feature=+crt-static".Trim()
    cargo build --release --locked --target $releaseTarget
    if ($LASTEXITCODE -ne 0) { throw 'A compilacao falhou.' }
} finally {
    $env:RUSTFLAGS = $previousRustFlags
}
$packageName = "test-craft-$version-windows-x64"
$stage = Join-Path $PWD "dist/$packageName"
New-Item -ItemType Directory -Force -Path $stage | Out-Null
[IO.File]::WriteAllText((Join-Path $PWD 'dist/release-notes.md'), $notes.Groups[1].Value.Trim() + [Environment]::NewLine, (New-Object Text.UTF8Encoding $false))
Copy-Item -LiteralPath "target/$releaseTarget/release/test-craft.exe" -Destination (Join-Path $stage 'test-craft.exe') -Force
foreach ($file in @('README.md', 'LICENSE', 'CREDITS.md', 'release-config.json')) {
    Copy-Item -LiteralPath $file -Destination $stage -Force
}
$zip = Join-Path $PWD "dist/$packageName.zip"
$files = @('test-craft.exe', 'README.md', 'LICENSE', 'CREDITS.md', 'release-config.json') | ForEach-Object { Join-Path $stage $_ }
Compress-Archive -LiteralPath $files -DestinationPath $zip -Force
$hash = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
[IO.File]::WriteAllText("$zip.sha256", "$hash  $packageName.zip" + [Environment]::NewLine, (New-Object Text.UTF8Encoding $false))
# Keep the installer marker out of the portable ZIP.
$marker = Join-Path $PWD 'dist/installed.txt'
[IO.File]::WriteAllText($marker, 'Data: %LOCALAPPDATA%\Test Craft\data', (New-Object Text.UTF8Encoding $false))
$wix = Join-Path $PWD '.tools/wix'
if (-not (Test-Path -LiteralPath (Join-Path $wix 'candle.exe'))) {
    New-Item -ItemType Directory -Force -Path $wix | Out-Null
    $archive = Join-Path $PWD '.tools/wix314-binaries.zip'
    if (-not (Test-Path -LiteralPath $archive)) {
        gh release download wix3141rtm --repo wixtoolset/wix3 --pattern wix314-binaries.zip --dir .tools
        if ($LASTEXITCODE -ne 0) { throw 'Falha ao obter WiX Toolset.' }
    }
    if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash -ne '6AC824E1642D6F7277D0ED7EA09411A508F6116BA6FAE0AA5F2C7DAA2FF43D31') { throw 'WiX: checksum invalido.' }
    Expand-Archive -LiteralPath $archive -DestinationPath $wix -Force
}
$object = Join-Path $PWD 'dist/test-craft.wixobj'
$msi = Join-Path $PWD "dist/$packageName.msi"
& (Join-Path $wix 'candle.exe') -nologo -arch x64 "-dVersion=$version" "-dStage=$stage" "-dMarker=$marker" -out $object installer/test-craft.wxs
if ($LASTEXITCODE -ne 0) { throw 'Falha ao compilar o instalador.' }
$validationArgs = @()
# Isolated environments may not expose the Windows Installer service required by ICE.
# CI keeps full ICE validation enabled by default.
if ($SkipMsiValidation) { $validationArgs = @('-sval'); Write-Warning 'ICE validation skipped: verify the MSI on Windows before publishing.' }
& (Join-Path $wix 'light.exe') -nologo -ext (Join-Path $wix 'WixUIExtension.dll') -cultures:pt-BR @validationArgs -out $msi $object
if ($LASTEXITCODE -ne 0) { throw 'Falha ao gerar o MSI.' }
$hash = (Get-FileHash -LiteralPath $msi -Algorithm SHA256).Hash.ToLowerInvariant()
[IO.File]::WriteAllText("$msi.sha256", "$hash  $packageName.msi" + [Environment]::NewLine, (New-Object Text.UTF8Encoding $false))
Write-Host "Release pronta: $msi e $zip"
