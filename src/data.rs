use std::io::prelude::*;
use std::io::{BufReader, BufRead};
use std::fs::File;
use std::path::{PathBuf, Path};
use std::collections::HashSet;

use noisy_float::prelude::*;
use failure::Error;

use err::MinistatFailure;
use args::Opt;

pub struct Dataset {
    pub path: PathBuf,
    pub data: Vec<f64>,
}

impl Dataset {
    pub fn from_reader<R: Read, P: AsRef<Path>>(r: BufReader<R>,
                                                name: P,
                                                col: u8,
                                                split_chars: &HashSet<char>)
                                                -> Result<Self, Error> {
        let mut rv = Vec::new();
        for (i, line) in r.lines().enumerate() {
            let line = line?;
            let val = line.split(|x| split_chars.contains(&x))
                .skip((col - 1) as usize)
                .next();
            if let Some(val) = val {
                let parsed = val.parse::<f64>()
                    .map_err(|_| {
                        MinistatFailure::InvalidData {
                            file: name.as_ref().to_string_lossy().into_owned(),
                            line_no: i + 1,
                        }
                    })?;
                if parsed.is_finite() {
                    rv.push(r64(parsed));
                }

            }
        }
        rv.sort();

        Ok(Dataset {
            path: name.as_ref().into(),
            data: rv.into_iter().map(|x| x.raw()).collect(),
        })
    }
}

pub fn load_data(opt: &Opt) -> Result<Vec<Dataset>, Error> {
    use std::io::BufReader;
    use std::io;
    let mut datas: Vec<Dataset> = Vec::new();
    let split_chars: HashSet<char> = opt.delimiter.chars().collect();
    if opt.files.is_empty() {
        let reader = BufReader::new(io::stdin());
        let name = "stdin";
        datas.push(Dataset::from_reader(reader, name, opt.column.0, &split_chars)?);
    } else {
        for fname in opt.files.iter() {
            let f = File::open(fname)?;
            let reader = BufReader::new(f);
            datas.push(Dataset::from_reader(reader, fname, opt.column.0, &split_chars)?);
        }
    }
    Ok(datas)
}
