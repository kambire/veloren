@echo off
title Veloren MMORPG - Launcher & Updater
cd /d "%~dp0"

if exist "VelorenLauncher.exe" (
    start "" "VelorenLauncher.exe"
    exit
)

if exist "target\release\veloren-updater.exe" (
    start "" "target\release\veloren-updater.exe"
    exit
)

if exist "target\debug\veloren-voxygen.exe" (
    start "" "target\debug\veloren-voxygen.exe"
    exit
)

echo No se encontro VelorenLauncher.exe ni el binario del juego.
echo Compilalo con cargo build o ejecuta VelorenLauncher.
pause
