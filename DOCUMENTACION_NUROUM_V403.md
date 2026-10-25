# Documentación Técnica e Ingeniería Inversa: Cámara PTZ Nuroum V403 (Hamedal V403)

**Fecha de auditoría:** 23 de octubre de 2026  
**Modelo de hardware:** Nuroum V403 / Hamedal Cam V403 (`device_type: 033`)  
**Versión de firmware analizada:** `General_MC_[Hamedal_V403]_V1.100.1.6.R.240930.bin`  
**Unidades de referencia en red:** `192.168.1.101` (CÁM 1) y `192.168.1.102` (CÁM 2)

---

## 1. Arquitectura Interna de Hardware y Firmware

A partir del paquete de actualización oficial de NearSync (`General_MC_[Hamedal_V403]_V1.100.1.6.R.240930.bin`), se extrajo la imagen `updateAll.img` (cabecera `ZL-DVR` / `ZENOTECH`) y se descomprimieron los nodos LZO del sistema de archivos **UBIFS** embebido.

* **SoC y Sistema Operativo:** Rockchip ARM Linux embebido (con aceleración NPU RockX `person_detection_v2.data` para autoencuadre e IA).
* **Identificadores USB (cuando se conecta por cable):**
  * **Fabricante:** `rockchip` / `AuditoryWorks` (`Hamedal Cam V403`)
  * **VID / PID:** `0x0627` (`1575`) / `0xA6B5` (`42677`)
  * **Interfaces USB compuestas (`usb_config.sh`):** UVC (vídeo), UAC1 (audio) y **HID (`MI_04`, Usage Page `0x0081`)** empleada por la librería propietaria `axdp.dll` de **NearSync**.
* **Secuencia de arranque (`/user/etc/init.sh`):**
  1. Monta la partición persistente de configuración UBIFS en `/mnt/mtd` (`ubi1_0`, partición `mtd5` `config`) y la de respaldo en `/extern` (`ubi2_0`, partición `mtd6` `backup`).
  2. Si no existe `/mnt/mtd/Config/config.ini`, copia la plantilla por defecto `/user/cfg/defaultConfig.ini`.
  3. Descomprime `/user/lib.tgz` (que incluye `libndi.so.5.1.0`, `librkaiq.so`, `librockchip_mpp.so`) en `/var`.
  4. Inicia la red (`netinit`), el demonio `telnetd` y ejecuta `/usr/bin/ii`, que lanza el binario maestro de la cámara: **`/var/hunter`** (ejecutable ELF ARM de 32 bits, `5.36 MB`, compilado con símbolos de depuración).

---

## 2. Mapa de Puertos y Servicios de Red

El escaneo de puertos y el análisis de `/var/hunter` confirman los siguientes servicios activos en la IP de la cámara:

| Puerto | Protocolo | Servicio / Módulo en `/var/hunter` | Función |
| :--- | :--- | :--- | :--- |
| **`1259`** | **UDP** | `libCommProtocol` (`MEDIA_SerialProtocolInit`) | Control PTZ y Presets mediante **VISCA sobre IP** (configurable en `config.ini` -> `[networkCfg] udpPort = 1259`). |
| **`554`** | **TCP** | `RtspServer` (`ZLMediaKit`) | Flujo de vídeo RTSP directo: `rtsp://<IP>:554/live/mainStream`. |
| **`5960` / `5961`** | **TCP** | `libndi.so.5.1.0` (`mediaNdi.cpp`) | Transmisión de vídeo/audio NDI\|HX y recepción de órdenes PTZ/Presets NDI. |
| **`8090`** | **TCP** | `WebApi.cpp` / `HttpProtHandle` | Servidor HTTP interno con API JSON-RPC en `/index/api/remote`. |
| **`23`** | **TCP** | `telnetd` | Consola Linux embebida (protegida por hash MD5-crypt en `/etc/passwd`). |
| **`5353`** | **UDP** | `avahi-daemon` | Anuncio mDNS de flujos `_ndi._tcp.local.`. |

---

## 3. Comportamiento Exacto del Protocolo VISCA sobre IP (`UDP 1259`)

El módulo `commCtlProtocol.cpp` (`CtrlDeviceMotion`) junto con `MEDIA_SerialProtocolInit` presenta particularidades críticas que lo diferencian de una cámara Sony VISCA estándar:

### 3.1. Respuesta estática del socket UDP
Ante cualquier paquete enviado a `UDP 1259`, la cámara responde siempre con una ráfaga fija de 4 datagramas UDP:
1. `90 50 02 FF`
2. `90 50 00 FF`
3. `90 50 00 00 00 00 FF`
4. `90 41 FF 90 51 FF` (ACK + Completion)

> **Consecuencia importante:** La cámara **no devuelve coordenadas reales** ante consultas de telemetría (`81 09 06 12 FF` o `81 09 04 47 FF`). Además, enviar consultas `81 09 06 12 FF` de forma periódica mientras se mueve el joystick satura la cola interna del controlador de motores y provoca **órdenes fantasma de retorno al CENTRO (Home)**. Por ello, **nunca debe realizarse polling periódico por UDP 1259**.

### 3.2. Comandos VISCA soportados en hardware
En `MEDIA_SerialProtocolInit`, el fabricante conectó únicamente dos callbacks (`MEDIA_SerialCommandPtz` y `MEDIA_SerialCommandPreset`):

| Acción | Trama Hexadecimal VISCA | Notas de comportamiento en la Nuroum V403 |
| :--- | :--- | :--- |
| **Giro Izquierda** | `81 01 06 01 VV WW 01 03 FF` | Operativo. `VV` y `WW` se aceptan (`01`–`18`), pero el motor gira a velocidad angular fija de firmware. |
| **Giro Derecha** | `81 01 06 01 VV WW 02 03 FF` | Operativo. Velocidad angular constante. |
| **Inclinación Arriba** | `81 01 06 01 VV WW 03 01 FF` | Operativo. Velocidad angular constante. |
| **Inclinación Abajo** | `81 01 06 01 VV WW 03 02 FF` | Operativo. Velocidad angular constante. |
| **Diagonales (45°)** | `81 01 06 01 VV WW 0p 0t FF` | **NO SOPORTADO EN HARDWARE / FIRMWARE.** El parser `CtrlDeviceMotion` descarta el byte de Tilt si Pan != 0x03 (ver sección 3.3). |
| **Parada Pan/Tilt (STOP)** | `81 01 06 01 VV WW 03 03 FF` | Detiene los motores de giro e inclinación inmediatamente. |
| **Ir al Centro (HOME)** | `81 01 06 04 FF` | Vuelve a `(pan=0, tilt=0)`. |
| **Zoom Acercar (Tele)** | `81 01 04 07 2p FF` (o `02 FF`) | Velocidad constante de zoom digital/óptico. |
| **Zoom Alejar (Wide)** | `81 01 04 07 3p FF` (o `03 FF`) | Velocidad constante de zoom. |
| **Zoom Parada (Stop)** | `81 01 04 07 00 FF` | Detiene el zoom. |
| **Guardar Punto (Preset)** | `81 01 04 3F 01 0p FF` | Guarda en `/mnt/mtd/Config/config.ini` bajo `[preset.p]` (`zoom`, `pan`, `tilt`). |
| **Llamar Punto (Recall)** | `81 01 04 3F 02 0p FF` | Mueve la cámara a la posición guardada en `[preset.p]`. |

### 3.3. Limitación de Movimiento Diagonal e Ingeniería Inversa de `hunter`

Durante la auditoría del binario `/var/hunter` se desensamblaron las funciones `CtrlDeviceMotion` (`0x00229800 - 0x0022A428`), `MEDIA_SerialCommandPtz` (`0x001279A8`) y `REMOTE_CommandPtz` (`0x002108F4`).

#### 1. Parser de tramas VISCA en `commCtlProtocol.cpp`:
Al recibir la trama `81 01 06 01 VV WW [PanDir] [TiltDir] FF`:
* En la dirección `0x0022A27C`, el código evalúa `PanDir` (Byte 5):
  * Si `PanDir == 0x02` (Derecha): asigna `type = 0` (PAN), `dir = 1` y **salta de inmediato a la ejecución del motor**, sin leer jamás `TiltDir`.
  * Si `PanDir == 0x01` (Izquierda): asigna `type = 0` (PAN), `dir = 0` y salta directamente a la ejecución.
  * **Únicamente si `PanDir == 0x03` (Pan STOP)**, el flujo de ejecución llega a `0x0022A408` donde finalmente lee `TiltDir` (Byte 6: Arriba/Abajo).
* **Conclusión:** Cualquier comando VISCA diagonal estándar que envíe Pan y Tilt simultáneos provoca que el firmware ejecute **exclusivamente el movimiento horizontal (Pan)** y descarte en su totalidad la orden vertical.

#### 2. Controlador de motores en hardware (`REMOTE_CommandPtz`):
* La función `REMOTE_CommandPtz(int type, int dir, int enable, int speed)` solo acepta tipos `0` (Pan), `1` (Tilt), `2` (Zoom) y `3` (Stop). No existe ningún tipo ni estructura interna para movimiento simultáneo multi-eje.
* La variable global de hardware en memoria (`0x52B0A4`) almacena un único código de acción a la vez para el microcontrolador de los motores paso a paso.

#### 3. Solución implementada en el cliente (`src-tauri/src/visca/client.rs`):
Para evitar que el movimiento en diagonal quede atascado arbitrariamente en el eje horizontal, se implementa la **estrategia de selección por eje predominante (Dominant Axis Selection)**:
* Si `abs_x >= abs_y`: se envía comando cardinal puro de Pan (Izquierda o Derecha).
* Si `abs_y > abs_x`: se envía comando cardinal puro de Tilt (Arriba o Abajo).
Esto proporciona un control de joystick completamente fluido, predecible y sin comportamientos anómalos.

### 3.4. Comandos VISCA ignorados por el firmware
Cualquier otro comando VISCA estándar es descartado en la rama `Unknow ptz control` de `CtrlDeviceMotion`:
* **Balance de Blancos (`81 01 04 35 xx FF`, Ganancia R/B `81 01 04 03/04 ...`):** Ignorado por UDP 1259.
* **Espejo / Volteo de imagen (`81 01 04 61 xx FF` / `81 01 04 66 xx FF`):** Ignorado por UDP 1259.
* **Exposición / Brillo / Contraste (`81 01 04 39 ...` / `4E ...`):** Ignorado por UDP 1259.
* **Enfoque (`81 01 04 38 ...` / `08 ...`):** Aunque `libCommProtocol` parsea el comando (`remoteCommandFocus`), `Media_V403` no registra el callback serie de enfoque en `MEDIA_SerialProtocolInit`.

### 3.5. Dependencia de Flujo de Vídeo Activo (Modo Privacidad y Corte de Servomotores en Standby)

Durante los diagnósticos de hardware se constató una particularidad crítica de diseño que condiciona el movimiento físico del gimbal:

1. **Directiva de Privacidad en Flash (`/mnt/mtd/Config/config.ini`):**
   El firmware de fábrica mantiene activada la directiva:
   ```ini
   [privacy]
   enable = 1
   ```
2. **Corte de potencia de los drivers paso a paso (*Stepper Drivers*):**
   Cuando **ningún receptor está consumiendo activamente el flujo de vídeo** de la cámara (por ejemplo, si OBS Studio, vMix o NDI Studio Monitor están cerrados y no hay conexiones RTSP ni UVC activas):
   * El procesador de señal de imagen (ISP) entra en estado de reposo para ahorrar energía y disipar calor.
   * El controlador de hardware **desactiva la etapa de potencia de los servomotores**, bloqueando el movimiento mecánico del cabezal.
3. **Comportamiento engañoso del socket de red UDP 1259:**
   * El demonio principal `/var/hunter` **sigue 100 % vivo en la red** y procesa cualquier comando VISCA entrante (Pan, Tilt, Zoom, Home o Presets).
   * La cámara responde siempre con tramas afirmativas de ACK (`90 41 FF`) y Completion (`90 51 FF`).
   * Sin embargo, **el gimbal no se moverá físicamente** mientras el subsistema de captura e imagen permanezca inactivo.
4. **Activación automática al transmitir vídeo (*Wake on Stream*):**
   * En el instante en que una aplicación receptora (OBS Studio, NDI Studio Monitor, vMix o reproductor RTSP) abre el flujo de vídeo, la cámara despierta, energiza de inmediato los motores paso a paso y el joystick y presets responden con absoluta fluidez.

#### 3.5.1. Prueba Experimental de Despertar Proactivo (Enganche NDI / Descartado)

Se evaluó la posibilidad de simular un enganche de consumo de vídeo (`TCP 5961`) desde el backend de control antes de cada movimiento para escenarios donde el joystick reside en un PC distinto al que recibe la imagen de OBS.
* **Resultado de las pruebas de campo:** El mecanismo fue **descartado**. Abrir y cerrar conexiones NDI intermitentes colisiona con los temporizadores internos de la máquina de estados de `/var/hunter` y produce respuestas erráticas en los motores.
* **Solución arquitectónica estándar:** Mantener la sesión de streaming NDI activa de forma continuada en el receptor (OBS, vMix o NDI Studio Monitor), lo que mantiene la etapa de potencia de los motores energizada de forma natural y estable.

---

## 4. API HTTP JSON-RPC Oculta en el Puerto `TCP 8090`

El binario `/var/hunter` levanta un servidor HTTP basado en `ZLMediaKit` en el puerto **`8090`**. Aunque la ruta raíz `/` devuelve `404 Not Found!!!`, existe un endpoint RPC funcional:

* **Endpoint:** `POST http://<IP_CAMARA>:8090/index/api/remote`
* **Cabecera requerida:** `Content-Type: application/json`

### 4.1. Módulos registrados en `HttpProtHandle`
El análisis de llamadas a `HttpRegistProt` dentro de `/var/hunter` muestra 9 clases registradas:
1. **`Focus`** (`FocusServer` — **100 % operativo**)
2. **`System`** (`SystemServer` — **100 % operativo**)
3. **`DevicePwr`** (`RebootServer` — Reinicio `Reboot` y apagado `ShutDown`)
4. **`RecordOps`** (`recordServer` — Grabación en tarjeta SD local)
5. **`RTSP`** (`RtspServer` — Autenticación RTSP)
6. **`KeepAlive`** (`KeepAliveServer`)
7. **`Debug`** (`DebugServer`)
8. **`Upgrade`** (`UpgraderServer` — Actualización OTA/HTTP)
9. **`Option`** (`ConfigServer` — Registrado, pero **sin tablas de imagen vinculadas** en la V403; devuelve `{"code": -500, "msg": "NotFound"}` porque `Config::attach` nunca es invocado para `VideoIn`).

### 4.2. Ejemplos de llamadas JSON-RPC operativas en `:8090/index/api/remote`

#### Consultar modo de enfoque (`Auto` / `Manual`):
```json
// Petición POST a http://192.168.1.101:8090/index/api/remote
{
  "Seq": 1,
  "Class": "Focus",
  "Method": "GetMode",
  "Param": {}
}

// Respuesta HTTP 200:
{
  "code": 0,
  "data": {
    "Class": "Focus",
    "Method": "GetMode",
    "Param": { "Mode": "Auto", "Result": true },
    "Seq": 1
  },
  "msg": "success"
}
```

#### Cambiar modo de enfoque (`Auto` o `Manual`) y enfoque manual (`Far` / `Near`):
```json
// Cambiar a enfoque Automático o Manual:
{
  "Seq": 2,
  "Class": "Focus",
  "Method": "SetMode",
  "Param": { "Mode": "Auto" }
}

// Mover enfoque manual (cuando Mode == "Manual"):
{
  "Seq": 3,
  "Class": "Focus",
  "Method": "ManualFocus",
  "Param": { "Dir": "Far", "Enable": true }
}
```

#### Consultar y cambiar velocidad de Zoom (`Fast` / `Slow`):
```json
// Consultar velocidad de Zoom:
{
  "Seq": 4,
  "Class": "Focus",
  "Method": "GetZoomSpeedMode",
  "Param": {}
}

// Cambiar velocidad de Zoom ("Fast" o "Slow"):
{
  "Seq": 5,
  "Class": "Focus",
  "Method": "SetZoomSpeedMode",
  "Param": { "SpeedMode": "Fast" }
}
```

#### 4.2.1. Implementación de Auto-Recuperación y Calibración de Enfoque en el Controlador

Dado que la cámara carece de controles analógicos de enfoque en su mando infrarrojo físico y que VISCA serie no expone el callback de enfoque en la V403, este controlador aprovecha la API HTTP en el puerto `8090` para ofrecer dos niveles de protección de nitidez:

1. **Auto-Saneamiento Silencioso en el Arranque / Descubrimiento (`discovery.rs`):**
   * Cada vez que una cámara es descubierta o recupera conectividad, el backend en Rust dispara en segundo plano `http_set_focus_auto(&ip)` para garantizar que el parámetro interno `Focus.Mode` esté fijado en `"Auto"`. Esto evita que la cámara quede desenfocada de forma permanente tras cortes de red o reinicios.
2. **Botón Táctico de Re-calibración de Enfoque en Ajustes (`settings.html`):**
   * En la vista de configuración se ha integrado el botón `[ENFOQUE]` que invoca el comando Tauri `ptz_recalibrate_focus(camera_id)`:
     1. Reasegura el modo `"Auto"` por HTTP JSON-RPC.
     2. Dispara un ciclo óptico de micro-zoom VISCA (Tele 350 ms $\rightarrow$ Wide 350 ms $\rightarrow$ Stop).
     3. El desplazamiento mecánico de los elementos ópticos fuerza al procesador Rockchip a activar su bucle de contraste de alta frecuencia, clavando la nitidez de la escena en menos de 1 segundo sin requerir la aplicación NearSync ni cableado USB.

---

## 5. Cómo Funcionan los Ajustes de Imagen (Balance de Blancos, Color y Espejo)

El rastreo de referencias cruzadas (XREFs) en `/var/hunter` hacia las funciones del ISP de Rockchip (`ISP_SetWhiteBalance`, `SAMPLE_COMMON_ISP_SET_WhiteBalanceMode`, `SAMPLE_COMMON_ISP_SET_ManualWhiteBalance`, `MEDIA_CommonSetColor`, `MEDIA_CommonEnableMirror`, `MEDIA_CommonEnableFlip`) demuestra que **solo existen dos puntos de entrada en el firmware** capaces de modificar la imagen de la cámara:

1. **Vía USB (`libHid` / `vdo_V520dJsonConfig` + `MEDIA_UvcSetColor`):**
   * Cuando la cámara se conecta por USB al PC, expone el dispositivo HID `VID_0627&PID_A6B5`.
   * El programa **NearSync** utiliza `axdp.dll` para enviar tramas JSON de configuración (`parseJsonCfg_image` -> `iWhiteBalanceMode`, `iColorTemp`, `iMirror`, `iFlip`, `iBrightness`, `iContrast`, `iSaturation`, `iHue`, `iSharpen`) a través de ese canal USB HID, y controles DirectShow/UVC para PTZ y balance de blancos.
2. **Vía Mando a Distancia Infrarrojo IR (`REMOTE_EnterMenuCmdCtrl` -> `MEDIA_MenuMsgProcess`):**
   * Aunque el binario `/var/hunter` incorpora el código del menú en pantalla (`vdo_OsdMenuConfigV403`, heredado de la familia V415/V520D), **el mando a distancia físico incluido con la Nuroum V403 es básico y carece de botón de Menú**, por lo que en la práctica **solo es posible modificar estos ajustes conectando la cámara por USB con NearSync**.

### 5.1. Persistencia en Memoria Flash (`/mnt/mtd/Config/config.ini`)
Todos los ajustes realizados mediante NearSync por USB se guardan automáticamente en la partición flash UBIFS `/mnt/mtd/Config/config.ini`. La estructura interna extraída del firmware es:

```ini
;camera application configuration file

[audio]
audioenable = 1

[ispColor]
;[0--100]
brightness = 50
contrast = 50
saturation = 43
hue = 50

[whiteBalance]
;auto:1,manual:15
mode = 1
colortemp = 4900

[exposure]
;auto:0,manual:1
mode = 0
compensation = 90
frequency = 1

[videoView]
mirror = 0
flip = 0
rotate90 = 0

[alg]
aimode = 0
speakertrack = 4
tracktime = 3
splitcnt = 6

[ptz]
zoom = 0
pan = 0
tilt = 0

[preset.0]
zoom = -1
pan = 0
tilt = 0
; ... (preset.1 a preset.9)

[osd]
mirror = 1

[privacy]
enable = 1

[networkCfg]
udpPort = 1259

[wdr]
mode = 0
```

> **Conclusión práctica:** Como `/mnt/mtd/Config/config.ini` reside en memoria flash no volátil (`mtd5`), cualquier ajuste de Balance de Blancos, Color o Espejo que configures una vez (con el mando IR en directo sobre NDI, o una vez por USB con NearSync) **se conserva permanentemente aunque apagues o desconectes la cámara de la corriente**.

---

## 6. Credenciales Internas, Hashes de Firmware y Estado de Telnet (`TCP 23`)

Durante el análisis de las particiones UBIFS (`/etc/passwd`, `/etc/passwd-`, `/etc/shadow`, `/etc/inittab` y `/user/cfg/CustomConfig`) y la investigación en repositorios públicos de seguridad de cámaras IP embebidas (`smartvision.dev`, GitHub), se identificaron las siguientes credenciales y claves del sistema:

### 6.1. Acceso Telnet (`TCP 23`) y Hashes de `root`
1. **Hash activo en `/etc/passwd`:**
   ```text
   root:$1$w4uYby9X$MZBZYSSEjhCvwafKv0v2t1:0:0::/root:/bin/sh
   ```
   * **Algoritmo:** MD5-crypt (`$1$`), sal `w4uYby9X`.
   * **Estado en la red:** Este mismo hash `\$1\$w4uYby9X\$MZBZYSSEjhCvwafKv0v2t1` proviene de una plantilla OEM cerrada compartida con módulos de cámaras IP embebidas (HiLinux / Rockchip). En la literatura pública de investigación de hardware (`smartvision.dev` y listas de hashes OEM en GitHub), **su texto plano permanece sin romper mediante diccionarios públicos**, ya que fue generado con una cadena aleatoria larga de fábrica para bloquear el acceso por Telnet en producción.
2. **Hash histórico en `/etc/passwd-` (copia previa del sistema base):**
   ```text
   root:ab8nBoH3mb8.g:0:0::/root:/bin/sh
   ```
   * **Algoritmo:** DES-crypt tradicional.
   * **Contraseña original conocida:** **`helpme`** (contraseña por defecto clásica del SDK base HiLinux antes de que el fabricante ejecutara `passwd` para sustituirla por el hash MD5-crypt definitivo).
3. **Consola serie física (UART en placa base — `/etc/inittab`):**
   * En `/etc/inittab` figura la directiva `::respawn:-/bin/sh`, lo que significa que el puerto serie UART físico de la placa electrónica interna sí entrega una consola `root` directa sin pedir contraseña, mientras que el servicio `telnetd` por red invoca `/bin/login` y exige el hash de `/etc/passwd`.

### 6.2. Otras Claves Extraídas de `/user/cfg/CustomConfig`
* **Secreto de la WebApi HTTP (`TCP 8090` — `ZLMediaKit`):**
  * **`secret`:** `035c73f7-bb6b-4889-a715-d9eb2d1925cc`
  * Permite consultar endpoints administrativos del servidor de streaming en el puerto `8090` añadiendo `?secret=035c73f7-bb6b-4889-a715-d9eb2d1925cc` (por ejemplo: `http://192.168.1.101:8090/index/api/getApiList?secret=035c73f7-bb6b-4889-a715-d9eb2d1925cc` o `getMediaList`).
* **Credenciales RTSP (`TCP 554` — desactivadas por defecto con `EnableAuth: false`):**
  * **Usuario:** `admin`
  * **Realm:** `Hamedal`
  * **Hash MD5 almacenado:** `637a462359efc2fa757674d624c42a78`

---

## 7. Comparativa de Flujos de Vídeo (NDI vs RTSP) y Colisión del Nombre NDI

### 7.1. Diferencia de resolución y tasa de fotogramas
La consulta al endpoint interno `http://<IP>:8090/index/api/getMediaList?secret=035c73f7-bb6b-4889-a715-d9eb2d1925cc` y la negociación SDP revelan una diferencia importante entre ambos protocolos:

| Protocolo | Puerto / Ruta | Resolución Nativa | Fotogramas (FPS) | Códec |
| :--- | :--- | :--- | :--- | :--- |
| **NDI\|HX** | `TCP 5960` (control) / `TCP 5961` (flujo) | **`1920 × 1080` (Full HD 1080p)** | **`30 fps`** | H.264 (NDI\|HX) |
| **RTSP** | `rtsp://<IP>:554/live/mainStream` | **`1280 × 720` (HD 720p)** | **`25 fps`** | H.264 (`profile-level-id=4D401F`) |

### 7.2. Causa del nombre NDI duplicado `LOCALHOST (Nuroum Cam V403-)`
En `mediaNdi.cpp` (`MEDIA_NdiInit`), el binario `/var/hunter`:
1. Lee la clave `devname` de `/mnt/mtd/Sys/usbInfo.ini` (`"Nuroum Cam V403"`).
2. Concatena `"-"` y trata de añadir la dirección IP local (`Get devIp=%s`).
3. Durante el arranque en frío (`/user/etc/init.sh`), `MEDIA_NdiInit` se ejecuta antes de que el cliente DHCP obtenga la dirección IP en `eth0`, por lo que `devIp` resulta vacío (`""`) y todas las cámaras se anuncian en el puerto `TCP 5960` con idéntico descriptor XML:
   ```xml
   <ndi><services><service name="LOCALHOST (Nuroum Cam V403-)" port="5961"><groups><group>public</group></groups></service></services></ndi>
   ```

### 7.3. Cómo distinguir y renombrar múltiples cámaras Nuroum V403 en una aplicación receptora NDI propia (Cámara Virtual)
Al desarrollar una aplicación propia receptora NDI (por ejemplo, un receptor NDI a Cámara Virtual):
1. **Conexión directa por `p_url_address` en el SDK de NDI (`NDIlib_source_t`):**
   * Aunque ambas cámaras compartan `p_ndi_name = "LOCALHOST (Nuroum Cam V403-)"`, el SDK de NDI permite especificar la dirección IP y puerto exactos en `NDIlib_source_t::p_url_address` (por ejemplo `"192.168.1.101:5961"` y `"192.168.1.102:5961"`, o `"192.168.1.101:5960"`), diferenciando cada cámara física por su IP/MAC sin depender del nombre duplicado.
2. **Proxy / Reescritura del XML en `TCP 5960`:**
   * Alternativamente, si se utiliza descubrimiento estándar NDI, basta con interceptar el banner XML de 132 bytes del puerto `5960` para inyectar un atributo `name` único por cámara (según su MAC o IP) y redirigir el flujo `TCP 5961` en calidad nativa `1080p @ 30 fps`.


