@echo off
setlocal EnableExtensions DisableDelayedExpansion
cd /d "%~dp0"
title Aurora AI - Windows installer build
chcp 65001 >nul
rem AURORA AI v0.8.6 - FINAL BUILD
rem Expected output: Aurora-AI-Setup-0.8.6.exe
rem Developer fallback (same full check sequence): call npm run dist

echo ============================================================
echo                 AURORA AI v0.8.6 BUILD
echo ============================================================
echo.
echo This window stays open after success or failure.
echo Full diagnostic output: BUILD-AURORA-output.log
echo Build summary: BUILD-AURORA-last-run.log
echo.
where powershell.exe >nul 2>nul
if errorlevel 1 (
  echo ERROR: Windows PowerShell was not found.
  echo Windows PowerShell is included with supported Windows 10/11 installations.
  echo.
  pause
  exit /b 1
)
powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%~dp0BUILD-AURORA.ps1"
set "BUILD_EXIT=%ERRORLEVEL%"
echo.
if not "%BUILD_EXIT%"=="0" (
  echo Build failed. The complete error and npm debug details are in BUILD-AURORA-output.log.
) else (
  echo Build succeeded. Verify the installer in the dist folder.
)
echo.
echo Press any key to close this window.
pause >nul
exit /b %BUILD_EXIT%
