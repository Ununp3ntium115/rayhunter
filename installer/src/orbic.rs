#[cfg(target_os = "windows")]
use std::io::stdin;

use std::io::ErrorKind;
use std::path::Path;
use std::time::{Duration, Instant};

use adb_client::{ADBDeviceExt, ADBUSBDevice, RustADBError, search_adb_devices};
use anyhow::{Context, Result, anyhow, bail};
use nusb::Interface;
use nusb::transfer::{Control, ControlType, Recipient, RequestBuffer};
use sha2::{Digest, Sha256};
use tokio::time::sleep;

use crate::RAYHUNTER_DAEMON_INIT;
use crate::connection::{DeviceConnection, check_free_space, install_config, install_wifi_tools};
use crate::output::{print, println};
use crate::util::open_usb_device;

pub const ORBIC_NOT_FOUND: &str = r#"No Orbic device found.
Make sure your device is plugged in and turned on.

If you're sure you've plugged in an Orbic device via USB, there may be a bug in
our installer. Please file a bug with the output of `lsusb` attached."#;

const ORBIC_BUSY: &str = r#"The Orbic is plugged in but is being used by another program.

Please close any program that might be using your USB devices.
If you have adb installed you may need to kill the adb daemon"#;

#[cfg(any(target_os = "macos", target_os = "windows"))]
const ORBIC_BUSY_MAC: &str = r#"Permission denied.

On macOS or windows this might be caused by another program using the Orbic.
Please close any program that might be using your Orbic.
If you have adb installed you may need to kill the adb daemon"#;

#[cfg(target_os = "windows")]
const WINDOWS_WARNING: &str = r#""WINDOWS IS NOT FULLY SUPPORTED

THIS MAY BRICK YOUR DEVICE

PLEASE INSTALL FROM MACOS OR LINUX INSTEAD IF POSSIBLE"#;

const VENDOR_ID: u16 = 0x05c6;
const PRODUCT_ID: u16 = 0xf601;

const INTERFACE: u8 = 1;

/// ADB-based connection wrapper for DeviceConnection trait
pub struct AdbConnection<'a> {
    device: &'a mut ADBUSBDevice,
}

impl DeviceConnection for AdbConnection<'_> {
    /// Runs through /bin/rootshell so commands execute as root (install_wifi_tools needs
    /// chmod on root-owned files). setup_rootshell must have succeeded before an
    /// AdbConnection is created; callers in this module (setup_rayhunter) enforce that
    /// ordering.
    async fn run_command(&mut self, command: &str) -> Result<String> {
        adb_command(
            self.device,
            &["/bin/rootshell", "-c", &format!("\"{command}\"")],
        )
    }

    async fn write_file(&mut self, path: &str, content: &[u8]) -> Result<()> {
        install_file(self.device, path, content).await
    }
}

#[cfg(target_os = "windows")]
const RNDIS_INTERFACE: u8 = 0;

#[cfg(not(target_os = "windows"))]
const RNDIS_INTERFACE: u8 = 1;

#[cfg(target_os = "windows")]
async fn confirm() -> Result<bool> {
    println!("{}", WINDOWS_WARNING);
    print!("Do you wish to proceed? Enter 'yes' to install> ");
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    Ok(input.trim() == "yes")
}

pub async fn install(reset_config: bool) -> Result<()> {
    println!(
        "WARNING: The orbic USB installer is not recommended for most usecases. Consider using ./installer orbic instead, unless you want ADB access for other purposes."
    );

    #[cfg(target_os = "windows")]
    {
        let confirmation = confirm().await?;
        if confirmation != true {
            println!("Install aborted. Your device has not been modified.");
            return Ok(());
        }
    }

    let mut adb_device = force_debug_mode().await?;
    print!("Installing rootshell... ");
    setup_rootshell(&mut adb_device).await?;
    println!("done");
    print!("Installing rayhunter... ");
    let mut adb_device = setup_rayhunter(adb_device, reset_config).await?;
    println!("done");
    print!("Testing rayhunter... ");
    test_rayhunter(&mut adb_device).await?;
    println!("done");
    Ok(())
}

/// Opens an interactive ADB shell, or runs `command` as root if it is non-empty.
pub async fn shell(command: &[String]) -> Result<()> {
    if command.is_empty() {
        println!(
            "WARNING: The orbic USB installer is not recommended for most usecases. Consider using ./installer util orbic-shell instead, unless you want ADB access for other purposes."
        );
        println!("opening shell");
    }

    let mut adb_device = get_adb().await?;
    if command.is_empty() {
        adb_device.shell(&mut std::io::stdin(), Box::new(std::io::stdout()))?;
    } else {
        let command = rootshell_command(&command.join(" "));
        let output = adb_command(&mut adb_device, &["/bin/rootshell", "-c", &command])?;
        print!("{output}");
    }
    Ok(())
}

/// Single-quotes `command` so the ADB shell passes it unchanged to `rootshell -c`, which then
/// runs it (including redirects like `> /usrdata/mode.cfg`) as root.
fn rootshell_command(command: &str) -> String {
    format!("'{}'", command.replace('\'', r"'\''"))
}

async fn force_debug_mode() -> Result<ADBUSBDevice> {
    println!("Forcing a switch into the debug mode to enable ADB");
    enable_command_mode()?;
    print!("ADB enabled, waiting for reboot... ");
    let mut adb_device = get_adb().await?;
    adb_setup_serial(&mut adb_device).await?;
    println!("it's alive!");
    print!("Waiting for atfwd_daemon to startup... ");
    adb_command(&mut adb_device, &["pgrep", "atfwd_daemon"])?;
    println!("done");
    Ok(adb_device)
}

async fn setup_rootshell(adb_device: &mut ADBUSBDevice) -> Result<()> {
    let rootshell_bin = crate::get_file!("FILE_ROOTSHELL");

    install_file(adb_device, "/bin/rootshell", rootshell_bin).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    adb_at_syscmd(adb_device, "chown root /bin/rootshell").await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    adb_at_syscmd(adb_device, "chmod 4755 /bin/rootshell").await?;
    let output = adb_command(adb_device, &["/bin/rootshell", "-c", "id"])?;
    if !output.contains("uid=0") {
        bail!("rootshell is not giving us root.");
    }
    Ok(())
}

async fn setup_rayhunter(mut adb_device: ADBUSBDevice, reset_config: bool) -> Result<ADBUSBDevice> {
    let rayhunter_daemon_bin = crate::get_file!("FILE_RAYHUNTER_DAEMON");

    {
        let mut conn = AdbConnection {
            device: &mut adb_device,
        };
        let needed_kb = rayhunter_daemon_bin.len() as u64 / 1024 + 5 * 1024;
        check_free_space(&mut conn, "/data", needed_kb).await?;
    }

    adb_at_syscmd(
        &mut adb_device,
        "mkdir -p /data/rayhunter/scripts /data/rayhunter/bin",
    )
    .await?;
    install_file(
        &mut adb_device,
        "/data/rayhunter/rayhunter-daemon",
        rayhunter_daemon_bin,
    )
    .await?;

    {
        let mut conn = AdbConnection {
            device: &mut adb_device,
        };
        install_config(&mut conn, "orbic", reset_config).await?;
        install_wifi_tools(&mut conn).await?;
    }

    install_file(
        &mut adb_device,
        "/etc/init.d/rayhunter_daemon",
        RAYHUNTER_DAEMON_INIT.as_bytes(),
    )
    .await?;
    install_file(
        &mut adb_device,
        "/etc/init.d/misc-daemon",
        include_bytes!("../../dist/scripts/misc-daemon"),
    )
    .await?;
    adb_at_syscmd(&mut adb_device, "chmod 755 /etc/init.d/rayhunter_daemon").await?;
    adb_at_syscmd(&mut adb_device, "chmod 755 /etc/init.d/misc-daemon").await?;
    println!("done");
    print!("Waiting for reboot... ");
    adb_at_syscmd(&mut adb_device, "shutdown -r -t 1 now").await?;
    // first wait for shutdown (it can take ~10s)
    tokio::time::timeout(Duration::from_secs(30), async {
        while let Ok(dev) = adb_echo_test(adb_device).await {
            adb_device = dev;
            sleep(Duration::from_secs(1)).await;
        }
    })
    .await
    .context("Orbic took too long to shutdown")?;
    // now wait for boot to finish
    get_adb().await
}

/// Test rayhunter on the device over adb without forwarding.
pub async fn test_rayhunter(adb_device: &mut ADBUSBDevice) -> Result<()> {
    const MAX_FAILURES: u32 = 10;
    let mut failures = 0;
    while failures < MAX_FAILURES {
        if let Ok(output) = adb_command(
            adb_device,
            &["wget", "-O", "-", "http://localhost:8080/index.html"],
        ) && output.contains("html")
        {
            return Ok(());
        }
        failures += 1;
        sleep(Duration::from_secs(3)).await;
    }
    bail!("timeout reached! failed to reach rayhunter, something went wrong :(")
}

async fn install_file(adb_device: &mut ADBUSBDevice, dest: &str, payload: &[u8]) -> Result<()> {
    const MAX_FAILURES: u32 = 5;
    let mut failures = 0;
    loop {
        match install_file_impl(adb_device, dest, payload).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                if failures > MAX_FAILURES {
                    return Err(e);
                } else {
                    sleep(Duration::from_secs(1)).await;
                    failures += 1;
                }
            }
        }
    }
}

async fn install_file_impl(
    adb_device: &mut ADBUSBDevice,
    dest: &str,
    mut payload: &[u8],
) -> Result<()> {
    // Prevent path traversal attacks by rejecting paths containing '..'.
    let dest_path = Path::new(dest);
    if dest_path
        .components()
        .any(|c| c == std::path::Component::ParentDir)
    {
        bail!("Invalid input: {}", dest_path.display());
    }
    let file_name = dest_path
        .file_name()
        .ok_or_else(|| anyhow!("{dest} does not have a file name"))?
        .to_str()
        .ok_or_else(|| anyhow!("{dest}'s file name is not UTF8"))?
        .to_owned();
    let push_tmp_path = format!("/tmp/{file_name}");
    let mut hasher = Sha256::new();
    hasher.update(payload);
    let file_hash_bytes = hasher.finalize();
    let file_hash = format!("{file_hash_bytes:x}");
    adb_device.push(&mut payload, &push_tmp_path)?;
    adb_at_syscmd(adb_device, &format!("mv {push_tmp_path} {dest}")).await?;
    let file_info = adb_device
        .stat(dest)
        .context("Failed to stat transfered file")?;
    if file_info.file_size == 0 {
        bail!("File transfer unsuccessful\nFile is empty");
    }
    let expected_size = payload.len() as u32;
    if file_info.file_size != expected_size {
        // Size mismatch most often means /data was full: `mv` from /tmp
        // silently left the old file in place on BusyBox when /data had no room.
        bail!(
            "File transfer unsuccessful\n\
             Size mismatch at {dest}: expected {expected_size}B, found {}B\n\
             Hint: the device's /data partition may be full. Free up space and re-run the installer.",
            file_info.file_size
        );
    }
    let output = adb_command(adb_device, &["sha256sum", dest])?;
    if !output.contains(&file_hash) {
        bail!(
            "File transfer unsuccessful\n\
             Bad hash at {dest}: expected {file_hash}, got {output}\n\
             Hint: if the hashes differ consistently, the device filesystem may be corrupted or full."
        );
    }
    Ok(())
}

fn adb_command(adb_device: &mut ADBUSBDevice, command: &[&str]) -> Result<String> {
    let mut buf = Vec::<u8>::new();
    adb_device.shell_command(command, &mut buf)?;
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

/// Creates an ADB interface instance.
///
/// This function waits for the ADB device then checks that an ADB shell command runs.
async fn get_adb() -> Result<ADBUSBDevice> {
    const MAX_FAILURES: u32 = 10;
    let mut failures = 0;
    loop {
        // Tethering changes the Orbic USB composition and can expose ADB under
        // a different product ID. Prefer the known product, then search for a
        // single ADB interface and require the Orbic Qualcomm vendor before
        // opening it. Do not use ADBUSBDevice::autodetect(), which can select
        // an unrelated Android device.
        let device = match ADBUSBDevice::new(VENDOR_ID, PRODUCT_ID) {
            Err(RustADBError::DeviceNotFound(_)) => match search_adb_devices()? {
                Some((vendor_id, product_id)) if is_orbic_usb_vendor(vendor_id) => {
                    ADBUSBDevice::new(vendor_id, product_id)
                }
                Some((vendor_id, product_id)) => Err(RustADBError::DeviceNotFound(format!(
                    "ADB device {vendor_id:04x}:{product_id:04x} is not an Orbic"
                ))),
                None => Err(RustADBError::DeviceNotFound(
                    "cannot find an Orbic ADB interface".into(),
                )),
            },
            result => result,
        };
        match device {
            Ok(dev) => match adb_echo_test(dev).await {
                Ok(dev) => return Ok(dev),
                Err(e) => {
                    if failures > MAX_FAILURES {
                        return Err(e);
                    } else {
                        sleep(Duration::from_secs(1)).await;
                        failures += 1;
                    }
                }
            },
            Err(RustADBError::IOError(e)) if e.kind() == ErrorKind::ResourceBusy => {
                bail!(ORBIC_BUSY);
            }
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            Err(RustADBError::IOError(e)) if e.kind() == ErrorKind::PermissionDenied => {
                bail!(ORBIC_BUSY_MAC);
            }
            Err(RustADBError::DeviceNotFound(_)) => {
                tokio::time::timeout(Duration::from_secs(30), wait_for_usb_device())
                    .await
                    .context("Timeout waiting for Orbic to reconnect")??;
            }
            Err(e) => {
                if failures > MAX_FAILURES {
                    return Err(e.into());
                } else {
                    sleep(Duration::from_secs(1)).await;
                    failures += 1;
                }
            }
        }
    }
}

async fn adb_echo_test(mut adb_device: ADBUSBDevice) -> Result<ADBUSBDevice> {
    let mut buf = Vec::<u8>::new();
    // Random string to echo
    let test_echo = "qwertyzxcvbnm";
    let thread = std::thread::spawn(move || {
        // This call to run a shell command is run on a separate thread because it can block
        // indefinitely until the command runs, which is undesirable.
        adb_device.shell_command(&["echo", test_echo], &mut buf)?;
        Ok::<(ADBUSBDevice, Vec<u8>), RustADBError>((adb_device, buf))
    });
    sleep(Duration::from_secs(1)).await;
    if thread.is_finished()
        && let Ok(Ok((dev, buf))) = thread.join()
        && let Ok(s) = std::str::from_utf8(&buf)
        && s.contains(test_echo)
    {
        return Ok(dev);
    }
    //  I'd like to kill the background thread here if that was possible.
    bail!("Could not communicate with the Orbic. Try disconnecting and reconnecting.");
}

async fn wait_for_usb_device() -> Result<()> {
    loop {
        if let Some((vendor_id, _product_id)) = search_adb_devices()?
            && is_orbic_usb_vendor(vendor_id)
        {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn adb_setup_serial(adb_device: &mut ADBUSBDevice) -> Result<()> {
    Ok(adb_device.get_transport_mut().claim_interface(INTERFACE)?)
}

async fn adb_at_syscmd(adb_device: &mut ADBUSBDevice, command: &str) -> Result<()> {
    adb_serial_cmd(adb_device, &format!("AT+SYSCMD={command}")).await
}

async fn adb_serial_cmd(adb_device: &mut ADBUSBDevice, command: &str) -> Result<()> {
    let mut data = String::new();
    data.push_str("\r\n");
    data.push_str(command);
    data.push_str("\r\n");

    let timeout = Duration::from_secs(2);

    // Set up the serial port appropriately
    adb_device
        .get_transport_mut()
        .send_usb_class_control_msg(INTERFACE, 0x22, 3, 1, &[], timeout)
        .context("Failed to send control request")?;

    // Send the command
    adb_device
        .get_transport_mut()
        .usb_bulk_write(INTERFACE, 0x2, data.as_bytes(), timeout)
        .context("Failed to write command")?;

    // The Orbic may split the echoed command and final response across multiple USB packets.
    // Read until a terminal modem response instead of assuming exactly one packet for each.
    let deadline = Instant::now() + timeout;
    let mut response = Vec::new();
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            bail!(
                "Timed out waiting for response to {command}: {}",
                String::from_utf8_lossy(&response)
            );
        }

        let mut packet = [0; 256];
        let bytes_read = adb_device
            .get_transport_mut()
            .usb_bulk_read(INTERFACE, 0x82, &mut packet, remaining)
            .context("Failed to read response")?;
        response.extend_from_slice(&packet[..bytes_read]);

        match serial_response_status(&response) {
            SerialResponseStatus::Pending => {}
            SerialResponseStatus::Success => break,
            SerialResponseStatus::Error => {
                bail!(
                    "Device rejected command {command}: {}",
                    String::from_utf8_lossy(&response)
                );
            }
        }
    }

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
enum SerialResponseStatus {
    Pending,
    Success,
    Error,
}

fn serial_response_status(response: &[u8]) -> SerialResponseStatus {
    let response = String::from_utf8_lossy(response);
    if response.contains("\r\nERROR\r\n") {
        SerialResponseStatus::Error
    } else if response.contains("\r\nOK\r\n") {
        SerialResponseStatus::Success
    } else {
        SerialResponseStatus::Pending
    }
}

/// Sends an AT command to the usb device over the serial port
///
/// First establish a USB handle and context by calling `open_orbic()`
pub async fn send_serial_cmd(interface: &Interface, command: &str) -> Result<()> {
    let mut data = String::new();
    data.push_str("\r\n");
    data.push_str(command);
    data.push_str("\r\n");

    let timeout = Duration::from_secs(2);

    let enable_serial_port = Control {
        control_type: ControlType::Class,
        recipient: Recipient::Interface,
        request: 0x22,
        value: 3,
        index: 1,
    };

    // Set up the serial port appropriately
    interface
        .control_out_blocking(enable_serial_port, &[], timeout)
        .context("Failed to send control request")?;

    // Send the command
    tokio::time::timeout(timeout, interface.bulk_out(0x2, data.as_bytes().to_vec()))
        .await
        .context("Timed out writing command")?
        .into_result()
        .context("Failed to write command")?;

    let deadline = Instant::now() + timeout;
    let mut response = Vec::new();
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            bail!(
                "Timed out waiting for response to {command}: {}",
                String::from_utf8_lossy(&response)
            );
        }
        let packet =
            tokio::time::timeout(remaining, interface.bulk_in(0x82, RequestBuffer::new(256)))
                .await
                .context("Timed out reading response")?
                .into_result()
                .context("Failed to read response")?;
        response.extend_from_slice(&packet);
        match serial_response_status(&response) {
            SerialResponseStatus::Pending => {}
            SerialResponseStatus::Success => break,
            SerialResponseStatus::Error => {
                bail!(
                    "Device rejected command {command}: {}",
                    String::from_utf8_lossy(&response)
                );
            }
        }
    }

    Ok(())
}

/// Send a command to switch the device into generic mode, exposing serial
///
/// If the device reboots while the command is still executing you may get a pipe error here, not sure what to do about this race condition.
pub fn enable_command_mode() -> Result<()> {
    if open_orbic()?.is_some() {
        println!("Device already in command mode. Doing nothing...");
        return Ok(());
    }

    let timeout = Duration::from_secs(1);

    if let Some(device) = open_usb_device(VENDOR_ID, 0xf626)? {
        let enable_command_mode = Control {
            control_type: ControlType::Vendor,
            recipient: Recipient::Device,
            request: 0xa0,
            value: 0,
            index: 0,
        };
        let interface = device
            .detach_and_claim_interface(RNDIS_INTERFACE)
            .context("detach_and_claim_interface(1) failed")?;
        if let Err(e) = interface.control_out_blocking(enable_command_mode, &[], timeout) {
            // If the device reboots while the command is still executing we
            // may get a pipe error here
            if e == nusb::transfer::TransferError::Stall {
                return Ok(());
            }
            bail!("Failed to send device switch control request: {0}", e)
        }
        return Ok(());
    }

    bail!(ORBIC_NOT_FOUND);
}

/// Get an Interface for the orbic device
pub fn open_orbic() -> Result<Option<Interface>> {
    // Device after initial mode switch
    if let Some(device) = open_usb_device(VENDOR_ID, PRODUCT_ID)? {
        let interface = device
            .detach_and_claim_interface(INTERFACE) // will reattach drivers on release
            .context("detach_and_claim_interface(1) failed")?;
        return Ok(Some(interface));
    }

    // Device with rndis enabled as well
    if let Some(device) = open_usb_device(VENDOR_ID, 0xf622)? {
        let interface = device
            .detach_and_claim_interface(INTERFACE) // will reattach drivers on release
            .context("detach_and_claim_interface(1) failed")?;
        return Ok(Some(interface));
    }

    Ok(None)
}

fn is_orbic_usb_vendor(vendor_id: u16) -> bool {
    vendor_id == VENDOR_ID
}

#[cfg(test)]
mod tests {
    #[test]
    fn rootshell_command_single_quotes() {
        assert_eq!(
            super::rootshell_command("echo 3 > /usrdata/mode.cfg"),
            "'echo 3 > /usrdata/mode.cfg'"
        );
        assert_eq!(
            super::rootshell_command("echo 'a b'"),
            r"'echo '\''a b'\'''"
        );
    }

    use super::{SerialResponseStatus, is_orbic_usb_vendor, serial_response_status};

    #[test]
    fn accepts_orbic_vendor_regardless_of_usb_product_mode() {
        assert!(is_orbic_usb_vendor(0x05c6));
    }

    #[test]
    fn rejects_unrelated_usb_vendors() {
        assert!(!is_orbic_usb_vendor(0x18d1));
    }

    #[test]
    fn fragmented_serial_response_is_pending_until_ok_marker_arrives() {
        let mut response = Vec::from(&b"\r\nAT+SYSCMD=mv /tmp/rootshell /bin/rootshell\r\n"[..]);

        assert_eq!(
            serial_response_status(&response),
            SerialResponseStatus::Pending
        );

        response.extend_from_slice(b"\r\nOK\r\n");

        assert_eq!(
            serial_response_status(&response),
            SerialResponseStatus::Success
        );
    }

    #[test]
    fn serial_error_response_is_reported_as_error() {
        assert_eq!(
            serial_response_status(b"\r\nERROR\r\n"),
            SerialResponseStatus::Error
        );
    }
}
