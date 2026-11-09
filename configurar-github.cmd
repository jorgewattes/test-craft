@echo off
title Conectar Test Craft ao GitHub
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\setup-github.ps1" -Login
if errorlevel 1 (
    echo.
    echo A conexao ainda nao foi concluida. Confira a mensagem acima.
)
pause
