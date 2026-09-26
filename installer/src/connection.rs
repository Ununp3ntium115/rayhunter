use std::future::Future;
use std::net::SocketAddr;

use anyhow::{Result, bail};

use crate::output::{print, println};

/// Abstraction for device communication (telnet or ADB)
pub trait DeviceConnection {
    /// Run a shell command and return its output
    fn run_command(&mut self, command: &str) -> impl Future<Output = Result<String>> + Send;

    /// Write a file to the device
    fn write_file(&mut self, path: &str, content: &[u8])
    -> impl Future<Output = Result<()>> + Send;
}

/// Fails if the filesystem containing `path` has fewer than `needed_kb` kilobytes free.
/// Silently succeeds when `df` output cannot be parsed; the subsequent push will report its
/// own error if the device really is out of space.
pub async fn check_free_space<C: DeviceConnection>(
    conn: &mut C,
    path: &str,
    needed_kb: u64,
) -> Result<()> {
    // `tail -n 1` handles BusyBox `df` wrapping long filesystem names to a second line.
    let output = conn
        .run_command(&format!("df -k '{path}' 2>/dev/null | tail -n 1"))
        .await?;
    let Some(available_kb) = parse_df_available_kb(&output) else {
        // df output is unparseable on this firmware; proceed and let the push fail on its own.
        return Ok(());
    };
    if available_kb < needed_kb {
        bail!(
            "Not enough free space on the filesystem containing '{path}'.\n\
             Need at least {} MB but only {} MB available.\n\
             \n\
             To free space, delete old recordings on the device:\n  \
               rm -rf /data/rayhunter/qmdl/*",
            needed_kb / 1024,
            available_kb / 1024,
        );
    }
    Ok(())
}

/// Parse the "Available" column (4th field, 0-indexed col 3) from a POSIX `df -k` data line.
fn parse_df_available_kb(output: &str) -> Option<u64> {
    // POSIX df -k columns: Filesystem  1K-blocks  Used  Available  Use%  Mountpoint
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4
            && let Ok(kb) = parts[3].parse::<u64>()
        {
            return Some(kb);
        }
    }
    None
}

/// Check if a file exists using a DeviceConnection
pub async fn file_exists<C: DeviceConnection>(conn: &mut C, path: &str) -> bool {
    conn.run_command(&format!("test -f '{path}' && echo exists || echo missing"))
        .await
        .map(|output| output.contains("exists"))
        .unwrap_or(false)
}

/// Shared config installation logic. Installs to /data/rayhunter/config.toml which resolves
/// through the symlink to the actual data directory.
pub async fn install_config<C: DeviceConnection>(
    conn: &mut C,
    device_type: &str,
    reset_config: bool,
) -> Result<()> {
    let config_path = "/data/rayhunter/config.toml";
    if reset_config || !file_exists(conn, config_path).await {
        let config = crate::CONFIG_TOML.replace(
            r#"#device = "orbic""#,
            &format!(r#"device = "{device_type}""#),
        );
        conn.write_file(config_path, config.as_bytes()).await?;
    } else {
        println!("Config file already exists, skipping (use --reset-config to overwrite)");
    }
    Ok(())
}

/// Install wifi tools (wpa_supplicant, wpa_cli, iw) to /data/rayhunter/bin.
///
/// Skips any binary that is already present on the device (e.g. provided by firmware),
/// since those may be newer or better-integrated than the bundled versions.
///
/// In debug builds the wpa-supplicant binaries may not be bundled (build.rs sets the
/// env vars to empty in that case); when so, this is a no-op so devs don't have to
/// build wpa-supplicant just to install on Orbic.
pub async fn install_wifi_tools<C: DeviceConnection>(conn: &mut C) -> Result<()> {
    if env!("FILE_WPA_SUPPLICANT").is_empty() {
        println!("wifi tools were not built into this installer, skipping");
        return Ok(());
    }
    let tools: &[(&str, &str, &[u8])] = &[
        (
            "wpa_supplicant",
            "/data/rayhunter/bin/wpa_supplicant",
            crate::get_file!("FILE_WPA_SUPPLICANT"),
        ),
        (
            "wpa_cli",
            "/data/rayhunter/bin/wpa_cli",
            crate::get_file!("FILE_WPA_CLI"),
        ),
        ("iw", "/data/rayhunter/bin/iw", crate::get_file!("FILE_IW")),
    ];
    for &(name, dest, payload) in tools {
        if device_has_binary(conn, name).await {
            println!("{name} already on device, skipping");
        } else {
            conn.write_file(dest, payload).await?;
            conn.run_command(&format!("chmod +x {dest}")).await?;
        }
    }
    Ok(())
}

async fn device_has_binary<C: DeviceConnection>(conn: &mut C, name: &str) -> bool {
    // `command -v` is a POSIX shell builtin, so it works on minimal busybox firmware
    // even when /usr/bin/which is absent.
    conn.run_command(&format!(
        "\"command -v {name} >/dev/null 2>&1 && echo FOUND || echo MISSING\""
    ))
    .await
    .map(|out| out.contains("FOUND"))
    .unwrap_or(false)
}

/// Check if a directory exists using a DeviceConnection
pub async fn dir_exists<C: DeviceConnection>(conn: &mut C, path: &str) -> bool {
    conn.run_command(&format!("test -d '{path}' && echo exists || echo missing"))
        .await
        .map(|output| output.contains("exists"))
        .unwrap_or(false)
}

/// Check if a path is a symlink using a DeviceConnection
pub async fn is_symlink<C: DeviceConnection>(conn: &mut C, path: &str) -> bool {
    conn.run_command(&format!("test -L '{path}' && echo yes || echo no"))
        .await
        .map(|output| output.contains("yes"))
        .unwrap_or(false)
}

/// Read the target of a symlink using a DeviceConnection
pub async fn readlink<C: DeviceConnection>(conn: &mut C, path: &str) -> Result<String> {
    // Use a prefix marker to find the actual output line, since some shells (TP-Link) echo
    // back the command and run_command appends protocol lines.
    let output = conn
        .run_command(&format!("echo RL:$(readlink '{path}')"))
        .await?;

    for line in output.lines() {
        if let Some(target) = line.trim().strip_prefix("RL:") {
            return Ok(target.to_string());
        }
    }

    bail!("unexpected readlink output: {output:?}");
}

/// Set up the data directory at `data_dir` and create a symlink from `/data/rayhunter` to it.
///
/// Handles migration from old locations:
/// - If `/data/rayhunter` is a real directory, moves its contents to `data_dir`
/// - If `/data/rayhunter` is a symlink to a different location, moves from the old target
/// - If `/data/rayhunter` doesn't exist, just creates the symlink
/// - If `/data/rayhunter` is a symlink to `data_dir`, does nothing
pub async fn setup_data_directory<C: DeviceConnection>(conn: &mut C, data_dir: &str) -> Result<()> {
    if data_dir == "/data/rayhunter" {
        bail!("data_dir must not be /data/rayhunter");
    }

    if data_dir.contains("'") {
        bail!("data_dir must not contain an apostrophe (')");
    }

    // Determine where old data lives, if anywhere
    let old_data_source = if is_symlink(conn, "/data/rayhunter").await {
        let current_target = readlink(conn, "/data/rayhunter").await?;
        if current_target == data_dir {
            println!("Data directory already configured at {data_dir}");
            return Ok(());
        }
        conn.run_command("rm -f /data/rayhunter").await?;
        // The old symlink target is where data actually lives
        if dir_exists(conn, &current_target).await {
            Some(current_target)
        } else {
            None
        }
    } else if dir_exists(conn, "/data/rayhunter").await {
        if dir_exists(conn, data_dir).await {
            bail!("Both /data/rayhunter and {data_dir} exist and are directories.");
        }
        // Real directory (pre-migration Orbic state)
        Some("/data/rayhunter".to_string())
    } else {
        None
    };

    // Migrate old data if present
    if let Some(old_source) = &old_data_source {
        // Stop rayhunter-daemon so it doesn't write during migration.
        // The device will be rebooted at the end of installation anyway.
        print!("Stopping rayhunter-daemon ... ");
        let _ = conn
            .run_command("/etc/init.d/rayhunter_daemon stop 2>/dev/null; true")
            .await;
        println!("ok");

        print!("Migrating data from {old_source} to {data_dir} ... ");

        // mv old data into its place. If source and destination are on the same filesystem,
        // this is an instant rename.
        // XXX: DeviceConnection::run_command does not expose the exit code of the ran command. It
        // probably should, or a utility for it should exist?
        let mv_output = conn
            .run_command(&format!("mv '{old_source}' '{data_dir}' && echo MV_OK"))
            .await?;
        if mv_output.contains("MV_OK") {
            println!("ok");
        } else {
            bail!("Failed to move data from {old_source} to {data_dir}:\n{mv_output}");
        }
    } else {
        // No migration needed, just ensure the target directory exists
        conn.run_command(&format!("mkdir -p '{data_dir}'")).await?;
    }

    // Create the symlink
    print!("Creating symlink /data/rayhunter -> {data_dir} ... ");
    conn.run_command("mkdir -p /data").await?;
    conn.run_command(&format!("ln -sf '{data_dir}' /data/rayhunter"))
        .await?;
    println!("ok");

    Ok(())
}

/// Telnet-based connection wrapper
pub struct TelnetConnection {
    pub addr: SocketAddr,
    pub wait_for_prompt: bool,
}

impl TelnetConnection {
    pub fn new(addr: SocketAddr, wait_for_prompt: bool) -> Self {
        Self {
            addr,
            wait_for_prompt,
        }
    }
}

impl DeviceConnection for TelnetConnection {
    async fn run_command(&mut self, command: &str) -> Result<String> {
        crate::util::telnet_send_command_with_output(
            self.addr,
            command,
            self.wait_for_prompt,
            std::time::Duration::from_secs(10),
        )
        .await
    }

    async fn write_file(&mut self, path: &str, content: &[u8]) -> Result<()> {
        crate::util::telnet_send_file(self.addr, path, content, self.wait_for_prompt).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_df_posix_output() {
        let output = "/dev/block/xxx  512000  400000  112000  78% /data";
        assert_eq!(parse_df_available_kb(output), Some(112000));
    }

    #[test]
    fn parse_df_busybox_wrapped_filesystem_name() {
        // BusyBox df wraps long names; tail -n 1 gives us only the numbers line.
        let output = "112000";
        // With just one token this won't parse (need 4 fields), but tail -n1 gives the data row.
        let output2 = "/dev/ubi0_0      204800   98304   106496  48% /data";
        assert_eq!(parse_df_available_kb(output2), Some(106496));
    }

    #[test]
    fn parse_df_returns_none_for_garbage() {
        assert_eq!(parse_df_available_kb("no numbers here"), None);
        assert_eq!(parse_df_available_kb(""), None);
    }
}
