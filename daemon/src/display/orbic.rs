use crate::config;
use crate::display::DisplayState;
use crate::display::generic_framebuffer::{self, Dimensions, GenericFramebuffer};
use async_trait::async_trait;
use log::{debug, warn};

use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

const FB_PATH: &str = "/dev/fb0";

// Sysfs paths for the Orbic RC400L display controller (QC SPI panel, verified on hardware).
// sleep_mode=0 and bl_gpio=0 indicate the display is blanked by the stock UI.
// Waking requires writing 1 to bl_gpio then sleep_mode.
const SYSFS_BASE: &str = "/sys/devices/78b6000.spi/spi_master/spi1/spi1.0";
const SYSFS_SLEEP_MODE: &str = "/sys/devices/78b6000.spi/spi_master/spi1/spi1.0/sleep_mode";
const SYSFS_BL_GPIO: &str = "/sys/devices/78b6000.spi/spi_master/spi1/spi1.0/bl_gpio";

// Global suspend control: "mem" = auto-suspend enabled, "off" = disabled.
// keep_screen_on forces "off" and restores "mem" on shutdown.
const SYSFS_AUTOSLEEP: &str = "/sys/power/autosleep";

const KEEP_SCREEN_POLL_MS: u64 = 500;

async fn read_sysfs_bool(path: &str) -> Option<bool> {
    match tokio::fs::read_to_string(path).await {
        Ok(s) => match s.trim() {
            "0" => Some(false),
            "1" => Some(true),
            _ => None,
        },
        Err(_) => None,
    }
}

async fn read_sysfs_string(path: &str) -> Option<String> {
    tokio::fs::read_to_string(path)
        .await
        .ok()
        .map(|s| s.trim().to_string())
}

async fn write_sysfs(path: &str, value: &[u8]) {
    if let Err(e) = tokio::fs::write(path, value).await {
        warn!(
            "keep_screen_on: write '{}' to {path}: {e}",
            std::str::from_utf8(value).unwrap_or("<bytes>")
        );
    }
}

fn spawn_keep_screen_on(task_tracker: &TaskTracker, shutdown_token: CancellationToken) {
    task_tracker.spawn(async move {
        if tokio::fs::metadata(SYSFS_BASE).await.is_err() {
            warn!("keep_screen_on: Orbic sysfs path not found: {SYSFS_BASE}");
            return;
        }

        let autosleep_available = tokio::fs::metadata(SYSFS_AUTOSLEEP).await.is_ok();
        if !autosleep_available {
            warn!("keep_screen_on: autosleep sysfs not found: {SYSFS_AUTOSLEEP}");
        }

        loop {
            if shutdown_token.is_cancelled() {
                break;
            }

            // sleep_mode=0 or bl_gpio=0 means the display blanked
            let sleep_mode = read_sysfs_bool(SYSFS_SLEEP_MODE).await;
            let bl_gpio = read_sysfs_bool(SYSFS_BL_GPIO).await;
            let needs_wake = matches!(sleep_mode, Some(false)) || matches!(bl_gpio, Some(false));

            let autosleep = if autosleep_available {
                read_sysfs_string(SYSFS_AUTOSLEEP).await
            } else {
                None
            };
            let needs_autosleep_off = matches!(autosleep.as_deref(), Some("mem"));

            if needs_wake || needs_autosleep_off {
                debug!(
                    "keep_screen_on: waking (sleep_mode={sleep_mode:?}, bl_gpio={bl_gpio:?}, autosleep={autosleep:?})"
                );
                if autosleep_available && needs_autosleep_off {
                    write_sysfs(SYSFS_AUTOSLEEP, b"off").await;
                }
                if needs_wake {
                    // Observed wake sequence: backlight first, then display resume
                    write_sysfs(SYSFS_BL_GPIO, b"1").await;
                    write_sysfs(SYSFS_SLEEP_MODE, b"1").await;
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(KEEP_SCREEN_POLL_MS)).await;
        }

        // Restore autosleep on shutdown
        if autosleep_available
            && matches!(
                read_sysfs_string(SYSFS_AUTOSLEEP).await.as_deref(),
                Some("off")
            )
        {
            debug!("keep_screen_on: restoring autosleep to \"mem\"");
            write_sysfs(SYSFS_AUTOSLEEP, b"mem").await;
        }
    });
}

#[derive(Copy, Clone, Default)]
struct Framebuffer;

#[async_trait]
impl GenericFramebuffer for Framebuffer {
    fn dimensions(&self) -> Dimensions {
        // TODO actually poll for this, maybe w/ fbset?
        Dimensions {
            height: 128,
            width: 128,
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

        tokio::fs::write(FB_PATH, &raw_buffer).await.unwrap();
    }
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
