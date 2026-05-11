@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0launch-asset-forge.ps1" %*
exit /b %ERRORLEVEL%
