#![allow(unused)]

use clap::{Parser, builder::Str};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {

    // Target and Directory, REQUIRED
    ///Pattern to search for
    pub target: String,
    #[arg(required = true)]
    ///Directory(s) to search in
    pub directory: Vec<String>,
    //

    //Search conditions, OPTIONAL
    #[arg(long)]
    ///Stop searching for patterns after a certain count
    pub max_count: Option<usize>,
    #[arg(short, long, default_value_t = false)]
    ///Check for whole pattern instances only
    pub whole: bool,
    #[arg(short, long, default_value_t = false)]
    ///Check for insensitive cases
    pub insensitive: bool,
    #[arg(short, long, default_value_t = false)]
    ///Perform a recursive search on the chosen directory
    pub recursive: bool,
    //

    //Display options, OPTIONAL 
    #[arg(short, long, default_value_t = false)]
    ///Display line numbers
    pub line_numbers: bool,
    #[arg(short, long, default_value_t = false)]
    ///Display count
    pub count: bool,
    #[arg(long, visible_alias = "nf", default_value_t = false)]
    ///Display count
    pub no_filename: bool,
    #[arg(short, long, default_value_t = false)]
    ///Return only matched pattern at line
    pub only: bool, //This does not YET work with CONTEXT --> REMINDER, DO THAT
    #[arg(short = 'v', long, default_value_t = false, conflicts_with = "only")]
    ///Return lines not matched
    pub invert: bool,
    #[arg(short, long, default_value_t = false, conflicts_with_all = ["invert", "only", "line_numbers", "count", "no_filename"])]
    ///Print nothing if matched at least once
    pub quiet: bool,
    #[arg(long, visible_alias = "cl")]
    ///Highlight pattern with desired color
    pub color: Option<String>,
    #[arg(long, conflicts_with_all = ["before_context", "after_context", "only", "invert"])]
    ///Lines to display before and after matched line
    pub context: Option<usize>,
    #[arg(long, conflicts_with_all = ["context", "after_context",  "only", "invert"])]
    ///Lines to display before matched line
    pub before_context: Option<usize>,
    #[arg(long, conflicts_with_all = ["context", "before_context", "only", "invert"])]
    ///Lines to display after matched line
    pub after_context: Option<usize>,
    //
    
}

impl Default for Args {
    fn default() -> Self {
        Self { 
            target: "hello".to_string(), 
            directory: vec!["src".to_string()], 
            max_count: None, 
            whole: false, 
            insensitive: false, 
            recursive: false, 
            line_numbers: false, 
            count: false, 
            no_filename: false,
            only: false, 
            invert: false, 
            quiet: false, 
            color: None,
            context: None, 
            before_context: None, 
            after_context: None,
        }
    }
}

impl Args {
    pub fn empty_dir() -> Self {
        Self { 
            target: "".to_string(), 
            directory: vec!["".to_string()], 
            max_count: None, 
            whole: false, 
            insensitive: false, 
            recursive: false, 
            line_numbers: false, 
            count: false, 
            no_filename: false,
            only: false, 
            invert: false,  
            quiet: false, 
            color: None,
            context: None, 
            before_context: None, 
            after_context: None,
        }
    }
    pub fn empty_file() -> Self {
        Self { 
            target: "".to_string(), 
            directory: vec!["src/empty.txt".to_string()], 
            max_count: None, 
            whole: false, 
            insensitive: false, 
            recursive: false, 
            line_numbers: false, 
            count: false, 
            no_filename: false,
            only: false, 
            invert: false, 
            quiet: false, 
            color: None,
            context: None, 
            before_context: None, 
            after_context: None,
        }
    }
}

//Grouping arguments for better readability
#[derive(Debug, Clone, Copy)]
pub struct PossibleArgs {
    pub whole: bool,
    pub insensitive: bool, 
    pub only: bool, 
    pub invert: bool, 
    pub max_count: Option<usize>,
    pub context: Option<Context>,
}

impl Default for PossibleArgs {
    fn default() -> Self {
        Self { 
            whole: false, 
            insensitive: false, 
            only: false, 
            invert: false, 
            max_count: None, 
            context: None, 
        }
    }
}

impl PossibleArgs {
    pub fn all_only_args(count: usize) -> Self {
        Self { 
            whole: true, 
            insensitive: true, 
            only: true, 
            invert: false, 
            max_count: Some(count), 
            context: None, 
        }
    }
    pub fn all_invert_args(count: usize) -> Self {
        Self { 
            whole: true, 
            insensitive: true, 
            only: false, 
            invert: true, 
            max_count: Some(count), 
            context: None, 
        }
    }
}

//enum to describe which context was chosen (before, after, or full)
#[derive(Debug, Clone, Copy)]
pub enum Context {
    Full(usize),
    Right(usize),
    Left(usize),
}

// can use conflicts_with = "other_tag_name" for tags that cant work together
