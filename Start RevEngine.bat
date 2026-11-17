@echo off
setlocal
cd /d "%~dp0"
if not exist "RevEngine.exe" (
  echo RevEngine.exe nao foi encontrado. Extraia o pacote completo.
  pause
  exit /b 1
)
start "" "%~dp0RevEngine.exe"
