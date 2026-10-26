@echo off
title NDI PTZ Joystick Controller - Compilador Portable Release
echo ======================================================
echo    Compilando Release Portable para Windows 10
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
    echo [ERROR] No se encontro Cargo/Rust en el sistema.
    pause
    exit /b 1
)

:: Mapear unidad virtual N: para garantizar rutas sin espacios
subst N: /d >nul 2>&1
subst N: "%~dp0." >nul 2>&1

if exist "N:\src-tauri" (
    cd /d "N:\src-tauri"
) else (
    cd /d "%~dp0src-tauri"
)

echo [1/2] Compilando binario optimizado (Release)...
set RUSTFLAGS=-A linker-messages
cargo build --release

if errorlevel 1 (
    echo [ERROR] Error durante la compilacion.
    cd /d "%~dp0"
    subst N: /d >nul 2>&1
    pause
    exit /b 1
)

cd /d "%~dp0"
subst N: /d >nul 2>&1

echo.
echo [2/2] Empaquetando en la carpeta 'dist_portable'...
if not exist "dist_portable" mkdir "dist_portable"

copy /Y "src-tauri\target\release\ndi-ptz-joystick-controller.exe" "dist_portable\NDI-PTZ-Controller.exe" >nul
copy /Y "src-tauri\target\release\WebView2Loader.dll" "dist_portable\WebView2Loader.dll" >nul

:: Limpiar archivos ZIP previos para evitar anidamiento circular
if exist "dist_portable\NDI-PTZ-Controller-Windows-Portable.zip" del /F /Q "dist_portable\NDI-PTZ-Controller-Windows-Portable.zip" >nul 2>&1
if exist "NDI-PTZ-Controller-Windows-Portable.zip" del /F /Q "NDI-PTZ-Controller-Windows-Portable.zip" >nul 2>&1

echo Generando archivo ZIP comprimido dentro de 'dist_portable'...
powershell -NoProfile -Command "Compress-Archive -Path 'dist_portable\*' -DestinationPath 'dist_portable\NDI-PTZ-Controller-Windows-Portable.zip' -Force"

echo.
echo ======================================================
echo  [LISTO] Paquete portable generado con exito:
echo  - Carpeta: %~dp0dist_portable\
echo  - Archivo ZIP: %~dp0dist_portable\NDI-PTZ-Controller-Windows-Portable.zip
echo ======================================================
echo.
echo Para probar en otro PC Windows 10, simplemente copia
echo el archivo ZIP o la carpeta 'dist_portable' a un pendrive.
echo.
pause
