---
name: review-project
description: Request a configured isolated review of committed project changes through review MCP and return its validated report to the calling workflow.
---

Select the configured tool whose description and machine contract cover the requested review. Use the project's configured Git boundary: checkout root, original base and committed candidate. Uncommitted edits are not part of the review. For a repeated review pass the persistent previous JSON report and retain its original base.

Call that MCP tool with `root`, `base`, `candidate` and optional `previous_report`. The client timeout must exceed the runner timeout plus snapshot/report overhead. The worker reads the review configuration from `capabilities.review.config` in `agentrig.yaml`. Use `just review config-check` to diagnose configuration errors.

Return the report verdict, findings, technical/cleanup errors and persistent report location to the calling workflow. A technical failure is not PASS. Do not launch extra critics, reinterpret verdicts, change ledger or accept a workflow stage as part of this review. Any fixes belong to the caller's separately authorized work.
