use noisy_float::prelude::*;
use std::borrow::Borrow;
use std::iter;

use crate::args::Opt;
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
    symbols: &'static [char],
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
    bar: '_',
    symbols: &CLASSIC_SYMBOLS,
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
    bar: '─',
    symbols: &UNICODE_SYMBOLS,
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

    pub fn draw<T: Borrow<[f64]>>(&self, data: &[T], stats: &[Stats], opt: &Opt) {
        let charset = if opt.modern_chars {
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

        println!(
            "{}{}{}",
            charset.ul,
            charset.horiz.to_string().repeat(col_count),
            charset.ur
        );

        for row in (0..max_height).rev() {
            let mut row_text = String::new();
            for col in &columns {
                if col.len() > row {
                    row_text.push(charset.symbols[col[row]]);
                } else {
                    row_text.push(' ');
                }
            }

            println!("{}{}{}", charset.vert, row_text, charset.vert);
        }
        let draw_on_bar = |bar: &mut Vec<char>, stat: &Stats| {
            let std_low = discretize(stat.mean - stat.stddev);
            let std_high = discretize(stat.mean + stat.stddev);
            bar[std_low] = charset.bar_start;
            bar[std_high] = charset.bar_end;
            for bar_segment in bar.iter_mut().take(std_high).skip(std_low + 1) {
                if *bar_segment == ' ' {
                    *bar_segment = charset.bar;
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
                println!(
                    "{}{}{}",
                    charset.vert,
                    bar.into_iter().collect::<String>(),
                    charset.vert
                );
            }
        } else {
            let mut bar = make_bar();
            for stat in stats.iter() {
                draw_on_bar(&mut bar, stat);
            }
            println!(
                "{}{}{}",
                charset.vert,
                bar.into_iter().collect::<String>(),
                charset.vert
            );
        }

        println!(
            "{}{}{}",
            charset.ll,
            charset.horiz.to_string().repeat(col_count),
            charset.lr
        );
    }
}

pub fn plot_graph<T: Borrow<[f64]>>(width: u16, opt: &Opt, stats: &[Stats], data: &[T]) {
    let plot = Plot::new(width, stats);
    if plot.is_err() {
        return;
    }
    let plot = plot.unwrap();
    plot.draw(data, stats, opt);
}
