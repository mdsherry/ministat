use anyhow::Error;
use noisy_float::prelude::*;
use std::borrow::Borrow;
use std::io::Write;
use std::iter;

use crate::args::Opt;
use crate::data::Dataset;
use crate::err::MinistatFailure;
use crate::stats::Stats;

pub static CLASSIC_SYMBOLS: [char; 8] = [' ', 'x', '+', '*', '%', '#', '@', 'O'];
pub static UNICODE_SYMBOLS: [char; 8] = [' ', '●', '○', '◾', '◽', '◆', '◇', '▲'];

struct DrawingChars {
    ul: char,
    ur: char,
    horiz: char,
    vert: char,
    ll: char,
    lr: char,
    bar_start: char,
    bar_end: char,
    bar: char,
}

static CLASSIC_CHARS: DrawingChars = DrawingChars {
    ul: '+',
    ur: '+',
    horiz: '-',
    vert: '|',
    ll: '+',
    lr: '+',
    bar_start: '|',
    bar_end: '|',
    bar: '_'
};
static MODERN_CHARS: DrawingChars = DrawingChars {
    ul: '┌',
    ur: '┐',
    horiz: '─',
    vert: '│',
    ll: '└',
    lr: '┘',
    bar_start: '├',
    bar_end: '┤',
    bar: '─'
};

pub struct Plot {
    width: u16,
    max: f64,
    min: f64,
}

impl Plot {
    pub fn new(width: u16, stats: &[Stats]) -> Result<Self, MinistatFailure> {
        let max = stats
            .iter()
            .map(|stat| r64(stat.max))
            .chain(stats.iter().map(|stat| r64(stat.mean + stat.stddev)))
            .max()
            .ok_or(MinistatFailure::NoPlotPossible)?
            .raw();

        let min = stats
            .iter()
            .map(|stat| r64(stat.min))
            .chain(stats.iter().map(|stat| r64(stat.mean - stat.stddev)))
            .min()
            .ok_or(MinistatFailure::NoPlotPossible)?
            .raw();
        Ok(Plot { width, min, max })
    }

    pub fn draw<W, T>(&self, f: &mut W, data: &[T], stats: &[Stats], symbols: &[char], opt: &Opt) -> Result<(), Error> 
    where
        W: Write,
        T: Borrow<[f64]>
     {
        let drawing_chars = if opt.modern_chars {
            &MODERN_CHARS
        } else {
            &CLASSIC_CHARS
        };
        let col_count = (self.width - 2) as usize;
        let mut columns: Vec<Vec<usize>> = iter::repeat(Vec::new()).take(col_count).collect();
        let dx = (self.max - self.min) / ((col_count - 1) as f64);
        let zero_point = self.min - 0.5 * dx;
        let discretize = |pt: f64| ((pt - zero_point) / dx) as usize;

        for (idx, dataset) in data.iter().enumerate() {
            let mut height = 0;
            let mut last_seen = None;
            for datum in dataset.borrow().iter() {
                let x = discretize(*datum);
                if opt.stack {
                    columns[x].push(idx + 1);
                } else {
                    match last_seen {
                        Some(last) if last == x => {
                            height += 1;
                        }
                        _ => {
                            height = 1;
                            last_seen = Some(x);
                        }
                    }
                    if columns[x].len() < height {
                        columns[x].push(idx + 1);
                    } else {
                        columns[x][height - 1] |= idx + 1;
                    }
                }
            }
        }

        let max_height = columns.iter().map(|c| c.len()).max().unwrap();

        writeln!(f,
            "{}{}{}",
            drawing_chars.ul,
            drawing_chars.horiz.to_string().repeat(col_count),
            drawing_chars.ur
        )?;

        for row in (0..max_height).rev() {
            let mut row_text = String::new();
            for col in &columns {
                if col.len() > row {
                    row_text.push(symbols[col[row]]);
                } else {
                    row_text.push(' ');
                }
            }

            writeln!(f, "{}{}{}", drawing_chars.vert, row_text, drawing_chars.vert)?;
        }
        let draw_on_bar = |bar: &mut Vec<char>, stat: &Stats| {
            let std_low = discretize(stat.mean - stat.stddev);
            let std_high = discretize(stat.mean + stat.stddev);
            bar[std_low] = drawing_chars.bar_start;
            bar[std_high] = drawing_chars.bar_end;
            for bar_segment in bar.iter_mut().take(std_high).skip(std_low + 1) {
                if *bar_segment == ' ' {
                    *bar_segment = drawing_chars.bar;
                }
            }
            bar[discretize(stat.mean)] = 'A';
            bar[discretize(stat.median)] = 'M';
        };
        let make_bar = || iter::repeat(' ').take(col_count).collect::<Vec<_>>();
        if opt.separate_lines {
            for stat in stats.iter() {
                let mut bar = make_bar();
                draw_on_bar(&mut bar, stat);
                writeln!(f,
                    "{}{}{}",
                    drawing_chars.vert,
                    bar.into_iter().collect::<String>(),
                    drawing_chars.vert
                )?;
            }
        } else {
            let mut bar = make_bar();
            for stat in stats.iter() {
                draw_on_bar(&mut bar, stat);
            }
            writeln!(f,
                "{}{}{}",
                drawing_chars.vert,
                bar.into_iter().collect::<String>(),
                drawing_chars.vert
            )?;
        }

        writeln!(f,
            "{}{}{}",
            drawing_chars.ll,
            drawing_chars.horiz.to_string().repeat(col_count),
            drawing_chars.lr
        )?;
        Ok(())
    }
}

pub fn plot_graph<W, T>(f: &mut W, width: u16, opt: &Opt, stats: &[Stats], data: &[T], char_set: &[char]) -> Result<(), Error>
where W: Write, T: Borrow<[f64]> {
    let plot = Plot::new(width, stats)?;
    plot.draw(f, data, stats, char_set, opt)?;
    Ok(())
}


pub fn print_heading<W>(f: &mut W, sets: &[Dataset], symbols: &[char]) -> Result<(), Error> where W: Write {
    for (symbol, set) in symbols.iter().skip(1).zip(sets.iter()) {
        writeln!(f, "{} {}", symbol, set.path.to_string_lossy())?;
    }
    Ok(())
}


#[cfg(test)]
mod test {
    use std::{path::PathBuf, str::FromStr};

    use crate::{print_heading, data::Dataset, stats::Stats, args::Opt, plot::{CLASSIC_SYMBOLS, UNICODE_SYMBOLS}};
    use super::Plot;

    #[test]
    fn test_print_heading_classic() {
        let mut buf = Vec::new();
        let datasets = vec![
            Dataset { path: PathBuf::from_str("file1").unwrap(), data: vec![1.0, 2.0, 3.0, 4.0] },
            Dataset { path: PathBuf::from_str("file2").unwrap(), data: vec![1.0, 2.0, 3.0, 4.0] },
            Dataset { path: PathBuf::from_str("file3").unwrap(), data: vec![1.0, 2.0, 3.0, 4.0] },
            Dataset { path: PathBuf::from_str("file4").unwrap(), data: vec![1.0, 2.0, 3.0, 4.0] },
            Dataset { path: PathBuf::from_str("file5").unwrap(), data: vec![1.0, 2.0, 3.0, 4.0] },
            Dataset { path: PathBuf::from_str("file6").unwrap(), data: vec![1.0, 2.0, 3.0, 4.0] },
            Dataset { path: PathBuf::from_str("file7").unwrap(), data: vec![1.0, 2.0, 3.0, 4.0] },
        ];
        print_heading(&mut buf, &datasets, &CLASSIC_SYMBOLS).unwrap();
        assert_eq!("x file1\n+ file2\n* file3\n% file4\n# file5\n@ file6\nO file7\n", std::str::from_utf8(&buf).unwrap());
    }
    #[test]
    fn test_print_heading_modern() {
        let mut buf = Vec::new();
        let datasets = vec![
            Dataset { path: PathBuf::from_str("file1").unwrap(), data: vec![1.0, 2.0, 3.0, 4.0] },
            Dataset { path: PathBuf::from_str("file2").unwrap(), data: vec![1.0, 2.0, 3.0, 4.0] },
            Dataset { path: PathBuf::from_str("file3").unwrap(), data: vec![1.0, 2.0, 3.0, 4.0] },
            Dataset { path: PathBuf::from_str("file4").unwrap(), data: vec![1.0, 2.0, 3.0, 4.0] },
            Dataset { path: PathBuf::from_str("file5").unwrap(), data: vec![1.0, 2.0, 3.0, 4.0] },
            Dataset { path: PathBuf::from_str("file6").unwrap(), data: vec![1.0, 2.0, 3.0, 4.0] },
            Dataset { path: PathBuf::from_str("file7").unwrap(), data: vec![1.0, 2.0, 3.0, 4.0] },
        ];
        print_heading(&mut buf, &datasets, &UNICODE_SYMBOLS).unwrap();
        assert_eq!("● file1\n○ file2\n◾ file3\n◽ file4\n◆ file5\n◇ file6\n▲ file7\n", std::str::from_utf8(&buf).unwrap());
    }

    #[test]
    // We only handle 7 datasets, because that's how many symbols we have
    // We check earlier that we don't have too many datasets
    fn test_print_heading_too_many() {
        let mut buf = Vec::new();
        let datasets = vec![
            Dataset { path: PathBuf::from_str("file1").unwrap(), data: vec![1.0, 2.0, 3.0, 4.0] },
            Dataset { path: PathBuf::from_str("file2").unwrap(), data: vec![1.0, 2.0, 3.0, 4.0] },
            Dataset { path: PathBuf::from_str("file3").unwrap(), data: vec![1.0, 2.0, 3.0, 4.0] },
            Dataset { path: PathBuf::from_str("file4").unwrap(), data: vec![1.0, 2.0, 3.0, 4.0] },
            Dataset { path: PathBuf::from_str("file5").unwrap(), data: vec![1.0, 2.0, 3.0, 4.0] },
            Dataset { path: PathBuf::from_str("file6").unwrap(), data: vec![1.0, 2.0, 3.0, 4.0] },
            Dataset { path: PathBuf::from_str("file7").unwrap(), data: vec![1.0, 2.0, 3.0, 4.0] },
            Dataset { path: PathBuf::from_str("file8").unwrap(), data: vec![1.0, 2.0, 3.0, 4.0] },
        ];
        print_heading(&mut buf, &datasets, &CLASSIC_SYMBOLS).unwrap();
        assert_eq!("x file1\n+ file2\n* file3\n% file4\n# file5\n@ file6\nO file7\n", std::str::from_utf8(&buf).unwrap());
    }

 
    #[test]
    fn test_plot() {
        let data = vec![
            vec![1., 2., 4., 8., 16.,], // mean 6.2, median 4.0
            vec![5., 6., 7., 8., 9.,], // mean and median: 7.0
        ];
        let stats: Vec<_> = data.iter().map(|d| Stats::from_dataset(&*d)).collect();
        let plot = Plot::new(30, &stats).unwrap();
        let mut buf = Vec::new();
        let opt = Opt {
            separate_lines: true,
            stack: true,
            ..Opt::default()
        };
        plot.draw(&mut buf, &data, &stats, &CLASSIC_SYMBOLS, &opt).unwrap();
        assert_eq!("\
+----------------------------+
|             +              |
|  xx   x+ + +x +           x|
||______M__A__________|      |
|         |__M_|             |
+----------------------------+
", std::str::from_utf8(&buf).unwrap());
    }
    #[test]
    fn test_plot_stacked() {
        let data = vec![
            vec![1., 2., 4., 8., 16.,], // mean 6.2, median 4.0
            vec![5., 6., 7., 8., 9.,], // mean and median: 7.0
        ];
        let stats: Vec<_> = data.iter().map(|d| Stats::from_dataset(&*d)).collect();
        let plot = Plot::new(30, &stats).unwrap();
        let mut buf = Vec::new();
        let opt = Opt {
            separate_lines: false,
            stack: false,
            ..Opt::default()
        };
        plot.draw(&mut buf, &data, &stats, &[' ', '1', '2', '3', '4', '5', '6'], &opt).unwrap();
        assert_eq!("\
+----------------------------+
|  11   12 2 23 2           1|
||______M_|A_M_|______|      |
+----------------------------+
", std::str::from_utf8(&buf).unwrap());
    }

    #[test]
    fn test_plot_modern() {
        let data = vec![
            vec![1., 2., 4., 8., 16.,], // mean 6.2, median 4.0
            vec![5., 6., 7., 8., 9.,], // mean and median: 7.0
        ];
        let stats: Vec<_> = data.iter().map(|d| Stats::from_dataset(&*d)).collect();
        let plot = Plot::new(30, &stats).unwrap();
        let mut buf = Vec::new();
        let opt = Opt {
            separate_lines: false,
            stack: false,
            modern_chars: true,
            ..Opt::default()
        };
        plot.draw(&mut buf, &data, &stats, &UNICODE_SYMBOLS, &opt).unwrap();
        assert_eq!("\
┌────────────────────────────┐
│  ●●   ●○ ○ ○◾ ○           ●│
│├──────M─├A─M─┤──────┤      │
└────────────────────────────┘
", std::str::from_utf8(&buf).unwrap());
    }
}