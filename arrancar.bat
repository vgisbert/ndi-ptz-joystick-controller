@echo off
title NDI PTZ Joystick Controller - Launcher
echo ======================================================
echo    NDI PTZ Joystick Controller - Broadcast Suite
echo    Hardware Target: 2x Nuroum V403 (VISCA over IP)
echo ======================================================
echo.

set CARGO_PATH=%USERPROFILE%\.cargo\bin
set WINGET_MINGW=%LOCALAPPDATA%\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\bin

if exist "%WINGET_MINGW%" (
    set "PATH=%WINGET_MINGW%;%PATH%"
)
if exist "%CARGO_PATH%" (
    set "PATH=%CARGO_PATH%;%PATH%"
)

where cargo >nul 2>&1
if errorlevel 1 (
    echo [ERROR] No se encontro Cargo/Rust en el sistema ni en %CARGO_PATH%.
    echo Asegurate de que Rustup este instalado.
    pause
    exit /b 1
)

echo [1/2] Verificando compilador Rust y toolchain...
cargo --version
rustc --version
echo.

echo [2/2] Preparando entorno de compilacion y arranque...
:: Cerrar cualquier instancia previa para evitar bloqueo del ejecutable al compilar
taskkill /F /IM ndi-ptz-joystick-controller.exe >nul 2>&1

:: Mapear unidad virtual N: para garantizar rutas sin espacios para windres/MinGW
subst N: /d >nul 2>&1
subst N: "%~dp0." >nul 2>&1

if exist "N:\src-tauri" (
    cd /d "N:\src-tauri"
) else (
    cd /d "%~dp0src-tauri"
)

cargo run

:: Limpiar mapeo virtual al salir
cd /d "%~dp0"
subst N: /d >nul 2>&1

if errorlevel 1 (
    echo.
    echo [ERROR] Se produjo un error al ejecutar la aplicacion.
    pause
    exit /b 1
)

pause
exit /b 0
