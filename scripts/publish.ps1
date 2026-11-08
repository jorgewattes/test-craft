param([string]$Repository = '')
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
# Restrict Git's ownership exception to this explicitly selected project.
function git { & git.exe -c "safe.directory=$PWD" @args }
function Check-Exit([string]$Action) { if ($LASTEXITCODE -ne 0) { throw "Falhou: $Action" } }
gh auth status
Check-Exit 'Autentique-se antes com gh auth login'
$account = gh api user --jq .login
Check-Exit 'Consultar conta GitHub'
if (-not $Repository) { $Repository = "$account/caderno" }
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
gh auth setup-git
Check-Exit 'Configurar credenciais Git'
git push -u origin HEAD
Check-Exit 'Enviar codigo'
git push origin $tag
Check-Exit 'Enviar tag'
Write-Host "Codigo publico: https://github.com/$Repository"
Write-Host "Aguarde o workflow Windows concluir: https://github.com/$Repository/actions"
Write-Host "Release: https://github.com/$Repository/releases"
