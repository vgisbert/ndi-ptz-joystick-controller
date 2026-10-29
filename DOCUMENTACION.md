# NDI PTZ Joystick Controller — Documentación Técnica

## 1. Visión General del Proyecto

**NDI PTZ Joystick Controller** es una aplicación de escritorio de alto rendimiento y grado broadcast para Windows, diseñada para el control simultáneo e independiente de cámaras PTZ que operan bajo el protocolo NDI y control IP.

La aplicación opera bajo un concepto de **demonio de autodescubrimiento en red** que localiza dinámicamente las cámaras presentes en la subred y permite desplegar **ventanas flotantes desacopladas e independientes** para cada dispositivo.

> [!NOTE]
> **Alcance del sistema:** Este software está especializado y optimizado exclusivamente para el **control cinemático, telemetría y gestión de presets PTZ**. La captura, conmutación y visualización de vídeo NDI se delega a aplicaciones independientes que se desarrollarán en fases posteriores, garantizando una latencia mínima de control y un consumo de recursos ultra-bajo.

---

## 2. Hardware Objetivo: Nuroum V403 PTZ (2 Unidades)

El despliegue inicial y calibración se realiza con **2 unidades Nuroum V403 PTZ** detectadas y verificadas activas en la red local:

- **Unidades de Referencia en Red (Ejemplo de Configuración):**
  - **CAM-01:** `192.168.1.101` | MAC: `98:6E:E8:XX:XX:01` | Puerto: `UDP 1259` | Estado: **ACTIVA / ON**
  - **CAM-02:** `192.168.1.102` | MAC: `98:6E:E8:XX:XX:02` | Puerto: `UDP 1259` | Estado: **ACTIVA / ON**
- **Tipo de dispositivo:** Cámara PTZ para conferencias y transmisiones profesionales con soporte NDI|HX.
- **Protocolo de Control:** VISCA over IP (UDP / TCP).
- **Puertos de control confirmados:**
  - `UDP 1259` (puerto por defecto VISCA-over-IP activo y verificado en ambas unidades).
  - `UDP 5353` (anuncios de servicio mDNS/Bonjour).
  - `TCP 80` / `HTTP 8080` (interfaz web de configuración interna).
- **Mecanismo de Autodescubrimiento:**
  - Anuncios mDNS/Bonjour en puerto `UDP 5353` (`_ndi._tcp.local`, `_ndi-hx._tcp.local` o servicios VISCA/HTTP).
  - Escaneo dirigido / Asignación manual de IP para entornos con filtrado multicast en routers/switches.
- **Rango y Dinámica de Movimiento:**
  - **Panorámica (Pan):** $\pm 170^\circ$ (resolución precisa de pasos de motor, velocidad $0x01$ a $0x18$).
  - **Inclinación (Tilt):** $-30^\circ$ a $+90^\circ$ (velocidad $0x01$ a $0x14$).
  - **Zoom Óptico:** Zoom variable continuo (Tele / Wide) con reporte de posición.
- **Bancos de Memoria (Presets):**
  - Mínimo 8 presets directos por cámara accesibles mediante macros (GOTO 01 a 08).
  - Almacenamiento directo en la memoria NVRAM de la propia cámara Nuroum V403.
- **Requisito Operativo Fundamental (Wake on Stream):**
  - La etapa de potencia de los motores paso a paso de la cámara solo se energiza cuando existe un receptor activo consumiendo vídeo (OBS Studio vía NDI, NDI Studio Monitor, vMix o RTSP). Si no hay recepción de vídeo activa, la cámara entra en modo de reposo/privacidad (`[privacy] enable = 1`) y no moverá físicamente los ejes aunque el demonio de red responda con `ACK` a los comandos VISCA.

---

## 3. Arquitectura del Software (Tauri v2 + Rust)

El proyecto adopta la arquitectura **Tauri 2.0 con backend en Rust**, seleccionada para cumplir con la premisa de **ejecutable portable sin dependencias en el equipo destino**.

```
┌─────────────────────────────────────────────────────────────┐
│                 WINDOWS WORKSTATION (OS)                    │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                 TAURI RUNTIME (Rust)                │   │
│   │                                                     │   │
│   │  ┌─────────────────┐       ┌─────────────────────┐  │   │
│   │  │  mDNS Discovery │       │   VISCA-over-IP     │  │   │
│   │  │  Daemon (5353)  │       │   Engine (UDP/Winsock│  │   │
│   │  └────────┬────────┘       └──────────┬──────────┘  │   │
│   │           │                           │             │   │
│   │           └─────────────┬─────────────┘             │   │
│   │                         │ Event Bus / IPC           │   │
│   └─────────────────────────┼───────────────────────────┘   │
│                             │                               │
│          ┌──────────────────┴──────────────────┐            │
│          ▼                                     ▼            │
│   ┌───────────────────────┐         ┌────────────────────┐  │
│   │ Floating Window CAM 1 │         │ Floating Window 2  │  │
│   │ (Nuroum V403 #1)      │         │ (Nuroum V403 #2)   │  │
│   │ [WebView2 - Isolated] │         │ [WebView2-Isolated]│  │
│   └───────────────────────┘         └────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                               ▲
                        LAN / UDP (PoE)
                               ▼
        ┌────────────────────────┐  ┌────────────────────────┐
        │  Nuroum V403 (CAM-01)  │  │  Nuroum V403 (CAM-02)  │
        │  192.168.1.101:1259    │  │  192.168.1.102:1259    │
        └────────────────────────┘  └────────────────────────┘
```

### 3.1. Modelo Multiventana Modular y Escalable (1 Diseño para N Cámaras)

Para optimizar el uso del espacio de trabajo en configuraciones multimonitor y permitir que la solución escale sin duplicar código, la interfaz se divide en dos tipos de ventanas flotantes nativas:

1. **Ventana Flotante del Demonio (Hub / Monitor Global):**
   - Ventana compacta e independiente con estado persistente del sistema de red.
   - Indicador de estado general (**`SISTEMA NOMINAL // TODO OK`** en verde/cian).
   - Métricas de escucha mDNS en puerto `UDP 5353`.
   - Listado en vivo de cámaras descubiertas con sus IPs (`192.168.1.101`, `192.168.1.102`), latencia en milisegundos y botón de despliegue/fijado de su ventana flotante correspondiente.

2. **Ventana Flotante Parametrizada por Cámara (Instancia por Dispositivo):**
   - Una ventana flotante independiente y desacoplada por cada cámara detectada (`cam_id`, `ip`, `port`).
   - Reutiliza exactamente la misma plantilla de interfaz y lógica de control para **$N$ cámaras** (sirve tanto para 2 cámaras Nuroum V403 como para 8 o más).
   - **Comportamiento en Pérdida de Conexión (Modo Grayscale):**
     - Si la cámara deja de responder al heartbeat VISCA (> 2.5s), la ventana activa automáticamente una transición CSS a **escala de grises completa (`filter: grayscale(100%) opacity(60%)`)**.
     - Se superpone un distintivo de advertencia táctico (**`[ENLACE PERDIDO - REINTENTANDO...]`**) y se bloquean las entradas del joystick y botones de presets para evitar desbordamiento de búfer.
     - En el instante en que el cable o la cámara se restablecen, la ventana recupera instantáneamente su color cian/ámbar y su operatividad al 100%.

### 3.2. Ventajas Clave de la Arquitectura
1. **Ejecutable Único y Ultra-ligero:** Binario portable PE final de aproximadamente **12 a 15 MB**.
2. **Cero Dependencias Externas:**
   - Aprovecha el componente nativo **Microsoft Edge WebView2 Evergreen**, preinstalado en el 100% de sistemas Windows 10 y Windows 11 modernos.
   - No requiere instalar Node.js, Python ni runtimes de C++ en la máquina cliente.
3. **Multi-Ventana Nativa Real:** Tauri permite crear ventanas flotantes independientes del sistema operativo con control total de:
   - Posicionamiento, arrastre y fijación en pantalla (*Always On Top* / `push_pin`).
   - Dimensionado mínimo y memoria de coordenadas en pantalla al reiniciar la app.
4. **Control en Tiempo Real con Rust:**
   - La capa de red de Rust envía paquetes UDP VISCA a alta frecuencia (bucle de 60-100 Hz) con amortiguación de jitter y sin pausas de recolección de basura (*Garbage Collector*).

---

## 4. Sistema de Diseño: Tactical Broadcast Console

La interfaz visual reproduce fielmente el diseño industrial documentado en `STITCH/tactical_broadcast_console/DESIGN.md`:

- **Chasis y Superficies:**
  - Fondo de consola: `#111319`
  - Chasis de módulo: `#191c21` / `#1d2025`
  - Receptáculos mecanizados de pantalla: `#0b0e13`
- **Acentos Emisivos y Tally:**
  - **Cian Precisión (`#00F0FF`):** Selección activa, retículas de joystick, telemetría y velocidad activa.
  - **Ámbar Táctico (`#FFB300`):** Presets activos, advertencias y modo Preview (PVW).
  - **Rojo Tally (`#FF1744`):** Cámara en emisión activa (PGM) y pulsación sostenida de memorización de preset (MEM).
- **Controles Especializados:**
  - **Cardán Virtual / Joystick Vectorial:** Respuesta polar con limitador de radio, cálculo de deflexión $X/Y$, zona muerta ajustable (*deadzone*) y tecla rápida `ESPACIO` para retorno al punto central.
  - **Modo Creep (25% Slew Rate):** Suavizado no lineal para paneos lentos cinematográficos.
  - **OLED Telemetry Display:** Bloques de visualización con tipografía monospace `JetBrains Mono` con cifras tabulares.
  - **Teclas de Presets con Doble Acción:** Botón GOTO (llamada inmediata) y botón MEM con timer de confirmación de 1 segundo para evitar reprogramaciones accidentales.

---

## 5. Estrategia de Portabilidad y Datos Locales

Para asegurar que la aplicación sea **100% portable y autónoma**, toda la información persistente se almacena localmente mediante la función canónica `get_app_base_dir()`:

1. **Resolución Automática del Directorio Base:**
   * **Modo Desarrollo (`arrancar.bat` / `cargo run`):** Los archivos de configuración se guardan en la misma carpeta raíz donde reside `arrancar.bat`.
   * **Modo Portable / Release (`.exe` empaquetado):** Toda la información se guarda en la misma carpeta donde esté ubicado el ejecutable `.exe` (ej. `dist_portable\` o pendrive USB).
2. **Archivos de Configuración Persistente:**
   * **`config.json`:** Archivo JSON legible y estructurado que almacena todos los perfiles de cámara (IPs fijas, nombres, puertos VISCA/NDI, velocidades de joystick, zoom y enfoque), los 9 nombres de presets personalizados por cámara, la cámara activa seleccionada (`activeCameraId`), el idioma (`es`/`en`) y los ajustes de inversión de controles (*Mirror*). Se sincroniza de forma reactiva y bidireccional entre UI y disco.
   * **`window_states.json`:** Almacena las geometrías de ventana multimonitor (coordenadas X/Y, dimensiones ancho/alto y estado siempre visible *Always on Top*).
3. **Aislamiento Total de WebView2 (`.data/`):**
   * Se redirige la variable de entorno de Windows `WEBVIEW2_USER_DATA_FOLDER` a la subcarpeta local `.data/` dentro del directorio base, impidiendo que el motor WebView2 escriba en `%LOCALAPPDATA%` y garantizando que el paquete pueda trasladarse entre equipos manteniendo todos los datos intactos.

---

## 6. Referencias Técnicas y Componentes de Código Abierto Integrados

Para optimizar los tiempos de desarrollo y maximizar la fiabilidad del sistema, se integran las siguientes soluciones y algoritmos de referencia:

### 6.1. Backend Rust
- **`grafton-visca` (`GrantSparks/grafton-visca`):**
  - Crate pura en Rust para VISCA-over-IP con soporte asíncrono Tokio.
  - Implementa serialización y deserialización binaria de paquetes VISCA (`Pan/Tilt`, `Zoom`, `Preset Recall/Store`, `Focus`), manejo de sockets UDP y verificación de respuestas ACK/Completion de la cámara.
- **`mdns-sd` (`openssl/tokio` compatible):**
  - Escáner de servicios multicast DNS en `UDP 5353` para detectar fuentes `_ndi._tcp.local` de forma autónoma, sin requerir la instalación de NewTek NDI Tools ni servicios externos de Bonjour en Windows.

### 6.2. Frontend & Cinemática de Control
- **Algoritmo de Curvas Exponenciales (Curvas EXPO):**
  - Derivado de las implementaciones de referencia de controladores PTZ (ej. `visca-ptz-web-controller`), permite mapear la deflexión analógica del joystick virtual con una función de aceleración no lineal:
    $$V_{out} = \text{sign}(D) \cdot |D|^{\gamma} \cdot V_{max}$$
    donde $\gamma \approx 1.8 - 2.2$. Esto confiere precisión quirúrgica en el centro para micro-ajustes y velocidad máxima ágil en deflexión completa.
- **Heartbeat & Resiliencia de Conexión:**
  - Patrón de keep-alive periódico y reconexión en background inspirado en `DanTappan/VISCA-Controller`, asegurando que cortes momentáneos de cable de red no bloqueen la interfaz de usuario ni las ventanas flotantes.

### 6.3. Solución a Rutas con Espacios en Compiladores GNU/Windows
- **Mapeo Virtual de Unidad (subst N:):**
  - La herramienta `windres.exe` de GNU MinGW falla cuando las rutas de preprocesamiento de recursos de Windows (`.rc`) contienen espacios en blanco. Para neutralizar este problema sin alterar las carpetas del usuario, el lanzador [`arrancar.bat`](file:///d:/Documentos/Antigravity/NDI%20PTZ%20Joystick%20Controller/arrancar.bat) mapea temporalmente la raíz del proyecto a una unidad virtual `N:\` durante la compilación y la desmonta al cerrar, asegurando compatibilidad universal.

---

## 7. Sistema de Internacionalización (i18n) e Idiomas Extensibles

La consola cuenta con un motor de internacionalización desacoplado y reactivo implementado en [`frontend/i18n.js`](file:///d:/Documentos/Antigravity/NDI%20PTZ%20Joystick%20Controller/frontend/i18n.js):

- **Idiomas Nativos Preconfigurados:**
  - **`SP` (Español):** Idioma por defecto de la aplicación.
  - **`EN` (Inglés):** Set completo traducido con terminología broadcast internacional.
- **Botón de Cambio Rápido de Idioma:**
  - Situado en la cabecera superior de todas las ventanas (`index.html`, `settings.html`, `floating.html`).
  - Muestra la insignia del idioma activo (ej. `SP` o `EN`). Al pulsarlo ciclea de manera instantánea entre los idiomas disponibles.
  - Sincronización multi-ventana en tiempo real mediante eventos `storage` de HTML5: al cambiar de idioma en una ventana, todas las demás ventanas abiertas se actualizan en vivo.
- **Cómo Añadir Nuevos Idiomas Editando un Fichero:**
  1. Abrir [`frontend/i18n.js`](file:///d:/Documentos/Antigravity/NDI%20PTZ%20Joystick%20Controller/frontend/i18n.js) con cualquier editor de texto plano (Bloc de notas, VS Code, etc.).
  2. En el objeto `I18N_LOCALES`, duplicar el bloque de `EN` o `SP`.
  3. Asignar el nuevo código de idioma como clave (por ejemplo `FR` para Francés, `DE` para Alemán, `PT` para Portugués, `IT` para Italiano).
  4. Traducir las cadenas de texto del bloque duplicado y guardar el archivo.
  5. El motor de i18n detectará automáticamente el nuevo idioma y lo incorporará en la rotación del botón superior de la barra de herramientas sin necesidad de recompilar el backend en Rust.

---

## 8. Escalamiento en Red Local a Más de 2 Cámaras (N > 2)

El sistema ha sido evaluado técnica y experimentalmente para escenarios donde operan 3, 4, 8 o más cámaras simultáneas en el mismo segmento de red:

### 8.1. Tráfico de Red: Control VISCA vs Vídeo NDI|HX

Existe una asimetría radical entre el tráfico de control cinemático y el tráfico de transporte audiovisual:

1. **Tráfico de Control PTZ (UDP 1259):**
   * El controlador en Rust implementa instancias dedicadas de `ViscaClient` en `ViscaManager` que únicamente transmiten paquetes de joystick (20 Hz) hacia la **cámara activa seleccionada**.
   * Las cámaras no seleccionadas consumen **0 kbps** durante la operación.
   * El demonio de supervisión periódica emite una única trama de 5 bytes cada 2 segundos por cámara online para verificar latencia (RTT). Con 8 cámaras, este consumo es de apenas **< 1 kbps**, completamente despreciable.
2. **Tráfico de Vídeo NDI|HX (TCP 5961):**
   * Cada cámara Nuroum V403 emite su flujo 1080p a unos **15 – 25 Mbps**.
   * **2 cámaras:** ~40 – 50 Mbps (asumible en redes Wi-Fi 5/6 o switches básicos de 100 Mbps).
   * **4 cámaras:** ~80 – 100 Mbps (hace mandatorio cableado Cat 5e/Cat 6 y switch Gigabit).
   * **8 cámaras:** ~160 – 200 Mbps de ancho de banda continuo hacia la estación receptora (OBS Studio / vMix).

### 8.2. Requisitos de Infraestructura de Red para N > 2 Cámaras
* **Switches Gestionados con IGMP Snooping:**
  * Al superar 2 cámaras emitiendo en la misma subred, es imperativo activar **IGMP Snooping** en el switch de red. Esto evita que los paquetes multicast de autodescubrimiento mDNS (`UDP 5353`) y los flujos NDI inunden todos los puertos físicos de la LAN (*broadcast storm*), aislando el tráfico hacia los equipos que efectivamente consumen la señal.
* **Segmentación de Subred / VLAN:**
  * Para instalaciones con 4 o más cámaras, se recomienda aislar los dispositivos de producción en una VLAN propia dedicada (p. ej. `192.168.9.0/24`) separada del tráfico ofimático o de invitados.

### 8.3. Persistencia, Concurrencia y Ergonomía Visual
* **Persistencia por IP (`localStorage`):**
  * La base de perfiles (`ndi_cam_profiles`) es un diccionario clave-valor sin límite de entradas indexado directamente por la IP de cada cámara.
* **Validación Anti-duplicados de Nombres:**
  * La función `isCameraNameDuplicate()` itera de forma reactiva sobre la totalidad del array `onlineCameras`. Ninguna cámara puede duplicar el nombre de ninguna otra en la red, evitando confusiones operativas en directo.
* **Interfaz de Cabecera:**
  * El contenedor de pestañas cuenta con desplazamiento horizontal nativo (`overflow-x-auto`). Con 2 cámaras entran holgadamente una al lado de la otra; para 4 o más unidades en producción, se recomienda el uso de atajos de teclado de conmutación rápida para alternar entre dispositivos sin necesidad de desplazar la barra con el ratón.

---

## 9. Registro de Mejoras y Optimizaciones Recientes

1. **Identificación 100% por IP:**
   * Eliminados todos los tecnicismos y dependencias visuales de direcciones MAC físicas en la interfaz y perfiles locales, alineando el controlador con el estándar de la industria (OBS, vMix, PTZOptics).
2. **Validación Anti-Duplicados en Tiempo Real:**
   * Detección instantánea en el campo de nombre de cámara (`cfg-cam-name`) con advertencia en rojo pulsante (`¡REPETIDO!`) y auto-recuperación al desenfocar.
3. **Secuencia de Centrado de Grado Broadcast (`send_ptz_home`):**
   * Inyección de comando `STOP` preventivo $\rightarrow$ retardo de 40 ms para estabilizar la máquina de estados de los servomotores $\rightarrow$ comando `HOME` $\rightarrow$ retardo de 40 ms $\rightarrow$ retransmisión `HOME` redundante contra pérdidas UDP.
4. **Autoenfoque Silencioso y Recalibración Rápida:**
   * Reaseguramiento automático de `Focus.Mode = "Auto"` mediante HTTP JSON-RPC (puerto 8090) en un hilo nativo independiente de runtime, y botón táctil `[ENFOQUE]` en configuración.
5. **Compactación Vertical de la Consola (312 px):**
   * Eliminación de las bandas y líneas vacías intermedias mediante empaquetado continuo (`justify-start gap-1`), retirada del pie redundante en ajustes con auto-guardado en el botón superior `VOLVER` y reducción de altura mínima a 250 px en `tauri.conf.json`.
6. **Protocolo de Limpieza Pre y Post Aplicación (Windows Job Objects):**
   * Asignación del proceso a un objeto de trabajo del kernel (`JobObject`) con la bandera `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, garantizando el exterminio automático de cualquier proceso hijo huérfano.
   * Limpieza preventiva de instancias previas zombi en el arranque (`main.rs`) antes de inicializar ventanas multimedia o sockets de red.
7. **Resiliencia de Sockets UDP y Reinicio Remoto Seguro (Reboot):**
   * Sockets UDP desacoplados que no colapsan con el error `WSAECONNRESET` (10054) cuando la cámara se reinicia físicamente.
   * Endpoint de reinicio remoto seguro vía API HTTP/JSON-RPC con modal de confirmación, cuenta atrás interactiva de 18 segundos y reactivación automática de control.
8. **Modo de Cámara y Rehabilitación de Nitidez:**
   * Indicador compacto de estado de captura (`MODO MANUAL` / `ÓPTIMO`) y botón de recalibración táctica de nitidez (`[ REHABILITAR NITIDEZ ]`).
9. **Telemetría en Vivo y Panel Unificado de Configuración:**
   * Monitorización de salud en una sola línea horizontal continua (`VISCA UDP`, `HTTP 8090`, `FPS REAL: 25` constante según PAL 50Hz anti-flicker).
   * Eliminación completa de duplicados de IP y modo manual en la interfaz, con altura optimizada a 465 px sin recortes.
10. **Búsqueda en Red Silenciosa y Estable (Solo Arranque y Ajustes):**
    * El proceso de escaneo en red (tabla ARP de Windows `arp -a` y barrido UDP 1259) se ejecuta únicamente en el arranque de la aplicación y mientras se permanece dentro de la ventana de configuración (`settings.html`).
    * En la ventana principal de operación (`index.html`), el escaneo programado en segundo plano queda completamente suspendido, evitando saturación de la cola ARP de Windows, latencias en los comandos VISCA y consumo innecesario de CPU y ancho de banda.
11. **Persistencia Local Canónica en Carpeta del Ejecutable y `arrancar.bat`:**
    * Implementación de la función `get_app_base_dir()` para unificar el guardado de `config.json`, `window_states.json` y la caché aislada de WebView2 (`.data/`) directamente en la carpeta del ejecutable o en la carpeta raíz de `arrancar.bat` en desarrollo.




