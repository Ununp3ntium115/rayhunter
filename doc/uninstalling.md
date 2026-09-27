# Uninstalling

There is no automated uninstallation routine, so this page documents the routine for some devices.

## Orbic

Run `./installer util orbic-shell --admin-password mypassword`. Refer to the
installation instructions for how to find out the admin password.

Inside, run:

```shell
echo 3 > /usrdata/mode.cfg  # only relevant if you previously installed via ADB installer
rm -rf /data/rayhunter /etc/init.d/rayhunter_daemon /bin/rootshell
reboot
```

Your device is now Rayhunter-free, and should no longer be rooted.

## TPLink

1. Run `./installer util tplink-shell` to obtain rootshell on the device.
2. Remove Rayhunter files and the init script:

   ```shell
   rm /data/rayhunter   # removes the symlink (recordings stay on the SD card)
   rm /etc/init.d/rayhunter_daemon
   update-rc.d rayhunter_daemon remove
   ```

   If you also want to erase your recordings from the SD card, run the following
   **before** removing the symlink:

   ```shell
   rm -rf "$(readlink /data/rayhunter)"
   ```

3. (hardware revision v4.0+ only) In `Settings > NAT Settings > Port Triggers` in TP-Link's admin UI, remove any leftover port triggers.

## UZ801

0. (Optional): Back up the qmdl folder with all of the captures:
`adb pull /data/rayhunter/qmdl .`
1. Run `adb shell` to get a root shell on the device
2. Delete the /data/rayhunter folder: `rm -rf /data/rayhunter`
3. Modify the initmifiservice.sh script to remove the rayhunter 
startup line:
```sh
mount -o remount,rw /system
busybox vi /system/bin/initmifiservice.sh
```
Then type 999G (shift+g), then type dd. Then press the colon key (:) and type wq. Finally, press Enter.
4. Lastly, run `setprop persist.sys.usb.config rndis`.
5. Type `reboot` to reboot the device.

## T-Mobile TMOHS1 / Wingtech CT2MHS01

First obtain a root shell on the device. The T-Mobile TMOHS1 and Wingtech CT2MHS01 both
use ADB:

```bash
# T-Mobile TMOHS1
./installer util tmobile-start-adb --admin-password Admin0123!
adb shell

# Wingtech CT2MHS01
./installer util wingtech-start-adb --admin-ip 192.168.0.1 --admin-password <password>
adb shell
```

Inside the shell, run:

```shell
rm -rf /data/rayhunter
rm -f /etc/init.d/rayhunter_daemon /etc/init.d/misc-daemon
update-rc.d rayhunter_daemon remove
reboot
```

## PinePhone

The PinePhone installer uses ADB:

```bash
adb shell
```

Inside the shell, run:

```shell
rm -rf /data/rayhunter
rm -f /etc/init.d/rayhunter_daemon /etc/init.d/misc-daemon
reboot
```
