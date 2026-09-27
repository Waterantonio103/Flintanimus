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
        
        //cloning works but is potentially expensive here, think about alternative
        let to_read = results.clone();
    
        for (file, matches) in results {
            let count = matches.len();
            let count_condition = args.count;

            let quiet = args.quiet;

            let line_numbers = args.line_numbers;

            let no_filename = args.no_filename;
            
            let all_args = count_condition || line_numbers;
            let no_matches = matches.is_empty();

            if !no_matches {
                if !quiet {
                    if !no_filename {
                        println!("{file:?}");
                    }
                    if count_condition {
                        if line_numbers {
                            for (line_number, line) in &matches {
                                println!("{line_number}:{line}");
                            }
                        }
                        println!("{count}");
                    }
                    if line_numbers {
                        for (line_number, line) in &matches {
                            println!("{line_number}:{line}");
                        }
                    }

                    if !all_args {
                        for (_, line) in &matches {
                            println!("{line}");
                        }
                    }
                }
            }
        }
        //check for no matches
        let no_matches_at_all = to_read.values().all(|x| x.is_empty());
        if no_matches_at_all {
            println!("0 matches found");
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
