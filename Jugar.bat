@echo off
title World of Azeria - Launcher
cd /d "%~dp0"

if exist "AzeriaLauncher.exe" (
    start "" "AzeriaLauncher.exe"
    exit
)

if exist "target\release\AzeriaLauncher.exe" (
    start "" "target\release\AzeriaLauncher.exe"
    exit
)

if exist "target\debug\AzeriaLauncher.exe" (
    start "" "target\debug\AzeriaLauncher.exe"
    exit
)

if exist "WorldOfAzeria.exe" (
    start "" "WorldOfAzeria.exe"
    exit
)

if exist "target\release\WorldOfAzeria.exe" (
    start "" "target\release\WorldOfAzeria.exe"
    exit
)

if exist "target\debug\WorldOfAzeria.exe" (
    start "" "target\debug\WorldOfAzeria.exe"
    exit
)

echo No se encontro el lanzador ni el ejecutable de World of Azeria.
pause
