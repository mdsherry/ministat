use crate::err::*;
use crate::t_table::T_CONFIDENCES;
use clap::Parser;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[clap(name = "ministat", about = "A Rust port of the ministat utility")]
pub struct Opt {
    #[clap(short = 'n', long = "raw")]
    /// Just report the raw statistics of the input, suppress the ASCII-art plot
    /// and the relative comparisons.
    pub raw_stats: bool,

    #[clap(short = 's', long = "separate")]
    /// Print the average/median/stddev bars on separate lines in the ASCII-art
    /// plot, to avoid overlap.
    pub separate_lines: bool,

    #[clap(short = 'A', long = "stats-only")]
    /// Print statistics only. Suppress the graph.
    pub stats_only: bool,

    #[clap(short = 'm', long = "modern")]
    /// Use non-ASCII characters for drawing the graph
    pub modern_chars: bool,

    #[clap(short = 't', long = "stack")]
    /// Stack datapoints in the graph instead of overlapping them
    pub stack: bool,

    #[clap(short = 'C', long = "column", default_value = "1")]
    /// Specify which column of data to use. By default the first column in the
    /// input file(s) are used.
    pub column: Column,

    #[clap(short = 'c', long = "confidence", default_value = "95")]
    /// Specify desired confidence level for Student's T analysis.  Possible
    /// values are 80, 90, 95, 98, 99 and 99.5%
    pub confidence_level: Confidence,

    #[clap(short = 'd', long = "delimit", default_value = " \t")]
    /// Specifies the column delimiter characters, default is SPACE and TAB.
    /// See strtok(3) for details.
    pub delimiter: String,

    #[clap(short = 'w', long = "width")]
    /// Width of ASCII-art plot in characters, default is terminal width, or 74.
    pub width: Option<u16>,

    #[clap(parse(from_os_str))]
    /// Files containing datapoints to compute statistics for
    pub files: Vec<PathBuf>,
}

impl Default for Opt {
    fn default() -> Self {
        Self {
            raw_stats: false,
            separate_lines: false,
            stats_only: false,
            modern_chars: false,
            stack: false,
            column: Column(1),
            confidence_level: Confidence(95),
            delimiter: " \t".into(),
            width: None,
            files: vec![]
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Column(pub u8);
impl FromStr for Column {
    type Err = MinistatFailure;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<i8>() {
            Ok(c) if c >= 1 => Ok(Column(c as u8)),
            _ => Err(MinistatFailure::InvalidColumn {
                provided_column: s.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Confidence(pub usize);
impl FromStr for Confidence {
    type Err = MinistatFailure;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        T_CONFIDENCES
            .iter()
            .position(|c| c == &s)
            .ok_or_else(|| MinistatFailure::InvalidConfidence {
                provided_confidence: s.to_string(),
            })
            .map(Confidence)
    }
}
