@echo off
setlocal
title Publicar Test Craft no GitHub
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0publish.ps1"
set "publish_exit=%errorlevel%"
if errorlevel 1 (
    echo.
    echo A publicacao nao foi concluida. Confira a mensagem acima.
) else (
    echo.
    echo Fontes enviados. Aguarde o workflow Windows em Actions publicar a release.
)
echo.
pause
exit /b %publish_exit%
