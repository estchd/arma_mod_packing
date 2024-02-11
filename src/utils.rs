use std::fs;
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::path::Path;

pub fn check_source_and_destination<A: AsRef<Path>, B: AsRef<Path>>(source_folder: A, destination_folder: B, source_is_folder: bool, destination_is_folder: bool) -> Result<(), Error> {
    check_path(source_folder, source_is_folder, !source_is_folder, true, false)?;
    check_path(destination_folder, destination_is_folder, !destination_is_folder, false, true)?;

    Ok(())
}

pub fn check_path<P: AsRef<Path>>(path: P, must_be_directory: bool, must_be_file: bool, must_exist: bool, create_if_doesnt_exist: bool) -> Result<(),Error> {
    let path: &Path = path.as_ref();

    if must_exist {
        check_existing_path(path, must_be_directory, must_be_file)?;
    }

    if create_if_doesnt_exist {
        if must_be_directory {
            return fs::create_dir_all(path);
        }

        let base_path = path.parent().ok_or(Error::new(ErrorKind::Other, "Cannot create parent directory"))?;

        fs::create_dir_all(base_path)?;

        if must_be_file {
            File::create(path)?;
        }
    }

    Ok(())
}

pub fn check_existing_path<P: AsRef<Path>>(path: P, must_be_directory: bool, must_be_file: bool) -> Result<(), Error> {
    let path: &Path = path.as_ref();

    if !path.exists() {
        return Err(Error::new(ErrorKind::NotFound, "Path doesn't exist"));
    }

    check_path_type(path, must_be_directory, must_be_file)
}

pub fn check_path_type<P: AsRef<Path>>(path: P, must_be_directory: bool, must_be_file: bool) -> Result<(), Error> {
    let path: &Path = path.as_ref();

    if must_be_directory && !path.is_dir() {
        return Err(Error::new(ErrorKind::NotADirectory, "Path is not a directory"));
    }
    if must_be_file && !path.is_file() {
        return Err(Error::new(ErrorKind::IsADirectory, "Path is not a file"));
    }

    Ok(())
}