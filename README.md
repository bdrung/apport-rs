Apport benchmark written in Rust
================================

This project re-implements the Python database benchmarks in Rust for
[apport-retrace needs more than 1 GB memory (when using sandbox)](https://bugs.launchpad.net/ubuntu/+source/apport/+bug/2073787).
This code is hacked together quick and dirty. Documentation is the source code.

## Ryzen 7 5700G desktop with noble Contents

```
$ hyperfine -r 10 "target/release/apport -V1" "target/release/apport -V2" "target/release/apport -V3" --export-markdown noble.md
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

```
$ du -B M -s *noble*sqlite3 | sort -n
390M	contents-noble_v3.sqlite3
657M	contents-noble_v2.sqlite3
720M	contents-noble_v1.sqlite3
```

## Ryzen 7 5700G desktop with jammy Contents

```
$ hyperfine -r 10 "target/release/apport -j -V1" "target/release/apport -j -V2" "target/release/apport -j -V3" --export-markdown jammy.md
Benchmark 1: target/release/apport -j -V1
  Time (mean ± σ):     26.923 s ±  0.263 s    [User: 23.622 s, System: 3.129 s]
  Range (min … max):   26.549 s … 27.374 s    10 runs

Benchmark 2: target/release/apport -j -V2
  Time (mean ± σ):     27.289 s ±  0.233 s    [User: 24.343 s, System: 2.767 s]
  Range (min … max):   27.058 s … 27.860 s    10 runs

Benchmark 3: target/release/apport -j -V3
  Time (mean ± σ):     26.664 s ±  0.197 s    [User: 24.900 s, System: 1.602 s]
  Range (min … max):   26.422 s … 27.072 s    10 runs

Summary
  target/release/apport -j -V3 ran
    1.01 ± 0.01 times faster than target/release/apport -j -V1
    1.02 ± 0.01 times faster than target/release/apport -j -V2
```

| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `target/release/apport -j -V1` | 26.923 ± 0.263 | 26.549 | 27.374 | 1.01 ± 0.01 |
| `target/release/apport -j -V2` | 27.289 ± 0.233 | 27.058 | 27.860 | 1.02 ± 0.01 |
| `target/release/apport -j -V3` | 26.664 ± 0.197 | 26.422 | 27.072 | 1.00 |

```
$ du -B M -s *jammy*sqlite3 | sort -n
693M	contents-jammy_v3.sqlite3
1275M	contents-jammy_v2.sqlite3
1468M	contents-jammy_v1.sqlite3
```

## Raspberry Pi Zero 2W with noble Contents

```
$ hyperfine -r 10 "target/release/apport -V1" "target/release/apport -V2" "target/release/apport -V3" --export-markdown noble.md
Benchmark 1: target/release/apport -V1
  Time (mean ± σ):     167.837 s ±  1.025 s    [User: 139.219 s, System: 13.616 s]
  Range (min … max):   165.820 s … 169.569 s    10 runs

Benchmark 2: target/release/apport -V2
  Time (mean ± σ):     174.587 s ±  1.447 s    [User: 142.980 s, System: 13.264 s]
  Range (min … max):   172.364 s … 177.241 s    10 runs

Benchmark 3: target/release/apport -V3
  Time (mean ± σ):     162.242 s ±  1.943 s    [User: 142.013 s, System: 8.973 s]
  Range (min … max):   159.858 s … 165.877 s    10 runs

Summary
  target/release/apport -V3 ran
    1.03 ± 0.01 times faster than target/release/apport -V1
    1.08 ± 0.02 times faster than target/release/apport -V2
```

| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `target/release/apport -V1` | 167.837 ± 1.025 | 165.820 | 169.569 | 1.03 ± 0.01 |
| `target/release/apport -V2` | 174.587 ± 1.447 | 172.364 | 177.241 | 1.08 ± 0.02 |
| `target/release/apport -V3` | 162.242 ± 1.943 | 159.858 | 165.877 | 1.00 |

## Raspberry Pi Zero 2W with jammy Contents

```
$ hyperfine -r 10 "target/release/apport -j -V1" "target/release/apport -j -V2" "target/release/apport -j -V3" --export-markdown jammy.md
Benchmark 1: target/release/apport -j -V1
  Time (mean ± σ):     507.049 s ±  2.166 s    [User: 432.024 s, System: 36.914 s]
  Range (min … max):   503.985 s … 509.524 s    10 runs

Benchmark 2: target/release/apport -j -V2
  Time (mean ± σ):     516.578 s ±  2.211 s    [User: 439.608 s, System: 33.816 s]
  Range (min … max):   511.680 s … 519.266 s    10 runs

Benchmark 3: target/release/apport -j -V3
  Time (mean ± σ):     476.953 s ±  2.101 s    [User: 432.375 s, System: 21.169 s]
  Range (min … max):   473.181 s … 479.840 s    10 runs

Summary
  target/release/apport -j -V3 ran
    1.06 ± 0.01 times faster than target/release/apport -j -V1
    1.08 ± 0.01 times faster than target/release/apport -j -V2
```

| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `target/release/apport -j -V1` | 507.049 ± 2.166 | 503.985 | 509.524 | 1.06 ± 0.01 |
| `target/release/apport -j -V2` | 516.578 ± 2.211 | 511.680 | 519.266 | 1.08 ± 0.01 |
| `target/release/apport -j -V3` | 476.953 ± 2.101 | 473.181 | 479.840 | 1.00 |
