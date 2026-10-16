use crate::{
    battery::{BatteryState, get_level_from_percentage_file},
    error::RayhunterError,
};
use std::path::Path;

const BATTERY_CAPACITY_FILE: &str = "/sys/class/power_supply/battery/capacity";
const USB_PRESENT_FILE: &str = "/sys/class/power_supply/usb/present";
const DC_PRESENT_FILE: &str = "/sys/class/power_supply/dc/present";

pub async fn get_battery_state() -> Result<BatteryState, RayhunterError> {
    let level =
        get_level_from_percentage_file(Path::new(BATTERY_CAPACITY_FILE)).await?;

    let usb_present = tokio::fs::read_to_string(USB_PRESENT_FILE)
        .await
        .map(|s| s.trim_end().starts_with('1'))
        .unwrap_or(false);
    let dc_present = tokio::fs::read_to_string(DC_PRESENT_FILE)
        .await
        .map(|s| s.trim_end().starts_with('1'))
        .unwrap_or(false);

    Ok(BatteryState {
        level,
        is_plugged_in: usb_present || dc_present,
    })
}
