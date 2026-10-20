# 5G NR threat model and detector design (private working note)

Status: design only. No detector is implemented and none should be until the capability gaps below are closed.
Date: 2026-10-19. AI-assisted working note; the maintainer reviews and rewrites before any public use. Not a PR description. Do not paste into public issues, PRs or comments (see `rayhunter-ai-contribution-policy.md` in this folder).
Privacy: contains no subscriber identifiers, exact cell identities, coordinates or capture payloads.
Verification legend used below: VERIFIED = checked against this checkout or a source read this session; MIRROR = read from a third-party mirror of a spec, not the 3GPP/ETSI text; UNVERIFIED = recalled or carried over, not checked here.

## 1. Baseline (verified in this checkout)

Rayhunter observes nothing about 5G today.

| Fact | Where |
|---|---|
| Log code `0xb821` (NR RRC OTA) is requested in the DIAG log mask | `lib/src/log_codes.rs:29`, `lib/src/diag_device.rs:50` |
| It is parsed only as `NrRrcOtaMessage { msg: Vec<u8> }`, raw bytes with no ASN.1 decode | `lib/src/diag/diaglog/mod.rs:81-85` |
| `InformationElement::FiveG` is a unit-variant placeholder; analyzers use it only in unit tests | `lib/src/analysis/information_element.rs:22-29` |
| No NR handling in the GSMTAP conversion | `lib/src/gsmtap/parser.rs` (no NR match) |
| LTE RRC decoding is generated from LTE ASN.1 only | `telcom-parser/` |
| No 5GS NAS (5GMM/5GSM) decoding: the pinned `pycrate-rs` (rev `9e72e40` in `Cargo.lock`) generates only `emm` and `esm` modules; its only 5G strings are field names inside two EPS NAS messages | `Cargo.lock` pin; `pycrate-rs/src/nas/generated/{emm,esm}` (VERIFIED by directory listing and grep) |
| NAS log codes in the mask are the LTE EMM/ESM and UMTS codes only | `lib/src/diag_device.rs:52-56` |

Consequence: any 5G heuristic written today would run against opaque bytes. Do not claim 5G detection.

## 2. Target device (what is known and unknown)

Known from repository evidence and the owner's own recording metadata:
- Device variant `r500l5` ("Orbic Speed 5G UW"); recording manifest reports `device_model: r500l5`, Linux 4.14.206, armv7.
- Earlier session notes (not independently re-verified here): Qualcomm SDX55 modem, Verizon firmware family `ORB500L5_V2.0.x`.
- A recording from this device (about 22 to 25 hours, parsed in this checkout) contains NR RRC OTA records (`0xb821`): 194 in the first export and 214 in the later export of the same session, against 11,804 and 12,479 LTE RRC OTA records (`0xb0c0`). VERIFIED by counting log codes in the local QMDL files. These records are present in the capture but are not decoded. Their presence is consistent with NSA (EN-DC, LTE anchor) operation; that is an INFERENCE and does not prove NSA or rule out SA.

Unknown (record as unknown, do not assume):
- Supported NR bands, therefore FR1 versus FR2 (mmWave) capability.
- Whether the device and modem firmware support SA, NSA, or both, and which the operator enables.
- The SIM's actual operator. A broadcast PLMN seen in LTE system information is an unauthenticated observation, not proof of the subscription.
- Whether the modem emits 5GS NAS in any DIAG log code, and which code.

"5G Ultra Wideband" is operator marketing, not a protocol or spectrum definition. It must not be used as a signal.

## 3. Sources and verification status

Read from search results or pages during this working session (MIRROR where noted). The secured session cannot re-fetch pages, so links below were not re-opened after drafting; confirm each against the official text before relying on it:
- 3GPP TS 33.501 (security architecture and procedures for the 5G system), Release 19, status page lists 19.7.0: https://www.3gpp.org/dynareport/33501.htm (listed date not independently confirmed).
- TS 33.501 clause 6.7.2 (NAS security mode command): the SMC replays the UE security capabilities so a UE can detect a man-in-the-middle bidding-down of the Registration Request; mismatch leads the UE to reject. Mirror: https://itecspec.com/3gpp/33.501/s/6.7.2 (mirror, not the 3GPP text).
- TS 33.501 Annex D and clause 5.5.2 (as cited by TS 33.512): NIA0 is allowed only on signalling radio bearers for unauthenticated emergency sessions of UEs in limited service mode, never on data radio bearers; AMF should disable NIA0 where unauthenticated emergency is not a regulatory requirement. Mirrors: https://itecspec.com/3gpp/33.501/s/d.1 and https://itecspec.com/3gpp/33.501/s/10.2.2.1.
- ETSI TS 124 501 V19.5.0 (2026-02), 5GS NAS: https://www.etsi.org/deliver/etsi_ts/124500_124599/124501/19.05.00_60/ts_124501v190500p.pdf . Clause 5.4.2 (security mode control, 5.4.2.3 to 5.4.2.5 confirmed) and 5.4.3 (identification; identity types SUCI, 5G-GUTI, IMEI, 5G-S-TMSI, IMEISV, MAC address, EUI-64; UE regenerates a SUCI only if timer T3519 is not running). Mirrors: https://whatthespec.net/sos/spec/ts-24501/5.4.3.1 , .../5.4.3.3 , https://itecspec.com/3gpp/24.501/s/9.11.3.3 .
- 3GPP TR 33.809, study on 5G enhancements against false base stations (V18.1.0 seen as a draft mirror; not approved normative text): https://www.3gpp.org/DynaReport/33809.htm .
- Chlosta, Rupprecht, Pöpper, Holz, "5G SUCI-catchers: still catching them all?", WiSec 2021 (SUCI linkability via AKA; proof of concept in a SA network): https://casa.rub.de/forschung/publikationen/detail/5g-suci-catchers-still-catching-them-all
- Marlin (NDSS 2025): pre-authentication identity exposure ratios, no validated field false-positive rate: https://www.cise.ufl.edu/~tucker/docs/2025-marlin.pdf (UNVERIFIED here: finding carried over from an earlier multi-source research run that cross-checked it; LTE-focused).
- SeaGlass (PoPETs 2017): error rates reported only from synthetic injection: https://homes.cs.washington.edu/~yoshi/papers/seaglass-imsi-catcher-2017.pdf (UNVERIFIED here: carried over from the same earlier run; URL not re-opened).

Cited by name but clause numbers NOT yet confirmed against the official text (confirm before relying):
- TS 38.331 (NR RRC): SIB1, RRCRelease including `redirectedCarrierInfo`, security mode procedures. https://www.3gpp.org/dynareport/38331.htm
- TS 33.501 clause on subscription identifier privacy (SUCI concealment), assumed in the 6.12 range.
- TS 37.340 (EN-DC / multi-radio dual connectivity) for NSA behaviour.
- The 5GMM cause value for "UE security capabilities mismatch" in a Security Mode Reject (recalled as #23, unconfirmed).
- Whether TS 24.501 permits IMEI or IMEISV identity requests before NAS security is active (clause 5.4.3.2, not read).

## 4. Terminology (keep distinct)

- IMSI: LTE/legacy permanent subscriber identity.
- SUPI: 5G permanent subscription identifier (may contain an IMSI). Never sent in the clear over the air in 5G.
- SUCI: concealed SUPI, sent instead. A SUCI by itself is NOT suspicious and NOT an attack signal.
- 5G-GUTI: temporary identity (analogue of the LTE GUTI/TMSI).
- PEI: permanent equipment identifier; IMEI and IMEISV are PEI formats.

## 5. Threat model (what a rogue gNB/IMSI catcher could try, and what we can see)

| Threat | Passive observable (if decoded) | Notes |
|---|---|---|
| Force the UE off NR (LTE or legacy) | RRC release/redirect sequences; NR to LTE to legacy chain | Direct NR redirect targets are NR or E-UTRA (to confirm in 38.331), so a legacy downgrade crosses LTE |
| Bid down NAS/AS algorithms | SMC replayed capabilities versus Registration Request; selected algorithms; UE Security Mode Reject | Strongest protocol-level signal; see D1/D2 |
| Harvest identities pre-security | Identity Request types and ratios; Registration Reject patterns | Only a ratio against a baseline is meaningful (Marlin); one request is normal |
| Link a known SUCI (SUCI-catcher) | Not detectable by a passive UE-side logger; needs network-side or targeted tests | Chlosta et al. |
| Fake cell broadcast | SIB1 PLMN/TAC/cell/PCI anomalies | Broadcast is unauthenticated; ordinary changes are normal; informational only |

Passive limits: after NAS security is activated, payloads are ciphered unless the modem logs them deciphered (chipset-specific, unverified for this modem). Pre-security messages and the SMC are the observable window.

## 6. Detector candidates (design only)

Conventions: severity is a proposal; every detector must report `unknown` when its inputs are missing and must never convert missing data into "clean".

### D1 Replayed-capability mismatch or UE Security Mode Reject (high confidence)
- Observables: 5GS Registration Request (UE security capability IE); Security Mode Command (replayed capabilities, selected NEA/NIA, ngKSI); Security Mode Reject.
- Expected benign sequence: Registration Request, Authentication, Security Mode Command with capabilities equal to the request, Security Mode Complete.
- Features: replayed_caps == requested_caps; reject cause.
- Confounders: truncated capture (request missing), capture started mid-procedure, modem rewriting capability bits before logging.
- Severity: HIGH if both messages observed and differ, or the UE rejects with a capability-mismatch cause. `unknown` if the request was not captured.
- Evidence requirement: both messages from the same connection in order.
- FP risk: low if inputs are complete; FN risk: high (a successful MitM may keep the UE-visible values consistent, and the UE rejects before we see completion).
```
on RegistrationRequest(conn, caps): cache[conn] = caps
on SecurityModeCommand(conn, replayed, sel):
    if conn not in cache: emit Unknown("request not captured")
    elif replayed != cache[conn]: emit HIGH("replayed capabilities differ")
on SecurityModeReject(conn, cause):
    if cause == CAPABILITY_MISMATCH: emit HIGH("UE rejected SMC: capability mismatch")
```

### D2 Null integrity selected outside an emergency context (high confidence)
- Observable: Security Mode Command selected integrity algorithm; registration type / emergency indicator.
- Rule from TS 33.501 Annex D / 5.5.2: NIA0 only for unauthenticated emergency sessions (limited service mode).
- Confounders: genuine emergency registration; unreadable registration type.
- Severity: HIGH if NIA0 is selected and the registration is not an emergency registration; NEA0 alone is LOW (not constrained by the sources read).
- FP/FN: FP low; FN high (needs a visible SMC).
```
on SecurityModeCommand(conn, sel):
    if sel.integrity == NIA0 and not emergency(conn): emit HIGH
    elif sel.cipher == NEA0: emit LOW  # informational
```

### D3 NR to LTE to legacy downgrade chain (low to medium)
- Observables: NR RRCRelease with redirected E-UTRA carrier; subsequent LTE release/redirect to GERAN or UTRA; absence of voice-fallback context.
- Confounders: EPS fallback for voice, coverage edge, operator policy, NSA anchor transitions.
- Severity: LOW for a single hop; MEDIUM only for a correlated chain within a short window with no fallback context; never HIGH on its own.
- FP risk high; evidence requirement: whole chain in order.

### D4 Pre-security identity exposure ratio (medium, needs a baseline)
- Observables: counts of Identity Requests by type before security activation; Registration Rejects.
- Method: compare against a per-device, per-operator baseline measured over normal operation, in the style of Marlin; no fixed threshold until a baseline exists.
- Confounders: re-registration with an unknown 5G-GUTI, roaming, new AMF. Open question: which identity types may be requested before security (24.501 5.4.3.2).
- Severity: MEDIUM at most, and only as a ratio anomaly.

### Explicit non-detectors
PLMN/TAC/PCI/frequency changes on their own, a carrier label such as "5G UW", a SUCI, a single Identity Request, availability of a null algorithm, or a single RRC release must never raise an alert by themselves.

## 7. Capability gaps (ordered; all required before D1 to D4 can run)

1. Confirm the modem emits 5GS NAS in DIAG and find the log code (candidates `0xb80a`, `0xb80b` are unverified). Needs owner-authorized capture; no device change was made for this note.
2. 5GS NAS decoder (24.501): the pinned `pycrate-rs` has no 5GS modules (verified above), so this needs either an upstream `pycrate-rs` extension or a new decoder.
3. NR RRC decoder (38.331 ASN.1): regenerate via the `telcom-parser` workflow (see its README) or a separate crate; size and licensing impact to be assessed.
4. GSMTAP/pcap representation for NR RRC and 5GS NAS so `rayhunter-check` and Wireshark can round-trip.
5. EN-DC handling: NR configuration embedded in LTE RRC reconfiguration messages; confirm the LTE ASN.1 version covers it. A historical note from an earlier corpus review (carried over from memory, UNVERIFIED here) reported unsupported LTE SIB12 additions, which suggests ASN.1 version gaps exist.
6. Analyzer plumbing: a 5G `InformationElement` variant carrying decoded data, registry entry, `AnalyzerMetadata.key`, `version`, `default_enabled`, and `doc/heuristics.md` entries.

## 8. Privacy-safe fixture plan

- Build synthetic messages from the spec's ASN.1/IE definitions using the standard test PLMN (MCC 001, MNC 01; general knowledge of the ITU test network, not checked in this session) and made-up identifiers. No real SUCI, 5G-GUTI, IMEI, cell identity, coordinates or payload bytes from owner captures.
- Per detector, fixtures for: positive, negative, benign confounder (emergency, EPS fallback, re-registration), truncated input (must give Unknown), and out-of-order input.
- Keep real captures in restricted evidence storage; derive minimized hand-written fixtures only.

## 9. Acceptance criteria

- Each detector has positive, negative, benign and truncated tests, and reports Unknown for missing inputs.
- No change to LTE or other-device results (existing test suite and replay of existing recordings unchanged).
- Metadata `version` bumped and documented; no threshold shipped without a measured baseline.
- No hardware claim without hardware evidence; local tests are reported as local only.
- No public prose written by AI.

## 10. Next hardware evidence needed (not performed)

- Exact supported NR bands and SA/NSA capability from the manufacturer specification or regulatory filing.
- An owner-authorized, time-boxed capture across a normal re-registration to learn which log codes carry 5GS NAS and NR RRC on this modem. This changes the DIAG mask and restarts the daemon, so it needs explicit approval.

## 11. Open decisions for the maintainer

- Whether to pursue the NR RRC decoder or start with 5GS NAS (D1/D2 need NAS only).
- Whether a D4 baseline can be collected without new storage cost on a small `/data` partition.
