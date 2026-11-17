@echo off
cd /d "%~dp0"
if exist "target\verify-build\release\revengine.exe" (
  start "" "%~dp0target\verify-build\release\revengine.exe"
  exit /b 0
)
if not exist "target\release\revengine.exe" (
  echo RevEngine ainda nao foi compilada. Execute: cargo build --release
  pause
  exit /b 1
)
start "" "%~dp0target\release\revengine.exe"
