# ADR-002 - NO_EVIDENCE is never PASS

Status: ACCEPTED
Date: 2026-09-27

## Decision

A rule that observes zero applicable targets cannot pass.

For policy/compliance/quality assertions:
- zero observations -> NO_EVIDENCE;
- partial required fields -> INCOMPLETE_EVIDENCE;
- full compliant evidence -> PASS;
- observed violation -> FAIL.

This rule applies across CLI reports, JSON output, experiment summaries and future GUI.
