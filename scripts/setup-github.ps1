param([switch]$Login, [switch]$CheckOnly)
$ErrorActionPreference = 'Stop'
$projectRoot = (Get-Item -LiteralPath (Split-Path $PSScriptRoot -Parent)).FullName.Replace('\', '/')
Set-Location -LiteralPath $projectRoot
# Trust only this checkout for these Git commands, including the first remote lookup.
# A local config entry cannot fix ownership checks; avoid persistent/global exceptions.
$gitTrustArguments = @('-c', "safe.directory=$projectRoot")

function Invoke-CheckedGit {
    param([string[]]$GitArguments)
    & git @gitTrustArguments @GitArguments
    if ($LASTEXITCODE -ne 0) { throw 'Falha na configuracao local do Git.' }
}

try {
    if ($Login -and $CheckOnly) { throw 'Use -Login ou -CheckOnly separadamente.' }
    $runningInOfflineSandbox = [Security.Principal.WindowsIdentity]::GetCurrent().Name -match '(^|\\)CodexSandboxOffline$'
    $offlineSandboxMessage = 'O Craft iniciou o Codex como CodexSandboxOffline. Execute a verificacao e a publicacao em uma sessao do Codex ou do PowerShell aberta pelo usuario do Windows conectado ao GitHub.'
    if ($Login -and $runningInOfflineSandbox) { throw $offlineSandboxMessage }
    foreach ($command in @('git', 'gh')) {
        if (-not (Get-Command $command -ErrorAction SilentlyContinue)) {
            throw "Instale $command e abra novamente este arquivo."
        }
    }
    $repository = (Get-Content release-config.json -Raw | ConvertFrom-Json).repository
    if ($repository -ne 'jorgewattes/test-craft') { throw 'Repositorio inesperado em release-config.json.' }
    $expected = "https://github.com/$repository.git"
    $gitRoot = & git @gitTrustArguments rev-parse --show-toplevel
    if ($LASTEXITCODE -ne 0 -or $gitRoot.Replace('\', '/') -ne $projectRoot) {
        throw 'Esta pasta precisa ser a raiz do repositorio Git do Test Craft.'
    }
    $origin = & git @gitTrustArguments remote get-url origin
    if ($LASTEXITCODE -ne 0) { throw 'Este projeto precisa ser um repositorio Git com origin configurado.' }
    if ($origin -ne $expected) { throw "Origin diferente do esperado: configure $expected antes de continuar." }
    $pushOrigin = & git @gitTrustArguments remote get-url --push origin
    if ($LASTEXITCODE -ne 0 -or $pushOrigin -ne $expected) { throw 'O destino de envio do origin precisa corresponder ao repositorio.' }

    if (-not $CheckOnly) {
        # Reset inherited credential helpers for github.com in this repository only.
        # Authentication stays with GitHub CLI; this file contains no credentials.
        $gitDirectory = & git @gitTrustArguments rev-parse --absolute-git-dir
        if ($LASTEXITCODE -ne 0) { throw 'Diretorio Git indisponivel.' }
        $helperFile = Join-Path $gitDirectory 'codex-github.gitconfig'
        $helperConfig = "[credential `"https://github.com`"]`n    helper =`n    helper = !gh auth git-credential`n"
        [IO.File]::WriteAllText($helperFile, $helperConfig, (New-Object Text.UTF8Encoding $false))
        $includes = @(& git @gitTrustArguments config --local --get-all include.path)
        if ($LASTEXITCODE -notin @(0, 1)) { throw 'Falha ao ler a configuracao Git.' }
        if ($includes -notcontains 'codex-github.gitconfig') {
            Invoke-CheckedGit @('config', '--local', '--add', 'include.path', 'codex-github.gitconfig')
        }
        Write-Host "Projeto conectado ao destino $repository."
    }

    if ($Login) {
        Write-Host 'Conclua a autorizacao na pagina oficial do GitHub. Nao envie senhas ou tokens pelo chat.'
        & gh auth login --hostname github.com --git-protocol https --web --scopes workflow
        if ($LASTEXITCODE -ne 0) { throw 'Login nao concluido. Abra configurar-github.cmd para tentar novamente.' }
    }
    & gh auth status --hostname github.com
    if ($LASTEXITCODE -ne 0) {
        if ($runningInOfflineSandbox) { throw $offlineSandboxMessage }
        throw 'Login do GitHub pendente ou invalido. Abra configurar-github.cmd e autorize no navegador.'
    }
    $accessJson = & gh api "repos/$repository" --jq '{repository: .full_name, push: .permissions.push}'
    if ($LASTEXITCODE -ne 0) { throw 'Nao foi possivel verificar o acesso ao repositorio.' }
    $access = $accessJson | ConvertFrom-Json
    if ($access.repository -ne $repository -or $access.push -ne $true) {
        throw 'A conta conectada precisa de permissao de escrita em jorgewattes/test-craft.'
    }
    if (-not $CheckOnly) {
        $identityJson = & gh api user --jq '{login: .login, id: .id}'
        if ($LASTEXITCODE -ne 0) { throw 'Nao foi possivel verificar a identidade GitHub.' }
        $identity = $identityJson | ConvertFrom-Json
        if (-not $identity.login -or -not $identity.id) { throw 'Identidade GitHub incompleta.' }
        $authorName = & git @gitTrustArguments config user.name
        if (-not $authorName) { Invoke-CheckedGit @('config', '--local', 'user.name', $identity.login) }
        $authorEmail = & git @gitTrustArguments config user.email
        if (-not $authorEmail) { Invoke-CheckedGit @('config', '--local', 'user.email', "$($identity.id)+$($identity.login)@users.noreply.github.com") }
    }
    $helpers = @(& git @gitTrustArguments config --get-all credential.https://github.com.helper)
    if ($LASTEXITCODE -ne 0 -or $helpers[-1] -ne '!gh auth git-credential') {
        throw 'Execute scripts/setup-github.ps1 sem -CheckOnly para configurar o acesso do Git.'
    }
    $authorName = & git @gitTrustArguments config user.name
    $authorEmail = & git @gitTrustArguments config user.email
    if (-not $authorName -or -not $authorEmail) {
        throw 'Identidade de commits ausente. Execute scripts/setup-github.ps1 sem -CheckOnly.'
    }
    Write-Host 'Pronto: autenticacao e permissao de escrita verificadas. O Codex pode preparar e enviar atualizacoes quando solicitado.'
    exit 0
} catch {
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
}
