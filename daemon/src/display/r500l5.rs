use async_trait::async_trait;
use log::warn;
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::config;
use crate::display::DisplayState;
use crate::display::generic_framebuffer::{self, Dimensions, GenericFramebuffer};

const FB_PATH: &str = "/dev/fb0";

// R500L5 has a 2.4-inch color IPS touch panel. Exact resolution needs hardware
// verification (probe with `fbset -fb /dev/fb0` on device); 240x320 is a common
// size for hotspot touchscreens of this class and matches FCC teardown photos.
//
// TODO: replace with verified sysfs paths from `ls /sys/devices/` on device.
// On the Orbic RC400L the panel lives at 78b6000.spi; the SDX55 SPI bus address
// will differ. Until verified this just logs a warning and does nothing.
const SYSFS_AUTOSLEEP: &str = "/sys/power/autosleep";
const FB_WIDTH: u32 = 240;
const FB_HEIGHT: u32 = 320;

#[derive(Copy, Clone, Default)]
struct Framebuffer;

#[async_trait]
impl GenericFramebuffer for Framebuffer {
    fn dimensions(&self) -> Dimensions {
        Dimensions {
            width: FB_WIDTH,
            height: FB_HEIGHT,
        }
    }

    async fn write_buffer(&mut self, buffer: Vec<(u8, u8, u8)>) {
        let mut raw_buffer = Vec::with_capacity(buffer.len() * 2);
        for (r, g, b) in buffer {
            let mut rgb565: u16 = (r as u16 & 0b11111000) << 8;
            rgb565 |= (g as u16 & 0b11111100) << 3;
            rgb565 |= (b as u16) >> 3;
            raw_buffer.extend(rgb565.to_le_bytes());
        }
        if let Err(e) = tokio::fs::write(FB_PATH, &raw_buffer).await {
            log::warn!("r500l5 display: write to {FB_PATH} failed: {e}");
        }
    }
}

fn spawn_keep_screen_on(task_tracker: &TaskTracker, shutdown_token: CancellationToken) {
    task_tracker.spawn(async move {
        // Disable autosleep so the screen stays on, then restore it on exit.
        // TODO: also poke the panel's own sleep_mode/bl_gpio sysfs nodes once
        // their SDX55 paths are known (probe with `ls /sys/devices/` on device).
        let autosleep_available = tokio::fs::metadata(SYSFS_AUTOSLEEP).await.is_ok();
        if !autosleep_available {
            warn!(
                "keep_screen_on: /sys/power/autosleep not found on R500L5 — screen-on unimplemented"
            );
            return;
        }
        if let Err(e) = tokio::fs::write(SYSFS_AUTOSLEEP, b"off").await {
            warn!("keep_screen_on: failed to disable autosleep: {e}");
        }
        shutdown_token.cancelled().await;
        if let Err(e) = tokio::fs::write(SYSFS_AUTOSLEEP, b"mem").await {
            warn!("keep_screen_on: failed to restore autosleep: {e}");
        }
    });
}

pub fn update_ui(
    task_tracker: &TaskTracker,
    config: &config::Config,
    shutdown_token: CancellationToken,
    ui_update_rx: Receiver<DisplayState>,
) {
    if config.keep_screen_on {
        spawn_keep_screen_on(task_tracker, shutdown_token.clone());
    }
    generic_framebuffer::update_ui(
        task_tracker,
        config,
        Framebuffer,
        shutdown_token,
        ui_update_rx,
    )
}
