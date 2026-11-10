# Authenticode uses an existing certificate/private key in the Windows store.
# Never create a self-signed certificate or change machine trust to hide warnings.
function New-WindowsSigningConfig {
    [CmdletBinding()]
    param(
        [string]$Thumbprint,
        [ValidateSet('CurrentUser', 'LocalMachine')][string]$Store = 'CurrentUser',
        [string]$SignToolPath,
        [string]$TimestampUrl = 'http://timestamp.digicert.com',
        [switch]$AllowUnsigned,
        [string]$AuthorizedUnsignedVersion = ''
    )
    if ($AllowUnsigned -and $env:GITHUB_REF_TYPE -eq 'tag' -and
        ($AuthorizedUnsignedVersion -notmatch '^\d+\.\d+\.\d+$' -or $env:GITHUB_REF_NAME -ne "v$AuthorizedUnsignedVersion")) {
        throw 'Uma release por tag exige assinatura ou autorizacao explicita para esta versao em release-config.json.'
    }
    if (-not $Thumbprint) {
        if (-not $AllowUnsigned) {
            throw 'Certificado ausente. Configure TEST_CRAFT_SIGNING_THUMBPRINT com um certificado confiavel de assinatura de codigo. Para preparacao local sem assinatura, use -AllowUnsigned; o aviso de autor desconhecido permanecera.'
        }
        Write-Warning 'Pacote SEM ASSINATURA: o Windows continuara mostrando autor desconhecido.'
        return [pscustomobject]@{ Enabled = $false }
    }
    $Thumbprint = ($Thumbprint -replace '\s', '').ToUpperInvariant()
    if ($Thumbprint -notmatch '^[A-F0-9]{40}$') { throw 'Thumbprint de assinatura invalido (40 caracteres hexadecimais).' }
    $certificate = Get-Item -LiteralPath "Cert:\$Store\My\$Thumbprint" -ErrorAction Stop
    $now = Get-Date
    if (-not $certificate.HasPrivateKey) { throw 'O certificado selecionado nao possui chave privada acessivel.' }
    if ($certificate.NotBefore -gt $now -or $certificate.NotAfter -le $now) { throw 'O certificado selecionado esta fora do prazo de validade.' }
    if ('1.3.6.1.5.5.7.3.3' -notin @($certificate.EnhancedKeyUsageList | ForEach-Object { $_.ObjectId.Value })) {
        throw 'O certificado selecionado nao permite assinatura de codigo.'
    }
    $chain = New-Object System.Security.Cryptography.X509Certificates.X509Chain
    try {
        if (-not $chain.Build($certificate)) { throw 'O certificado nao possui uma cadeia confiavel no Windows.' }
    } finally { $chain.Dispose() }
    $timestampUri = $null
    if (-not [Uri]::TryCreate($TimestampUrl, [UriKind]::Absolute, [ref]$timestampUri) -or $timestampUri.Scheme -notin @('http', 'https')) {
        throw 'Informe um servidor de carimbo de tempo RFC 3161 HTTP/HTTPS.'
    }
    if (-not $SignToolPath) {
        $command = Get-Command signtool.exe -ErrorAction SilentlyContinue
        if ($command) { $SignToolPath = $command.Source }
        else {
            $sdk = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits/10/bin'
            $SignToolPath = Get-ChildItem "$sdk/*/x64/signtool.exe" -ErrorAction SilentlyContinue |
                Sort-Object { [version]$_.Directory.Parent.Name } -Descending |
                Select-Object -First 1 -ExpandProperty FullName
        }
    }
    if (-not $SignToolPath -or -not (Test-Path -LiteralPath $SignToolPath -PathType Leaf)) {
        throw 'SignTool ausente. Instale o Windows SDK ou configure TEST_CRAFT_SIGNTOOL_PATH.'
    }
    return [pscustomobject]@{
        Enabled = $true; Thumbprint = $Thumbprint; Store = $Store
        SignToolPath = $SignToolPath; TimestampUrl = $TimestampUrl
    }
}

function Assert-WindowsSignature {
    param([string]$Path, [string]$SignToolPath, [string]$Thumbprint)
    & $SignToolPath verify /pa /all /tw $Path
    if ($LASTEXITCODE -ne 0) { throw "Assinatura Authenticode invalida ou sem carimbo de tempo: $Path" }
    $signature = Get-AuthenticodeSignature -LiteralPath $Path
    if ($signature.Status -ne 'Valid' -or -not $signature.TimeStamperCertificate -or
        $signature.SignerCertificate.Thumbprint -ne $Thumbprint) {
        throw "Assinatura, identidade ou carimbo de tempo inesperado: $Path"
    }
}

function Invoke-WindowsSigning {
    param([string]$Path, $Config)
    if (-not $Config.Enabled) { return }
    $storeArgs = @('/s', 'My')
    if ($Config.Store -eq 'LocalMachine') { $storeArgs += '/sm' }
    & $Config.SignToolPath sign /sha1 $Config.Thumbprint @storeArgs /fd SHA256 /tr $Config.TimestampUrl /td SHA256 /d 'Test Craft' $Path
    if ($LASTEXITCODE -ne 0) { throw "Falha ao assinar: $Path" }
    Assert-WindowsSignature -Path $Path -SignToolPath $Config.SignToolPath -Thumbprint $Config.Thumbprint
}
