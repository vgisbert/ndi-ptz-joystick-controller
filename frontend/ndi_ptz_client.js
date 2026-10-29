/**
 * NDI PTZ Tactical Broadcast Console — IPC Client Bridge
 * Communicates with Tauri Rust backend over IPC (window.__TAURI__.core.invoke)
 */

(function () {
    const isTauri = typeof window !== 'undefined' && window.__TAURI__ !== undefined;

    const NDI = {
        isTauriAvailable: () => isTauri,

        async getCameras() {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('get_cameras');
            }
            console.log('[NDI Mock] get_cameras invoked');
            return [
                { id: '192.168.1.101', mac: '', name: 'CÁM 1', ip: '192.168.1.101', visca_port: 1259, is_manual: false, is_online: true, latency_ms: 4 }
            ];
        },

        async scanAllCameras() {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('scan_all_cameras');
            }
            return await this.getCameras();
        },

        async addCamera(ip, port = 1259, name = null) {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('add_camera', { ip, port, name });
            }
            console.log('[NDI Mock] add_camera:', { ip, port, name });
            return { id: `cam-${ip.replace(/\./g, '-')}`, name: name || `Cámara ${ip}`, ip, visca_port: port, is_manual: true, is_online: true, latency_ms: null };
        },

        async updateCameraConfig(cameraId, ip, port = 1259, name = null) {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('update_camera_config', { cameraId, ip, port, name });
            }
            console.log('[NDI Mock] update_camera_config:', { cameraId, ip, port, name });
            return { id: cameraId, name: name || `Cámara ${ip}`, ip, visca_port: port, is_manual: true, is_online: true, latency_ms: null };
        },

        async scanCameraIp(cameraId, hintIp, excludeIp) {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('scan_camera_ip', { cameraId, hintIp, excludeIp });
            }
            console.log('[NDI Mock] scan_camera_ip:', { cameraId, hintIp, excludeIp });
            return { id: cameraId, name: 'CÁM', ip: hintIp, visca_port: 1259, is_manual: true, is_online: false, latency_ms: null };
        },

        async removeCamera(cameraId) {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('remove_camera', { cameraId });
            }
            console.log('[NDI Mock] remove_camera:', cameraId);
            return true;
        },

        async sendVector(cameraId, x, y, speedScale = 1.0) {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('ptz_vector', { cameraId, x, y, speedScale });
            }
            // Mock logging throttled
        },

        async stop(cameraId) {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('ptz_stop', { cameraId });
            }
            console.log('[NDI Mock] ptz_stop:', cameraId);
        },

        async home(cameraId) {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('ptz_home', { cameraId });
            }
            console.log('[NDI Mock] ptz_home:', cameraId);
        },

        async zoom(cameraId, action, speed = 4) {
            // action: 'tele' | 'wide' | 'stop'
            if (isTauri) {
                return await window.__TAURI__.core.invoke('ptz_zoom', { cameraId, action, speed });
            }
            console.log('[NDI Mock] ptz_zoom:', cameraId, action, speed);
        },

        async focus(cameraId, action, speed = 4) {
            // action: 'auto' | 'manual' | 'far' | 'near' | 'stop' | 'onepush'
            if (isTauri) {
                return await window.__TAURI__.core.invoke('ptz_focus', { cameraId, action, speed });
            }
            console.log('[NDI Mock] ptz_focus:', cameraId, action, speed);
        },

        async recalibrateFocus(cameraId) {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('ptz_recalibrate_focus', { cameraId });
            }
            console.log('[NDI Mock] ptz_recalibrate_focus:', cameraId);
            return true;
        },

        async rebootCamera(cameraId) {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('camera_reboot', { cameraId });
            }
            console.log('[NDI Mock] camera_reboot:', cameraId);
            return true;
        },

        async getCameraHealth(cameraId) {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('camera_get_health', { cameraId });
            }
            return {
                is_online: true,
                visca_latency_ms: 2,
                http_ok: true,
                http_rtt_ms: 35,
                active_streams_count: 0,
                total_readers: 0,
                bitrate_mbps: 0.0,
                fps: 25,
                resolution: '1080p',
                uptime_seconds: 0,
                cpu_load_pct: 0.0
            };
        },

        async preset(cameraId, action, presetNum) {
            // action: 'recall' | 'store' | 'clear'
            if (isTauri) {
                return await window.__TAURI__.core.invoke('ptz_preset', { cameraId, action, presetNum });
            }
            console.log('[NDI Mock] ptz_preset:', cameraId, action, presetNum);
        },

        async sendRawVisca(cameraId, hexBytes) {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('send_raw_visca', { cameraId, hexBytes });
            }
            console.log('[NDI Mock] send_raw_visca:', cameraId, hexBytes);
        },

        async getTelemetry(cameraId) {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('ptz_inquiry_telemetry', { cameraId });
            }
            return {
                camera_id: cameraId,
                pan_deg: 124.50,
                tilt_deg: -12.30,
                pan_raw: 1766,
                tilt_raw: -174,
                zoom_ratio: 4.80,
                zoom_raw: 6912,
                focus_mode: "AUTO",
                latency_ms: 4,
                is_online: true
            };
        },

        async ping(cameraId) {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('ptz_ping', { cameraId });
            }
            return 3;
        },

        async openCameraWindow(cameraId) {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('open_camera_window', { cameraId });
            }
            console.log('[NDI Mock] open_camera_window:', cameraId);
            window.open(`floating.html?cam=${cameraId}`, '_blank', 'width=460,height=740');
        },

        async toggleWindowPin() {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('toggle_window_pin');
            }
            console.log('[NDI Mock] toggle_window_pin');
            return true;
        },

        async isWindowPinned() {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('is_window_pinned');
            }
            return false;
        },

        async minimizeWindow() {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('minimize_window');
            }
            console.log('[NDI Mock] minimize_window');
        },

        async closeWindow() {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('close_window');
            }
            window.close();
        },

        async setWindowSize(width, height) {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('set_window_size', { width, height });
            }
        },

        async setSettingsActive(active) {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('set_settings_active', { active });
            }
            console.log('[NDI Mock] set_settings_active:', active);
        },

        async getAppConfig() {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('get_app_config');
            }
            return {};
        },

        async saveAppConfig(config) {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('save_app_config', { config });
            }
        },

        async getAppBaseDir() {
            if (isTauri) {
                return await window.__TAURI__.core.invoke('get_app_base_dir_path');
            }
            return '';
        },

        async syncConfigFromDisk() {
            if (!isTauri) return;
            try {
                const cfg = await window.__TAURI__.core.invoke('get_app_config');
                if (cfg && typeof cfg === 'object' && Object.keys(cfg).length > 0) {
                    if (cfg.profiles) {
                        localStorage.setItem('ndi_cam_profiles', JSON.stringify(cfg.profiles));
                    }
                    if (cfg.activeCameraId) {
                        localStorage.setItem('ndi_active_cam', cfg.activeCameraId);
                    }
                    if (cfg.language) {
                        localStorage.setItem('ndi_ptz_language', cfg.language);
                    }
                    if (cfg.mirrorSettings && typeof cfg.mirrorSettings === 'object') {
                        for (const [k, v] of Object.entries(cfg.mirrorSettings)) {
                            localStorage.setItem(`ndi_mirror_std_${k}`, v);
                        }
                    }
                    console.log('[NDI Config] Configuración sincronizada desde config.json.');
                } else {
                    await this.syncConfigToDisk();
                }
            } catch (e) {
                console.warn('[NDI Config] Error al sincronizar config desde disco:', e);
            }
        },

        async syncConfigToDisk() {
            if (!isTauri) return;
            try {
                const rawProfiles = localStorage.getItem('ndi_cam_profiles') || localStorage.getItem('ndi_cam_profiles_mac');
                const profiles = rawProfiles ? JSON.parse(rawProfiles) : {};
                const activeCameraId = localStorage.getItem('ndi_active_cam') || '';
                const language = localStorage.getItem('ndi_ptz_language') || 'es';

                const mirrorSettings = {};
                for (let i = 0; i < localStorage.length; i++) {
                    const key = localStorage.key(i);
                    if (key && key.startsWith('ndi_mirror_std_')) {
                        const camId = key.replace('ndi_mirror_std_', '');
                        mirrorSettings[camId] = localStorage.getItem(key);
                    }
                }

                const config = {
                    activeCameraId,
                    language,
                    mirrorSettings,
                    profiles
                };

                await window.__TAURI__.core.invoke('save_app_config', { config });
                console.log('[NDI Config] Configuración guardada en config.json.');
            } catch (e) {
                console.warn('[NDI Config] Error al guardar config en disco:', e);
            }
        },

        onCameraDiscovered(callback) {
            if (isTauri && window.__TAURI__.event) {
                return window.__TAURI__.event.listen('camera-discovered', (event) => {
                    callback(event.payload);
                });
            }
        },

        onCamerasUpdated(callback) {
            if (isTauri && window.__TAURI__.event) {
                return window.__TAURI__.event.listen('cameras-updated', (event) => {
                    callback(event.payload);
                });
            }
        }
    };

    window.NDIPtz = NDI;
    console.log('[NDI PTZ] Bridge IPC cargado correctamente. Modo Tauri:', isTauri);
})();
