# No certificates or trust settings are created or changed by these checks.
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'windows-signing.ps1')
function Assert-Rejected {
    param([scriptblock]$Action, [string]$Expected)
    try { & $Action | Out-Null }
    catch {
        if ($_.Exception.Message -notlike "*$Expected*") { throw }
        return
    }
    throw "Expected rejection: $Expected"
}
$previousRefType = $env:GITHUB_REF_TYPE
$previousRefName = $env:GITHUB_REF_NAME
try {
    $env:GITHUB_REF_TYPE = ''
    Assert-Rejected { New-WindowsSigningConfig } 'Certificado ausente'
    $unsigned = New-WindowsSigningConfig -AllowUnsigned -WarningAction SilentlyContinue
    if ($unsigned.Enabled) { throw 'Unsigned preparation must never claim signing.' }
    Assert-Rejected { New-WindowsSigningConfig -Thumbprint 'invalid' -AllowUnsigned } 'Thumbprint'
    $env:GITHUB_REF_TYPE = 'tag'
    $env:GITHUB_REF_NAME = 'v0.4.1'
    Assert-Rejected { New-WindowsSigningConfig -AllowUnsigned } 'release por tag exige assinatura'
    Assert-Rejected { New-WindowsSigningConfig -AllowUnsigned -Thumbprint ('A' * 40) } 'release por tag exige assinatura'
    Assert-Rejected { New-WindowsSigningConfig -AllowUnsigned -AuthorizedUnsignedVersion '0.4.2' } 'release por tag exige assinatura'
    Assert-Rejected { New-WindowsSigningConfig -AllowUnsigned -AuthorizedUnsignedVersion 'invalid' } 'release por tag exige assinatura'
    Assert-Rejected { New-WindowsSigningConfig -AuthorizedUnsignedVersion '0.4.1' } 'Certificado ausente'
    $unsignedTag = New-WindowsSigningConfig -AllowUnsigned -AuthorizedUnsignedVersion '0.4.1' -WarningAction SilentlyContinue
    if ($unsignedTag.Enabled) { throw 'Authorized unsigned tag must never claim signing.' }
    $env:GITHUB_REF_NAME = 'v0.4.2'
    Assert-Rejected { New-WindowsSigningConfig -AllowUnsigned -AuthorizedUnsignedVersion '0.4.1' } 'release por tag exige assinatura'
} finally {
    $env:GITHUB_REF_TYPE = $previousRefType
    $env:GITHUB_REF_NAME = $previousRefName
}

# Exercise invocation and failure propagation without accessing any private key.
# These are policy tests; real cryptographic verification still requires a certificate.
$script:toolCalls = @()
$script:toolExitCode = 0
$script:testSignature = [pscustomobject]@{
    Status = 'Valid'; SignerCertificate = [pscustomobject]@{ Thumbprint = ('A' * 40) }
    TimeStamperCertificate = [pscustomobject]@{ Subject = 'Test timestamp fixture' }
}
function Test-SignTool {
    $script:toolCalls += ,@($args)
    $global:LASTEXITCODE = $script:toolExitCode
}
function Get-AuthenticodeSignature {
    param([string]$LiteralPath)
    return $script:testSignature
}
$fixture = [pscustomobject]@{
    Enabled = $true; Thumbprint = ('A' * 40); Store = 'LocalMachine'
    SignToolPath = 'Test-SignTool'; TimestampUrl = 'http://timestamp.invalid'
}
Invoke-WindowsSigning -Path 'fixture with spaces.exe' -Config $fixture
if ($script:toolCalls.Count -ne 2 -or $script:toolCalls[0][0] -ne 'sign' -or
    $script:toolCalls[1][0] -ne 'verify' -or '/sm' -notin $script:toolCalls[0] -or
    '/pa' -notin $script:toolCalls[1] -or '/tw' -notin $script:toolCalls[1] -or
    $script:toolCalls[0][-1] -ne 'fixture with spaces.exe') { throw 'Signing and verification invocation failed.' }
$script:toolExitCode = 1
Assert-Rejected { Invoke-WindowsSigning -Path 'fixture.exe' -Config $fixture } 'Falha ao assinar'
Assert-Rejected { Assert-WindowsSignature -Path 'fixture.exe' -SignToolPath Test-SignTool -Thumbprint $fixture.Thumbprint } 'Authenticode invalida'
$script:toolExitCode = 0
$script:testSignature.Status = 'HashMismatch'
Assert-Rejected { Assert-WindowsSignature -Path 'fixture.exe' -SignToolPath Test-SignTool -Thumbprint $fixture.Thumbprint } 'inesperado'
$script:testSignature.Status = 'Valid'
Assert-Rejected { Assert-WindowsSignature -Path 'fixture.exe' -SignToolPath Test-SignTool -Thumbprint ('B' * 40) } 'inesperado'
$script:testSignature.TimeStamperCertificate = $null
Assert-Rejected { Assert-WindowsSignature -Path 'fixture.exe' -SignToolPath Test-SignTool -Thumbprint $fixture.Thumbprint } 'inesperado'
Write-Host 'Signing safeguards: 16 checks passed. Signature invocation uses fixtures; real signing needs a trusted certificate.'
