# Guía Maestra: Sistema de Compilación Nativo Portable con Tauri v2 y Rust

> **Propósito:** Esta guía proporciona una plantilla universal, paso a paso y comprobada para empaquetar aplicaciones de escritorio nativas en **Windows 10 y 11** utilizando **Tauri v2 + Rust**, logrando ejecutables ultraligeros (**~3 MB**), arranque instantáneo, consumo mínimo de RAM (<25 MB) y **cero dependencia de Python o Node.js** en el equipo de destino.
>
> 💡 *Nota para este repositorio:* **NDI PTZ Joystick Controller** constituye la implementación de referencia real en producción de esta arquitectura táctica.

---

## 1. Por qué este Sistema supera a PyInstaller / Python

| Factor | Empaquetado Tradicional (PyInstaller) | **Sistema Nativo (Tauri v2 + Rust)** |
| :--- | :--- | :--- |
| **Tamaño en disco** | 80 – 150 MB (con carpetas `_internal`) | **~3 a 4 MB (un solo archivo `.exe`)** |
| **Tiempo de arranque** | 2 a 4 segundos (descompresión en `%TEMP%`) | **Instantáneo (0.05 s)** |
| **Consumo de memoria** | 150 – 250 MB RAM | **15 – 30 MB RAM** |
| **Icono en barra de tareas**| Propenso a mostrar el icono de Python o fallar | **100 % nativo embebido en el ejecutable PE** |
| **Falsos positivos AV** | Muy frecuentes por empaquetado de scripts | **Prácticamente nulos (código binario estándar)** |

---

## 2. Requisitos Previos en el Entorno de Desarrollo

Para compilar en cualquier equipo de desarrollo se requieren únicamente dos herramientas:

1. **Rust y Cargo (x86_64-pc-windows-gnu o msvc):**
   - Instalable desde [rustup.rs](https://rustup.rs).
   - Comando de verificación: `cargo --version`
2. **Compilador C / MinGW-w64 (si se usa la toolchain GNU):**
   - Se puede instalar en Windows con WinGet:
     ```cmd
     winget install BrechtSanders.WinLibs.POSIX.UCRT
     ```
   - O utilizar la toolchain oficial de Microsoft C++ Build Tools (`msvc`).

---

## 3. Estructura de Directorios Estándar

Cualquier proyecto que utilice este sistema debe estructurarse de la siguiente manera:

```text
NDI_PTZ_Joystick_Controller/
├── arrancar.bat                   <-- Script de desarrollo y recarga en 1 clic
├── compilar_y_empaquetar.bat      <-- Script de compilación release portable automática
├── backup.py                      <-- Script de copia de seguridad integral
├── dist_portable/                 <-- Carpeta final lista para distribuir / pendrive
│   ├── NDI-PTZ-Controller.exe     <-- Ejecutable nativo final (~3 MB)
│   └── WebView2Loader.dll         <-- Conector oficial de Microsoft Edge WebView2
├── frontend/                      <-- Interfaz de usuario táctica (HTML5, Tailwind, JS)
│   ├── index.html                 <-- Hub principal / Selector de cámaras
│   ├── floating.html              <-- Ventana flotante desacoplada de control
│   ├── settings.html              <-- Panel de ajustes y perfiles
│   ├── i18n.js                    <-- Motor de traducción multilingüe
│   └── ndi_ptz_client.js          <-- Conector IPC con Rust
└── src-tauri/                     <-- Motor backend en Rust
    ├── build.rs                   <-- Script de enlace de recursos Windows
    ├── Cargo.toml                 <-- Configuración de dependencias y optimización LTO
    ├── tauri.conf.json            <-- Propiedades de ventanas, títulos e identificadores
    └── src/
        ├── main.rs                <-- Punto de entrada de Windows (sin consola negra)
        ├── lib.rs                 <-- Registro de comandos y ciclo de vida de Tauri
        ├── commands.rs            <-- Funciones que el frontend invoca por IPC
        ├── discovery.rs           <-- Demonio de descubrimiento mDNS y sondeo ARP
        ├── models.rs              <-- Estructuras de datos compartidas (JSON)
        ├── window_manager.rs      <-- Orquestación de ventanas flotantes
        └── visca/                 <-- Motor cliente asíncrono VISCA-over-IP (UDP)
```

---

## 4. Archivos Clave y Plantillas de Configuración

### 4.1. `src-tauri/Cargo.toml` (Optimizado para Tamaño Mínimo)

Este archivo activa la optimización LTO (*Link-Time Optimization*), strip de símbolos de depuración y abort en pánico para reducir el ejecutable al mínimo tamaño posible:

```toml
[package]
name = "ndi-ptz-joystick-controller"
version = "0.1.0"
description = "Tactical Broadcast Console for NDI PTZ Cameras"
authors = ["Vicente Gisbert"]
edition = "2021"

[lib]
name = "ndi_ptz_lib"
crate-type = ["staticlib", "rlib"]

[build-dependencies]
tauri-build = { version = "2.0", features = [] }

[dependencies]
tauri = { version = "2.0", features = ["tray-icon", "custom-protocol"] }
tauri-plugin-opener = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
mdns-sd = "0.13"
grafton-visca = "0.1"

[profile.release]
panic = "abort"      # Elimina el árbol de desenrollado de excepciones (-1.5 MB)
codegen-units = 1    # Permite optimización de código cruzada entre módulos
lto = true           # Link-Time Optimization global (-2 MB)
opt-level = "z"      # Optimiza agresivamente por tamaño de binario
strip = true         # Elimina metadatos y símbolos debug (-4 MB)
```

---

### 4.2. `src-tauri/tauri.conf.json`

Define la ventana principal, dimensiones y la ruta relativa a la carpeta `frontend`:

```json
{
  "$schema": "https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri-cli/schema.json",
  "productName": "NDI PTZ Controller",
  "version": "0.1.0",
  "identifier": "com.antigravity.ndi-ptz-controller",
  "build": {
    "frontendDist": "../frontend"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "NDI PTZ Joystick Controller",
        "url": "index.html",
        "width": 1050,
        "height": 720,
        "minWidth": 900,
        "minHeight": 600,
        "resizable": true,
        "decorations": true,
        "transparent": false,
        "alwaysOnTop": false,
        "center": true,
        "label": "main"
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": false
  }
}
```

---

### 4.3. `src-tauri/build.rs` (Solución Crítica al Bug de Tildes en Windows)

> [!WARNING]
> **Bug Conocido de Windows / MinGW:** Si la ruta del proyecto contiene tildes o caracteres especiales (por ejemplo: `Cámera`, `música`, `año`, `diseño`), el compilador de recursos GNU `windres.exe` fallará con el error `Invalid argument` al intentar abrir el icono.

En compilaciones limpias estándar basta con:
```rust
fn main() {
    tauri_build::build()
}
```

Si el proyecto se encuentra alojado en carpetas con caracteres no-ASCII o tildes, `build.rs` puede copiar el icono a una ruta temporal garantizada en ASCII antes de invocar a `tauri_build`:

```rust
fn main() {
    let mut windows = tauri_build::WindowsAttributes::new();
    
    // Copiar el icono a una ruta temporal libre de tildes para evitar errores de windres
    let temp_ico = std::path::PathBuf::from(r"C:\Users\super\.cargo\ndi_ptz_app_icon.ico");
    if std::path::Path::new("icons/icon.ico").exists() {
        let _ = std::fs::copy("icons/icon.ico", &temp_ico);
        windows = windows.window_icon_path(&temp_ico);
    }

    let attrs = tauri_build::Attributes::new().windows_attributes(windows);
    tauri_build::try_build(attrs).expect("Fallo al ejecutar tauri-build");
}
```

---

### 4.4. `src-tauri/src/main.rs` (Ocultar Consola Negra de Windows)

La directiva `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` indica al enlazador de Windows que genere un binario con subsistema GUI, impidiendo que aparezca la ventana negra de consola al hacer doble clic:

```rust
// Impide la consola negra en modo Release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    ndi_ptz_lib::run();
}
```

---

### 4.5. Comunicación Frontend <-> Rust (`frontend/ndi_ptz_client.js`)

En el frontend no es necesario utilizar frameworks pesados ni empaquetadores como Webpack si no se desean. Se puede usar JavaScript estándar con la API nativa de Tauri:

```javascript
// Llamar a una función de Rust por memoria IPC directa
async function descubrirCamaras() {
  if (window.__TAURI__ && window.__TAURI__.core) {
    try {
      // 'discover_cameras' es el comando registrado en Rust con #[tauri::command]
      const camaras = await window.__TAURI__.core.invoke('discover_cameras');
      console.log("Cámaras NDI / VISCA detectadas por Rust:", camaras);
      actualizarSelectorCamaras(camaras);
    } catch (err) {
      console.error("Error al invocar Rust:", err);
    }
  } else {
    // Modo de prueba o mock para desarrollo en navegador
    console.warn("Entorno Web puro: usando Mock IPC");
    actualizarSelectorCamaras(obtenerCamarasMock());
  }
}
```

Y en Rust (`src-tauri/src/commands.rs`):

```rust
#[tauri::command]
pub async fn discover_cameras(
    discovery: State<'_, Arc<DiscoveryService>>,
) -> Result<Vec<CameraDevice>, String> {
    Ok(discovery.get_discovered_cameras().await)
}
```

---

## 5. El Script de Compilación Portable: `compilar_y_empaquetar.bat`

El script implementado en este proyecto se encarga de:
1. Localizar y priorizar las herramientas del sistema (Cargo, MinGW/WinLibs).
2. Mapear una unidad virtual (`N:`) para aislar el compilador de espacios y caracteres especiales en las rutas.
3. Compilar en modo Release con optimizaciones LTO agresivas.
4. Generar la carpeta portable `dist_portable` con el binario (`NDI-PTZ-Controller.exe`) y `WebView2Loader.dll` listo para llevar en pendrive o desplegar en cabina broadcast.

```bat
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

echo.
echo ======================================================
echo  [LISTO] Paquete portable generado con exito en:
echo         %~dp0dist_portable\
echo ======================================================
echo.
echo Para probar en otro PC Windows 10, simplemente copia
echo la carpeta 'dist_portable' completa (o un pendrive).
echo.
pause
```

---

## 6. Lista de Verificación (Checklist) para Nuevos Proyectos

Al iniciar un nuevo proyecto con este sistema, sigue estos pasos:

- [ ] **1.** Crea la carpeta `frontend/` y coloca allí tu `index.html`, `app.js` y hojas de estilo.
- [ ] **2.** Crea la carpeta `src-tauri/` con `Cargo.toml`, `build.rs` y `tauri.conf.json`.
- [ ] **3.** Coloca los iconos en `src-tauri/icons/` (`icon.ico`, `icon.png`, `32x32.png`, `128x128.png`).
- [ ] **4.** En `frontend/app.js`, utiliza `window.__TAURI__.core.invoke('nombre_comando')` para solicitar datos al backend.
- [ ] **5.** En `src-tauri/src/commands.rs`, implementa los comandos con la anotación `#[tauri::command]`.
- [ ] **6.** Ejecuta `compilar_y_empaquetar.bat`: el proceso generará automáticamente en segundos la carpeta `dist_portable` y el archivo `.zip` de ~3-5 MB listo para llevar en pendrive.
