# Rayhunter Orbic/shared backlog map

Live snapshot: EFForg/rayhunter has 91 open issues in the current API result. The fork `Ununp3ntium115/rayhunter` has GitHub Issues disabled and one closed PR; upstream issues are used as the evidence source.

Selected for this board: 67 open issue/feature lanes (direct Orbic or shared/general), plus 3 reviewed branch lanes and 1 source TODO/maintenance lane. Unrelated device-only issues were excluded unless their text indicated shared/general impact.

## Board

- Board: `rayhunter-orbic` — Rayhunter Orbic & Shared Features
- Parent gate: `t_bf6d3c0a` — blocked intentionally until the operator starts a bounded batch
- Child cards: 71, currently held in `todo` behind the parent gate
- Every child includes evidence, pseudocode, acceptance criteria, and a receipt requirement.

## Issue and feature lanes

| Issue | Scope | Title | Board card | Evidence |
|---:|---|---|---|---|
| #1160 | direct-orbic | Full screen status incorrect while off but charging on orbic | `t_e7edf1c3` | https://github.com/EFForg/rayhunter/issues/1160 |
| #1144 | shared-general | Extract shared components for GUI | `t_4ec82505` | https://github.com/EFForg/rayhunter/issues/1144 |
| #1139 | shared-general | Refactor analysis format to permit more than one event per (analyzer, msg) | `t_f0f1b9e7` | https://github.com/EFForg/rayhunter/issues/1139 |
| #1137 | direct-orbic | add a device connection screen to the GUI installer | `t_d00b5a12` | https://github.com/EFForg/rayhunter/issues/1137 |
| #1133 | shared-general | Persist device metadata for recordings | `t_78ae8d77` | https://github.com/EFForg/rayhunter/issues/1133 |
| #1129 | direct-orbic | RC400L documentation mentions it supports 5G bands when it is not a 5G device | `t_74c07a6e` | https://github.com/EFForg/rayhunter/issues/1129 |
| #1072 | shared-general | LPP heuristic | `t_c8163937` | https://github.com/EFForg/rayhunter/issues/1072 |
| #1047 | shared-general | Build the first GPS sensor/client (browser location API) | `t_3f4ea929` | https://github.com/EFForg/rayhunter/issues/1047 |
| #1033 | direct-orbic | Wifi Client option fails to activate, and makes web server inaccessible. | `t_2f594a51` | https://github.com/EFForg/rayhunter/issues/1033 |
| #1031 | shared-general | A full release workflow | `t_f581f5b7` | https://github.com/EFForg/rayhunter/issues/1031 |
| #1017 | shared-general | Add documentation about how to add a heuristic | `t_195c1b2d` | https://github.com/EFForg/rayhunter/issues/1017 |
| #1013 | shared-general | shove unparsed 3g/2g traffic into pcap | `t_44836819` | https://github.com/EFForg/rayhunter/issues/1013 |
| #1000 | shared-general | Wifi based heuristics | `t_27bf0438` | https://github.com/EFForg/rayhunter/issues/1000 |
| #990 | shared-general | ROADMAP | `t_26e082ff` | https://github.com/EFForg/rayhunter/issues/990 |
| #984 | direct-orbic | Opt-in HTTP tap for the raw /dev/diag byte stream | `t_ad85afc6` | https://github.com/EFForg/rayhunter/issues/984 |
| #920 | direct-orbic | Orbic USB installer is becoming harder to maintain | `t_ebe3a77e` | https://github.com/EFForg/rayhunter/issues/920 |
| #916 | direct-orbic | Feature: optional keep-screen-on for Orbic | `t_42f6a384` | https://github.com/EFForg/rayhunter/issues/916 |
| #914 | shared-general | Feature: Add additional ui_level to show preset/custom PNGs for Low/Medium/High detections. | `t_8817bd8d` | https://github.com/EFForg/rayhunter/issues/914 |
| #901 | direct-orbic | [Installer] Orbic RC400L ORB400L_V1.2.8_BVZRT: orbic + orbic-usb both fail (telnet + AT+SYSCMD rootshell) | `t_4550acbc` | https://github.com/EFForg/rayhunter/issues/901 |
| #880 | direct-orbic | add on-device function to fully dis/enable wifi on the Orbic RC400L | `t_9634fe70` | https://github.com/EFForg/rayhunter/issues/880 |
| #868 | shared-general | We need more contributors | `t_1931edfd` | https://github.com/EFForg/rayhunter/issues/868 |
| #833 | shared-general | UZ801 displays Test Heuristic Warnings without SIM Card. | `t_19eb5726` | https://github.com/EFForg/rayhunter/issues/833 |
| #807 | direct-orbic | Unable to switch from teathered to wifi install | `t_f932760f` | https://github.com/EFForg/rayhunter/issues/807 |
| #785 | shared-general | M7350 initialization failed error code -1 test heuristic shows no errors | `t_5c633ea2` | https://github.com/EFForg/rayhunter/issues/785 |
| #769 | direct-orbic | PowerShell not showing up in contextual menu on Windows 11 Pro | `t_7c384844` | https://github.com/EFForg/rayhunter/issues/769 |
| #762 | direct-orbic | Keep orbic-usb installer alive | `t_567716be` | https://github.com/EFForg/rayhunter/issues/762 |
| #756 | shared-general | [feature] Add heuristic for Timing Advance abnormalities | `t_6d62d9d5` | https://github.com/EFForg/rayhunter/issues/756 |
| #737 | direct-orbic | How to uninstall using orbic-shell? | `t_e2fd4edf` | https://github.com/EFForg/rayhunter/issues/737 |
| #736 | shared-general | Proxmity/ Signal Source Finder Module | `t_508f44cb` | https://github.com/EFForg/rayhunter/issues/736 |
| #733 | shared-general | build-firmware-devel produces too large binaries for moxee | `t_b82a8be2` | https://github.com/EFForg/rayhunter/issues/733 |
| #732 | direct-orbic | EFF Logo bugs out during charging | `t_f8ccd66e` | https://github.com/EFForg/rayhunter/issues/732 |
| #730 | shared-general | rayhunter-check: can not read the PCAP file from QCSuper although no Encapsulation | `t_a710ee6d` | https://github.com/EFForg/rayhunter/issues/730 |
| #719 | shared-general | rayhunter-check not give the warning even if SIB7 appear | `t_9a527334` | https://github.com/EFForg/rayhunter/issues/719 |
| #693 | direct-orbic | Network installer fails on Orbic RC400L: "exit code 0" not found in: command done, exit code 1 | `t_9c5b0cea` | https://github.com/EFForg/rayhunter/issues/693 |
| #607 | shared-general | Implement capture storage management in rayhunter-daemon | `t_399d8615` | https://github.com/EFForg/rayhunter/issues/607 |
| #591 | shared-general | [Feat] Add steps to documentation for developing new devices | `t_5c902206` | https://github.com/EFForg/rayhunter/issues/591 |
| #589 | direct-orbic | feat: Turn Orbic WiFi from AP to Client mode | `t_06b220a9` | https://github.com/EFForg/rayhunter/issues/589 |
| #557 | shared-general | IMSI Requested with out attach - False positive. | `t_711b55da` | https://github.com/EFForg/rayhunter/issues/557 |
| #543 | direct-orbic | Disconnected after Identity Request without Auth Accept | `t_ae44316f` | https://github.com/EFForg/rayhunter/issues/543 |
| #539 | direct-orbic | Prevent screen shut off (when plugged in) | `t_bd6dfd89` | https://github.com/EFForg/rayhunter/issues/539 |
| #534 | shared-general | New criterions and features | `t_115f3a40` | https://github.com/EFForg/rayhunter/issues/534 |
| #523 | direct-orbic | Cant access Orbic via ADB after enabling tethering | `t_73b43ef2` | https://github.com/EFForg/rayhunter/issues/523 |
| #510 | direct-orbic | Installer v0.5.0 fails unhelpfully on full file system | `t_98290123` | https://github.com/EFForg/rayhunter/issues/510 |
| #501 | shared-general | Allow users to set display name and notes for rayhunter recordings | `t_37132e68` | https://github.com/EFForg/rayhunter/issues/501 |
| #480 | shared-general | empty pcap and unhandled GsmRrSignallingMessage on PinePhone | `t_14309480` | https://github.com/EFForg/rayhunter/issues/480 |
| #462 | shared-general | Develop FlashCatch heuristic | `t_22e23b7a` | https://github.com/EFForg/rayhunter/issues/462 |
| #457 | shared-general | Support layer 2 MAC packets | `t_3ec71cbd` | https://github.com/EFForg/rayhunter/issues/457 |
| #441 | shared-general | Multiple ideas for reducing binary size | `t_96829736` | https://github.com/EFForg/rayhunter/issues/441 |
| #398 | shared-general | New possible indicators | `t_404a1775` | https://github.com/EFForg/rayhunter/issues/398 |
| #377 | shared-general | [Feature Request]: Move telcom parser to it's own crate | `t_6493de12` | https://github.com/EFForg/rayhunter/issues/377 |
| #363 | shared-general | [Feature Request] Add Count of Severity Level Warnings to UI | `t_65be73ca` | https://github.com/EFForg/rayhunter/issues/363 |
| #326 | shared-general | [Feature Request]: Record and show current and neighboring cells | `t_89f5f754` | https://github.com/EFForg/rayhunter/issues/326 |
| #300 | shared-general | [Feature Request]: Allow users to re-run analysis on old recordings | `t_54ebbe21` | https://github.com/EFForg/rayhunter/issues/300 |
| #261 | direct-orbic | [Feature Request]: A hardware-based CI testing rig for all supported devices | `t_6ea00559` | https://github.com/EFForg/rayhunter/issues/261 |
| #259 | direct-orbic | [Feature Request]: Orbic RC400L - change LTE bands for Europe | `t_014606e0` | https://github.com/EFForg/rayhunter/issues/259 |
| #207 | direct-orbic | Carrier unlock | `t_b17be10e` | https://github.com/EFForg/rayhunter/issues/207 |
| #195 | shared-general | Question on Dependency Security / Supply Chain Security | `t_8f5dbfd0` | https://github.com/EFForg/rayhunter/issues/195 |
| #160 | shared-general | CHOICE Additions not supported yet | `t_9c39815b` | https://github.com/EFForg/rayhunter/issues/160 |
| #154 | shared-general | Tool for nulling sensitive QMDL/PCAP fields | `t_6f7e602f` | https://github.com/EFForg/rayhunter/issues/154 |
| #153 | shared-general | Another indicator of IMSI catcher activity (compare public IP with announced IP ranges) | `t_810e4e03` | https://github.com/EFForg/rayhunter/issues/153 |
| #139 | direct-orbic | PerCodec:DecodeError:Requested Bits to decode 3, Remaining bits 1 | `t_47b6d7c4` | https://github.com/EFForg/rayhunter/issues/139 |
| #113 | shared-general | Support for detecting silent SMS messages | `t_1bfbac8f` | https://github.com/EFForg/rayhunter/issues/113 |
| #108 | shared-general | Telemetry | `t_1384a2d9` | https://github.com/EFForg/rayhunter/issues/108 |
| #98 | shared-general | Automated testing for RRC parser | `t_8dd407f4` | https://github.com/EFForg/rayhunter/issues/98 |
| #81 | direct-orbic | Include RSSI info in pcap  | `MISSING` | https://github.com/EFForg/rayhunter/issues/81 |
| #78 | shared-general | Setting to upload results to S3 Bucket  | `MISSING` | https://github.com/EFForg/rayhunter/issues/78 |
| #58 | shared-general | web UI: download entire QMDL store as zip | `t_23531757` | https://github.com/EFForg/rayhunter/issues/58 |

## Additional lanes

| Lane | Board card |
|---|---|
| Branch review: origin/fix/orbic-serial-response-framing-901-reviewed | `t_a87ab901` |
| Branch review: origin/fix/orbic-network-installer-693-reviewed | `t_5ddc132e` |
| Branch review: origin/fix/orbic-adb-tethering-reconnect-523-reviewed | `t_0aef7d67` |
| Source TODO/FIXME/maintenance sweep | `t_7a07b6e0` |

## Common pseudocode contract

```text
1. Verify current source behavior and issue status.
2. Trace affected callers, persistence/API contracts, and sibling-device paths.
3. For installer/device-control work, require vendor + product + interface + serial constraints; never trust USB vendor 0x05c6 alone.
4. Add regression fixtures/tests before implementation where practical.
5. Run targeted checks, then documented repository gates; build daemon frontend assets before Rust checks.
6. Separate local proof, provider CI, and hardware evidence.
7. Return a structured receipt and do not mutate upstream without explicit operator approval.
```

## Source maintenance markers

- `.cargo/audit.toml`: rustls-rustcrypto exception follow-up.
- `telcom-parser/README.md`: unfinished TODO section.
- `check/src/main.rs`: skip already-analyzed QMDL when matching PCAP.
- `installer/src/uz801.rs`: variant-specific IDs research.
- `lib/src/pcap.rs`: hard-coded radio/destination behavior.
- 2G downgrade heuristics: SIB-state tracking TODOs.
- Orbic/TP-Link display backends: display polling TODOs.
- `information_element.rs`: NB message mapping FIXME.
- `installer/src/connection.rs`: expose command exit status.

## Verification caveats

- This is a live inventory snapshot, not proof that any issue is still reproducible or that any branch is CI-ready.
- The source export used earlier ended before a final structured review report.
- The current checkout contains unrelated dirty paths; do not clean or overwrite them.
