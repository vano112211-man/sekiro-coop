@echo off
rem Builds the friend kit: D:\dev\coop-kit\sekiro-coop-friend.zip (DLL, me3 profile, launcher, README). No game files.
setlocal
set SRC=D:\dev\sekiro-coop
set RUN=D:\dev\coop-run
set OUT=D:\dev\coop-kit\sekiro-coop-friend

pushd %SRC%
set MINHOOK_LIB_DIR=D:\dev\minhook-build
cargo build --release -p sekiro-coop-dll
if errorlevel 1 (popd & echo BUILD FAILED - send the error above to Claude & exit /b 1)
popd

if exist "%OUT%" rmdir /S /Q "%OUT%"
mkdir "%OUT%\sekiro-coop-mods" || exit /b 1
copy /Y "%SRC%\target\release\sekiro_coop.dll" "%OUT%\sekiro-coop-mods\" >nul || exit /b 1
copy /Y "%RUN%\sekiro-coop.me3" "%OUT%\" >nul || exit /b 1
copy /Y "%RUN%\kit\play-steam.cmd" "%OUT%\" >nul || exit /b 1
copy /Y "%RUN%\kit\README.txt" "%OUT%\" >nul || exit /b 1
del "%OUT%.zip" 2>nul
powershell -NoProfile -Command "Compress-Archive -Path '%OUT%\*' -DestinationPath '%OUT%.zip'"
if errorlevel 1 (echo ZIP FAILED & exit /b 1)
certutil -hashfile "%OUT%.zip" SHA256
echo Kit ready: %OUT%.zip  (check it, then send it to your friend yourself)
