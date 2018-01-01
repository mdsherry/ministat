# Ministat

A port of Freebsd's `ministat` utility to Rust.

## Differences from `ministat`

  * Uses Welch's t test instead of Student's, as Welch's handles cases where the datasets have
    different variances.
  * Optional use of non-ASCII characters for drawing the plot (`-m` or `--modern`)
  * Stack datapoints instead of overlapping (`-t` or `--stack`)