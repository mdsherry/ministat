use noisy_float::prelude::*;
use std::borrow::Borrow;
use std::iter;

use SYMBOLS;
use stats::Stats;
use err::MinistatFailure;

pub struct Plot {
    width: u16,
    max: f64,
    min: f64,
}


impl Plot {
    pub fn new(width: u16, stats: &[Stats]) -> Result<Self, MinistatFailure> {
        let max = stats.iter()
            .map(|stat| r64(stat.max))
            .chain(stats.iter().map(|stat| r64(stat.mean + stat.stddev)))
            .max()
            .ok_or(MinistatFailure::NoPlotPossible)?
            .raw();

        let min = stats.iter()
            .map(|stat| r64(stat.min))
            .chain(stats.iter().map(|stat| r64(stat.mean - stat.stddev)))
            .min()
            .ok_or(MinistatFailure::NoPlotPossible)?
            .raw();
        Ok(Plot {
            width: width,
            min: min,
            max: max,
        })
    }

    pub fn draw<T: Borrow<[f64]>>(&self, data: &[T], stats: &[Stats], separate_lines: bool) {
        let col_count = (self.width - 2) as usize;
        let mut columns: Vec<Vec<usize>> = iter::repeat(Vec::new()).take(col_count).collect();
        let dx = (self.max - self.min) / ((col_count - 1) as f64);
        let zero_point = self.min - 0.5 * dx;
        let discretize = |pt: f64| ((pt - zero_point) / dx) as usize;
        let mut max_height = 0;
        for (idx, dataset) in data.iter().enumerate() {
            let mut height = 0;
            let mut last_seen = None;
            for datum in dataset.borrow().iter() {
                let x = discretize(*datum);
                match last_seen {
                    Some(last) if last == x => {
                        height += 1;
                    }
                    _ => {
                        if height > max_height {
                            max_height = height;
                        }
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
        println!("+{}+", "-".repeat(col_count));
        for row in (0..max_height).rev() {
            let mut row_text = String::new();
            for col in columns.iter() {
                if col.len() > row {
                    row_text.push(SYMBOLS[col[row]]);
                } else {
                    row_text.push(' ');
                }
            }
            println!("|{}|", row_text);
        }
        let draw_on_bar = |bar: &mut Vec<char>, stat: &Stats| {
            let std_low = discretize(stat.mean - stat.stddev);
            let std_high = discretize(stat.mean + stat.stddev);
            bar[std_low] = '|';
            bar[std_high] = '|';
            for i in (std_low + 1)..std_high {
                // Don't clobber other symbols
                if bar[i] == ' ' {
                    bar[i] = '_';
                }
            }
            bar[discretize(stat.mean)] = 'A';
            bar[discretize(stat.median)] = 'M';
        };
        let make_bar = || iter::repeat(' ').take(col_count).collect::<Vec<_>>();
        if separate_lines {
            for stat in stats.iter() {
                let mut bar = make_bar();
                draw_on_bar(&mut bar, stat);
                println!("|{}|", bar.into_iter().collect::<String>());
            }
        } else {
            let mut bar = make_bar();
            for stat in stats.iter() {
                draw_on_bar(&mut bar, stat);
            }
            println!("|{}|", bar.into_iter().collect::<String>());
        }


        println!("+{}+", "-".repeat(col_count));
    }
}

pub fn plot_graph<T: Borrow<[f64]>>(width: u16,
                                    separate_lines: bool,
                                    stats: &Vec<Stats>,
                                    data: &[T]) {
    let plot = Plot::new(width, stats);
    if plot.is_err() {
        return;
    }
    let plot = plot.unwrap();
    plot.draw(data, &stats, separate_lines);
}
