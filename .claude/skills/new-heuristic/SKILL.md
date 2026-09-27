---
name: new-heuristic
description: Scaffold a complete new rayhunter Analyzer with correct boilerplate — creates the file, wires it into all_analyzers(), and adds the doc entry
---

# New Heuristic Scaffolder

Ask the user for:
1. **Snake-case key** (e.g. `timing_advance_outlier`) — used as the filename, config key, and `metadata().key`. **This value is permanent — it becomes a user-visible config key that must never change between versions.**
2. **Struct name** (e.g. `TimingAdvanceAnalyzer`) — PascalCase Rust struct name
3. **Display name** (e.g. "Timing Advance Outlier") — shown in the web UI
4. **Description** — one sentence for `AnalyzerMetadata.description`
5. **Severity** — `Informational`, `Low`, `Medium`, or `High`
6. **Which `InformationElement` variant(s)** it inspects — check `lib/src/analysis/information_element.rs` for the full list

Then perform these steps in order:

## Step 1 — Create the analyzer file

Create `lib/src/analysis/<key>.rs`:

```rust
use crate::analysis::analyzer::{Analyzer, AnalyzerMetadata, Event, EventType};
use crate::analysis::information_element::InformationElement;
use chrono::{DateTime, FixedOffset};

pub struct <StructName>;

impl Default for <StructName> {
    fn default() -> Self {
        Self::new()
    }
}

impl <StructName> {
    pub fn new() -> Self {
        <StructName>
    }
}

impl Analyzer for <StructName> {
    fn metadata() -> AnalyzerMetadata {
        AnalyzerMetadata {
            key: "<key>".into(),
            default_enabled: true,
            name: "<Display Name>".into(),
            description: "<Description>".into(),
            version: 1,
        }
    }

    fn analyze_information_element(
        &mut self,
        ie: &InformationElement,
        _packet_num: usize,
        _timestamp: DateTime<FixedOffset>,
    ) -> Option<Event> {
        // TODO: match on the relevant IE variant(s) and return Some(Event {...}) or None
        let _ = ie;
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_target_ie_returns_none() {
        let mut a = <StructName>::new();
        // Test that unrelated IE types are ignored
        assert!(a.analyze_information_element(&InformationElement::GSM, 0,
            chrono::DateTime::parse_from_rfc3339("2025-01-01T00:00:00+00:00").unwrap()
        ).is_none());
    }
}
```

## Step 2 — Register in `mod.rs`

In `lib/src/analysis/mod.rs`, add:
```rust
pub mod <key>;
```

## Step 3 — Register in `all_analyzers()`

In `lib/src/analysis/analyzer.rs`:
- Add import: `use super::<key>::<StructName>;`
- Add to the array in `all_analyzers()`: `(StructName::metadata(), boxed(StructName::new())),`

## Step 4 — Add doc entry

In `doc/src/heuristics.md`, add a section:
```markdown
## <Display Name>

**Config key:** `<key>`  
**Default:** Enabled  
**Severity:** <Severity>

<Description>. Explain what this detects, why it matters for IMSI catcher detection,
known false positive conditions, and any limitations.
```

## Step 5 — Verify

Run `cargo check -p rayhunter` and fix any compile errors before declaring done.

## Constraints
- `metadata().key` is a config file key — once shipped it must never change (would break user configs). Choose carefully.
- Bump `metadata().version` whenever the heuristic's detection logic changes in a released version.
- Every analyzer needs at least one test covering the "returns None for irrelevant IE type" path.
