use std::fs::*;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub fn recurse_files(path: impl AsRef<Path> + std::fmt::Debug, recursive: bool) -> std::io::Result<Vec<PathBuf>> {
    //Purpose : Search designated path, with the option to search recursively until no more sub directories are left to search 
    let mut buf = vec![];
    let path = path.as_ref();

    //if path is a directory, search inside - else return the path as buf
    if path.is_dir() {
        let entries = match read_dir(path) {
            Ok(entries) => {entries},
            //Catch and ignore errors that would break program 
            Err(e) if matches!(
                e.kind(),
                ErrorKind::PermissionDenied |
                ErrorKind::NotFound
                ) => {return Ok(buf);}
            Err(e) => {
                eprintln!(
                    "FAILED: {} -> {}",
                    path.display(),
                    e
                );
    
                return Err(e)
            }
        };
    
        //iterate through every file/folder in directory
        for entry in entries {
            let entry = entry?;
            let meta = entry.metadata()?;
    
            //handle entry, if file, push to buffer, if dir, option to search again
            if meta.is_dir() && recursive {
                let mut subdir = recurse_files(entry.path(), recursive)?;
                buf.append(&mut subdir);
            }
    
            if meta.is_file() {
                buf.push(entry.path());
            }
        }
    
    } else if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }

    Ok(buf)
}