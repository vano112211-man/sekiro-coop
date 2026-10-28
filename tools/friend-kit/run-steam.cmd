@echo off
rem Sekiro co-op over Steam P2P with a friend: each player sees the other as a Wolf driven by the
rem game's own remote-player packets. Usage: run-steam.cmd host|client <friend SteamID64>
rem Without a SteamID the game starts and the log shows YOUR SteamID64.
rem Uses the test copy D:\SekiroCoop and the separate save coop-test.sl2.
setlocal
set ME3=D:\dev\me3-0.11.0\bin\me3.exe
set RUN=D:\dev\coop-run
set BUILD=D:\dev\sekiro-coop\target\release
set ROLE=%~1
if "%ROLE%"=="" set ROLE=host
set FRIEND=%~2

echo Building mod...
pushd D:\dev\sekiro-coop
set MINHOOK_LIB_DIR=D:\dev\minhook-build
cargo build --release -p sekiro-coop-dll
if errorlevel 1 (popd & echo BUILD FAILED - send the error above to Claude & exit /b 1)
popd
copy /Y "%BUILD%\sekiro_coop.dll" "%RUN%\sekiro-coop-mods\" >nul || (echo DLL copy failed & exit /b 1)
del "%LOCALAPPDATA%\sekiro-coop\sekiro-coop.log" "%RUN%\sekiro-coop.log" 2>nul

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
set SEKIRO_COOP_DP_LOOPBACK=
set SEKIRO_COOP_DP_SMOOTH=1
set SEKIRO_COOP_VTCOUNT=
set SEKIRO_COOP_DP_PROBE=

echo Launching Sekiro as %ROLE% (test copy, save coop-test.sl2), friend SteamID: %FRIEND%
"%ME3%" launch -g sekiro -e D:\SekiroCoop\sekiro.exe --disable-arxan true --online false --savefile coop-test.sl2 -p "%RUN%\sekiro-coop.me3"
copy /Y "%LOCALAPPDATA%\sekiro-coop\sekiro-coop.log" "%RUN%\sekiro-coop.log" >nul 2>&1
echo Done. Log: %RUN%\sekiro-coop.log
findstr /C:"steam" /C:"session established" "%RUN%\sekiro-coop.log"
