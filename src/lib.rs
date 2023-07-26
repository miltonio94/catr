use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool,
}

type MyResult<T> = Result<T, Box<dyn Error>>;

pub fn run(config: Config) -> MyResult<()> {
    for file in config.files {
        match open(&file) {
            Err(err) => eprintln!("Failed to open {}: {}", file, err),
            Ok(buffer) => {
                let line_number = if config.number_lines && !config.number_nonblank_lines {
                    Numbering::new_number()
                } else if config.number_nonblank_lines && !config.number_lines {
                    Numbering::new_number_nonblank()
                } else {
                    Numbering::new_no_number()
                };

                let file_content = process_data(buffer, line_number);

                for line in file_content {
                    println!("{}", line);
                }
            }
        }
    }
    Ok(())
}

enum Numbering {
    Number(u32),
    NumberNonblank(u32),
    NoNumber,
}

type Line = (String, Numbering);

impl Numbering {
    pub fn increment(self, line: String) -> Self {
        match self {
            Self::NoNumber => self,
            Self::Number(line_number) => Self::Number(line_number + 1),
            Self::NumberNonblank(line_number) => {
                if line.is_empty() {
                    self
                } else {
                    Self::NumberNonblank(line_number + 1)
                }
            }
        }
    }

    pub fn new_number() -> Self {
        Self::Number(1)
    }

    pub fn new_number_nonblank() -> Self {
        Self::NumberNonblank(1)
    }

    pub fn new_no_number() -> Self {
        Self::NoNumber
    }
}

fn process_data(file: Box<dyn BufRead>, numbering: Numbering) -> Vec<String> {
    let mut ret_lines = vec![];
    let mut count = numbering;
    for line in file.lines() {
        match line {
            Ok(line) => {
                let (l, c) = number_line(line, count);
                count = c;
                ret_lines.push(l);
            }
            Err(_) => (),
        }
    }
    ret_lines
}

fn number_line(line: String, numbering: Numbering) -> Line {
    match numbering {
        Numbering::NoNumber => (line, numbering),
        Numbering::Number(line_number) => (
            format!("{:>6}\t{}", line_number, line),
            numbering.increment(line),
        ),
        Numbering::NumberNonblank(line_number) => {
            if line.is_empty() {
                (line, numbering)
            } else {
                (
                    format!("{:>6}\t{}", line_number, line),
                    numbering.increment(line),
                )
            }
        }
    }
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("catr")
        .version("0.1.0")
        .author("Me!")
        .about("a rusty cat")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("number")
                .short("n")
                .long("number")
                .help("Number lines")
                .takes_value(false)
                .conflicts_with("number_nonblank"),
        )
        .arg(
            Arg::with_name("number_nonblank")
                .short("b")
                .long("number-nonblank")
                .help("Number non-blank lines")
                .takes_value(false),
        )
        .get_matches();

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        number_lines: matches.is_present("number"),
        number_nonblank_lines: matches.is_present("number_nonblank"),
    })
}
