@echo off
setlocal
title Verificar acesso ao GitHub
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0publish.ps1" -CheckAuth
set "auth_exit=%errorlevel%"
echo.
pause
exit /b %auth_exit%
