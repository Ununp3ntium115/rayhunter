# Adding a heuristic

Rayhunter's detection logic lives in Rust inside the `lib` crate. Each heuristic is a struct that implements the `Analyzer` trait and gets registered in a central list. This page walks you through all the moving parts.

## Concepts

**Analyzer** — a struct that inspects one packet at a time and optionally returns an `Event` when something suspicious is detected. Analyzers are stateful; they can accumulate context across packets to detect multi-step attack patterns.

**InformationElement (IE)** — the decoded contents of a single GSMTAP packet, as seen by analyzers. LTE RRC and NAS payloads are fully decoded; other types (GSM, UMTS) are passed through as opaque variants.

**Event** — the result an analyzer returns when it detects something. Has a `message` (human-readable explanation) and an `EventType` severity (`Low`, `Medium`, `High`, `Informational`).

## Step 1: Create the analyzer file

Create `lib/src/analysis/<key>.rs` where `<key>` is a permanent, snake_case identifier. This value becomes the user-visible config key and must never change once shipped.

The simplest complete analyzer (`lib/src/analysis/nas_null_cipher.rs`) looks like this:

```rust,ignore
use chrono::{DateTime, FixedOffset};

use super::analyzer::{Analyzer, AnalyzerMetadata, Event, EventType};
use super::information_element::{InformationElement, LteInformationElement};

pub struct MyAnalyzer;

impl Analyzer for MyAnalyzer {
    fn metadata() -> AnalyzerMetadata {
        AnalyzerMetadata {
            key: "my_analyzer".into(),       // permanent config key
            default_enabled: true,
            name: "My Analyzer".into(),
            description: "Detects suspicious behavior X.".into(),
            version: 1,
        }
    }

    fn analyze_information_element(
        &mut self,
        ie: &InformationElement,
        _packet_num: usize,
        _timestamp: DateTime<FixedOffset>,
    ) -> Option<Event> {
        // Return None for IEs you don't handle.
        let _lte_ie = match ie {
            InformationElement::LTE(inner) => inner,
            _ => return None,
        };

        // ... inspect the IE and return Some(Event{...}) when suspicious.
        None
    }
}
```

Key rules:
- Return `None` for every IE variant you don't handle.
- Keep state minimal. Use the Welford online algorithm (see `timing_advance.rs`) for running statistics rather than buffering raw values.
- Never call `.unwrap()` or `assert!` in non-test code. Use `?`, `match`, or `debug_assert!`.
- Bump `version` whenever the detection logic changes in a released version (changing `version` does not require changing `key`).

If your analyzer needs to know about packets it *didn't* analyze (e.g., to maintain a packet-count baseline), implement `report_skipped_packet`. The default is a no-op.

## Step 2: Register the module

In `lib/src/analysis/mod.rs`, add:

```rust,ignore
pub mod my_analyzer;
```

## Step 3: Add to `all_analyzers()`

In `lib/src/analysis/analyzer.rs`, add an import near the top:

```rust,ignore
use super::my_analyzer::MyAnalyzer;
```

Then add an entry inside `all_analyzers()`:

```rust,ignore
(MyAnalyzer::metadata(), boxed(MyAnalyzer::new())),
```

The `all_analyzers()` function is the single authoritative list; the order here determines the order events appear in the UI.

## Step 4: Update the config template

In `dist/config.toml.in`, add your key to the `[analyzers]` section:

```toml
my_analyzer = true
```

Missing keys fall back to `default_enabled` at runtime, so this is non-breaking — but adding it lets users see and override the toggle in their generated config.

## Step 5: Document the heuristic

In `doc/heuristics.md`, add a section:

```markdown
### My Analyzer (v1)

Short description of what this detects.

Longer explanation of:
- What cellular behavior triggers it
- Why that behavior is suspicious or indicative of an IMSI catcher
- Known false positive conditions (travel, airplane mode, SIM swap, etc.)
- Any v1 limitations (e.g., "does not yet partition by cell ID")
```

## Step 6: Write tests

Add a `#[cfg(test)] mod tests` block to your file. At minimum:

- A test confirming `analyze_information_element` returns `None` for an unrelated IE type.
- A test confirming it returns `Some(Event)` for a synthetic IE that matches the detection condition.

For analyzers with stateful logic, test the warmup period and the state transitions that lead to detection. See `timing_advance.rs` for an example.

## Verification

```sh
cargo check -p rayhunter
cargo clippy -p rayhunter -- -D warnings
cargo fmt --all --check
cargo test -p rayhunter
```

All four must be clean before submitting.

## Finding the right InformationElement variant

`lib/src/analysis/information_element.rs` contains the `InformationElement` and `LteInformationElement` enums with all supported packet types. For LTE RRC, the decoded ASN.1 types come from `telcom-parser`. For NAS, they come from `pycrate-rs`.

Use `tools/asn1grep.py` to search the ASN.1 specs in `telcom-parser/specs/`:

```sh
python3 tools/asn1grep.py "TimingAdvance"
```

## Example: the Timing Advance outlier analyzer

`lib/src/analysis/timing_advance.rs` is a recent, self-contained example that:
- Intercepts a non-GSMTAP IE type (`LteLl1ServingCellTiming`) added directly in `analyzer.rs`
- Uses Welford's online algorithm to detect statistical outliers without buffering
- Has four unit tests covering warmup, outlier detection, and false-positive suppression

It is a good template for any new numeric-threshold heuristic.
