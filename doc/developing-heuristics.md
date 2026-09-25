# Developing a new heuristic

This guide walks through adding a new IMSI catcher heuristic to Rayhunter. All analyzers live in [`lib/src/analysis/`](https://github.com/EFForg/rayhunter/blob/main/lib/src/analysis/) and share the same `Analyzer` trait.

## The `Analyzer` trait

```rust
pub trait Analyzer {
    fn metadata() -> AnalyzerMetadata where Self: Sized;

    fn analyze_information_element(
        &mut self,
        ie: &InformationElement,
        packet_num: usize,
        timestamp: DateTime<FixedOffset>,
    ) -> Option<Event>;

    // Optional — default is a no-op.
    fn report_skipped_packet(&mut self, _timestamp: DateTime<FixedOffset>) -> Option<Event> {
        None
    }
}
```

Implement `metadata()` and `analyze_information_element()` at minimum. Override `report_skipped_packet()` if your heuristic is time-based and needs to advance its clock even when a packet could not be decoded.

## `AnalyzerMetadata`

```rust
pub struct AnalyzerMetadata {
    pub key: Cow<'static, str>,   // stable config key, never rename it
    pub default_enabled: bool,
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub version: u32,             // bump on any behavioral change
}
```

- **`key`** is stored in users' `config.toml` files. Once published, renaming it silently disables the heuristic for existing users. Choose a name and keep it.
- **`version`** distinguishes results from old behavior in stored recordings. Bump it whenever the heuristic's detection logic changes in a way that would produce different results on the same capture.
- **`default_enabled`**: set `false` for noisy or experimental analyzers (like `TestAnalyzer`). Most production heuristics are `true`.

## `InformationElement`

`analyze_information_element` receives one decoded IE per call. The primary variants are:

```rust
pub enum InformationElement {
    LTE(Box<LteInformationElement>),
    NAS(NASMessage),
}
```

`LteInformationElement` covers RRC messages decoded by `telcom-parser`:

```rust
pub enum LteInformationElement {
    DlDcch(Box<DL_DCCH_Message>),
    BcchDlSch(BCCH_DL_SCH_Message),
    UlDcch(UL_DCCH_Message),
    // … other channel types
}
```

Pattern-match on the variant you care about and return `None` for everything else:

```rust
fn analyze_information_element(
    &mut self,
    ie: &InformationElement,
    _packet_num: usize,
    _timestamp: DateTime<FixedOffset>,
) -> Option<Event> {
    let InformationElement::LTE(lte_ie) = ie else { return None };
    let LteInformationElement::DlDcch(dcch) = &**lte_ie else { return None };
    // inspect dcch …
    None
}
```

Use `tools/asn1grep.py` to locate a specific type in the ASN.1 specs:

```sh
python tools/asn1grep.py SecurityModeCommand
```

## Returning an event

Return `Some(Event { event_type, message })` to report a detection. Use the lowest severity that accurately reflects risk:

| Severity | Meaning |
|----------|---------|
| `Informational` | Noteworthy but not suspicious; does not trigger an alert on the device display |
| `Low` | Mildly suspicious; seen under normal carrier behavior on some networks |
| `Medium` | Suspicious; rarely benign |
| `High` | Strong indicator of malicious activity |

Include identifying details in the message (cell ID, TAC, cipher algorithm, etc.) so analysts can correlate events in a PCAP.

## Minimal example

Here is the smallest possible analyzer: it fires on every SIB1 broadcast.

```rust
// lib/src/analysis/my_analyzer.rs

use chrono::{DateTime, FixedOffset};
use telcom_parser::lte_rrc::{BCCH_DL_SCH_MessageType, BCCH_DL_SCH_MessageType_c1};
use super::analyzer::{Analyzer, AnalyzerMetadata, Event, EventType};
use super::information_element::{InformationElement, LteInformationElement};

pub struct MyAnalyzer {}

impl Analyzer for MyAnalyzer {
    fn metadata() -> AnalyzerMetadata {
        AnalyzerMetadata {
            key: "my_analyzer".into(),
            default_enabled: false,
            name: "My Analyzer".into(),
            description: "Fires on every SIB1 for testing.".into(),
            version: 1,
        }
    }

    fn analyze_information_element(
        &mut self,
        ie: &InformationElement,
        _packet_num: usize,
        _timestamp: DateTime<FixedOffset>,
    ) -> Option<Event> {
        let InformationElement::LTE(lte_ie) = ie else { return None };
        let LteInformationElement::BcchDlSch(sch_msg) = &**lte_ie else { return None };
        let BCCH_DL_SCH_MessageType::C1(c1) = &sch_msg.message else { return None };
        let BCCH_DL_SCH_MessageType_c1::SystemInformationBlockType1(_sib1) = c1 else {
            return None;
        };
        Some(Event {
            event_type: EventType::Informational,
            message: "SIB1 received".to_string(),
        })
    }
}
```

## Registering the analyzer

Add it to `all_analyzers()` in [`lib/src/analysis/analyzer.rs`](https://github.com/EFForg/rayhunter/blob/main/lib/src/analysis/analyzer.rs):

```rust
// 1. Add the import at the top of analyzer.rs
use super::my_analyzer::MyAnalyzer;

// 2. Add an entry in all_analyzers()
(MyAnalyzer::metadata(), boxed(MyAnalyzer {})),
```

The order in `all_analyzers()` determines the display order in the web UI.

If your analyzer needs constructor parameters (e.g. `ImsiRequestedAnalyzer` receives `home_plmn`), pass them in the `boxed(...)` call.

## Documenting the heuristic for users

Add a section to [`doc/heuristics.md`](./heuristics.md) explaining:
- What signal the heuristic looks for.
- Why that signal is suspicious.
- Known false-positive conditions.

The key in the doc heading should match `AnalyzerMetadata::key` so users can cross-reference their `config.toml`.

## Testing

Write unit tests directly in your analyzer's file. The `process_row_records_skip_reason_and_event` pattern in `check/src/main.rs` shows how to call `Harness::analyze_qmdl_message`. For most analyzers, build an `InformationElement` from the relevant decoded struct and call `analyze_information_element()` directly:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    #[test]
    fn fires_on_sib1() {
        let mut analyzer = MyAnalyzer {};
        // Build an InformationElement::LTE(…) with a SIB1 payload and call:
        // let event = analyzer.analyze_information_element(&ie, 0, timestamp);
        // assert!(event.is_some());
    }
}
```

Capture real traffic with [`rayhunter-check`](./analyzing-a-capture.md) and run it over your QMDL file to validate false-positive rates on real recordings before opening a PR.
