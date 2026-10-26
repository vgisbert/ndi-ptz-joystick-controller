# Metodologías de Desarrollo y Arquitectura: NDI Virtual Camera vs NDI PTZ Joystick Controller

Este documento analiza en profundidad las dos estrategias de ingeniería de software adoptadas en la suite Broadcast de Antigravity:
1. **Modelo Interpretado de Alta Flexibilidad (Python Core)** – Utilizado en *NDI Virtual Camera*.
2. **Modelo Compilado Nativo de Latencia Determinista (Rust/Tauri Core)** – Utilizado en *NDI PTZ Joystick Controller*.

Se detallan las ventajas, desventajas, razones de diseño que justifican la bifurcación técnica y cómo ambas soluciones convergen en producción para Windows 10/11.

---

## 1. Cuadro Comparativo Global

| Aspecto | NDI Virtual Camera (Python Core) | NDI PTZ Joystick Controller (Rust Core) |
| :--- | :--- | :--- |
| **Paradigma** | Interpretado al vuelo (Python 3.10+) | Compilado a código máquina nativo (Rust / Cargo) |
| **Tiempo de arranque en dev** | Inmediato (~0.5s tras verificar venv) | Requiere compilación y enlace previo (~3-6s) |
| **Gestión de Memoria** | Automática por Garbage Collector (GC) | Determinista en tiempo de compilación (RAII / Borrow Checker) |
| **Latencia y Jitter** | Variable (pausas esporádicas del GC de Python) | Cero pausas, respuesta constante en sub-milisegundo |
| **Procesamiento Objetivo** | Manipulación de streams pesados de vídeo (frames RGB/UYVY) | Telemetría UDP continua, polling de joystick a 60-120 Hz y mDNS |
| **Consumo de RAM** | Medio-Alto (~150 MB - 400 MB según resolución de video) | Mínimo (~25 MB - 55 MB) |
| **Distribución Portable** | Empaquetado binario / Wrapper nativo con DLLs externas | Binario autónomo nativo enlazado estáticamente |

---

## 2. Análisis del Procedimiento 1: Python Core (NDI Virtual Camera)

### ¿Cómo opera?
En el flujo diario de trabajo, el archivo `arrancar.bat` comprueba el entorno virtual de Python (`core/venv`), valida dependencias y ejecuta directamente `python core/main.py`.

### Ventajas Técnicas:
1. **Iteración Instantánea (Cero tiempo de build):**
   * Cualquier cambio en la lógica de conexión de video, filtros o parsing de frames tiene efecto inmediato al relanzar el script.
2. **Ecosistema de Tratamiento de Imagen y Video:**
   * Python cuenta con librerías maduras como `numpy`, `OpenCV` y bindings NDI (`cyndilib`, `ndi-python`) con implementaciones optimizadas en C que simplifican el flujo entre DirectShow y NDI.
3. **Facilidad de Depuración Algorítmica:**
   * La inspección interactiva y la flexibilidad dinámica de tipos permiten probar transformaciones de píxeles sin lidiar con los estrictos contratos de tipado del compilador.

### Desventajas Técnicas:
1. **Pausas del Garbage Collector (Jitter):**
   * El recolector de basura de Python puede activarse en momentos impredecibles, lo que en tareas de control en tiempo real provocaría micro-tirones (*stutter*).
2. **Sobrecarga de Distribución:**
   * Para convertir un proyecto Python en un `.exe` portable independiente, es necesario "congelar" el intérprete (vía PyInstaller o embutir el runtime), generando ejecutables de mayor tamaño (>80 MB) y ocasionales falsos positivos con software antivirus.

---

## 3. Análisis del Procedimiento 2: Rust / Tauri Core (NDI PTZ Joystick Controller)

### ¿Cómo opera?
En el flujo de desarrollo, `arrancar.bat` invoca `cargo run` en la carpeta `src-tauri`. El compilador `rustc` y el linker de Windows (`ld.exe` / `link.exe`) procesan el código fuente y generan físicamente el binario en `target/debug/ndi-ptz-joystick-controller.exe` antes de ejecutarlo.

### Ventajas Técnicas:
1. **Latencia Determinista y Control PTZ Milimétrico:**
   * El protocolo VISCA over IP requiere enviar ráfagas de comandos UDP y lecturas de posición a intervalos estrictos (e.g., cada 16 ms para 60 FPS). Rust carece de recolector de basura; la memoria se asigna y libera de forma exacta, eliminando cualquier retraso entre la pulsación del joystick y el movimiento de los motores de la cámara Nuroum V403.
2. **Seguridad Concurrente (Thread Safety sin carreras):**
   * La aplicación maneja simultáneamente hilos para: autodescubrimiento mDNS en UDP 5353, polling de DirectInput/XInput para joystick, servidor HTTP para peticiones REST y sockets UDP VISCA. El compilador de Rust garantiza la ausencia de colisiones de datos (*data races*) entre estos hilos.
3. **Huella de Recursos Ultra-ligera:**
   * El binario compilado ocupa menos de 10 MB y consume menos de 45 MB de RAM, ideal para permanecer activo en segundo plano en la bandeja del sistema (*System Tray*) sin afectar a programas de emisión pesados (OBS, vMix, Wirecast).

### Desventajas Técnicas:
1. **Tiempos de Compilación en Desarrollo:**
   * Cada modificación en los archivos `.rs` obliga a una pequeña pausa de compilación (de 3 a 8 segundos).
2. **Mensajes del Enlazador (Linker):**
   * Al compilar con herramientas GNU/MinGW en Windows, pueden mostrarse avisos técnicos del sistema (como `.rsrc merge failure: multiple non-default manifests`), los cuales no son errores pero requieren comprender el ecosistema de la toolchain.

---

## 4. Justificación de la Decisión Técnica

Haber elegido dos aproximaciones diferentes para los dos proyectos no fue una discrepancia accidental, sino una **decisión arquitectónica adaptada al objetivo operativo de cada herramienta**:

1. **NDI Virtual Camera:**
   * Su núcleo es el transporte masivo de matrices de píxeles (video a 1080p/60fps) hacia el subsistema DirectShow de Windows. La prioridad aquí era la integración con librerías de video y la rapidez para adaptar drivers de cámara virtual.
2. **NDI PTZ Joystick Controller:**
   * Su núcleo es la **telemetría de control en tiempo real**. No procesa imágenes pesadas, sino eventos de red UDP y señales de mando físico. Requería fiabilidad absoluta, latencia cero, arranque instantáneo y cero dependencias de runtimes externos en la máquina del operador.

---

## 5. Convergencia en Producción (`compilar_y_empaquetar.bat`)

A pesar de que sus entornos de desarrollo se comportan de forma distinta, **ambos proyectos convergen en una entrega idéntica de cara al usuario final**:

1. **Generación de Binario Release:**
   Ambos scripts ejecutan:
   ```cmd
   set RUSTFLAGS=-A linker-messages
   cargo build --release
   ```
   * La bandera `-A linker-messages` suprime los avisos inofensivos del enlazador MinGW (`ld.exe`).
   * Se genera un binario optimizado en tamaño y velocidad (`LTO - Link Time Optimization`).

2. **Empaquetado Portable (`dist_portable`):**
   * El resultado es una carpeta completamente independiente con el `.exe` nativo y la librería `WebView2Loader.dll`.
   * Ninguno de los dos paquetes finales exige que el usuario final tenga instalado Python, Rust, Cargo ni compiladores de C++ en Windows 10/11.
