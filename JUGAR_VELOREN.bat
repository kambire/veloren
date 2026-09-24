@echo off
title World of Azeria - Launcher & Updater
cd /d "%~dp0"

if exist "target\release\veloren-updater.exe" (
    start "" "target\release\veloren-updater.exe"
    exit
)

if exist "target\debug\veloren-updater.exe" (
    start "" "target\debug\veloren-updater.exe"
    exit
)

if exist "AzeriaLauncher.exe" (
    start "" "AzeriaLauncher.exe"
    exit
)

start "" "target\debug\veloren-voxygen.exe"
