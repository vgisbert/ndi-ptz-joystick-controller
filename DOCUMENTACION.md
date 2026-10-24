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

Para asegurar que la aplicación sea **100% portable**:
1. **Modo Portable Automático:** La aplicación comprueba si existe un archivo `portable.flag` o `config.json` en su propio directorio de ejecución. De ser así, guarda todos los perfiles, calibraciones de joystick y nombres de presets en la misma carpeta, sin escribir en `%APPDATA%` ni en el registro de Windows.
2. **Exportación e Importación de Perfiles (.json):** Permite respaldar y cargar configuraciones completas de escena (ej. `nuroum_escenario_A.json`).
3. **Escritura Directa en NVRAM:** Opción de sincronizar los nombres y presets en la memoria no volátil de las cámaras Nuroum V403 para que persistan a reinicios físicos del equipamiento.

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



