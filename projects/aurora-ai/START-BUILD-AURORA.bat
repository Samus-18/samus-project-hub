@echo off
setlocal
cd /d "%~dp0"
title Aurora AI - Build launcher
echo Starting Aurora AI build in a persistent console...
echo The console will stay open after the build so errors remain visible.
echo.
start "Aurora AI build" "%ComSpec%" /d /k ""%~dp0BUILD-AURORA.bat""
