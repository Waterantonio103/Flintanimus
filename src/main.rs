#![allow(unused)]

use clap::Parser;
use std::error::Error;
use std::fs::{self, *};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::io::{self, BufRead};
use std::time::Instant;
use std::collections::HashMap;
use rayon::prelude::*;

use crate::{
    args::{Args, PossibleArgs, Context}, 
    help::help,
    filesearch::recurse_files,
    matching::{relaxed, strict},
    fileprocessing::find_lines_individual,
    errors::FileReadError,
};

mod errors;
mod help;
mod args;
mod filesearch;
mod fileprocessing;
mod matching;

fn main() -> Result<(), FileReadError> {
    //parse arguments and hand off to run function
    let args = Args::parse();
    run(&args)
}

fn run(args: &Args) -> Result<(), FileReadError> {
    //Purpose : extract path and pattern from command line, and run them through functions according to optional flags
    let paths = &args.directory;
    let target = &args.target;

    //get context count from command
    let context = if let Some(full) = args.context {
        Some(Context::Full(full))
    } else if let Some(right) = args.after_context {
        Some(Context::Right(right))
    } else if let Some(left) = args.before_context {
        Some(Context::Left(left))
    } else {
        None
    };

    //Grab extra flags
    let arguments = PossibleArgs {
        whole: args.whole,
        insensitive: args.insensitive,
        only: args.only,
        invert: args.invert,
        max_count: args.max_count,
        context: context,
    };

    //iterate through pathes (multiple pathes can be given at runtime -- each will be handled)
    for path in paths {
        let result = recurse_files(path, args.recursive)?;
        //think of handling empty file
        //Using parallel iteration to handoff files and folders to threads
        let results = result.par_iter()
            .map(|file| {
                find_lines_individual(
                    file, 
                    target.as_str(),
                    arguments,
                )
                    .map(|matches| (file, matches))
            })
            .collect::<Result<HashMap<&PathBuf, Vec<(usize, String)>>, FileReadError>>()?;
            // dbg!(&results);
        let to_read = results.clone();
    
        for (file, matches) in results {
            let count = matches.len();
            if !matches.is_empty() {
                if args.count && args.line_numbers {
                    println!("{file:?}");
                    for (line_number, line) in &matches {
                        println!("{line_number}:{line}");
                    }
                    println!("total matches: {count}");
                } else if args.count {
                    println!("{file:?}:{count}");
                } else if args.count && args.files_with_matches {
                    println!("{file:?}:{count}");
                } else if args.files_with_matches {
                    println!("{file:?}");
                } if args.files_without_matches {
                    //to pass it non-empty matches
                } if args.quiet {
                    //to pass it non-empty matches
                } else {
                    println!("{file:?}");
                    for (line_number, line) in &matches {
                        if args.line_numbers {
                            println!("{line_number}:{line}");
                        } else {
                            println!("{line}");
                        }
                    }
                }
            } else {
                if args.quiet {
                    //do nothing
                } else if args.files_without_matches {
                    println!("{file:?}");
                }
            }
        }
        //check for no matches
        let no_matches = to_read.values().all(|x| x.is_empty());
        if no_matches {
            println!("No matches found");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master() {
        let args = Args::default();
        run(&args).unwrap();
    }

    #[test]
    fn test_empty_dir() {
        let args = Args::empty_dir();
        run(&args).unwrap();
    }

    fn test_empty_file() {
        let args = Args::empty_file();
        run(&args).unwrap();
    }
}
