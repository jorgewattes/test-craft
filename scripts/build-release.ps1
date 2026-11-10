param(
    [string]$Repository = '',
    [switch]$SkipMsiValidation,
    [switch]$AllowUnsigned,
    [string]$SigningCertificateThumbprint = $env:TEST_CRAFT_SIGNING_THUMBPRINT,
    [ValidateSet('CurrentUser', 'LocalMachine')][string]$SigningCertificateStore = 'CurrentUser',
    [string]$SignToolPath = $env:TEST_CRAFT_SIGNTOOL_PATH,
    [string]$TimestampUrl = 'http://timestamp.digicert.com'
)
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
. (Join-Path $PSScriptRoot 'windows-signing.ps1')
$config = Get-Content -LiteralPath release-config.json -Raw | ConvertFrom-Json
if ($Repository) {
    if ($Repository -notmatch '^[A-Za-z0-9][A-Za-z0-9_.-]*/[A-Za-z0-9][A-Za-z0-9_.-]*$') { throw 'Use conta/repositorio.' }
    $config.repository = $Repository
}
[IO.File]::WriteAllText((Join-Path $PWD 'release-config.json'), ($config | ConvertTo-Json) + [Environment]::NewLine, (New-Object Text.UTF8Encoding $false))
$manifest = Get-Content -LiteralPath Cargo.toml -Raw
$version = [regex]::Match($manifest, '(?m)^version = "([0-9]+\.[0-9]+\.[0-9]+)"').Groups[1].Value
if (-not $version) { throw 'Versao invalida no Cargo.toml.' }
if ($env:GITHUB_REF_TYPE -eq 'tag' -and $env:GITHUB_REF_NAME -ne "v$version") { throw 'A tag precisa corresponder a versao do Cargo.toml.' }
$unsignedRelease = $config.unsignedReleaseVersion -ceq $version
# The reviewed exception applies only to this version, including on tag builds.
if ($unsignedRelease) { $SigningCertificateThumbprint = '' }
$signing = New-WindowsSigningConfig -Thumbprint $SigningCertificateThumbprint -Store $SigningCertificateStore -SignToolPath $SignToolPath -TimestampUrl $TimestampUrl -AllowUnsigned:($AllowUnsigned -or $unsignedRelease) -AuthorizedUnsignedVersion $config.unsignedReleaseVersion
if ($env:GITHUB_REF_TYPE -eq 'tag' -and $SkipMsiValidation) { throw 'Publicacao por tag exige validacao ICE.' }
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
$releaseNotes = $notes.Groups[1].Value.Trim()
if (-not $signing.Enabled) { $releaseNotes += "`n`nMSI e ZIP publicados sem assinatura digital. O Windows pode mostrar Autor desconhecido. Confira os arquivos SHA-256 que acompanham os pacotes." }
[IO.File]::WriteAllText((Join-Path $PWD 'dist/release-notes.md'), $releaseNotes + [Environment]::NewLine, (New-Object Text.UTF8Encoding $false))
Copy-Item -LiteralPath "target/$releaseTarget/release/test-craft.exe" -Destination (Join-Path $stage 'test-craft.exe') -Force
# Sign the staged executable before it enters either the ZIP or the MSI.
Invoke-WindowsSigning -Path (Join-Path $stage 'test-craft.exe') -Config $signing
foreach ($file in @('README.md', 'SIGNING.md', 'LICENSE', 'CREDITS.md', 'release-config.json')) {
    Copy-Item -LiteralPath $file -Destination $stage -Force
}
$zip = Join-Path $PWD "dist/$packageName.zip"
$files = @('test-craft.exe', 'README.md', 'SIGNING.md', 'LICENSE', 'CREDITS.md', 'release-config.json') | ForEach-Object { Join-Path $stage $_ }
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
# The cabinet is embedded; sign the final MSI after linking and before hashing.
Invoke-WindowsSigning -Path $msi -Config $signing
$hash = (Get-FileHash -LiteralPath $msi -Algorithm SHA256).Hash.ToLowerInvariant()
[IO.File]::WriteAllText("$msi.sha256", "$hash  $packageName.msi" + [Environment]::NewLine, (New-Object Text.UTF8Encoding $false))
$signatureFiles = @((Join-Path $stage 'test-craft.exe'), $msi) | ForEach-Object {
    $signature = Get-AuthenticodeSignature -LiteralPath $_
    [ordered]@{
        file = [IO.Path]::GetFileName($_); status = [string]$signature.Status
        publisher = $signature.SignerCertificate.Subject
        thumbprint = $signature.SignerCertificate.Thumbprint
        timestamped = [bool]$signature.TimeStamperCertificate
    }
}
$report = [ordered]@{ signed = [bool]$signing.Enabled; files = @($signatureFiles) } | ConvertTo-Json -Depth 4
[IO.File]::WriteAllText((Join-Path $PWD 'dist/signing-status.json'), $report, (New-Object Text.UTF8Encoding $false))
if ($signing.Enabled) { Write-Host "Release assinada e verificada: $msi e $zip" }
else { Write-Warning "Pacotes SEM ASSINATURA: $msi e $zip. Nao resolvem o aviso de autor desconhecido." }
