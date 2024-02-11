#![feature(io_error_more)]

mod args;
mod packers;
mod converters;
mod utils;
mod json_files;
mod signing;

use std::path::Path;
use clap::Parser;
use crate::args::Args;
use crate::packers::mod_packer::ArmaToolsModPacker;
use crate::packers::Packer;
use crate::json_files::path_json::model::path_json::PathJson;
use crate::json_files::path_json::path_json_handler::PathJsonHandler;

fn main() {
    let args = Args::parse();

    let paths: PathJson;

    if let Some(path_string) = args.path_json
    {
        let path = Path::new(path_string.as_str());

        if !path.exists()
        {
            eprintln!("No path.json found at: {path_string}");
            return;
        }

        let handler = PathJsonHandler::default();

        paths = handler.read_json(path).expect(format!("Cannot read path.json at: {path_string}").as_str());
    }
    else {
        let program_path = std::env::current_exe().expect("Cannot get path to current executable");

        let program_directory = program_path.parent().unwrap();

        let path = program_directory.join("path.json");

        let handler = PathJsonHandler::default();

        if path.exists()
        {
            paths = handler.read_json(path).expect(format!("Cannot read path.json in program directory: {program_directory:?}").as_str());
        }
        else
        {
            paths = PathJson::default();

            handler.write_json(&paths, path).expect(format!("Cannot write default path.json to program directory: {program_directory:?}").as_str());
        }
    }

    let mod_packer = ArmaToolsModPacker::create(paths);

    if args.unpack {
        mod_packer.unpack(args.source, args.destination).unwrap();
    }
    else if args.pack {
        mod_packer.pack(args.source, args.destination).unwrap();
    }
}