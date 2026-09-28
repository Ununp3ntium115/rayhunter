use log::{error, info, warn};
use rayhunter::Device;
use std::time::{Duration, Instant};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::config::{self, KeyInputMode};
use crate::diag::DiagDeviceCtrlMessage;

#[derive(Debug)]
enum Event {
    KeyDown,
    KeyUp,
}

const INPUT_EVENT_SIZE: usize = 32;

/// Taps closer together than this are treated as button bounce.
const MIN_TAP_GAP: Duration = Duration::from_millis(100);
/// Taps further apart than this start a new sequence.
const MAX_TAP_GAP: Duration = Duration::from_millis(800);

/// Orbic RC400L WiFi configuration; `<state>` under `<wlan><Feature>` enables both bands.
const ORBIC_WLAN_CONF: &str = "/usrdata/data/usr/wlan/wlan_conf_6174.xml";
const WLAN_ON: &str = "<wlan><Feature><state>1</state>";
const WLAN_OFF: &str = "<wlan><Feature><state>0</state>";

#[derive(Debug, PartialEq)]
enum TapAction {
    NewRecording,
    ToggleWifi,
}

/// Counts power-button taps and decides what a finished sequence means.
///
/// With only double-tap enabled, the action fires on the second tap. With triple-tap enabled,
/// a double tap has to wait until `MAX_TAP_GAP` passes without a third tap.
struct TapDetector {
    triple_tap_enabled: bool,
    count: u32,
    last_tap: Option<Instant>,
}

impl TapDetector {
    fn new(triple_tap_enabled: bool) -> Self {
        TapDetector {
            triple_tap_enabled,
            count: 0,
            last_tap: None,
        }
    }

    /// Records a key-up at `now`. Returns an action if the sequence is decided immediately.
    fn on_tap(&mut self, now: Instant) -> Option<TapAction> {
        match self.last_tap {
            Some(last) => {
                let elapsed = now.duration_since(last);
                if elapsed < MIN_TAP_GAP {
                    return None;
                }
                if elapsed > MAX_TAP_GAP {
                    self.count = 0;
                }
            }
            None => self.count = 0,
        }
        self.count += 1;
        self.last_tap = Some(now);

        match (self.count, self.triple_tap_enabled) {
            (2, false) => {
                self.reset();
                Some(TapAction::NewRecording)
            }
            (3, true) => {
                self.reset();
                Some(TapAction::ToggleWifi)
            }
            _ => None,
        }
    }

    /// When the pending sequence can no longer grow, returns the deadline to call `on_timeout`.
    fn deadline(&self) -> Option<Instant> {
        match self.last_tap {
            Some(last) if self.count > 0 => Some(last + MAX_TAP_GAP),
            _ => None,
        }
    }

    fn on_timeout(&mut self) -> Option<TapAction> {
        let action =
            (self.count == 2 && self.triple_tap_enabled).then_some(TapAction::NewRecording);
        self.reset();
        action
    }

    fn reset(&mut self) {
        self.count = 0;
        self.last_tap = None;
    }
}

/// Flips the WiFi feature state in the Orbic wlan config. Returns the new contents and whether
/// WiFi is now enabled, or `None` if the expected element isn't present.
fn toggle_wlan_state(xml: &str) -> Option<(String, bool)> {
    if xml.contains(WLAN_ON) {
        Some((xml.replacen(WLAN_ON, WLAN_OFF, 1), false))
    } else if xml.contains(WLAN_OFF) {
        Some((xml.replacen(WLAN_OFF, WLAN_ON, 1), true))
    } else {
        None
    }
}

async fn new_recording(diag_tx: &Sender<DiagDeviceCtrlMessage>) {
    if let Err(e) = diag_tx.send(DiagDeviceCtrlMessage::StopRecording).await {
        error!("Failed to send StopRecording: {e}");
    }
    if let Err(e) = diag_tx
        .send(DiagDeviceCtrlMessage::StartRecording { response_tx: None })
        .await
    {
        error!("Failed to send StartRecording: {e}");
    }
}

/// Toggles the Orbic WiFi hotspot. The setting is only read at boot, so this reboots the device;
/// the current recording is stopped first so the QMDL file is closed cleanly.
async fn toggle_wifi_and_reboot(diag_tx: &Sender<DiagDeviceCtrlMessage>) {
    let xml = match tokio::fs::read_to_string(ORBIC_WLAN_CONF).await {
        Ok(xml) => xml,
        Err(e) => {
            error!("Failed to read {ORBIC_WLAN_CONF}: {e}");
            return;
        }
    };
    let Some((new_xml, enabled)) = toggle_wlan_state(&xml) else {
        error!("WiFi state element not found in {ORBIC_WLAN_CONF}; not toggling WiFi");
        return;
    };
    if let Err(e) = tokio::fs::write(ORBIC_WLAN_CONF, new_xml).await {
        error!("Failed to write {ORBIC_WLAN_CONF}: {e}");
        return;
    }
    info!(
        "WiFi hotspot {} via triple tap, rebooting",
        if enabled { "enabled" } else { "disabled" }
    );
    if let Err(e) = diag_tx.send(DiagDeviceCtrlMessage::StopRecording).await {
        error!("Failed to send StopRecording: {e}");
    }
    // give the diag thread a moment to flush and close the recording
    tokio::time::sleep(Duration::from_secs(2)).await;
    let _ = tokio::process::Command::new("sync").status().await;
    if let Err(e) = tokio::process::Command::new("reboot").status().await {
        error!("Failed to reboot: {e}");
    }
}

pub fn run_key_input_thread(
    task_tracker: &TaskTracker,
    config: &config::Config,
    diag_tx: Sender<DiagDeviceCtrlMessage>,
    cancellation_token: CancellationToken,
) {
    let triple_tap_enabled = match config.key_input_mode {
        KeyInputMode::Disabled => return,
        KeyInputMode::DoubleTapPower => false,
        KeyInputMode::DoubleTapPowerTripleTapWifi => {
            if config.device == Device::Orbic {
                true
            } else {
                warn!("Triple-tap WiFi toggle is only supported on Orbic; using double-tap only");
                false
            }
        }
    };

    task_tracker.spawn(async move {
        // Open the input device
        let mut file = match File::open("/dev/input/event0").await {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to open /dev/input/event0: {e}");
                return;
            }
        };

        let mut buffer = [0u8; INPUT_EVENT_SIZE];
        let mut detector = TapDetector::new(triple_tap_enabled);
        let mut last_event_time: Option<Instant> = None;

        loop {
            let deadline = detector.deadline();
            let action = tokio::select! {
               _ = cancellation_token.cancelled() => {
                    info!("received key input shutdown");
                    return;
                }
                _ = async { tokio::time::sleep_until(deadline.unwrap().into()).await }, if deadline.is_some() => {
                    detector.on_timeout()
                }
                result = file.read_exact(&mut buffer) => {
                    if let Err(e) = result {
                        error!("failed to read key input: {e}");
                        return;
                    }

                    let now = Instant::now();

                    // On orbic it was observed that pressing the power button can trigger many
                    // successive events. Drop events that are too close together.
                    let bounced = last_event_time
                        .is_some_and(|last| now.duration_since(last) < Duration::from_millis(50));
                    last_event_time = Some(now);

                    match parse_event(buffer) {
                        Event::KeyUp if !bounced => detector.on_tap(now),
                        _ => None,
                    }
                }
            };

            match action {
                Some(TapAction::NewRecording) => new_recording(&diag_tx).await,
                Some(TapAction::ToggleWifi) => toggle_wifi_and_reboot(&diag_tx).await,
                None => {}
            }
        }
    });
}

fn parse_event(input: [u8; INPUT_EVENT_SIZE]) -> Event {
    if input[12] == 0 {
        Event::KeyUp
    } else {
        Event::KeyDown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_event_keydown_m7350_v5() {
        let input = [
            0x57, 0x6c, 0x09, 0x00, 0x7c, 0xfb, 0x03, 0x00, 0x01, 0x00, 0x74, 0x00, 0x01, 0x00,
            0x00, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert!(matches!(parse_event(input), Event::KeyDown));
    }

    #[test]
    fn test_parse_event_keyup_m7350_v5() {
        let input = [
            0x57, 0x6c, 0x09, 0x00, 0x1b, 0x15, 0x05, 0x00, 0x01, 0x00, 0x74, 0x00, 0x00, 0x00,
            0x00, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert!(matches!(parse_event(input), Event::KeyUp));
    }

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn double_tap_fires_immediately_without_triple_tap() {
        let t0 = Instant::now();
        let mut d = TapDetector::new(false);
        assert_eq!(d.on_tap(t0), None);
        assert_eq!(d.on_tap(t0 + ms(300)), Some(TapAction::NewRecording));
        assert_eq!(d.deadline(), None);
    }

    #[test]
    fn taps_too_far_apart_do_not_count() {
        let t0 = Instant::now();
        let mut d = TapDetector::new(false);
        assert_eq!(d.on_tap(t0), None);
        assert_eq!(d.on_tap(t0 + ms(1000)), None);
        assert_eq!(d.on_tap(t0 + ms(1300)), Some(TapAction::NewRecording));
    }

    #[test]
    fn bounce_is_ignored() {
        let t0 = Instant::now();
        let mut d = TapDetector::new(false);
        assert_eq!(d.on_tap(t0), None);
        assert_eq!(d.on_tap(t0 + ms(60)), None);
        assert_eq!(d.on_tap(t0 + ms(400)), Some(TapAction::NewRecording));
    }

    #[test]
    fn triple_tap_toggles_wifi() {
        let t0 = Instant::now();
        let mut d = TapDetector::new(true);
        assert_eq!(d.on_tap(t0), None);
        assert_eq!(d.on_tap(t0 + ms(300)), None);
        assert_eq!(d.on_tap(t0 + ms(600)), Some(TapAction::ToggleWifi));
        assert_eq!(d.deadline(), None);
    }

    #[test]
    fn double_tap_waits_for_timeout_when_triple_tap_enabled() {
        let t0 = Instant::now();
        let mut d = TapDetector::new(true);
        assert_eq!(d.on_tap(t0), None);
        assert_eq!(d.on_tap(t0 + ms(300)), None);
        assert_eq!(d.deadline(), Some(t0 + ms(300) + MAX_TAP_GAP));
        assert_eq!(d.on_timeout(), Some(TapAction::NewRecording));
        assert_eq!(d.deadline(), None);
    }

    #[test]
    fn single_tap_timeout_does_nothing() {
        let t0 = Instant::now();
        let mut d = TapDetector::new(true);
        assert_eq!(d.on_tap(t0), None);
        assert_eq!(d.on_timeout(), None);
    }

    #[test]
    fn toggle_wlan_state_flips_both_ways() {
        let on = "<root><wlan><Feature><state>1</state><x/></Feature></wlan></root>";
        let (off, enabled) = toggle_wlan_state(on).unwrap();
        assert!(!enabled);
        assert_eq!(
            off,
            "<root><wlan><Feature><state>0</state><x/></Feature></wlan></root>"
        );
        let (back, enabled) = toggle_wlan_state(&off).unwrap();
        assert!(enabled);
        assert_eq!(back, on);
    }

    #[test]
    fn toggle_wlan_state_rejects_unknown_format() {
        assert_eq!(toggle_wlan_state("<wlan><state>1</state></wlan>"), None);
    }
}
