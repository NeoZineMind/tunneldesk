@echo off
setlocal

call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64
set CARGO_BUILD_JOBS=1
cd /d "%~dp0.."
pnpm tauri:dev
