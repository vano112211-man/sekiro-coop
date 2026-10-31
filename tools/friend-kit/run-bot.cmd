@echo off
rem Bot test: Wolf clone driven by the game's own remote-player packets; a bot runs the Hero in circles
rem for 60 s, the mod logs smoothness numbers ('bot:' lines) and the game is closed when the bot is done.
rem Argument 1 = nosmooth (smoothing off) or lead (target 1.2x ahead to cancel the game's 1/1.2 damping).
rem Argument 2 = the clone's character type for 0x9f5170 (default 0; remote phantoms use 1, 2, 12).
rem The simulator mirrors the Hero's packets back with a delay; the clone is a kind-2 (Network) PlayerIns.
rem Uses the test copy and the separate save coop-test.sl2. Optional argument: warp interval in ms.
setlocal
set ME3=D:\dev\me3-0.11.0\bin\me3.exe
set RUN=D:\dev\coop-run
set BUILD=D:\dev\sekiro-coop\target\release

echo Building mod and simulator...
pushd D:\dev\sekiro-coop
set MINHOOK_LIB_DIR=D:\dev\minhook-build
cargo build --release -p sekiro-coop-dll -p peer-simulator
if errorlevel 1 (popd & echo BUILD FAILED - send the error above to Claude & exit /b 1)
popd

copy /Y "%BUILD%\sekiro_coop.dll" "%RUN%\sekiro-coop-mods\" >nul || (echo DLL copy failed & exit /b 1)
del "%LOCALAPPDATA%\sekiro-coop\sekiro-coop.log" 2>nul

rem Stop any simulator left from an earlier run, then start a fresh one with a fresh log.
taskkill /F /IM peer-simulator.exe >nul 2>&1
ping -n 2 127.0.0.1 >nul
del "%RUN%\peer.log" "%RUN%\peer.err" "%RUN%\sekiro-coop.log" 2>nul
start "peer-simulator" /MIN cmd /c ""%BUILD%\peer-simulator.exe" --bind 127.0.0.1:28001 --peer 127.0.0.1:28000 --as client --mirror --mirror-delay-ms 250 --mirror-offset 5 > "%RUN%\peer.log" 2> "%RUN%\peer.err""
ping -n 3 127.0.0.1 >nul
tasklist /FI "IMAGENAME eq peer-simulator.exe" | find /I "peer-simulator.exe" >nul
if errorlevel 1 (
  echo.
  echo ===== SIMULATOR DID NOT START - game not launched. Tell Claude. =====
  type "%RUN%\peer.err" 2>nul
  exit /b 1
)
rem Wait until the simulator has really started (its first log line); a freshly
rem built exe can sit for a while before it runs (run 39: alive but silent, no session).
set /a WAITED=0
:waitsim
findstr /C:"peer-simulator starting" "%RUN%\peer.log" >nul 2>&1
if not errorlevel 1 goto simok
set /a WAITED+=1
if %WAITED% GEQ 30 (
  echo.
  echo ===== SIMULATOR STARTED BUT IS SILENT AFTER 30 s - game not launched. Tell Claude. =====
  taskkill /F /IM peer-simulator.exe >nul 2>&1
  exit /b 1
)
ping -n 2 127.0.0.1 >nul
goto waitsim
:simok
echo Simulator running.

set SEKIRO_COOP_LOG=info,sekiro_coop=debug
set SEKIRO_COOP_PEER=host
set SEKIRO_COOP_BIND=127.0.0.1:28000
set SEKIRO_COOP_PEER_ADDR=127.0.0.1:28001
set SEKIRO_COOP_PUPPET_PROBE=1
set SEKIRO_COOP_PUPPET_SOURCE=
set SEKIRO_COOP_PUPPET_FREEZE_AI=
set SEKIRO_COOP_PUPPET_WARP_MS=
set SEKIRO_COOP_PUPPET_CATALOG=
set SEKIRO_COOP_PUPPET_CATALOG_MS=3000
set SEKIRO_COOP_PUPPET_SLOTSCAN=
set SEKIRO_COOP_SPAWN=
set SEKIRO_COOP_SPAWN_PLAYER=1
set SEKIRO_COOP_PUPPET_LOGIC_OFF=0
set SEKIRO_COOP_PUPPET_TEAM=-1
set SEKIRO_COOP_PUPPET_ANIM_PASSTHROUGH=
set SEKIRO_COOP_PUPPET_INPUT_COPY=
set SEKIRO_COOP_NET_PLAYER_KIND=2
set SEKIRO_COOP_CLONE_PAD=
set SEKIRO_COOP_CLONE_RANGES=none
set SEKIRO_COOP_PUPPET_NOPOS=1
set SEKIRO_COOP_PUPPET_CORRECT_ONLY=
set SEKIRO_COOP_CLONE_SUBOBJ=
set SEKIRO_COOP_CLONE_SLOT0=0
set SEKIRO_COOP_NET_PLAYER_POST=1
set SEKIRO_COOP_NET_PLAYER_TYPE=0
if not "%~2"=="" set SEKIRO_COOP_NET_PLAYER_TYPE=%~2
set SEKIRO_COOP_CLONE_DIFF=
set SEKIRO_COOP_BVAR_SCAN=
set SEKIRO_COOP_BVAR_COPY=
set SEKIRO_COOP_MODULE_LOG=
set SEKIRO_COOP_CLONE_LOCAL=
set SEKIRO_COOP_CLONE_NOOMIT=
set SEKIRO_COOP_VTCOUNT=
set SEKIRO_COOP_DP_PROBE=
set SEKIRO_COOP_DP_SESSION=1
set SEKIRO_COOP_DP_LOOPBACK=
set SEKIRO_COOP_DP_SEND_EVERY_FRAME=
set SEKIRO_COOP_DP_SMOOTH=1
if /I "%~1"=="nosmooth" set SEKIRO_COOP_DP_SMOOTH=
set SEKIRO_COOP_DP_LEAD=
if /I "%~1"=="lead" set SEKIRO_COOP_DP_LEAD=1.2
set SEKIRO_COOP_DP_TRACE=
set SEKIRO_COOP_BOT=1
set SEKIRO_COOP_BOT_AUTOLOAD=1
set SEKIRO_COOP_CLONE_OWN_PGD=2
set SEKIRO_COOP_ENEMY_PROBE=
set SEKIRO_COOP_ENEMY_PUPPET=
set SEKIRO_COOP_ENEMY_MIRROR=
set SEKIRO_COOP_ENEMY_REPLAY=
set SEKIRO_COOP_ENEMY_SYNC=
rem Test knob: sleep this many ms once a frame while the bot runs (14 gives ~38 fps).
rem e.g. set BOT_FRAME_SLEEP_MS=14 before running for ~38 fps
set SEKIRO_COOP_BOT_FRAME_SLEEP_MS=%BOT_FRAME_SLEEP_MS%
set SEKIRO_COOP_DP_T_GATE=
set SEKIRO_COOP_DP_SMOOTH_MS=
set SEKIRO_COOP_CLONE_PATCH=
set SEKIRO_COOP_PUPPET_PHASES=
if not "%~1"=="" if not "%~1"=="-" set SEKIRO_COOP_PUPPET_WARP_MS=%~1

echo Launching Sekiro (test copy, save coop-test.sl2)...
rem Close the game a few seconds after the bot reports (or after 6 minutes).
start "bot-watch" /MIN powershell -NoProfile -Command "$log=Join-Path $env:LOCALAPPDATA 'sekiro-coop\sekiro-coop.log'; $t0=Get-Date; while(((Get-Date)-$t0).TotalMinutes -lt 6){ if((Test-Path $log) -and (Select-String -Path $log -Pattern 'bot: done' -Quiet)){ Start-Sleep 3; break }; Start-Sleep 2 }; Stop-Process -Name sekiro -Force -ErrorAction SilentlyContinue"
"%ME3%" launch -g sekiro -e D:\SekiroCoop\sekiro.exe --disable-arxan true --online false --savefile coop-test.sl2 -p "%RUN%\sekiro-coop.me3"

copy /Y "%LOCALAPPDATA%\sekiro-coop\sekiro-coop.log" "%RUN%\sekiro-coop.log" >nul 2>&1
taskkill /F /IM peer-simulator.exe >nul 2>&1
echo Done. Game closed, simulator stopped.
