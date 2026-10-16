#!/bin/sh
# Rayhunter pre-start: persistent root shell + ethernet WAN uplink
# Deployed to /data/rayhunter/scripts/prestart.sh on R500L5 (Orbic RC500L5).
# Called by /etc/init.d/rayhunter_daemon before the daemon starts.

# Fix devpts ptmxmode so adb shell gets a real TTY (required for su)
mount -o remount,ptmxmode=666 devpts /dev/pts 2>/dev/null || true

# Root shell on port 24 for diagnostics
busybox nc -ll -p 24 -e /bin/sh &

# Accept traffic arriving on eth0 (default INPUT policy is DROP; wlan1 and
# bridge0 have ACCEPT rules but eth0 does not since it was added to LAN bridge
# during normal hotspot operation).
# Added early and again at T+15 because the system firewall script re-initialises
# chains after the network comes up and wipes our early insert.
add_eth0_rules() {
    iptables -I INPUT  -i eth0 -j ACCEPT 2>/dev/null || true
    iptables -I FORWARD -i eth0 -j ACCEPT 2>/dev/null || true
    iptables -I FORWARD -o eth0 -j ACCEPT 2>/dev/null || true
    # Allow ICMP (ping) so network reachability testing works
    iptables -C INPUT -p icmp -j ACCEPT 2>/dev/null || \
        iptables -I INPUT -p icmp -j ACCEPT 2>/dev/null || true
}
add_eth0_rules

# Convert eth0 from LAN bridge to WAN uplink
ip link set eth0 nomaster 2>/dev/null || true
ip addr flush dev eth0 2>/dev/null || true

# DHCP for WAN address from upstream router
udhcpc -i eth0 -n -q -t 10 2>/dev/null || true

# Re-assert the metric-5 default route and eth0/ICMP firewall rules after a delay.
# wifi_station's demote_cellular_default() runs ~5s after daemon start and
# demotes ALL non-wlan1 default routes to metric 1000.  The system firewall
# script also re-initialises iptables chains after the network comes up.
# We fire at 15s to restore both after everything settles.
(sleep 15
 GW=$(ip -4 addr show eth0 | awk '/inet / { split($2,a,"[./]"); print a[1]"."a[2]"."a[3]".1"; exit }')
 if [ -n "$GW" ]; then
     ip route del default dev eth0 2>/dev/null || true
     ip route add default via "$GW" dev eth0 metric 5 2>/dev/null || true
 fi
 # add_eth0_rules is inherited by this subshell; re-call to restore all rules.
 add_eth0_rules) &
