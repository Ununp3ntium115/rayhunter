# Adding a heuristic

Rayhunter's detection logic is organized into **analyzers** — stateful objects
that receive one decoded message at a time and emit zero or one `Event` in
response. Each analyzer corresponds to exactly one detection hypothesis (e.g.,
"the cell asked for an IMSI without authenticating first"). This page explains
how to write a new one, register it, and test it.

## Overview

The processing pipeline, from raw hardware to a report entry, looks like this:

```
/dev/diag → HDLC framing → QMDL message
          → GSMTAP packet (type + subtype)
          → InformationElement (decoded LTE RRC or NAS struct)
          → Analyzer::analyze_information_element()
          → Option<Event>
```

Every QMDL message produces at most one `InformationElement`. The `Harness`
in `lib/src/analysis/analyzer.rs` calls every enabled analyzer on each element
and collects the results into an `AnalysisRow`.

## The Analyzer trait

A new analyzer implements one trait in `lib/src/analysis/analyzer.rs`:

```rust
pub trait Analyzer {
    fn metadata() -> AnalyzerMetadata where Self: Sized;

    fn analyze_information_element(
        &mut self,
        ie: &InformationElement,
        packet_num: usize,
        timestamp: DateTime<FixedOffset>,
    ) -> Option<Event>;

    // Optional — useful for time-based analyzers.
    fn report_skipped_packet(&mut self, _timestamp: DateTime<FixedOffset>) -> Option<Event> {
        None
    }
}
```

`analyze_information_element` is called for every successfully decoded packet
regardless of type, so your implementation should return `None` quickly for
types it does not care about.

`report_skipped_packet` is called when a packet could not be decoded or was
intentionally skipped. Time-based analyzers (like `NoNasMessagesAnalyzer`) use
it to advance their clock even on undecoded traffic.

## InformationElement

The `InformationElement` enum in `lib/src/analysis/information_element.rs`
covers all packet types the daemon can decode:

```rust
pub enum InformationElement {
    GSM,
    UMTS,
    LTE(Box<LteInformationElement>),
    FiveG,
}
```

`LteInformationElement` sub-types correspond to LTE RRC channel types
(`BcchDlSch` for broadcast SIBs, `DlDcch` for unicast RRC messages, `NAS` for
Non-Access Stratum messages, and several others). Their inner message structs
come from `telcom_parser::lte_rrc` (generated ASN.1) and `pycrate_rs::nas` (NAS
decoder).

For most heuristics you only need LTE RRC. Pattern-match all the way down to
the message type you care about and return `None` for everything else. The
compiler will tell you if you missed an arm.

## AnalyzerMetadata

`metadata()` returns a **static** struct — it must not depend on instance
state:

```rust
pub struct AnalyzerMetadata {
    pub key: Cow<'static, str>,   // stable config key; never rename
    pub default_enabled: bool,
    pub name: Cow<'static, str>,  // user-facing display name
    pub description: Cow<'static, str>,
    pub version: u32,             // bump for substantial behavior changes
}
```

**`key`** is used in `config.toml` to enable/disable the analyzer and in
analysis reports to identify which analyzer fired. Once published, a key
must never change — users' saved configs would silently stop working.

**`version`** must be bumped whenever a change would cause the same recording
to produce meaningfully different results (new detection case, loosened/tightened
threshold, different event message format). This lets users know stored reports
may differ from a fresh re-analysis.

## A minimal example

The simplest analyzer is `TestAnalyzer` in `lib/src/analysis/test_analyzer.rs`.
It fires a `Low`-severity event for every SIB1 broadcast it sees:

```rust
pub struct TestAnalyzer {}

impl Analyzer for TestAnalyzer {
    fn metadata() -> AnalyzerMetadata {
        AnalyzerMetadata {
            key: "test_analyzer".into(),
            default_enabled: false,
            name: "Test Analyzer".into(),
            description: "Fires on every SIB1 to confirm the device is receiving traffic.".into(),
            version: 1,
        }
    }

    fn analyze_information_element(
        &mut self,
        ie: &InformationElement,
        _packet_num: usize,
        _timestamp: DateTime<FixedOffset>,
    ) -> Option<Event> {
        if let InformationElement::LTE(lte_ie) = ie
            && let LteInformationElement::BcchDlSch(msg) = &**lte_ie
            && let BCCH_DL_SCH_MessageType::C1(c1) = &msg.message
            && let BCCH_DL_SCH_MessageType_c1::SystemInformationBlockType1(_sib1) = c1
        {
            return Some(Event {
                event_type: EventType::Low,
                message: "SIB1 received".to_string(),
            });
        }
        None
    }
}
```

When you need state, add fields to the struct and initialize them in a `new()`
constructor:

```rust
pub struct MyAnalyzer {
    saw_attach: bool,
}

impl MyAnalyzer {
    pub fn new() -> Self {
        Self { saw_attach: false }
    }
}
```

## Navigating LTE RRC message types

The full generated ASN.1 types live in `telcom_parser/src/lte_rrc.rs` (a large
auto-generated file). The easiest way to explore it is with the included helper:

```sh
python3 tools/asn1grep.py SEQUENCE_OR_CHOICE_NAME
```

For example, `python3 tools/asn1grep.py SecurityModeCommand` prints the ASN.1
source for that type and all nested types, which shows you what Rust fields to
expect.

The `telcom_parser/specs/` directory contains the raw 3GPP ASN.1 spec files if
you need to trace the message structure further.

## Registering the analyzer

Once your struct is ready, add it to the `all_analyzers()` function in
`lib/src/analysis/analyzer.rs`:

```rust
fn all_analyzers(
    device_metadata: &DeviceMetadata,
) -> impl Iterator<Item = (AnalyzerMetadata, Box<dyn Analyzer + Send>)> {
    [
        // ... existing analyzers ...
        (MyAnalyzer::metadata(), boxed(MyAnalyzer::new())),
    ]
    .into_iter()
}
```

Add the corresponding `use` import at the top of the file with the other
analyzers.

The analyzer is now available in the config. Add it to `config.toml` to enable
it when testing:

```toml
[analyzer_config]
my_analyzer_key = true
```

## Severity levels

| `EventType` | When to use |
|-------------|-------------|
| `Informational` | Normal network events worth recording; not a warning |
| `Low` | Unusual but common; IMSI catcher possible but not likely |
| `Medium` | Suspicious combination of behaviors; investigation warranted |
| `High` | Strong indicator of an attack; alert the user prominently |

Prefer `Informational` for diagnostic events that help interpret the recording
context (e.g., "connected to new tower"). Reserve `High` for behaviors that have
a reliable real-world false-positive rate near zero.

## Testing

Unit tests for individual analyzers live alongside the analyzer source file.
Use `Harness::new_with_config` to create a test harness and check the events
returned. The `rayhunter-check` CLI (`cargo run -p rayhunter-check -- -p path/to/file.qmdl`)
lets you run all analyzers over a saved recording and review the events.

When writing tests, construct synthetic `InformationElement` values or load
real captures from `lib/tests/` (the test fixtures directory).

Run the full test suite with:

```sh
cargo test -p rayhunter
```

## Checklist before opening a PR

- [ ] `key` is unique across all analyzers in `all_analyzers()`
- [ ] `default_enabled: false` for noisy or experimental analyzers
- [ ] `version` starts at `1`
- [ ] The analyzer description in `metadata()` explains both what it detects
      **and** known false-positive conditions
- [ ] Added to `doc/heuristics.md` under **Available Analyzers**
- [ ] `cargo fmt --all --check` and `cargo clippy -- -D warnings` pass
- [ ] Tests cover at least the primary detection case and one non-firing case
