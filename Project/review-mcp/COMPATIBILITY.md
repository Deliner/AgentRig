# Codex compatibility evidence

Observed on Linux with codex-cli 0.153.4 and the configured gpt-5.6-luna model at high reasoning effort.

A real Codex process ran in bubblewrap with synthetic read-only project/input and hook mounts, its own writable work directory, a separate CODEX_HOME containing only a copied auth.json, private /tmp and PID namespace, system runtime files and no host home/project mount. Flags included ignore-user-config, ignore-rules, ephemeral mode and explicit trust for the vetted smoke hook. Both codex and its sibling codex-code-mode-host must be mounted; omitting the latter made tool execution fail and hit the probe timeout.

The initial instruction requested a final response without writing a file. Codex replied Ready. The configured Stop hook rejected the missing file using a blocking decision and requested the exact output. Codex continued, wrote review.json containing {"ok":true}, and exited successfully. This proves blocking continuation on the installed version; it does not yet prove the production validator, protected attempt counter, MCP timeout behavior or final isolation acceptance.

The smoke removed copied auth material after each invocation. Temporary probe directories were removed after recording this evidence. The implementation must retain a reproducible smoke test and independently verify filesystem visibility and isolation of user skills/config/hooks.

[Official Stop hook contract](https://learn.chatgpt.com/docs/hooks#stop) describes blocking continuation and hook trust. Actual installed behavior above determines compatibility for this implementation.

The initial production runner completed a review using gpt-5.6-luna/high:
valid PASS response, one Stop validation, exact JSON retained in the report,
and runtime cleanup confirmed absent. The fixture reviewed committed Python
code while the runner exported its Git snapshot. Deterministic Rust integration
tests additionally execute real bubblewrap with two simulated CLI processes,
assert read-only mounts and hidden host configuration, and exercise timeouts,
format exhaustion, failure reports and repeat-review findings. These tests do
not substitute for the remaining real MCP-client and Codex configuration smoke.
