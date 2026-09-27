# Rayhunter PCAP/Wireshark research pass — 2026-09-26

Scope

A read-only Wireshark/tshark pass was run over every `*.pcap*` file directly under `/Users/brodynielsen/Downloads`. Raw packet payloads were not exported or copied. This report intentionally does not repeat exact GPS values found in packet comments; the captures contain location metadata and must be treated as sensitive evidence.

Inventory evidence

- 36 PCAP/PCAPNG filenames discovered.
- 26 files contain packets; 10 are zero-packet files.
- 24 distinct SHA-256 contents; duplicate filename pairs were detected.
- 3,740 total frames across the non-empty files, including duplicate filenames.
- Non-empty captures decode as `rawip4 -> UDP -> GSMTAP -> LTE RRC/NAS-EPS`.
- GSMTAP UDP ports observed consistently: `13337` and `4729`.
- The captures are mostly LTE paging/RRC/NAS traffic; they are not ordinary Ethernet/IP application captures.
- `tshark`, `capinfos`, and `editcap` were available at `/usr/local/bin` and used for the pass.

Observed evidence

1. Capture metadata contains per-packet JSON comments with `unix_ts` set to the minimum signed 64-bit value and latitude/longitude fields. This occurred on every packet in the sampled non-empty captures. Exact coordinates are intentionally omitted here.
   - Impact: Rayhunter's packet timestamp/location provenance path may be writing a sentinel timestamp while still attaching valid-looking location fields.
   - Risk: downstream UI/export/analysis can silently treat invalid timestamps as real chronology, or leak location metadata into evidence without a clear privacy indicator.
   - Candidate issue: add explicit metadata validity states; reject or label sentinel timestamps; preserve a separate capture-time source and GPS-fix source.

2. One capture (`1790367566.pcapng`) contains a strong timestamp anomaly: its first decoded packet timestamp was approximately `315964824.864` while later packets were around `1790368xxx`, producing a multi-decade discontinuity in one file. This must be reproduced against the original capture and checked against pcapng interface/block timestamps before deciding whether the writer or source clock is at fault.
   - Impact: chronology, session grouping, rate calculations, and event correlation can be wrong.
   - Candidate issue: detect non-monotonic/implausible timestamp jumps during import and emit a visible integrity warning instead of normalizing silently.

3. LTE RRC content is useful and repeatedly includes paging, SIB1/SIB2/SIB3/SIB4/SIB5, RRC connection setup/release, service requests, security mode command/complete, UE capability exchange, measurement reports, and reconfiguration messages.
   - Paging dominates many short captures; longer captures include attach and default EPS bearer setup.
   - Several attach summaries explicitly report `PDN type IPv6 only allowed` and `CS domain not available`.
   - Candidate issue: expose a normalized per-capture protocol/event summary so users can distinguish paging-only captures, attach attempts, data-session setup, and release/reject outcomes.

4. NAS-EPS decode shows repeated dissector errors and warnings. Across the eventful files, Wireshark reports `Not a NAS EPS PD` for values that decode as other 3GPP domains, unknown message types, and extraneous data. Explicit `Malformed Packet (Exception occurred)` entries appeared in at least:
   - `1790400950.pcapng` (1)
   - `1790413135.pcapng` (1)
   - `1790459868.pcapng` (3)
   - `1790460052.pcapng` (1)
   Other files had NAS-EPS errors without an explicit malformed packet line.
   - Interpretation: this may be expected when raw GSMTAP direction/bitfields or encrypted/protected NAS payloads are incomplete, but the parser must not treat dissector disagreement as a confirmed protocol defect.
   - Candidate issue: add robust unknown/malformed frame accounting and preserve undecoded bytes/reason codes; never drop a frame solely because a higher-level dissector rejects it.

5. The GSMTAP signal and SNR fields decoded as zero for every analyzed frame, while ARFCN values varied (commonly 1100, 2050, 2460, and 5230; zero also appeared). This indicates either source captures omit measurements, the writer is defaulting to zero, or the selected GSMTAP fields do not match the modem's metadata encoding.
   - Candidate issue: distinguish `missing` from numeric zero for signal/SNR; audit GSMTAP header serialization and modem metric extraction; add fixtures for absent, zero, and valid negative dBm/SNR values.
   - Do not claim signal quality from these captures until the encoding is confirmed.

6. Duplicate and empty artifacts are common: exact duplicate pairs include `1790406227.pcapng` / `1790406227 (1).pcapng`, `1790410934.pcapng` / `1790410934 (1).pcapng`, and `1790459359.pcapng` / `1790459359 (1).pcapng`. Ten files are zero-packet captures, including repeated `1790410217*`, `1790411077*`, `1790411096*`, and related files.
   - Candidate issue: add evidence-ingest deduplication by content hash, explicit zero-frame capture classification, and a user-facing reason/status for empty output.

7. Capture framing is consistent enough to support a regression corpus: raw IPv4, UDP/GSMTAP, LTE RRC/NAS-EPS, and stable ports. Frame lengths ranged approximately from 46 to 408 bytes in analyzed files. This makes a small sanitized fixture set feasible without retaining full user captures.

Recommended Kanban work units

A. Capture metadata validity and privacy provenance

Pseudocode:

```text
for packet in capture:
    ts = packet.capture_timestamp
    gps = packet.metadata.gps
    if ts is sentinel_min_i64 or ts outside supported_epoch_range:
        packet.timestamp_state = INVALID_SENTINEL
    else:
        packet.timestamp_state = VALID
    if gps exists and not gps.valid_fix:
        packet.gps_state = PRESENT_BUT_UNCONFIRMED
    expose timestamp_state and gps_state in analysis/export
    redact or explicitly opt-in GPS in shared reports
```

Acceptance: sentinel timestamps cannot sort as valid chronology; invalid metadata is visible in CLI/UI/export; GPS is handled as sensitive; fixtures cover valid, absent, sentinel, and implausible values.

B. Timestamp discontinuity and capture-integrity analyzer

Pseudocode:

```text
previous = None
for frame in frames:
    delta = frame.ts - previous if previous else 0
    if previous and (delta < 0 or abs(delta) > configured_gap_limit):
        emit_integrity_event(TIMESTAMP_JUMP, frame.number, delta)
    previous = frame.ts
summarize monotonicity, gaps, duplicate timestamps, and interface clock domains
never rewrite timestamps silently
```

Acceptance: `1790367566.pcapng`-style jumps produce a deterministic warning and packet/frame reference; normal modem gaps do not become false failures; original timestamps remain available.

C. Unknown/malformed LTE/NAS frame preservation

Pseudocode:

```text
result = try_decode(frame)
if result.success:
    emit decoded event
else:
    emit undecoded event with layer, reason_code, raw_length, frame_number
    preserve frame bytes and GSMTAP direction/header metadata
    continue capture analysis
aggregate malformed/unknown counts separately from confirmed parser defects
```

Acceptance: explicit malformed NAS frames remain countable/exportable; parser never panics or drops the whole capture; unknown protocol discriminators are stable and privacy-safe; regression fixtures cover all observed failure classes.

D. GSMTAP measurement presence semantics

Pseudocode:

```text
signal = decode_gsmtap_signal(header)
snr = decode_gsmtap_snr(header)
for metric in [signal, snr]:
    if metric absent or header_not_supported:
        state = MISSING
    elif metric == encoded_zero and zero_is_not_valid_for_source:
        state = MISSING_OR_UNCONFIRMED
    else:
        state = PRESENT
never render MISSING as 0 dBm or 0 dB
```

Acceptance: valid negative measurements, true zero, and absent fields are distinct; reports clearly say when this corpus cannot establish RF quality; source-specific encoding is documented.

E. Capture triage summary and deduplication

Pseudocode:

```text
hash = sha256(file_bytes)
if hash already seen:
    classify DUPLICATE and link evidence, do not analyze twice
elif frame_count == 0:
    classify EMPTY with capinfos/read-error status
else:
    decode protocol/event counters and integrity warnings
write machine-readable summary with no raw IMSI/location by default
```

Acceptance: duplicate names collapse to one evidence item; empty captures have a useful diagnostic status; summaries include protocol/event counts and integrity results; default exports exclude sensitive identifiers and exact GPS.

Suggested source areas to inspect next

- `lib/src/pcap.rs` and analysis modules: timestamp, packet preservation, and protocol event summaries.
- `lib/src/analysis/*`: unknown/malformed LTE/NAS handling and SIB state.
- `daemon/src/*` and web UI: user-visible integrity/empty-capture warnings and privacy labeling.
- `check/src/main.rs`: skip/dedup logic and QMDL/PCAP association.
- Any GSMTAP writer/decoder path: signal/SNR presence and timestamp/GPS comment serialization.

Limits and non-claims

- This was a read-only tshark/capinfos pass, not a Wireshark GUI forensic review.
- No IMSI, TMSI, phone number, exact coordinate, raw payload, or decoded subscriber identity is included in this report.
- Wireshark expert output is evidence of dissector behavior, not proof that Rayhunter generated invalid NAS. Source-level correlation and sanitized fixtures are required.
- No code, hardware, device, GitHub issue, or upstream publication was mutated by this pass.

Evidence artifacts

- Inventory: `/Users/brodynielsen/GitRepos/hermes/cache/scratch/rayhunter-pcap-inventory-20260926.json`
- Protocol summary: `/Users/brodynielsen/GitRepos/hermes/cache/scratch/rayhunter-pcap-protocol-summary.json`
- Event summary: `/Users/brodynielsen/GitRepos/hermes/cache/scratch/rayhunter-pcap-event-summary.json`
- Expert summary: `/Users/brodynielsen/GitRepos/hermes/cache/scratch/rayhunter-pcap-expert-summary.json`
