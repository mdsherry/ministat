extern crate structopt;
#[macro_use]
extern crate structopt_derive;
#[macro_use]
extern crate failure;
extern crate noisy_float;
extern crate terminal_size;
extern crate kahan;

mod args;
mod err;
mod t_table;
mod stats;
mod plot;
mod data;

use err::*;
use args::Opt;
use stats::*;
use plot::plot_graph;
use data::{Dataset, load_data};

use structopt::StructOpt;

use terminal_size::terminal_size;
use failure::Error;

fn print_heading(sets: &Vec<Dataset>, modern_chars: bool) {
    let symbols = if modern_chars { &plot::UNICODE_SYMBOLS } else { &plot::CLASSIC_SYMBOLS };
    for (symbol, set) in symbols.iter().skip(1).zip(sets.iter()) {
        println!("{} {}", symbol, set.path.to_string_lossy());
    }
}

fn get_width(opt: &Opt) -> u16 {
    opt.width.or_else(|| terminal_size().map(|ts| (ts.0).0)).unwrap_or(74)
}

fn run(opt: Opt) -> Result<(), Error> {
    if opt.files.len() > 7 {
        return Err(MinistatFailure::TooManyDatasets { dataset_count: opt.files.len() }.into());
    }
    let datasets = load_data(&opt)?;
    for ds in datasets.iter() {
        if ds.data.len() < 3 {
            return Err(MinistatFailure::InsufficientData {
                    file: ds.path.to_string_lossy().into_owned(),
                }
                .into());
        }
    }
    print_heading(&datasets, opt.modern_chars);
    let stats: Vec<_> = datasets.iter()
        .map(|dataset| Stats::from_dataset(&dataset.data))
        .collect();

    if !opt.raw_stats {
        if !opt.stats_only {
            plot_graph(get_width(&opt),
                       &opt,
                       &stats,
                       &datasets.into_iter().map(|x| x.data).collect::<Vec<_>>());
        }
    }
    print_stats(&stats, opt.confidence_level.0, opt.raw_stats, opt.modern_chars);

    Ok(())
}

fn main() {
    let opt = Opt::from_args();

    match run(opt) {
        Ok(()) => (),
        Err(error) => eprintln!("{}, {}", error.cause(), error.backtrace()),
    }
}
