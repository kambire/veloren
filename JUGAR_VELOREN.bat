@echo off
title World of Azeria - Launcher
cd /d "%~dp0"

if exist "AzeriaLauncher.exe" (
    start "" "AzeriaLauncher.exe"
    exit
)

if exist "target\debug\veloren-updater.exe" (
    start "" "target\debug\veloren-updater.exe"
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

if exist "target\release\veloren-voxygen.exe" (
    start "" "target\release\veloren-voxygen.exe"
    exit
)

echo No se encontro el lanzador ni el ejecutable de World of Azeria.
pause
