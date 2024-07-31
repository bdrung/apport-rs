Apport benchmark written in Rust
================================

This project re-implements the Python database benchmarks in Rust for
[apport-retrace needs more than 1 GB memory (when using sandbox)](https://bugs.launchpad.net/ubuntu/+source/apport/+bug/2073787).
This code is hacked together quick and dirty. Documentation is the source code.

## Ryzen 7 5700G desktop with noble Contents

```
Benchmark 1: target/release/apport -V1
  Time (mean ± σ):      9.497 s ±  0.110 s    [User: 7.940 s, System: 1.439 s]
  Range (min … max):    9.370 s …  9.758 s    10 runs

Benchmark 2: target/release/apport -V2
  Time (mean ± σ):      9.747 s ±  0.104 s    [User: 8.291 s, System: 1.334 s]
  Range (min … max):    9.620 s …  9.911 s    10 runs

Benchmark 3: target/release/apport -V3
  Time (mean ± σ):      9.440 s ±  0.091 s    [User: 8.492 s, System: 0.825 s]
  Range (min … max):    9.309 s …  9.584 s    10 runs

Summary
  target/release/apport -V3 ran
    1.01 ± 0.02 times faster than target/release/apport -V1
    1.03 ± 0.01 times faster than target/release/apport -V2
```

| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `target/release/apport -V1` | 9.497 ± 0.110 | 9.370 | 9.758 | 1.01 ± 0.02 |
| `target/release/apport -V2` | 9.747 ± 0.104 | 9.620 | 9.911 | 1.03 ± 0.01 |
| `target/release/apport -V3` | 9.440 ± 0.091 | 9.309 | 9.584 | 1.00 |
