@echo off
rem Sekiro co-op over Steam (sekiro-coop, experimental). No IP, no port forwarding.
rem Usage:  play-steam.cmd host <friend SteamID64>     (the one whose world it is)
rem         play-steam.cmd client <friend SteamID64>
rem Without a SteamID the game starts anyway and the log shows YOUR SteamID64 to send to your friend.
setlocal
cd /d "%~dp0"

rem ---- edit these two lines if needed ----
set ME3=me3
rem Leave empty to let me3 find Sekiro in Steam; or point at a COPY of the game: set SEKIRO_EXE=D:\SekiroCoop\sekiro.exe
set SEKIRO_EXE=
rem ----------------------------------------

set ROLE=%~1
if "%ROLE%"=="" set ROLE=host
set FRIEND=%~2

where %ME3% >nul 2>&1
if errorlevel 1 if not exist "%ME3%" (
  echo me3 was not found. Install it from https://github.com/garyttierney/me3/releases ^(official^) or set ME3= to the full path of me3.exe in this file.
  pause
  exit /b 1
)
if not exist "%~dp0sekiro-coop-mods\sekiro_coop.dll" (echo sekiro_coop.dll is missing next to this script & pause & exit /b 1)

set SEKIRO_COOP_LOG=info,sekiro_coop=debug
set SEKIRO_COOP_PEER=%ROLE%
set SEKIRO_COOP_TRANSPORT=steam
set SEKIRO_COOP_STEAM_PEER=%FRIEND%
set SEKIRO_COOP_BIND=
set SEKIRO_COOP_PEER_ADDR=
set SEKIRO_COOP_PUPPET_PROBE=1
set SEKIRO_COOP_PUPPET_SOURCE=net
set SEKIRO_COOP_PUPPET_FREEZE_AI=
set SEKIRO_COOP_PUPPET_WARP_MS=
set SEKIRO_COOP_PUPPET_CATALOG=
set SEKIRO_COOP_PUPPET_CATALOG_MS=3000
set SEKIRO_COOP_PUPPET_SLOTSCAN=
set SEKIRO_COOP_SPAWN=5060:auto
set SEKIRO_COOP_SPAWN_PLAYER=
set SEKIRO_COOP_PUPPET_LOGIC_OFF=
set SEKIRO_COOP_PUPPET_TEAM=
set SEKIRO_COOP_PUPPET_ANIM_PASSTHROUGH=
set SEKIRO_COOP_PUPPET_INPUT_COPY=
set SEKIRO_COOP_PUPPET_NOPOS=
set SEKIRO_COOP_PUPPET_PHASES=


set EXEARG=
if not "%SEKIRO_EXE%"=="" set EXEARG=-e "%SEKIRO_EXE%"
del "%LOCALAPPDATA%\sekiro-coop\sekiro-coop.log" 2>nul

echo Starting Sekiro as %ROLE%. Friend SteamID64: %FRIEND%
echo The mod uses a separate save file coop.sl2 (your normal save is not touched).
"%ME3%" launch -g sekiro %EXEARG% --disable-arxan true --online false --savefile coop.sl2 -p "%~dp0sekiro-coop.me3"

copy /Y "%LOCALAPPDATA%\sekiro-coop\sekiro-coop.log" "%~dp0sekiro-coop.log" >nul 2>&1
echo.
echo ---- Steam lines from the log (send sekiro-coop.log next to this script if something failed) ----
findstr /C:"steam" "%~dp0sekiro-coop.log"
pause
