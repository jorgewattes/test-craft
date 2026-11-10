param([Parameter(Mandatory=$true)][string]$Job, [switch]$Quiet)
$ErrorActionPreference = 'Stop'
$config = Get-Content -LiteralPath $Job -Raw -Encoding UTF8 | ConvertFrom-Json
try {
    $running = Get-Process -Id $config.parentId -ErrorAction SilentlyContinue
    if ($running) { Wait-Process -Id $config.parentId -Timeout 120 -ErrorAction Stop }
    if ((Get-FileHash -LiteralPath $config.package -Algorithm SHA256).Hash -ine $config.sha256) { throw 'O pacote foi alterado depois do download.' }
    if ($config.installed) {
        $log = Join-Path ([IO.Path]::GetDirectoryName($Job)) 'installer.log'
        $arguments = @('/i', ('"{0}"' -f $config.package), '/passive', '/norestart', '/L*v', ('"{0}"' -f $log))
        $process = Start-Process -FilePath (Join-Path $env:SystemRoot 'System32\msiexec.exe') -ArgumentList $arguments -WindowStyle Hidden -Wait -PassThru
        if ($process.ExitCode -notin @(0, 3010)) { throw "Instalacao cancelada ou com erro ($($process.ExitCode)). Log: $log" }
    } else {
        $stage = Join-Path ([IO.Path]::GetDirectoryName($Job)) 'extracted'
        Expand-Archive -LiteralPath $config.package -DestinationPath $stage
        $new = Join-Path $stage 'test-craft.exe'
        if (-not (Test-Path -LiteralPath $new -PathType Leaf)) { throw 'Executavel ausente no pacote.' }
        $backup = $config.executable + '.previous'
        Copy-Item -LiteralPath $config.executable -Destination $backup -Force
        try { Copy-Item -LiteralPath $new -Destination $config.executable -Force }
        catch { Copy-Item -LiteralPath $backup -Destination $config.executable -Force; throw }
    }
    [IO.File]::WriteAllText($config.result, 'Atualizacao instalada. Abra o Test Craft novamente.', (New-Object Text.UTF8Encoding $false))
    if (-not $Quiet) {
        Add-Type -AssemblyName PresentationFramework
        [System.Windows.MessageBox]::Show('Atualizacao concluida. Abra o Test Craft novamente.', 'Test Craft') | Out-Null
    }
} catch {
    [IO.File]::WriteAllText($config.result, ('Falha na atualizacao: ' + $_.Exception.Message), (New-Object Text.UTF8Encoding $false))
    if (-not $Quiet) {
        Add-Type -AssemblyName PresentationFramework
        [System.Windows.MessageBox]::Show($_.Exception.Message, 'Test Craft - Atualizacao') | Out-Null
    }
    exit 1
}
