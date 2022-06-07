mod args;
mod data;
mod err;
mod plot;
mod stats;
mod t_table;

use args::Opt;
use data::{load_data, Dataset};
use err::*;
use plot::{plot_graph, print_heading, CLASSIC_SYMBOLS, UNICODE_SYMBOLS};
use stats::*;

use clap::Parser;

use anyhow::Error;
use terminal_size::terminal_size;

fn get_width(opt: &Opt) -> u16 {
    opt.width
        .or_else(|| terminal_size().map(|ts| (ts.0).0))
        .unwrap_or(74)
}
fn validate_datasets(datasets: &[Dataset]) -> Result<(), Error> {
    for ds in datasets {
        if ds.data.len() < 3 {
            Err(MinistatFailure::InsufficientData {
                file: ds.path.to_string_lossy().into_owned(),
            })?;
        }
    }
    Ok(())
}

fn get_symbols(opt: &Opt) -> Vec<char> {
    opt.symbols
        .as_deref()
        .map(|custom_symbols| [' '].into_iter().chain(custom_symbols.chars()).collect())
        .unwrap_or_else(|| {
            if opt.modern_chars {
                UNICODE_SYMBOLS.to_vec()
            } else {
                CLASSIC_SYMBOLS.to_vec()
            }
        })
}

fn run(opt: &Opt) -> Result<(), Error> {
    let mut stdout = std::io::stdout().lock();
    let symbols = get_symbols(opt);
    if opt.files.len() > symbols.len() - 1 {
        return Err(MinistatFailure::TooManyDatasets {
            dataset_count: opt.files.len(),
        }
        .into());
    }
    let datasets = load_data(opt)?;
    validate_datasets(&datasets)?;

    print_heading(&mut stdout, &datasets, &symbols)?;
    let stats: Vec<_> = datasets
        .iter()
        .map(|dataset| Stats::from_dataset(&dataset.data))
        .collect();

    if !opt.raw_stats && !opt.stats_only {
        plot_graph(
            &mut stdout,
            get_width(opt),
            opt,
            &stats,
            &datasets.into_iter().map(|x| x.data).collect::<Vec<_>>(),
            &symbols,
        )?;
    }
    print_stats(
        &mut stdout,
        &stats,
        opt.confidence_level.0,
        opt.raw_stats,
        &symbols,
    )?;

    Ok(())
}

fn main() -> Result<(), Error> {
    let opt = Opt::parse();

    run(&opt)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{args::Opt, get_symbols};

    #[test]
    fn test_custom_symbols() {
        let opt_classic = Opt {
            modern_chars: false,
            ..Opt::default()
        };
        let opt_modern = Opt {
            modern_chars: true,
            ..Opt::default()
        };
        let opt_custom = Opt {
            symbols: Some("a2³$e6".into()),
            ..Opt::default()
        };
        assert_eq!(
            vec![' ', 'x', '+', '*', '%', '#', '@', 'O'],
            get_symbols(&opt_classic)
        );
        assert_eq!(
            vec![' ', '●', '○', '◾', '◽', '◆', '◇', '▲'],
            get_symbols(&opt_modern)
        );
        assert_eq!(
            vec![' ', 'a', '2', '³', '$', 'e', '6'],
            get_symbols(&opt_custom)
        );
    }
}
