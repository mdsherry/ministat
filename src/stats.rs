use std::io::Write;

use anyhow::Error;

#[derive(Debug, Clone)]
pub struct Stats {
    pub n: usize,
    pub max: f64,
    pub min: f64,
    pub var: f64,
    pub stddev: f64,
    pub median: f64,
    pub mean: f64,
}

impl Stats {
    pub fn from_dataset(data: &[f64]) -> Self {
        use kahan::KahanSum;
        let mut s = KahanSum::new();
        let mut m = data[0];
        let mut max = m;
        let mut min = m;
        let mut total = KahanSum::new_with_value(m);
        for (k, datum) in data.iter().enumerate().skip(1) {
            let datum = *datum;
            total += datum;
            if datum < min {
                min = datum;
            }
            if datum > max {
                max = datum;
            }
            let old_m = m;
            m = old_m + (datum - old_m) / ((k + 1) as f64);
            s += (datum - old_m) * (datum - m);
        }
        let s = s.sum();
        let total = total.sum();

        let var = s / ((data.len() - 1) as f64);
        let mean = total / (data.len() as f64);
        let stddev = var.sqrt();
        let median = if data.len() % 2 == 1 {
            data[data.len() / 2]
        } else {
            (data[data.len() / 2] + data[data.len() / 2 - 1]) / 2.
        };
        Stats {
            max,
            min,
            stddev,
            median,
            mean,
            var,
            n: data.len(),
        }
    }
}

pub fn print_stats<W>(
    f: &mut W,
    stats: &[Stats],
    confidence_idx: usize,
    raw_stats: bool,
    symbols: &[char],
) -> Result<(), Error>
where
    W: Write,
{
    use crate::t_table::{T_CONFIDENCES, T_TABLE};

    let confidence_label = T_CONFIDENCES[confidence_idx];
    // This isn't necessary, but helps maintain symmetry between the header and data rows
    let symbol = ' ';
    writeln!(
        f,
        "{symbol} {N:>3} {Min:>13} {Max:>13} {Median:>13} {Avg:>13} {Stddev:>13}",
        symbol = symbol,
        N = "N",
        Min = "Min",
        Max = "Max",
        Median = "Median",
        Avg = "Avg",
        Stddev = "Stddev"
    )?;
    let mut first_stats = None;
    let fmt_decimal = |x| {
        format!("{:13.6}", x)
            .trim_start_matches('0')
            .trim_start_matches('.')
            .to_string()
    };
    for (&symbol, stats) in symbols.iter().skip(1).zip(stats.iter()) {
        writeln!(
            f,
            "{symbol} {N:>3} {Min:>13} {Max:>13} {Median:>13} {Avg:>13} \
                  {Stddev:>13}",
            symbol = symbol,
            N = stats.n,
            Min = fmt_decimal(stats.min),
            Max = fmt_decimal(stats.max),
            Median = fmt_decimal(stats.median),
            Avg = fmt_decimal(stats.mean),
            Stddev = fmt_decimal(stats.stddev)
        )?;
        if !raw_stats && first_stats.is_none() {
            first_stats = Some(stats.clone());
        } else if let Some(ref fs) = first_stats {
            let val = stats.var / (stats.n as f64) + fs.var / (fs.n as f64);
            // Because we the sample sizes and variances might differ, we
            // use https://en.wikipedia.org/wiki/Welch%27s_t-test
            // to compute a t value.
            let t = ((stats.mean - fs.mean) / val.sqrt()).abs();
            let a = stats.var.powi(2) / (stats.n * stats.n * (stats.n - 1)) as f64;
            let b = fs.var.powi(2) / (fs.n * fs.n * (fs.n - 1)) as f64;
            let v = val.powi(2) / (a + b);

            let v_floor = v as usize;
            let t_required = if v_floor > 1000 {
                T_TABLE[1000][confidence_idx]
            } else {
                T_TABLE[v_floor - 1][confidence_idx]
            };
            if t > t_required {
                writeln!(f, "Difference at {}% confidence", confidence_label)?;
                writeln!(
                    f,
                    "\t{:.6} +/- {:.6}",
                    stats.mean - fs.mean,
                    t_required * val
                )?;
                writeln!(
                    f,
                    "\t{:.6}% +/- {:.6}%",
                    (stats.mean - fs.mean) / fs.mean * 100.,
                    t_required * val * 100. / fs.mean
                )?;
                writeln!(f, "\t(Welch's t = {:.6})", t)?;
            } else {
                writeln!(
                    f,
                    "No difference proven at {}% confidence",
                    confidence_label
                )?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::plot::CLASSIC_SYMBOLS;

    use super::{print_stats, Stats};

    #[test]
    fn test_stats() {
        let data = vec![
            vec![1., 2., 4., 8., 16.], // mean 6.2, median 4.0
            vec![5., 6., 7., 8., 9.],  // mean and median: 7.0
        ];
        let stats: Vec<_> = data.iter().map(|d| Stats::from_dataset(&*d)).collect();
        let mut buf = vec![];
        print_stats(&mut buf, &stats, 2, false, &CLASSIC_SYMBOLS).unwrap();
        let s = std::str::from_utf8(&buf).unwrap();
        assert_eq!(
            "    N           Min           Max        Median           Avg        Stddev
x   5      1.000000     16.000000      4.000000      6.200000      6.099180
+   5      5.000000      9.000000      7.000000      7.000000      1.581139
No difference proven at 95% confidence
",
            s
        );
    }

    #[test]
    fn test_stats2() {
        let data = vec![
            vec![1., 2., 4., 8., 16.],     // mean 6.2, median 4.0
            vec![15., 16., 17., 18., 19.], // mean and median: 17.0
        ];
        let stats: Vec<_> = data.iter().map(|d| Stats::from_dataset(&*d)).collect();
        let mut buf = vec![];
        print_stats(&mut buf, &stats, 2, false, &CLASSIC_SYMBOLS).unwrap();
        let s = std::str::from_utf8(&buf).unwrap();
        assert_eq!(
            "    N           Min           Max        Median           Avg        Stddev
x   5      1.000000     16.000000      4.000000      6.200000      6.099180
+   5     15.000000     19.000000     17.000000     17.000000      1.581139
Difference at 95% confidence
\t10.800000 +/- 22.045013
\t174.193548% +/- 355.564726%
\t(Welch's t = 3.832777)
",
            s
        );
    }
}
