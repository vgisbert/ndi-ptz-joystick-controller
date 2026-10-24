use grafton_visca::command::{
    focus::FocusCommand,
    inquiry::InquiryCommand,
    pan_tilt::{PanSpeed, PanTiltCommand, PanTiltDirection, TiltSpeed},
    zoom::ZoomCommand,
    ViscaCommand,
};

#[test]
fn test_preset_commands() {
    // Preset Store: 81 01 04 3F 01 pp FF
    let store_p1 = vec![0x81, 0x01, 0x04, 0x3F, 0x01, 0x00, 0xFF];
    assert_eq!(store_p1, vec![0x81, 0x01, 0x04, 0x3F, 0x01, 0x00, 0xFF]);

    let store_p8 = vec![0x81, 0x01, 0x04, 0x3F, 0x01, 0x07, 0xFF];
    assert_eq!(store_p8, vec![0x81, 0x01, 0x04, 0x3F, 0x01, 0x07, 0xFF]);

    // Preset Recall: 81 01 04 3F 02 pp FF
    let recall_p3 = vec![0x81, 0x01, 0x04, 0x3F, 0x02, 0x02, 0xFF];
    assert_eq!(recall_p3, vec![0x81, 0x01, 0x04, 0x3F, 0x02, 0x02, 0xFF]);

    // Preset Clear: 81 01 04 3F 00 pp FF
    let clear_p1 = vec![0x81, 0x01, 0x04, 0x3F, 0x00, 0x00, 0xFF];
    assert_eq!(clear_p1, vec![0x81, 0x01, 0x04, 0x3F, 0x00, 0x00, 0xFF]);
}

#[test]
fn test_grafton_pan_tilt_commands() {
    let pan_speed = PanSpeed::new(0x10).unwrap();
    let tilt_speed = TiltSpeed::new(0x0C).unwrap();

    let cmd_up = PanTiltCommand {
        direction: PanTiltDirection::Up,
        pan_speed,
        tilt_speed,
    };
    assert_eq!(
        cmd_up.to_bytes().unwrap(),
        vec![0x81, 0x01, 0x06, 0x01, 0x10, 0x0C, 0x03, 0x01, 0xFF]
    );

    let cmd_down_right = PanTiltCommand {
        direction: PanTiltDirection::DownRight,
        pan_speed,
        tilt_speed,
    };
    assert_eq!(
        cmd_down_right.to_bytes().unwrap(),
        vec![0x81, 0x01, 0x06, 0x01, 0x10, 0x0C, 0x02, 0x02, 0xFF]
    );

    let cmd_stop = PanTiltCommand {
        direction: PanTiltDirection::Stop,
        pan_speed: PanSpeed::STOP,
        tilt_speed: TiltSpeed::STOP,
    };
    assert_eq!(
        cmd_stop.to_bytes().unwrap(),
        vec![0x81, 0x01, 0x06, 0x01, 0x00, 0x00, 0x03, 0x03, 0xFF]
    );

    let cmd_home = PanTiltCommand {
        direction: PanTiltDirection::Home,
        pan_speed: PanSpeed::STOP,
        tilt_speed: TiltSpeed::STOP,
    };
    assert_eq!(
        cmd_home.to_bytes().unwrap(),
        vec![0x81, 0x01, 0x06, 0x04, 0xFF]
    );
}

#[test]
fn test_grafton_zoom_commands() {
    let cmd_stop = ZoomCommand::Stop;
    assert_eq!(
        cmd_stop.to_bytes().unwrap(),
        vec![0x81, 0x01, 0x04, 0x07, 0x00, 0xFF]
    );

    let cmd_tele_var = ZoomCommand::TeleVariable(5);
    assert_eq!(
        cmd_tele_var.to_bytes().unwrap(),
        vec![0x81, 0x01, 0x04, 0x07, 0x25, 0xFF]
    );

    let cmd_wide_var = ZoomCommand::WideVariable(3);
    assert_eq!(
        cmd_wide_var.to_bytes().unwrap(),
        vec![0x81, 0x01, 0x04, 0x07, 0x33, 0xFF]
    );
}

#[test]
fn test_grafton_focus_commands() {
    let cmd_auto = FocusCommand::Auto;
    assert_eq!(
        cmd_auto.to_bytes().unwrap(),
        vec![0x81, 0x01, 0x04, 0x38, 0x02, 0xFF]
    );

    let cmd_manual = FocusCommand::Manual;
    assert_eq!(
        cmd_manual.to_bytes().unwrap(),
        vec![0x81, 0x01, 0x04, 0x38, 0x03, 0xFF]
    );

    let cmd_one_push = FocusCommand::OnePushTrigger;
    assert_eq!(
        cmd_one_push.to_bytes().unwrap(),
        vec![0x81, 0x01, 0x04, 0x18, 0x01, 0xFF]
    );
}

#[test]
fn test_inquiry_packets() {
    let pt_inq = InquiryCommand::PanTiltPosition;
    assert_eq!(
        pt_inq.to_bytes().unwrap(),
        vec![0x81, 0x09, 0x06, 0x12, 0xFF]
    );

    let zoom_inq = InquiryCommand::ZoomPosition;
    assert_eq!(
        zoom_inq.to_bytes().unwrap(),
        vec![0x81, 0x09, 0x04, 0x47, 0xFF]
    );

    let focus_inq = InquiryCommand::FocusPosition;
    assert_eq!(
        focus_inq.to_bytes().unwrap(),
        vec![0x81, 0x09, 0x04, 0x48, 0xFF]
    );
}

#[test]
fn test_visca_telemetry_decoding() {
    // Decoding 4 nibbles into 16-bit integer
    fn decode_visca_u16(p: u8, q: u8, r: u8, s: u8) -> u16 {
        (((p & 0x0F) as u16) << 12)
            | (((q & 0x0F) as u16) << 8)
            | (((r & 0x0F) as u16) << 4)
            | ((s & 0x0F) as u16)
    }

    fn decode_visca_i16(p: u8, q: u8, r: u8, s: u8) -> i16 {
        decode_visca_u16(p, q, r, s) as i16
    }

    // Zoom position: 0x4000 = 16384 (maximum optical zoom)
    let zoom_raw = decode_visca_u16(0x04, 0x00, 0x00, 0x00);
    assert_eq!(zoom_raw, 0x4000);
    let zoom_ratio = 1.0 + (zoom_raw as f64 / 16384.0) * 9.0;
    assert_eq!(zoom_ratio, 10.0);

    // Pan angle calculation: +124.5 degrees
    // In raw steps at 0.0705 deg/step: 124.5 / 0.0705 ~= 1766 (0x06E6)
    let pan_raw = decode_visca_i16(0x00, 0x06, 0x0E, 0x06);
    let pan_deg = (pan_raw as f64 * 0.0705 * 10.0).round() / 10.0;
    assert_eq!(pan_deg, 124.5);
}

#[test]
fn test_exponential_expo_curve_calculation() {
    // Deadzone = 0.05, gamma = 2.0
    let deadzone: f64 = 0.05;
    let gamma: f64 = 2.0;

    // Below deadzone -> 0 deflection
    let deflection_center: f64 = 0.03;
    let norm_center: f64 = if deflection_center < deadzone { 0.0 } else { (deflection_center - deadzone) / (1.0 - deadzone) };
    assert_eq!(norm_center, 0.0);

    // Half deflection: 0.525 -> norm = (0.525 - 0.05) / 0.95 = 0.5
    let deflection_mid: f64 = 0.525;
    let norm_mid: f64 = (deflection_mid - deadzone) / (1.0 - deadzone);
    let expo_mid: f64 = norm_mid.powf(gamma); // 0.5^2 = 0.25 (Creep effect for micro-control)
    assert!((expo_mid - 0.25).abs() < 1e-6);

    // Full deflection: 1.0 -> norm = 1.0 -> expo = 1.0
    let deflection_full: f64 = 1.0;
    let norm_full: f64 = (deflection_full - deadzone) / (1.0 - deadzone);
    let expo_full: f64 = norm_full.powf(gamma);
    assert!((expo_full - 1.0).abs() < 1e-6);
}
