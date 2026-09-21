@echo off
title Coachify Desktop App
cd /d "%~dp0"
set PATH=C:\Program Files\nodejs;C:\Users\User\.cargo\bin;C:\Users\User\AppData\Roaming\npm;%PATH%

echo ===================================================
echo Starting Coachify Desktop App (Tauri 2 + Vite + Core)
echo ===================================================

:: Ensure any stale dev processes or port 5173 bindings are cleared
taskkill /F /IM coachify.exe >nul 2>&1
for /f "tokens=5" %%p in ('netstat -aon ^| findstr ":5173 " ^| findstr "LISTENING"') do (
    taskkill /F /PID %%p >nul 2>&1
)

call "C:\Users\User\AppData\Roaming\npm\pnpm.cmd" dev
if %ERRORLEVEL% NEQ 0 (
    echo.
    echo Coachify exited with error code %ERRORLEVEL%.
    pause
)
