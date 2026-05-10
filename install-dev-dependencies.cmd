@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0install-dev-dependencies.ps1" %*
exit /b %ERRORLEVEL%
