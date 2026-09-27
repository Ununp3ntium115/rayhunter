# Heuristics

Rayhunter includes several analyzers to detect potential IMSI catcher activity. These can be enabled and disabled in your [configuration](./configuration.md) file.

## Available Analyzers

### IMSI Requested (v3)

This analyzer tests whether the eNodeB sends an IMSI or IMEI Identity Request NAS message under suspicious .

Mobile networks primarily request IMSI or IMEI from a mobile device during initial network attachment or when the network cannot identify the mobile device by its temporary identification (TMSI - *Temporary Mobile Subscriber Identity* or GUTI - *Globally Unique Temporary Identifier* in 4G/5G terminology).

IMSI request therefore usually happens when you first turn the device on especially after it has been off for a long time. Another possibility is, that you reboot your mobile device and your temporary ID expired. Sometimes temporary identification can expire if you have been in an area where there is absolutely no connection to your service provider or after you left your device on an airplane mode and then reconnect to the network (especially being disconnected for a long time). IMSI could also be requested when you connect to a new network (for instance for roaming), when you swap she SIM card or when your device moves to a new *Tracking Area* or *Location Area* and the network can not map the temporary identification to your device. IMSI number can also be requested after core network reboot.

It should also be noted that the network periodically reassigns your device new temporary identification to enhance security and avoid tracking, but in that cases usually does not request IMSI.

During these events the phone will typically go on to authenticate that the network is legitimate and then establish service with the network it is connected to. 

What we consider suspicious is the following chain of events:

* Phone connects to a new tower. 
* Tower asks for phones identity (IMEI or IMSI.)
* Authentication does *NOT* happen. 
* Tower requests phone to disconnect. 

Looking for this chain of events is much less prone to false positives than naively looking for any time the IMSI/IMEI is sent. We do still sometimes get false positives when users are in an airplane that is coming in for a landing however. This is likely due to having been disconnected for a while and then being over towers that are not able to route to your home network, but we are still researching.

This is the attack used by commercial IMSI catchers used by law enforcement. 

This heuristic will also alert you if any of the following happen:
* Identity is requested after authentication.
* Identity is requested without your phone connecting to the tower. 
* Identity is requested and then authentication doesn't happen shortly thereafter. 

This heuristic will also issue a notification every time your identity is sent to the network under non suspicious circumstances. This is for diagnostic purposes. 

### Connection Release/Redirected Carrier 2G Downgrade

This analyzer tests if a base station releases your device's connection and redirects your device to a 2G base station. This heuristic is useful, because some IMSI catchers may operate in a such way that they downgrade connection to 2G where they can intercept the communication (by performing man-in-the-middle attack).


### LTE SIB6/7 Downgrade (v2)

This analyzer tests if LTE base station is broadcasting a SIB type 6 and 7 messages which include 2G/3G frequencies with higher priorities.

SIB (*System Information Block*) Type 6 and 7 are specific types of broadcast messages sent by the base station (eNodeB in 4G networks) to mobile devices. They contain essential radio-related configuration parameters to help mobile device perform cell reselection.

This attack exploits the fact that SIB broadcast messages are not encrypted or authenticated. This allows them to pretend to be a legitimate cell by broadcasting fake system information in order to force mobile devices to downgrade from more secure 4G (LTE) to less secure 2G (GSM) network and then steal IMSI and/or perform man-in-the-middle attack. That is why this is also called a downgrade attack.

SIB6 is used for cell reselection to CDMA2000 systems which are not supported by many modern mobile phones, and SIB7 Provides the mobile device with information to perform cell reselection to GSM/EDGE networks. Therefore SIB6 messages are quite rare, while malformed SIB7 messages are much more frequent in practice. 

This heuristic is useful even in countries where 2g is still prevalent. A well behaved tower should always advertise its other 4g neighbors at a higher priority than 2g/3g neighbors. (Older versions of this heuristic were prone to false positives.)

### Null Cipher

This analyzer tests whether the cell suggests using a null cipher (EEA0) in the RRC layer. That means that encryption between your mobile device and base station is turned off.

Normally this should never happen, because null cipher is used almost exclusively for testing and debugging in labs or in controlled environments. Sometimes null cipher is used if encryption negotiation fails or isn’t supported (however in most networks this should not be the case). Also, some regulations allow unencrypted communications in **specific** emergency cases.

The general rule is that null cipher should never be used in commercial deployments, except in very controlled conditions (e.g., test labs) or in a very specific regulatory-approved use cases.

On the other hand, IMSI catchers often use null cipher to avoid setting up secure contexts (because they lack valid keys) and/or to trick mobile device into using unencrypted links (which makes eavesdropping easier).

### NAS Null Cipher

This analyzer tests whether the security mode command at the NAS layer suggests using a null cipher (EEA0). This would usually only happen after a mobile device has successfully authenticated with the MME (*Mobility Management Entity* - core network component that handles signaling and control) but still it shouldn't happen at all. This could be indicative of an attack though using SS7 (*Signaling System 7* - a set of telecommunication protocols used to set up and manage calls and other services) to get key material from the HLR (*Home Location Register* - a database in mobile telecommunications networks that stores subscriber information) of the mobile phone for a successful authentication.

It could also indicate an IMSI catcher which is connected to the mobile network MME and HLR through cooperation between government and telecom provider. Or it could be a false positive if the telecom provider is intending to use null ciphers (if encryption is illegal in some country, or they have some misconfiguration of the network), however this should be very rare case.

### Incomplete SIB

This analyzer tests whether the SIB1 message contains a complete SIB chain (SIB3, SIB5, etc.). A legitimate SIB1 message should contain timing information for at least 2 additional SIBs (SIB3, 4, and 5 being the most common) but a fake base station will often not bother to send additional SIBs beyond 1 and 2 (i. e. some IMSI catchers send just SIB1 and *one additional* SIB).

On its own this might just be a misconfigured base station (though we have only seen it in the wild under suspicious circumstances) but combined with other heuristics such as **IMSI Requested** detection it should be considered as a strong indicator of malicious activity.

### Diagnostic Information 
This analyzer displays some diagnostic information about when your device connects and disconnects from certain towers. It is helpful for analysis of suspicious PCAPs. The informational warnings in here can safely be ignored until there is a low, medium, or high severity warning. 

### No NAS Messages

*(disabled by default)*

This analyzer warns once per recording if Rayhunter receives diagnostic traffic spanning at least 5 minutes without observing a NAS (*Non-Access Stratum*) message. NAS is the signaling layer between your device and the core network, and a working SIM card normally produces NAS traffic during actions such as attachment, tracking area updates, and authentication.

The analyzer uses timestamps stored in the recording rather than the device's wall clock. This makes its results reproducible during reanalysis. Messages that Rayhunter cannot otherwise decode still advance the analyzer's clock. If any NAS message is observed, the analyzer is disabled for the rest of that recording.

If Rayhunter receives diagnostic traffic but never sees a NAS message, the SIM card may be missing, deactivated, improperly provisioned, or not seated correctly. Rayhunter may still capture broadcast messages from nearby towers, but most other heuristics depend on signaling involving the SIM and core network.

A recording containing no diagnostic messages at all cannot trigger this analyzer because it contains no timestamps with which to measure 5 minutes. Separately, while a recording is active, the web interface warns if no diagnostic message has arrived for 5 minutes; that warning is not part of any saved report.

This heuristic is experimental. It may produce a false positive if the device receives radio traffic but cannot reach a network that produces NAS traffic.

### LTE Positioning Protocol (LPP) (v1)

*(disabled by default)*

This analyzer detects LTE Positioning Protocol (LPP) messages used for device location tracking. LPP is a legitimate 3GPP protocol used by mobile networks to determine the geographic location of mobile devices for location-based services (such as emergency services, navigation, or location-dependent applications).

LPP operates at the EMM (Mobility Management) layer and is carried in Generic NAS Transport messages. The analyzer reports:

* **Downlink LPP messages** (network-to-UE): These are location requests or information from the network to the device.
* **Uplink LPP messages** (UE-to-network): These are responses or location information from the device to the network.

**False positives and legitimate use:**

This analyzer is informational only and does **not** indicate malicious activity. Legitimate mobile networks use LPP for:

* GPS-assisted location services (A-GPS)
* Emergency location services (E911/E112)
* Location-based services (navigation, geofencing)
* Network optimization and coverage analysis (MDT/minimization of drive tests)

Most users connected to legitimate networks may observe LPP messages, especially when:
* Location services are enabled on the device
* Connecting in areas served by modern 3GPP-compliant base stations
* Using emergency services or location-dependent applications

This heuristic is disabled by default because LPP activity is expected on legitimate networks. Enable it if you wish to monitor LPP activity or to correlate with other suspicious indicators detected by other heuristics.

### Test Analyzer

*(disabled by default)*

This analyzer is great for testing if your Rayhunter installation works. It will alert every time a new tower is seen (specifically every time a tower broadcasts a SIB1 message.) It is designed to be very noisy so we do not recommend leaving it on but if this alerts it means your Rayhunter device is working! 

### Timing Advance Outlier

Monitors the LTE Timing Advance (TA) value from baseband diagnostic logs (log code 0xb114). TA is the round-trip propagation delay between the device and the serving cell, expressed in units of 16 Ts (≈78 m one-way per index). A legitimate tower 1 km away has TA ≈ 13; a tower 10 km away has TA ≈ 128.

IMSI catchers are often physically close to the target (same room or street), which produces an unusually small TA. Conversely, a spoofed cell identity at an impossible distance produces an unusually large TA.

This heuristic tracks a running mean and standard deviation using Welford's online algorithm. After a 20-sample warmup period, a TA more than 2.5σ from the session mean triggers a Medium severity event.

**Known limitations (v1):** The distribution is global across all cells. A legitimate handover to a nearby small cell will produce a true TA drop that could trigger a false positive. Per-cell partitioning (keyed by PCI or EARFCN) is planned for a future version.

### Paging Message Storm (v1)

Detects excessive LTE paging messages that may indicate an IMSI catcher attempting to locate or track the device.

This analyzer triggers on two independent conditions:

**High Severity:** More than 50 paging messages (LTE_RRC.Paging on PCCH) within a 60-second time window. After alert fires, the detector resets to prevent duplicate alerts within that burst.

**Medium Severity:** Paging messages comprise more than 30% of captured traffic (once per recording after a 20-packet warmup). Normal captures show 1-5 paging messages; >30% traffic is anomalous.

**False positive conditions:** A busy urban cell legitimately produces high paging rates because all users sharing a paging occasion receive the broadcast message. ETWS (Earthquake and Tsunami Warning System) and CMAS (Commercial Mobile Alert System) emergency broadcasts also use the Paging message type. If other IMSI catcher heuristics do not corroborate, the paging rate may be legitimate.

### Rapid RRC Connection Setup (v1)

Detects multiple RRC connection setup messages within short time windows, which may indicate forced connection re-establishment attacks or IMSI catcher activity.

An RRCConnectionSetup message (sent on the downlink common control channel, DL-CCCH) is normally sent once per connection session when the network grants a mobile device's connection request. IMSI catchers may trigger rapid, repeated connection setups to force the device through multiple security procedures (capability exchange, security mode command, etc.), enabling cipher algorithm downgrade attacks. Each setup procedure exchanges encryption capabilities, creating opportunities to negotiate weaker algorithms.

This analyzer triggers on two independent conditions:

**High Severity:** More than 2 RRCConnectionSetup messages within a 300-second time window. After alert fires, the detector resets to prevent duplicate alerts within that burst.

**Medium Severity:** RRCConnectionSetup messages comprise more than 3% of total captured traffic (once per recording after a 100-packet warmup). Normal captures show <1% setup traffic; >3% is anomalous.

**False positive conditions:** Rapid idle-to-connected state cycling on unstable networks or with aggressive LTE connection policies may produce multiple setups. A carrier with misconfigured connection release timers could trigger false positives if the device rapidly disconnects and reconnects. If other IMSI catcher heuristics (e.g., null cipher, downgrade attacks) do not corroborate, the setup rate may reflect network instability rather than malicious activity.

**Limitations (v1):** This detector is blind to the sequence context (e.g., whether setups follow explicit RRCConnectionRelease messages with cause="other"). Such sequencing is documented in PCAP analysis but not yet correlated. A future version may correlate release-then-setup cycles for higher specificity.

### Connection Release Storm (v1)

Detects excessive RRC connection release messages that may indicate forced connection termination attacks.

An RRCConnectionRelease message (sent on the downlink dedicated control channel, DL-DCCH) normally occurs once per session when the network or device initiates disconnection. IMSI catchers may forcibly terminate connections to trigger device re-attachment cycles, enabling handover spoofing attacks or cipher algorithm downgrade through repeated security procedure re-execution.

Only releases with cause="other" are counted; releases due to load balancing or CS fallback are normal network behavior. Each release-then-reattach cycle re-executes authentication and capability exchange, creating opportunities for the attacker to negotiate weaker encryption algorithms.

This analyzer triggers on two independent conditions:

**High Severity:** More than 2 RRCConnectionRelease messages with cause="other" within a 10-second time window (i.e., 3+ consecutive rapid releases). After alert fires, the detector prevents immediate re-trigger by requiring a 10-second gap before the next HIGH alert.

**Medium Severity:** More than 3 RRCConnectionRelease messages with cause="other" within a 300-second time window (i.e., 4+). Fires once per recording to indicate sustained, moderately-paced forced termination activity.

**False positive conditions:** A heavily congested network or device with misconfigured connection policies may legitimately cycle connections rapidly. A misbehaving UE (user equipment) might re-initiate too quickly. If other IMSI catcher heuristics (e.g., null cipher, downgrade attacks) do not corroborate, the release rate may reflect network instability rather than malicious activity. Emergency fallback scenarios (e.g., moving between RATs) can also produce multiple rapid releases.

**Correlation:** Release storm activity strongly correlates with Rapid RRC Connection Setup detections, as each release is immediately followed by a new setup, paging, and service request. Detection of both together is a high-confidence indicator of active IMSI catcher attacks.

### Measurement Report Profiling (v1)

Detects aggressive measurement report patterns indicating signal strength profiling attacks by fake cells.

A MeasurementReport message (sent on the uplink dedicated control channel, UL-DCCH) normally contains signal strength metrics (RSRP/RSRQ) for the serving and neighbor cells. Legitimate networks request periodic measurement reports every 10-40 seconds to support mobility and handover decisions. IMSI catchers may request frequent measurement reports to build detailed UE location profiles, enabling handover spoofing, location tracking, or cipher algorithm downgrade attacks. The attacker uses the measurement data to predict optimal handover points and intercept the device during transitions.

This analyzer triggers on two independent conditions (both may fire during a single aggressive burst):

**Medium Severity:** More than 5 measurement reports within a 120-second time window (i.e., 6+). Fires once per recording to indicate sustained, aggressive profiling activity.

**High Severity:** 12 or more measurement reports within a 60-second time window (rate of 1 report per 5 seconds or faster). After alert fires, the detector requires a 60-second gap before re-firing to prevent duplicate alerts within the same burst. A typical aggressive profiling attack sequence: 6th report triggers MEDIUM alert, 12th report triggers HIGH alert, subsequent bursts only re-trigger HIGH after a 60-second cooldown.

**False positive conditions:** Event-triggered measurement reports (A2/A3 events) during high mobility scenarios (train, highway) produce legitimate bursts. Cell-edge oscillation in marginal coverage areas causes frequent re-measurement. Dense small-cell deployments (femtocells, picocells) with optimized MDT (Minimization of Drive Tests) configurations generate higher report densities per 3GPP standards. LTE handover preparation in congested urban areas legitimately increases report frequency. If other IMSI catcher heuristics (e.g., paging storm, connection release storm, null cipher) do not corroborate, the measurement rate may reflect network mobility optimization rather than malicious profiling activity.
