# Rayhunter Mom Test

Use this command to pressure-test a Rayhunter feature, issue, or proposed enhancement before implementation.

## Scope

This command applies only to Rayhunter / EFForg rayhunter, especially Orbic hardware and shared Rayhunter features. It is not a Velociraptor Claw workflow.

## Procedure

1. State the user problem in one sentence without naming the proposed solution.
2. Identify who has the problem: device owner, investigator, developer, maintainer, or contributor.
3. Replace hypothetical enthusiasm with concrete evidence: ask what the user currently does, what failed, how often it occurs, and what evidence exists in captures, logs, issues, or device behavior.
4. Separate observed behavior from interpretation and from the requested feature.
5. Identify the smallest testable improvement and the exact repository territory it would touch.
6. Check whether the issue is Orbic-specific or shared/general Rayhunter work.
7. Define a falsifiable acceptance test, including local tests, provider CI, and hardware validation when applicable.
8. If the evidence is insufficient, classify the proposal as `NEEDS_EVIDENCE` rather than inventing demand or implementation details.

## Required output

```markdown
## Problem

## Evidence observed

## Evidence missing

## User and affected workflow

## Orbic-specific or shared/general

## Smallest testable change

## Falsifiable acceptance test

## Decision
- PROCEED
- NEEDS_EVIDENCE
- REJECT_SPECULATION
```

Do not edit source, create a Kanban card, or claim product validation unless the user explicitly requests that follow-up.
