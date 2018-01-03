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
            max: max,
            min: min,
            stddev: stddev,
            median: median,
            mean: mean,
            var: var,
            n: data.len(),
        }
    }
}

pub fn print_stats(stats: &Vec<Stats>, confidence_idx: usize, raw_stats: bool, modern_chars: bool) {
    use plot::{UNICODE_SYMBOLS, CLASSIC_SYMBOLS};
    let symbols = if modern_chars { &UNICODE_SYMBOLS } else { &CLASSIC_SYMBOLS };
    use t_table::{T_TABLE, T_CONFIDENCES};

    let confidence_label = T_CONFIDENCES[confidence_idx];
    println!("{symbol} {N:>3} {Min:>13} {Max:>13} {Median:>13} {Avg:>13} {Stddev:>13}",
             symbol = ' ',
             N = "N",
             Min = "Min",
             Max = "Max",
             Median = "Median",
             Avg = "Avg",
             Stddev = "Stddev");
    let mut first_stats = None;
    let fmt_decimal =
        |x| format!("{:13.6}", x).trim_right_matches('0').trim_right_matches('.').to_string();
    for (&symbol, ref stats) in symbols.iter().skip(1).zip(stats.iter()) {
        println!("{symbol} {N:>3} {Min:>13} {Max:>13} {Median:>13} {Avg:>13} \
                  {Stddev:>13}",
                 symbol = symbol,
                 N = stats.n,
                 Min = fmt_decimal(stats.min),
                 Max = fmt_decimal(stats.max),
                 Median = fmt_decimal(stats.median),
                 Avg = fmt_decimal(stats.mean),
                 Stddev = fmt_decimal(stats.stddev));
        if !raw_stats && first_stats.is_none() {
            first_stats = Some(stats.clone());
        } else {
            if let &Some(ref fs) = &first_stats {
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
                    println!("Difference at {}% confidence", confidence_label);
                    println!("\t{:.6} +/- {:.6}", stats.mean - fs.mean, t_required * val);
                    println!("\t{:.6}% +/- {:.6}%",
                             (stats.mean - fs.mean) / fs.mean * 100.,
                             t_required * val * 100. / fs.mean);
                    println!("\t(Welch's t = {:.6})", t);
                } else {
                    println!("No difference proven at {}% confidence", confidence_label);
                }
            }
        }
    }
}
