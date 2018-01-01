use std::str::FromStr;
use std::path::PathBuf;
use err::*;
use t_table::T_CONFIDENCES;

#[derive(StructOpt, Debug)]
#[structopt(name="ministat", about = "A Rust port of the ministat utility")]
pub struct Opt {
    #[structopt(short="n")]
    /// Just report the raw statistics of the input, suppress the ASCII-art plot
    /// and the relative comparisons.
    pub raw_stats: bool,
    #[structopt(short="s")]
    /// Print the average/median/stddev bars on separate lines in the ASCII-art
    /// plot, to avoid overlap.
    pub separate_lines: bool,
    #[structopt(short="A")]
    /// Print statistics only. Suppress the graph.
    pub stats_only: bool,
    #[structopt(short="m")]
    /// Use non-ASCII characters for drawing the graph
    pub modern_chars: bool,
    #[structopt(short="C", default_value = "1")]
    /// Specify which column of data to use. By default the first column in the
    /// input file(s) are used.
    pub column: Column,
    #[structopt(short="c", default_value = "95")]
    /// Specify desired confidence level for Student's T analysis.  Possible
    /// values are 80, 90, 95, 98, 99 and 99.5%
    pub confidence_level: Confidence,
    #[structopt(short="d", default_value = " \t")]
    /// Specifies the column delimiter characters, default is SPACE and TAB.
    /// See strtok(3) for details.
    pub delimiter: String,
    #[structopt(short="w")]
    /// Width of ASCII-art plot in characters, default is terminal width, or 74.
    pub width: Option<u16>,
    /// Files containing datapoints to compute statistics for
    #[structopt(parse(from_os_str))]
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy)]
pub struct Column(pub u8);
impl FromStr for Column {
    type Err = MinistatFailure;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<i8>() {
            Ok(c) if c >= 1 => Ok(Column(c as u8)),
            _ => Err(MinistatFailure::InvalidColumn { provided_column: s.to_string() }),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Confidence(pub usize);
impl FromStr for Confidence {
    type Err = MinistatFailure;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        T_CONFIDENCES.iter().position(|c| c == &s).ok_or_else(|| {
            MinistatFailure::InvalidConfidence { provided_confidence: s.to_string() }
        }).map(|p| Confidence(p))
    }
}
