use std::error::Error;
use std::fs::{self, *};
use std::io::ErrorKind;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::io::{self, BufRead};
use std::collections::HashMap;
use std::usize;

use crate::args::{PossibleArgs ,Context};
use crate::matching::{strict::{find_match_strict, not_matched_strict}, relaxed::{find_match, not_matched}};
use crate::errors::{FileReadError};

pub fn find_lines_individual(
        file: &PathBuf,
        target: &str,
        args: PossibleArgs,
    ) -> Result<Vec<(usize, String)>, FileReadError> {

    //Purpose : find target lines in a file based on arguments passed

    let mut context_bool: bool = false;
    let mut lower: usize = 0;
    let mut upper: usize = 0;

    //getting context safely, handling no context if necessary
    if let Some(context) = args.context {
        match context {
            Context::Full(x) => {
                context_bool = true;
                lower = x;
                upper = lower;
            },
            Context::Right(y) => {
                context_bool = true;
                lower = 0;
                upper = y;
            },
            Context::Left(z) => {
                context_bool = true;
                lower = z;
                upper = 0;
            },
        }
    };

    let mut matches = Vec::new();

    let mut all_lines: Vec<(usize, String)> = Vec::new();
    let mut buf = Vec::new();

    for (line_number, line_result) in read_lines(file)? {
        let line = match line_result {
            Ok(valid_line) => {valid_line},
            Err(e) if matches!(e.kind(), ErrorKind::InvalidData) => {return Ok(buf)},
            Err(e) => {return Err(FileReadError::ReadError(e));}
        };

        //pushing current line to vector, will run for every line, hence all_lines
        all_lines.push((line_number, line.clone()));

        //stop when the matches length = desired max count, this does however apply to invert, meaning a max count of X will behave differently when 
        //invert is set to true or to false
        if args.max_count.is_some() && matches.len() == args.max_count.unwrap() {
            break;
        }

        //handling cases by handing to specialized functions
        if args.whole {
            if args.invert {
                if let Some(found) = not_matched_strict(line, line_number, target, args.insensitive) {
                    matches.push(found);
                }
            } else {
                if let Some(found) = find_match_strict(line, line_number, target, args.insensitive, args.only) {
                    matches.push(found);
                }
            }
        } else {
            if args.invert {
                if let Some(found) = not_matched(line, line_number, target, args.insensitive) {
                    matches.push(found);
                }
            } else {
                if let Some(found) = find_match(line, line_number, target, args.insensitive, args.only) {
                    matches.push(found);
                }
            }
        }
    }

    if context_bool {
        let ranges = bingus(&matches, &all_lines, lower, upper)?;
        //clear vec to send back correct lines
        matches.clear();

        for range in ranges {
            let get = all_lines.get(range);
            if let Some(slice_with_context) = get {
                matches.extend(slice_with_context.iter().cloned());
            };
        }

        return Ok(matches);
    } else {
        return Ok(matches);
    }

}

fn read_lines<P>(filename: P) -> io::Result<impl Iterator<Item = (usize, io::Result<String>)>>
where P: AsRef<Path>, {
    //Purpose : read all lines from a file and return an iterator over them (String)
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines().enumerate().map(|(mut line_number, line)| {(line_number + 1, line)}))
}

fn retrieve(matches: &Vec<(usize, String)>, all_lines: &Vec<(usize, String)>, lower: usize, upper: usize) -> Result<Vec<std::ops::Range<usize>>, FileReadError> {
    // Purpose : return ranges of desired context (lower and upper, e.g element at index 5 with context 2 -> (3..8))
    //this is with actual index!!!! -- delete index line if want line number instead
    if matches.is_empty() {
        return Ok(Vec::new());
    }

    const CLAMP_MIN: usize = usize::MIN;
    let upper_plus_one = upper.saturating_add(1);
    let Some(clamp_max) = all_lines.len().checked_sub(1).filter(|c| *c >= CLAMP_MIN + 1) else {
        return Err(FileReadError::ShortSlice);
    };
    let mut result = Vec::new();
    for (index, _) in matches {
        //use real_index if want actual index -- test first
        let real_index = index.saturating_sub(1);
        let start = real_index.saturating_sub(lower).clamp(CLAMP_MIN, clamp_max.saturating_add(1));
        let end = real_index.saturating_add(upper_plus_one).clamp(CLAMP_MIN, clamp_max.saturating_add(1));
        result.push(start..end)
    }

    Ok(result)
}

fn bingus(matches: &Vec<(usize, String)>, all_lines: &Vec<(usize, String)>, lower: usize, upper: usize) -> Result<Vec<std::ops::Range<usize>>, FileReadError> {
    //Purpose : Deal with overlapping ranges by merging them, e.g : (0..3), (1..6), (5..9), (12..15) -> (0..9), (12..15)

    //raw ranges
    let ranges = retrieve(matches, all_lines, lower, upper)?;

    let mut result = Vec::new();
    let mut iter = ranges.into_iter();

    //r is first element, .next takes that so the for loop inside starts at element after r : r2
    if let Some(mut r) = iter.next() {
        for r2 in iter {
            let overlap = r2.start <= r.end;
            if overlap {
                r.end = r.end.max(r2.end);
            } else {
                result.push(r);
                r = r2;
            }
        }
        result.push(r);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retrieve() {
        let result = retrieve(
            &vec![(1, "hello".to_string()), (2, "hellobabe".to_string()), (5, "hell".to_string())], 
            &vec![(1, "hello".to_string()), (2, "hellobabe".to_string()), (3, "bye".to_string()), (4, "".to_string()), (5, "hell".to_string())], 
            0, 
            3,
        ).unwrap();
        assert_eq!(result, vec![(0..4), (1..5), (4..5)]);
    }

    #[test]
    fn test_bingus() {
        let result = bingus(
            &vec![(1, "hello".to_string()), (2, "hellobabe".to_string()), (5, "hell".to_string())], 
            &vec![(1, "hello".to_string()), (2, "hellobabe".to_string()), (3, "bye".to_string()), (4, "".to_string()), (5, "hell".to_string())], 
            0, 
            3,
        ).unwrap();
        assert_eq!(result, vec![(0..5)]);
    }

    #[test]
    fn default_args() {
        let arguments = PossibleArgs::default();
        let result = find_lines_individual(&PathBuf::from("src/tests.txt"), "hello", arguments).unwrap();
        assert_eq!(result, vec![(1 ,"hello".to_string()), (4, "archhellobabes".to_string()), (6, "wtfhello".to_string()), (9, "hello".to_string())]);
    }

    #[test]
    fn all_only_args() {
        let arguments = PossibleArgs::all_only_args(5);
        let result = find_lines_individual(&PathBuf::from("src/tests.txt"), "hello", arguments).unwrap();
        assert_eq!(result, vec![(1, "hello".to_string()), (2, "Hello".to_string()), (3, "HeLlo".to_string()), (9, "hello".to_string())]);
    }

    #[test]
    fn all_invert_args() {
        let arguments = PossibleArgs::all_invert_args(10);
        let result = find_lines_individual(&PathBuf::from("src/tests.txt"), "hello", arguments).unwrap();
        assert_eq!(result, vec![(4, "archhellobabes".to_string()), (5, "babyHelLOlol".to_string()), (6, "wtfhello".to_string()), (7, "wtfHellO".to_string()), (8, "zooweemama".to_string()), (10, "abc".to_string()), (11, "fhiihf".to_string()), (12, "sjfpjw".to_string()), (13, "hwfheofih".to_string())]);
    }

}