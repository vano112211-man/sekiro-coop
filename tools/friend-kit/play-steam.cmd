@echo off
rem Sekiro co-op over Steam (sekiro-coop, experimental). No IP, no port forwarding. Your friend appears as a Wolf.
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

rem Find me3: PATH, a portable copy unpacked into this folder (me3\bin\me3.exe), or the installer's folder.
where %ME3% >nul 2>&1
if errorlevel 1 if exist "%~dp0me3\bin\me3.exe" set ME3=%~dp0me3\bin\me3.exe
if "%ME3%"=="me3" if exist "%LOCALAPPDATA%\Programs\me3\bin\me3.exe" set ME3=%LOCALAPPDATA%\Programs\me3\bin\me3.exe
where %ME3% >nul 2>&1
if errorlevel 1 if not exist "%ME3%" (
  echo me3 was not found.
  echo Download me3 0.11.0 from https://github.com/garyttierney/me3/releases ^(official^):
  echo  - portable: unpack the Windows zip into a folder named me3 next to this script, so that
  echo    me3\bin\me3.exe is next to play-steam.cmd; or
  echo  - installer: install it, then open a NEW PowerShell window and run this script again.
  pause
  exit /b 1
)
echo Using me3: %ME3%
if not exist "%~dp0sekiro-coop-mods\sekiro_coop.dll" (echo sekiro_coop.dll is missing next to this script & pause & exit /b 1)

set SEKIRO_COOP_LOG=info,sekiro_coop=debug
set SEKIRO_COOP_PEER=%ROLE%
set SEKIRO_COOP_TRANSPORT=steam
set SEKIRO_COOP_STEAM_PEER=%FRIEND%
set SEKIRO_COOP_BIND=
set SEKIRO_COOP_PEER_ADDR=
set SEKIRO_COOP_SEND_MANIP=0
set SEKIRO_COOP_PUPPET_PROBE=1
set SEKIRO_COOP_PUPPET_SOURCE=
set SEKIRO_COOP_SPAWN=
set SEKIRO_COOP_SPAWN_PLAYER=1
set SEKIRO_COOP_PUPPET_LOGIC_OFF=0
set SEKIRO_COOP_PUPPET_TEAM=-1
set SEKIRO_COOP_NET_PLAYER_KIND=2
set SEKIRO_COOP_CLONE_PAD=
set SEKIRO_COOP_CLONE_RANGES=none
set SEKIRO_COOP_PUPPET_NOPOS=1
set SEKIRO_COOP_NET_PLAYER_POST=1
set SEKIRO_COOP_NET_PLAYER_TYPE=0
set SEKIRO_COOP_DP_SESSION=1
set SEKIRO_COOP_CLONE_OWN_PGD=2
set SEKIRO_COOP_DP_LOOPBACK=
set SEKIRO_COOP_DP_SMOOTH=1
set SEKIRO_COOP_DP_TRACE=1
set SEKIRO_COOP_DP_LEAD=
set SEKIRO_COOP_VTCOUNT=
set SEKIRO_COOP_DP_PROBE=

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
