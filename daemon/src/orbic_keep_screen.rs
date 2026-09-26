use log::{error, info, warn};
use std::path::PathBuf;
use tokio::time::{Duration, interval};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

const POLL_INTERVAL_MS: u64 = 1000;
// bl_power: 0 = unblank (on), 1..4 = various blank states (FB_BLANK_*)
const BL_POWER_UNBLANK: &[u8] = b"0";

fn find_backlight_power_paths() -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir("/sys/class/backlight") else {
        return Vec::new();
    };
    entries
        .filter_map(|e| e.ok())
        .map(|e| e.path().join("bl_power"))
        .filter(|p| p.exists())
        .collect()
}

/// Spawns a task that re-enables the Orbic display backlight whenever it blanks.
/// Returns immediately if no backlight sysfs nodes are found.
pub fn run_keep_screen_on_task(
    task_tracker: &TaskTracker,
    cancellation_token: CancellationToken,
) {
    let paths = find_backlight_power_paths();
    if paths.is_empty() {
        warn!("keep_screen_on: no backlight nodes found under /sys/class/backlight — task not started");
        return;
    }
    info!(
        "keep_screen_on: monitoring {} backlight node(s): {:?}",
        paths.len(),
        paths
    );

    task_tracker.spawn(async move {
        let mut ticker = interval(Duration::from_millis(POLL_INTERVAL_MS));
        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    info!("keep_screen_on: shutting down");
                    return;
                }
                _ = ticker.tick() => {}
            }

            for path in &paths {
                let val = match tokio::fs::read(path).await {
                    Ok(v) => v,
                    Err(e) => {
                        error!("keep_screen_on: read {}: {e}", path.display());
                        continue;
                    }
                };
                // Only write if the screen is actually blanked (avoids unnecessary sysfs writes)
                if val.trim_ascii() != BL_POWER_UNBLANK
                    && let Err(e) = tokio::fs::write(path, BL_POWER_UNBLANK).await
                {
                    error!("keep_screen_on: write {}: {e}", path.display());
                }
            }
        }
    });
}
