param([string]$Repository = '', [switch]$CheckAuth)
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
# Restrict Git's ownership exception to this explicitly selected project.
function git { & git.exe -c "safe.directory=$PWD" @args }
function Check-Exit([string]$Action) { if ($LASTEXITCODE -ne 0) { throw "Falhou: $Action" } }
# Check the active credential before changing files or creating a repository.
$windowsUser = [Security.Principal.WindowsIdentity]::GetCurrent().Name
Write-Host "Usuario Windows desta execucao: $windowsUser"
if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    Write-Host 'GitHub CLI (gh) nao encontrado nesta sessao. O login no GitHub Desktop nao instala esse comando.'
    Write-Host 'Instale o GitHub CLI e abra novamente seu terminal para verificar o acesso.'
    exit 1
}
$ErrorActionPreference = 'Continue'
$account = gh api user --jq .login 2>$null
$authCode = $LASTEXITCODE
$ErrorActionPreference = 'Stop'
if ($authCode -ne 0 -or -not $account) {
    Write-Host 'Esta sessao nao conseguiu acessar sua conta pela API do GitHub.'
    Write-Host 'Seu login no CMD, PowerShell ou GitHub Desktop pode continuar valido no seu usuario Windows.'
    Write-Host 'Se esta execucao usa CodexSandboxOffline, ela nao tem o mesmo acesso do seu usuario.'
    Write-Host 'Abra scripts\verificar-login.cmd com dois cliques no seu Windows. Se confirmar o acesso, abra scripts\publicar.cmd no mesmo usuario.'
    Write-Host 'Se a verificacao tambem falhar no seu usuario, execute gh auth login nesse terminal e tente novamente.'
    Write-Host 'Nenhum repositorio foi criado ou alterado.'
    exit 1
}
$account = $account.Trim()
Write-Host "Conta GitHub confirmada: $account"
if ($CheckAuth) {
    Write-Host 'Acesso confirmado para esta execucao. Verificacao concluida sem publicar nem alterar arquivos.'
    return
}
if (-not $Repository) { $Repository = "$account/test-craft" }
if ($Repository -notmatch '^[A-Za-z0-9][A-Za-z0-9_.-]*/[A-Za-z0-9][A-Za-z0-9_.-]*$') { throw 'Use conta/repositorio.' }
if (-not (Test-Path -LiteralPath .git)) { git init -b main; Check-Exit 'Inicializar Git' }
$ErrorActionPreference = 'Continue'
$existing = gh repo view $Repository --json isPrivate --jq .isPrivate 2>$null
$viewCode = $LASTEXITCODE
$ErrorActionPreference = 'Stop'
if ($viewCode -eq 0) {
    if ($existing -eq 'true') { throw 'O repositorio existente e privado. Escolha outro nome para publicar de forma aberta.' }
} else {
    gh repo create $Repository --public --description 'Banco local de questoes calculadas e provas individualizadas em PDF para Windows'
    Check-Exit 'Criar repositorio publico'
}
$repoUrl = "https://github.com/$Repository.git"
$remotes = @(git remote)
$origin = if ($remotes -contains 'origin') { git remote get-url origin } else { '' }
if (-not $origin) { git remote add origin $repoUrl; Check-Exit 'Configurar origin' }
elseif ($origin -ne $repoUrl) { throw "Origin aponta para $origin. Use o repositorio correspondente." }
& "$PSScriptRoot/build-release.ps1" -Repository $Repository
if (-not $?) { throw 'Falha ao preparar release.' }
$version = [regex]::Match((Get-Content Cargo.toml -Raw), '(?m)^version = "([0-9]+\.[0-9]+\.[0-9]+)"').Groups[1].Value
$tag = "v$version"
# Explicit allowlist: banks, student lists and exports remain local.
git add -- Cargo.toml Cargo.lock src scripts .github .gitignore .gitattributes README.md LICENSE CREDITS.md MECHANICS.md mechanics.json release-config.json CHANGELOG.md
Check-Exit 'Adicionar fontes publicaveis'
git diff --cached --quiet
if ($LASTEXITCODE -eq 1) {
    $userId = gh api user --jq .id
    Check-Exit 'Consultar identidade GitHub'
    git -c "user.name=$account" -c "user.email=$userId+$account@users.noreply.github.com" commit -m "Release $tag"
    Check-Exit 'Registrar fontes'
}
git tag $tag
Check-Exit 'Criar tag (aumente a versao para uma nova release)'
# Use CLI credentials for these pushes without changing global Git settings.
git -c credential.helper= -c 'credential.https://github.com.helper=!gh auth git-credential' push -u origin HEAD
Check-Exit 'Enviar codigo'
git -c credential.helper= -c 'credential.https://github.com.helper=!gh auth git-credential' push origin $tag
Check-Exit 'Enviar tag'
Write-Host "Codigo publico: https://github.com/$Repository"
Write-Host "Aguarde o workflow Windows concluir: https://github.com/$Repository/actions"
Write-Host "Release: https://github.com/$Repository/releases"
