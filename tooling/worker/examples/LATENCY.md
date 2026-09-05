# Observed standalone latency

Measured 2026-09-05T01:50:15.733538+00:00 using discipline-worker 0.1.0 on Linux x86_64, 7.0.11-76070011-generic.

Each language uses a new consumer installation of its independent source example; its three operations share that installation. First is the first measured process for that operation; repeated results use 50 fresh processes. This is a cold application process, not a flushed OS page cache. Compilation and installation are excluded. Timings include process startup, configuration loading, filesystem work and captured output. The edit event has no transcript; it measures file guidance, not large transcript scanning. Lint runs the five default rules on the small example source tree.

| Project | Operation | First ms | Repeated median ms | Repeated p95 ms |
| --- | --- | ---: | ---: | ---: |
| python | SessionStart | 1.711 | 1.395 | 1.816 |
| python | PreToolUse edit | 1.299 | 1.364 | 1.933 |
| python | small source lint | 1.985 | 1.949 | 2.283 |
| rust | SessionStart | 1.757 | 1.360 | 1.629 |
| rust | PreToolUse edit | 1.338 | 1.334 | 1.462 |
| rust | small source lint | 2.050 | 1.882 | 2.043 |

These local observations describe this workload, not a latency guarantee. No additional cache, daemon or parallel scheduler was introduced for this measurement. Reproduce with `python3 tooling/worker/examples/measure.py --worker /absolute/path/discipline-worker --output /path/to/report.md`.
