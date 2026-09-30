use clap::Parser;
// use std::error::Error;
// use std::fs::{self, *};
// use std::io::ErrorKind;
use std::path::{PathBuf};
// use std::io::{self, BufRead};
// use std::time::Instant;
use std::collections::HashMap;
use rayon::prelude::*;
use aho_corasick::{AhoCorasick, PatternID};
use colored::Colorize;

use crate::{
    args::{Args, PossibleArgs, Context}, 
    // help::help,
    filesearch::recurse_files,
    // matching::{relaxed, strict},
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

    let mut empty_checker = Vec::new();
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

        let count_condition = args.count;
        let quiet = args.quiet;
        let line_numbers = args.line_numbers;
        let no_filename = args.no_filename;
        let all_args = count_condition || line_numbers;

        for (file, mut matches) in results {
            let filename = file.display().to_string();
            let count = matches.len();
            let no_matches = matches.is_empty();

            if !no_matches {
                empty_checker.push(0);

                if let Some(chosen_color) = &args.color {
                    matches.clear();
                    matches = format_with_color(matches, target, chosen_color);
                    dbg!(&matches);
                }
                dbg!(&matches);

                if !quiet {
                    if !no_filename {
                        println!("{filename}");
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
    }
    
    //check for no matches
    let no_matches_at_all= empty_checker.is_empty();
    if no_matches_at_all  {
        println!("0 matches found");
    }

    Ok(())
}

fn format_with_color(lines: Vec<(usize, String)>, target: &str, color: &String) -> Vec<(usize, String)> {
    let mut colored_lines = Vec::new();
    let ac = AhoCorasick::new(&[target]).unwrap();
    // dbg!(&ac);
    for (index, (line_number, line)) in lines.into_iter().enumerate() {
        for mat in ac.find_iter(&line) {
            dbg!(&line);
            colored_lines.push((line_number, line.clone()));
            let new_line = &mut colored_lines[index].1;
            let pattern = &new_line[mat.start()..mat.end()];
            let colored_pattern = format!("{}", pattern.red());
            new_line.replace_range(mat.start()..mat.end(), &colored_pattern);
        }
    }
    dbg!(&colored_lines);
    colored_lines
}

#[cfg(test)]
mod tests {
    #![allow(unused)]
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
