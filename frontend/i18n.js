/**
 * NDI PTZ Tactical Broadcast Console — Sistema de Internacionalización (i18n)
 * 
 * ============================================================================
 * GUÍA PARA AÑADIR NUEVOS IDIOMAS MEDIANTE LA EDICIÓN DE ESTE FICHERO:
 * ============================================================================
 * 1. Para agregar un nuevo idioma (por ejemplo, Francés 'FR' o Alemán 'DE'):
 *    - Copia todo el bloque de 'EN' o 'SP' dentro del objeto I18N_LOCALES.
 *    - Pega la copia a continuación y cambia la clave (ej. 'FR' o 'DE').
 *    - Traduce los textos entre comillas a la derecha de cada propiedad.
 *    - Guarda este archivo.
 * 2. ¡Listo! El botón superior detectará automáticamente el nuevo idioma y
 *    permitirá seleccionarlo cíclicamente al hacer clic (SP -> EN -> FR -> ...).
 */

const I18N_LOCALES = {
  // ==========================================
  // SET 1: ESPAÑOL (SP) — POR DEFECTO
  // ==========================================
  SP: {
    _label: "SP",
    _name: "Español",

    // Títulos de Ventanas
    app_title: "Controlador PTZ NDI",
    app_title_settings: "Controlador PTZ NDI — Configuración",
    app_title_floating: "Controlador PTZ NDI — Ventana Flotante",

    // Botones de Cabecera y Ventana
    btn_lang_title: "Cambiar idioma (Haga clic para alternar)",
    btn_pin_title: "Fijar ventana siempre visible",
    btn_pin_floating_title: "Fijar Ventana Flotante (Always on Top)",
    btn_settings_title: "Configuración",
    btn_minimize_title: "Minimizar",
    btn_close_title: "Cerrar Ventana",
    btn_back: "VOLVER",

    // Pestañas de Cámaras y Estados de Red
    tabs_searching: "BUSCANDO...",
    tabs_no_cameras: "0 CÁM ONLINE",
    status_no_network: "SIN RED",
    cam_prefix: "CÁM",

    // Overlay de Modo Desconectado
    offline_no_signal: "SIN SEÑAL DE CÁMARA",
    offline_subtext: "Buscando cámaras Nuroum V403 en UDP 1259...",
    offline_connect_hint: "Conecte una cámara Nuroum V403 a la red local",
    offline_scanning_subnet: "Escaneando subred local UDP 1259...",
    offline_btn_scan: "BUSCAR AHORA",
    offline_btn_settings: "AJUSTES",

    // Tira de Telemetría (OLED)
    telem_pan: "GIRO",
    telem_tilt: "INCL.",
    telem_zoom: "ZOOM",
    telem_focus: "ENFOQUE",
    badge_center: "CENTRO",
    badge_manual: "MANUAL",
    badge_visca_active: "VISCA UDP 1259 // ACTIVO",
    badge_free_preset: "PRESET LIBRE",

    // Joystick y Botones de Movimiento
    btn_dir_up_title: "Inclinar arriba (Flecha Arriba)",
    btn_dir_down_title: "Inclinar abajo (Flecha Abajo)",
    btn_dir_left_title: "Girar a la izquierda (Flecha Izquierda)",
    btn_dir_right_title: "Girar a la derecha (Flecha Derecha)",
    btn_mirror: "ESPEJO ↔",
    btn_mirror_title: "Invertir sentido Derecha/Izquierda del Joystick y flechas para coincidir con Espejo en OBS",
    btn_zoom_in: "ACERCAR +",
    btn_zoom_in_title: "Acercar imagen",
    btn_zoom_out: "ALEJAR -",
    btn_zoom_out_title: "Alejar imagen",
    btn_zoom_wide: "ALEJAR -",
    btn_zoom_tele: "ACERCAR +",
    btn_home_center: "CENTRAR [ESPACIO]",
    btn_home_center_title: "Centrar cámara (Barra Espaciadora)",
    btn_home_floating_title: "Centrar cámara (ESPACIO)",
    btn_creep: "CREEP 25%",
    btn_creep_title: "Modo Creep 25% para paneo lento cinematográfico",
    radar_vector: "Radar Vectorial",

    // Puntos de Parada (Presets) y Memoria
    presets_goto_title: "IR A PUNTO (1-8)",
    presets_goto_floating: "Ir a Preset (GOTO)",
    presets_instant_press: "PULSACIÓN INSTANTÁNEA",
    mem_help_label: "GUARDAR: MANTENER 2s",
    mem_saving: "GUARDANDO P{n} (2s)...",
    mem_saved_badge: "P{n} GUARDADO",
    mem_saved_feedback: "P{n} GUARDADO OK",
    mem_nvram_help: "Guardar en NVRAM (Mantener 2s)",
    default_presets: ["ESCEN.", "TECLADO", "BATERÍA", "VOZ", "BAJO", "GUIT.", "PÚBLICO", "TELÓN"],
    default_presets_floating: ["ESCENARIO", "TECLADO", "BATERÍA", "VOZ PPAL", "BAJO", "GUITARRA", "PÚBLICO", "TELÓN"],

    // Pantalla de Configuración (settings.html)
    sec_identification: "Identificación de Cámara",
    lbl_cam_name: "NOM:",
    err_duplicate_name: "¡REPETIDO!",
    err_duplicate_name_title: "Este nombre ya lo utiliza otra cámara",
    btn_ip_title: "Ver IP para OBS, estado ON/OFFLINE y buscar cámaras en la red",
    btn_refocus: "ENFOQUE",
    btn_refocus_title: "Restablecer Autoenfoque y recalibrar nitidez de la lente",
    sec_optics_status: "Modo de Cámara & Nitidez",
    lbl_focus_system: "MODO CÁMARA",
    mode_manual_val: "MANUAL",
    lbl_isp_sharpness: "NITIDEZ / ISP",
    lbl_refocus_title: "Recalibración de Nitidez",
    btn_recalibrate_action: "REHABILITAR NITIDEZ",
    lbl_reboot_row: "Reinicio de Cámara",
    btn_reboot_device: "REINICIAR (REBOOT)",
    modal_reboot_title: "¿REINICIAR LINUX ROCKCHIP?",
    modal_reboot_msg: "Se reiniciará el procesador Linux Rockchip mediante llamada RPC (DevicePwr.Reboot) y reinicio de servidor. El flujo se interrumpirá durante ~15-20s.",
    rebooting_in_progress: "REINICIANDO LINUX ROCKCHIP...",
    btn_confirm_reboot: "CONFIRMAR REINICIO",
    sec_camera_health: "Diagnóstico y Salud",
    health_nominal: "NOMINAL // ÓPTIMO",
    health_idle: "EN ESPERA",
    health_offline: "DESCONECTADA",
    lbl_diag_bitrate: "BITRATE:",
    lbl_diag_res: "RES:",
    lbl_diag_uptime: "ACTIVO:",
    btn_refresh: "REFRESCAR",
    refocusing: "ENFOCANDO...",
    refocus_success: "FOCO OK",
    sec_presets_title: "Etiquetas de Puntos (1-8)",
    sec_presets_by_ip: "GUARDADO POR IP",
    btn_go: "IR",
    btn_go_title: "Ir al punto {n}",
    btn_export: "EXPORTAR",
    btn_save_and_back: "GUARDAR Y VOLVER",

    // Modal de Configuración IP y Red
    modal_network_title: "Red · {name}",
    lbl_cam_status: "Estado Cámara:",
    status_checking: "COMPROBANDO...",
    lbl_mac_id: "Dirección IP:",
    lbl_last_ip: "Última IP Asignada (para OBS / UDP 1259):",
    btn_verify_ip: "PROBAR",
    btn_verify_ip_title: "Probar esta IP",
    btn_scan_network: "BÚSQUEDA EN LA RED",
    scan_feedback_default: "Detecta cámaras activas y reasignaciones DHCP",
    btn_accept: "ACEPTAR",
    scan_searching_btn: "BUSCANDO EN LA RED...",
    scan_scanning_feedback: "Escaneando subred UDP 1259...",
    feedback_cam_active_obs: "Cámara activa para OBS en {ip}",
    feedback_no_response_scan: "Sin respuesta · Pulsa BÚSQUEDA EN LA RED",
    feedback_response_ok: "Cámara activa en {ip}",
    feedback_no_response_ip: "Sin respuesta en {ip} · Cámara en OFFLINE",
    feedback_cams_found: "{n} cámara(s) ONLINE · IP actual: {ip}",
    feedback_no_cams_found: "No se encontraron cámaras · Estado: OFFLINE",
    feedback_network_error: "Sin respuesta en red · Estado: OFFLINE"
  },

  // ==========================================
  // SET 2: ENGLISH (EN)
  // ==========================================
  EN: {
    _label: "EN",
    _name: "English",

    // Window Titles
    app_title: "NDI PTZ Controller",
    app_title_settings: "NDI PTZ Controller — Settings",
    app_title_floating: "NDI PTZ Controller — Floating Window",

    // Header & Window Buttons
    btn_lang_title: "Change language (Click to toggle)",
    btn_pin_title: "Pin window always on top",
    btn_pin_floating_title: "Pin Floating Window (Always on Top)",
    btn_settings_title: "Settings",
    btn_minimize_title: "Minimize",
    btn_close_title: "Close Window",
    btn_back: "BACK",

    // Camera Tabs & Network Status
    tabs_searching: "SCANNING...",
    tabs_no_cameras: "0 CAM ONLINE",
    status_no_network: "NO NETWORK",
    cam_prefix: "CAM",

    // Disconnected Mode Overlay
    offline_no_signal: "NO CAMERA SIGNAL",
    offline_subtext: "Searching Nuroum V403 cameras on UDP 1259...",
    offline_connect_hint: "Connect a Nuroum V403 camera to local network",
    offline_scanning_subnet: "Scanning local subnet UDP 1259...",
    offline_btn_scan: "SCAN NOW",
    offline_btn_settings: "SETTINGS",

    // Telemetry Strip (OLED)
    telem_pan: "PAN",
    telem_tilt: "TILT",
    telem_zoom: "ZOOM",
    telem_focus: "FOCUS",
    badge_center: "CENTER",
    badge_manual: "MANUAL",
    badge_visca_active: "VISCA UDP 1259 // ACTIVE",
    badge_free_preset: "FREE PRESET",

    // Joystick & Directional Buttons
    btn_dir_up_title: "Tilt up (Up Arrow)",
    btn_dir_down_title: "Tilt down (Down Arrow)",
    btn_dir_left_title: "Pan left (Left Arrow)",
    btn_dir_right_title: "Pan right (Right Arrow)",
    btn_mirror: "MIRROR ↔",
    btn_mirror_title: "Invert Joystick and arrow Left/Right direction to match OBS Mirror",
    btn_zoom_in: "ZOOM IN +",
    btn_zoom_in_title: "Zoom in",
    btn_zoom_out: "ZOOM OUT -",
    btn_zoom_out_title: "Zoom out",
    btn_zoom_wide: "ZOOM OUT -",
    btn_zoom_tele: "ZOOM IN +",
    btn_home_center: "CENTER [SPACE]",
    btn_home_center_title: "Center camera (Spacebar)",
    btn_home_floating_title: "Center camera (SPACE)",
    btn_creep: "CREEP 25%",
    btn_creep_title: "Creep mode 25% for slow cinematic panning",
    radar_vector: "Vector Radar",

    // Presets & Memory
    presets_goto_title: "GO TO PRESET (1-8)",
    presets_goto_floating: "Go to Preset (GOTO)",
    presets_instant_press: "INSTANT PRESS",
    mem_help_label: "SAVE: HOLD 2s",
    mem_saving: "SAVING P{n} (2s)...",
    mem_saved_badge: "P{n} SAVED",
    mem_saved_feedback: "P{n} SAVED OK",
    mem_nvram_help: "Save to NVRAM (Hold 2s)",
    default_presets: ["STAGE", "KEYBOARD", "DRUMS", "VOCALS", "BASS", "GUITAR", "AUDIENCE", "CURTAIN"],
    default_presets_floating: ["STAGE", "KEYBOARD", "DRUMS", "MAIN VOCAL", "BASS", "GUITAR", "AUDIENCE", "CURTAIN"],

    // Settings Screen (settings.html)
    sec_identification: "Camera Identification",
    lbl_cam_name: "NAME:",
    err_duplicate_name: "DUPLICATE!",
    err_duplicate_name_title: "This name is already used by another camera",
    btn_ip_title: "View IP for OBS, ON/OFFLINE status and scan cameras on network",
    btn_refocus: "FOCUS",
    btn_refocus_title: "Reset Autofocus and recalibrate lens sharpness",
    sec_optics_status: "Camera Mode & Sharpness",
    lbl_focus_system: "CAMERA MODE",
    mode_manual_val: "MANUAL",
    lbl_isp_sharpness: "SHARPNESS / ISP",
    lbl_refocus_title: "Recalibrate Sharpness",
    btn_recalibrate_action: "RESTORE SHARPNESS",
    lbl_reboot_row: "Camera Reboot",
    btn_reboot_device: "REBOOT CAMERA",
    modal_reboot_title: "REBOOT LINUX ROCKCHIP?",
    modal_reboot_msg: "The Rockchip Linux processor will reboot via RPC call and media server restart. Stream will be interrupted for ~15-20s.",
    rebooting_in_progress: "REBOOTING LINUX ROCKCHIP...",
    btn_confirm_reboot: "CONFIRM REBOOT",
    sec_camera_health: "Diagnostics & Health",
    health_nominal: "NOMINAL // OPTIMAL",
    health_idle: "STANDBY",
    health_offline: "OFFLINE",
    lbl_diag_bitrate: "BITRATE:",
    lbl_diag_res: "RES:",
    lbl_diag_uptime: "UPTIME:",
    btn_refresh: "REFRESH",
    refocusing: "FOCUSING...",
    refocus_success: "FOCUS OK",
    sec_presets_title: "Preset Labels (1-8)",
    sec_presets_by_ip: "SAVED BY IP",
    btn_go: "GO",
    btn_go_title: "Go to preset {n}",
    btn_export: "EXPORT",
    btn_save_and_back: "SAVE & BACK",

    // IP & Network Modal
    modal_network_title: "Network · {name}",
    lbl_cam_status: "Camera Status:",
    status_checking: "CHECKING...",
    lbl_mac_id: "IP Address:",
    lbl_last_ip: "Last Assigned IP (for OBS / UDP 1259):",
    btn_verify_ip: "TEST",
    btn_verify_ip_title: "Test this IP",
    btn_scan_network: "NETWORK SCAN",
    scan_feedback_default: "Detects active cameras and DHCP reassignments",
    btn_accept: "ACCEPT",
    scan_searching_btn: "SCANNING NETWORK...",
    scan_scanning_feedback: "Scanning UDP 1259 subnet...",
    feedback_cam_active_obs: "Camera active for OBS at {ip}",
    feedback_no_response_scan: "No response · Click NETWORK SCAN",
    feedback_response_ok: "Camera active at {ip}",
    feedback_no_response_ip: "No response at {ip} · Camera is OFFLINE",
    feedback_cams_found: "{n} camera(s) ONLINE · Current IP: {ip}",
    feedback_no_cams_found: "No cameras found · Status: OFFLINE",
    feedback_network_error: "No response on network · Status: OFFLINE"
  }
};

/**
 * Motor de Traducción de la Aplicación
 */
class I18nManager {
  constructor() {
    this.STORAGE_KEY = 'ndi_ptz_lang';
    this.DEFAULT_LANG = 'SP';
    this.locales = I18N_LOCALES;
    this.listeners = new Set();
    this._currentLang = null;
  }

  getAvailableLanguages() {
    return Object.keys(this.locales);
  }

  getCurrentLanguage() {
    if (this._currentLang && this.locales[this._currentLang]) {
      return this._currentLang;
    }
    try {
      const saved = localStorage.getItem(this.STORAGE_KEY);
      if (saved && this.locales[saved]) {
        this._currentLang = saved;
        return saved;
      }
    } catch (e) {}
    this._currentLang = this.DEFAULT_LANG;
    return this.DEFAULT_LANG;
  }

  setLanguage(lang) {
    if (!this.locales[lang]) {
      lang = this.DEFAULT_LANG;
    }
    this._currentLang = lang;
    try {
      localStorage.setItem(this.STORAGE_KEY, lang);
      if (window.NDIPtz && window.NDIPtz.syncConfigToDisk) {
        window.NDIPtz.syncConfigToDisk();
      }
    } catch (e) {}

    this.applyToDocument();
    this.notifyListeners(lang);
  }

  cycleLanguage() {
    const list = this.getAvailableLanguages();
    if (list.length === 0) return this.DEFAULT_LANG;
    const current = this.getCurrentLanguage();
    const idx = list.indexOf(current);
    const next = list[(idx + 1) % list.length];
    this.setLanguage(next);
    return next;
  }

  t(key, params = {}) {
    const lang = this.getCurrentLanguage();
    const dict = this.locales[lang] || this.locales[this.DEFAULT_LANG];
    let text = dict ? dict[key] : null;

    if (text === undefined || text === null) {
      const fallback = this.locales[this.DEFAULT_LANG];
      text = fallback ? fallback[key] : key;
    }

    if (typeof text === 'string') {
      for (const [pKey, pVal] of Object.entries(params)) {
        text = text.replace(new RegExp(`\\{${pKey}\\}`, 'g'), pVal);
      }
    }
    return text;
  }

  getDefaultPresets(isFloating = false) {
    const lang = this.getCurrentLanguage();
    const dict = this.locales[lang] || this.locales[this.DEFAULT_LANG];
    if (isFloating && dict.default_presets_floating) {
      return [...dict.default_presets_floating];
    }
    return dict.default_presets ? [...dict.default_presets] : [...this.locales.SP.default_presets];
  }

  onLanguageChange(callback) {
    this.listeners.add(callback);
    return () => this.listeners.delete(callback);
  }

  notifyListeners(lang) {
    for (const cb of this.listeners) {
      try {
        cb(lang);
      } catch (err) {
        console.error('[i18n] Error in listener:', err);
      }
    }
  }

  applyToDocument(root = (typeof document !== 'undefined' ? document : null)) {
    if (!root) return;
    const currentLang = this.getCurrentLanguage();

    // Actualizar atributo lang en <html>
    if (document.documentElement) {
      document.documentElement.lang = currentLang.toLowerCase();
    }

    // Traducir título del documento según corresponda
    const titleAttr = document.querySelector('title[data-i18n]');
    if (titleAttr) {
      const key = titleAttr.getAttribute('data-i18n');
      document.title = this.t(key);
    }

    // Traducir elementos con data-i18n (contenido de texto)
    root.querySelectorAll('[data-i18n]').forEach(el => {
      const key = el.getAttribute('data-i18n');
      if (key) {
        el.textContent = this.t(key);
      }
    });

    // Traducir atributos title (tooltips) con data-i18n-title
    root.querySelectorAll('[data-i18n-title]').forEach(el => {
      const key = el.getAttribute('data-i18n-title');
      if (key) {
        el.title = this.t(key);
      }
    });

    // Traducir placeholders con data-i18n-placeholder
    root.querySelectorAll('[data-i18n-placeholder]').forEach(el => {
      const key = el.getAttribute('data-i18n-placeholder');
      if (key) {
        el.placeholder = this.t(key);
      }
    });

    // Actualizar botones de idioma en la página (#btn-lang)
    const btnLangs = root.querySelectorAll('#btn-lang, .btn-i18n-toggle');
    btnLangs.forEach(btn => {
      btn.textContent = currentLang;
      btn.title = this.t('btn_lang_title');
    });
  }

  init() {
    this.applyToDocument();

    // Configurar listeners en botones de cambio de idioma
    if (typeof document !== 'undefined') {
      document.addEventListener('click', (e) => {
        const btn = e.target.closest('#btn-lang, .btn-i18n-toggle');
        if (btn) {
          e.preventDefault();
          e.stopPropagation();
          this.cycleLanguage();
        }
      });

      // Escuchar cambios de idioma en otras ventanas (sincronización multi-ventana vía storage)
      window.addEventListener('storage', (e) => {
        if (e.key === this.STORAGE_KEY) {
          this._currentLang = e.newValue;
          this.applyToDocument();
          this.notifyListeners(this.getCurrentLanguage());
        }
      });
    }
  }
}

// Instancia global accesible desde cualquier script
if (typeof window !== 'undefined') {
  window.I18N_LOCALES = I18N_LOCALES;
  window.I18n = new I18nManager();

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => window.I18n.init());
  } else {
    window.I18n.init();
  }
}
