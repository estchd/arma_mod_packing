use std::collections::HashSet;
use std::fs;
use std::io::{Error};
use std::path::{Path, PathBuf};
use ignore::{WalkBuilder};
use remove_empty_subdirs::remove_empty_subdirs;
use crate::converters::config_converter::ArmaToolsConfigConverter;
use crate::converters::FileConverter;
use crate::converters::paa_converter::ArmaToolsPAAConverter;
use crate::json_files::key_json::key_json_handler::KeyJsonHandler;
use crate::json_files::path_json::model::path_json::PathJson;
use crate::packers::Packer;
use crate::packers::pbo_packer::{ArmaToolsPBOPacker};
use crate::signing::pbo_signer::{ArmaToolsPBOSigner, PBOSigner};
use crate::utils::check_source_and_destination;

pub trait ModPacker : Packer {}

pub struct ArmaToolsModPacker {
    pbo_packer: ArmaToolsPBOPacker<String>,
    config_converter: ArmaToolsConfigConverter<String>,
    paa_converter: ArmaToolsPAAConverter<String>,
    rvmat_converter: ArmaToolsConfigConverter<String>,
    pbo_signer: ArmaToolsPBOSigner<String>
}

impl ModPacker for ArmaToolsModPacker {}

impl Packer for ArmaToolsModPacker
{
    type PackError = Error;
    type UnpackError = Error;

    fn pack<SourcePath: AsRef<Path>, DestPath: AsRef<Path>>(&self, source_folder: SourcePath, destination_folder: DestPath) -> Result<(), Self::PackError>
    {
        // Input:
        // @Mod
        //  |- mod.cpp
        //  |- logo.png
        //  |- IMPORTANT_SECRET.md
        //  |- README.md
        //  |- .modignore       // contains misc_development and IMPORTANT_SECRET.md
        //  |- .convertignore
        //  |...
        //  |- [misc_development]
        //      |- README.md
        //      |- [Documentation]
        //          |- index.html
        //          |...
        //      |...
        //  |- [mod_guide]
        //      |- intro.md
        //      |- tutorial.html
        //  |- [keys]
        //      |- first_authority.bikey
        //      |...
        //      |- last_authority.bikey
        //      |- first_authority.biprivatekey
        //      |...
        //      |- last_authority.biprivatekey
        //  |- [addons]
        //      |- first
        //          |- config.cpp
        //          |- pbo.json
        //          |- [key.json]           // Without key.json, resulting .pbo is not signed
        //          |- [Data]
        //              |- tex.png
        //              |- mat.rvmat
        //              |...
        //      |...
        //      |- last
        //          |...

        // - Copy all files not excluded by .modignore into output folder
        // - Convert all files not excluded by .convertignore and delete originals (in output folder)
        // - Copy all key.json files as [mod_name]_key.json into addons folder
        // - Build all folders containing pbo.json into .pbo files in addons folder and delete originals (in output folder)
        // - Sign each pbo with a corresponding key.json
        // - Copy each used .bikey into keys folder
        // - Delete key.json files
        // - Prune empty folders

        // Output:
        // @Mod
        //  |- mod.cpp
        //  |- meta.cpp
        //  |- logo.paa
        //  |- README.md
        //  |- [mod_guide]
        //      |- intro.md
        //      |- tutorial.html
        //  |...
        //  |- keys
        //      |- first_authority.bikey
        //      |...
        //      |- last_authority.bikey
        //  |- addons
        //      |- first.pbo
        //          |- config.bin
        //          |- [Data]
        //              |- tex.paa
        //              |- mat.rvmat
        //              |...
        //      |...
        //      |- last.pbo
        //          |...
        //      |- first.bisign
        //      |...
        //      |- last.bisign

        if destination_folder.as_ref().exists()
        {
            fs::remove_dir_all(&destination_folder).unwrap();
        }

        check_source_and_destination(&source_folder, &destination_folder, true, true).unwrap();

        let addons_folder_path = destination_folder.as_ref().join("addons");
        let keys_folder_path = destination_folder.as_ref().join("keys");

        Self::copy_raw_files_for_packing(&source_folder, &destination_folder).unwrap();

        Self::delete_mod_ignore_files(&destination_folder).unwrap();

        self.convert_raw_files(&destination_folder).unwrap();

        Self::delete_convert_ignore_files(&destination_folder).unwrap();

        let pbo_paths: HashSet<PathBuf> = Self::identify_pbo_folders(&destination_folder).unwrap();

        Self::copy_key_files_from_pbo_folders(&addons_folder_path, &pbo_paths).unwrap();

        Self::delete_non_addon_key_files(&destination_folder, &addons_folder_path).unwrap();

        self.pack_pbo_folders(&addons_folder_path, &pbo_paths).unwrap();

        Self::delete_original_pbo_folders(&pbo_paths).unwrap();

        let used_bikeys: HashSet<String> = self.sign_packed_pbos(&addons_folder_path, &keys_folder_path).unwrap();

        Self::delete_private_keys(&keys_folder_path).unwrap();

        Self::delete_unused_bikeys(&keys_folder_path, &used_bikeys).unwrap();

        Self::delete_addon_key_json_files(&addons_folder_path).unwrap();

        Self::prune_empty_directories(&destination_folder).unwrap();

        Ok(())
    }

    fn unpack<SourcePath: AsRef<Path>, DestPath: AsRef<Path>>(&self, source_folder: SourcePath, destination_folder: DestPath) -> Result<(), Self::UnpackError>
    {
        // Input:
        // @Mod
        //  |- mod.cpp
        //  |- meta.cpp
        //  |- logo.paa
        //  |...
        //  |- [keys]
        //      |- first_authority.bikey
        //      |...
        //      |- last_authority.bikey
        //  |- [Addons]
        //      |- first.pbo
        //          |- config.bin
        //          |- [Data]
        //              |- tex.paa
        //              |- mat.rvmat
        //              |...
        //      |...
        //      |- last.pbo
        //          |...
        //      |- first.bisign
        //      |...
        //      |- last.bisign

        // Output:
        // @Mod
        //  |- mod.cpp
        //  |- logo.png
        //  |...
        //  |- addons
        //      |- first
        //          |- config.cpp
        //          |- pbo.json
        //          |- [Data]
        //              |- tex.png
        //              |- mat.rvmat
        //              |...
        //      |...
        //      |- last
        //          |...


        if destination_folder.as_ref().exists()
        {
            fs::remove_dir_all(&destination_folder).unwrap();
        }

        check_source_and_destination(source_folder.as_ref(), destination_folder.as_ref(), true, true).unwrap();

        let addons_folder_path = destination_folder.as_ref().join("addons");
        let keys_folder_path = destination_folder.as_ref().join("keys");

        fs::create_dir_all(&addons_folder_path).unwrap();
        fs::create_dir_all(&keys_folder_path).unwrap();

        Self::copy_files_in_folder(source_folder.as_ref(), destination_folder.as_ref(), true).unwrap();
        self.unpack_pbos_in_folder(source_folder.as_ref(), destination_folder.as_ref(), true).unwrap();

        self.convert_files(destination_folder.as_ref(), true).unwrap();

        Ok(())
    }
}

impl ArmaToolsModPacker
{
    pub fn create(paths: PathJson) -> Self
    {
        let paa_converter = ArmaToolsPAAConverter
        {
            tool_path: paths.paa_converter_path,
        };

        let config_converter = ArmaToolsConfigConverter
        {
            tool_path: paths.config_converter_path,
        };

        let rvmat_converter = ArmaToolsConfigConverter
        {
            tool_path: paths.rvmat_converter_path
        };

        let pbo_packer = ArmaToolsPBOPacker
        {
            tool_path: paths.pbo_packer_path,
            prefix: None,
        };

        let pbo_signer = ArmaToolsPBOSigner
        {
            tool_path: paths.pbo_signer_path
        };

        Self {
            pbo_packer,
            config_converter,
            paa_converter,
            rvmat_converter,
            pbo_signer
        }
    }

    fn copy_raw_files_for_packing<A: AsRef<Path>, B: AsRef<Path>>(source_folder: A, destination_folder: B) -> Result<(), Error>
    {
        let walk = WalkBuilder::new(&source_folder)
            .standard_filters(false)
            .add_custom_ignore_filename(".modignore")
            .build();

        for entry in walk
        {
            let entry = entry.unwrap();

            if !entry.path().is_file()
            {
                continue;
            }

            if entry.file_name().to_str().unwrap() == ".modignore"
            {
                continue;
            }

            Self::copy_file(entry.path(), &source_folder, &destination_folder).unwrap();
        }

        Ok(())
    }

    fn copy_file<A: AsRef<Path>, B: AsRef<Path>, C: AsRef<Path>>(file_path: A, source_root_path: B, destination_root_path: C) -> Result<(), Error>
    {
        if !file_path.as_ref().starts_with(&source_root_path)
        {
            let file_path = file_path.as_ref();
            let source_root_path = source_root_path.as_ref();

            panic!("File path: {0:?} is not inside Source Root Path: {1:?}", file_path, source_root_path);
        }

        let relative_file_path = file_path.as_ref().strip_prefix(&source_root_path).unwrap();

        let destination_path = destination_root_path.as_ref().join(relative_file_path);

        let destination_parent_path = destination_path.parent().unwrap();

        fs::create_dir_all(destination_parent_path).unwrap();

        fs::copy(file_path, destination_path).unwrap();

        Ok(())
    }

    fn delete_mod_ignore_files<A: AsRef<Path>>(mod_folder: A) -> Result<(), Error>
    {
        let mut ignore_files = HashSet::<PathBuf>::new();

        let walk = WalkBuilder::new(&mod_folder)
            .standard_filters(false)
            .filter_entry(|entry| {
                if entry.path().is_dir()
                {
                    return true;
                }

                let file_name = entry.file_name().to_str();

                return if let Some(file_name) = file_name
                {
                    file_name == ".modignore"
                }
                else {
                    false
                };
            })
            .build();

        for entry in walk
        {
            let entry = entry.unwrap();

            if entry.path().is_dir()
            {
                continue;
            }

            ignore_files.insert(PathBuf::from(entry.path()));
        }

        for ignore_file in ignore_files
        {
            fs::remove_file(&ignore_file).unwrap();
        }

        Ok(())
    }

    fn convert_raw_files<A: AsRef<Path>>(&self, mod_folder: A) -> Result<(), Error>
    {
        let mut files_to_convert: Vec<PathBuf> = vec![];

        let walk = WalkBuilder::new(&mod_folder)
            .standard_filters(false)
            .add_custom_ignore_filename(".convertignore")
            .build();

        for entry in walk
        {
            let entry = entry.unwrap();

            if !entry.path().is_file()
            {
                continue;
            }

            if entry.file_name().to_str().unwrap() == ".convertignore"
            {
                continue;
            }

            files_to_convert.push(PathBuf::from(entry.path()));
        }

        for entry_path in &files_to_convert
        {
            println!("Converting: {:?}", entry_path);

            let entry_extension = entry_path.extension().unwrap().to_str().unwrap();

            match entry_extension
            {
                "pac" |
                "tga" |
                "jpg" |
                "png" => {
                    let converted_path = entry_path.with_extension("paa");

                    self.paa_converter.binarize(entry_path, &converted_path).unwrap();

                    fs::remove_file(entry_path).unwrap();
                }
                "cpp" => {
                    let converted_path = entry_path.with_extension("bin");

                    self.config_converter.binarize(entry_path, &converted_path).unwrap();

                    fs::remove_file(entry_path).unwrap();
                }
                "rvmat" => {
                    // files with the .rvmat extension are being converted from .rvmat (plaintext) to .rvmat (binarized)
                    // so we have to first use a different extension .brvmat, and convert into that
                    // after that, we can delete the original file and rename the converted extension into .rvmat

                    let converted_path = entry_path.with_extension("brvmat");

                    self.rvmat_converter.binarize(entry_path, &converted_path).unwrap();

                    fs::remove_file(entry_path).unwrap();

                    fs::copy(&converted_path, entry_path).unwrap();

                    fs::remove_file(converted_path).unwrap();
                }
                _ => {
                    panic!("Cannot convert file {entry_path:?} with extension {entry_extension}, consider adding that file");
                }
            }
        }

        Ok(())
    }

    fn delete_convert_ignore_files<A: AsRef<Path>>(mod_folder: A) -> Result<(), Error>
    {
        let mut ignore_files = HashSet::<PathBuf>::new();

        let walk = WalkBuilder::new(&mod_folder)
            .standard_filters(false)
            .filter_entry(|entry| {
                if entry.path().is_dir()
                {
                    return true;
                }

                let file_name = entry.file_name().to_str();

                return if let Some(file_name) = file_name
                {
                    file_name == ".convertignore"
                }
                else {
                    false
                };
            })
            .build();

        for entry in walk
        {
            let entry = entry.unwrap();

            if entry.path().is_dir()
            {
                continue;
            }

            ignore_files.insert(PathBuf::from(entry.path()));
        }

        for ignore_file in ignore_files
        {
            fs::remove_file(&ignore_file).unwrap();
        }

        Ok(())
    }

    fn identify_pbo_folders<A: AsRef<Path>>(mod_folder: A) -> Result<HashSet<PathBuf>, Error>
    {
        let mut pbo_json_files = HashSet::<PathBuf>::new();

        let walk = WalkBuilder::new(&mod_folder)
            .standard_filters(false)
            .filter_entry(|entry| {
                if entry.path().is_dir()
                {
                    return true;
                }

                let file_name = entry.file_name().to_str();

                return if let Some(file_name) = file_name
                {
                    file_name == "pbo.json"
                }
                else {
                    false
                };
            })
            .build();

        let mut pbo_directories = HashSet::<PathBuf>::new();

        for entry in walk
        {
            let entry = entry.unwrap();

            if entry.path().is_dir()
            {
                continue;
            }

            let mut skip: bool = false;

            for pbo_directory in &pbo_directories
            {
                if entry.path().starts_with(pbo_directory)
                {
                    skip = true;
                    break;
                }
            }

            if skip
            {
                continue;
            }

            pbo_json_files.insert(PathBuf::from(entry.path()));
            pbo_directories.insert(PathBuf::from(entry.path().parent().unwrap()));
        }

        Ok(pbo_directories)
    }

    fn copy_key_files_from_pbo_folders<A: AsRef<Path>>(addons_folder: A, pbo_folders: &HashSet<PathBuf>) -> Result<(), Error>
    {
        for pbo_folder in pbo_folders
        {
            println!("Copying Key file from: {:?}", pbo_folder);

            for child_entry in pbo_folder.read_dir().unwrap()
            {
                let child_entry = child_entry.unwrap();

                let child_path = child_entry.path();

                if child_path.is_dir()
                {
                    continue;
                }

                let file_name = child_path.file_name().unwrap();

                let file_name = file_name.to_str();

                if file_name.is_none()
                {
                    continue;
                }

                let file_name = file_name.unwrap();

                if file_name != "key.json"
                {
                    continue;
                }

                let parent_folder_name = pbo_folder.file_name().unwrap().to_str().unwrap();

                let new_key_file_name = format!("{parent_folder_name}_key.json");

                fs::copy(child_entry.path(), &addons_folder.as_ref().with_file_name(new_key_file_name)).unwrap();
            }
        }

        Ok(())
    }

    fn delete_non_addon_key_files<A: AsRef<Path>, B: AsRef<Path>>(mod_folder: A, addons_folder: B) -> Result<(), Error>
    {
        let mut key_json_files = HashSet::<PathBuf>::new();

        let walk = WalkBuilder::new(&mod_folder)
            .standard_filters(false)
            .filter_entry(|entry| {
                if entry.path().is_dir()
                {
                    return true;
                }

                let file_name = entry.file_name().to_str();

                return if let Some(file_name) = file_name
                {
                    file_name == "key.json"
                }
                else {
                    false
                };
            })
            .build();

        for entry in walk
        {
            let entry = entry.unwrap();

            if entry.path().is_dir()
            {
                continue;
            }

            if entry.path().parent().is_some() &&
                entry.path().parent().unwrap() == addons_folder.as_ref()
            {
                continue;
            }

            key_json_files.insert(PathBuf::from(entry.path()));
        }

        for key_json_file in key_json_files
        {
            fs::remove_file(&key_json_file).unwrap();
        }

        Ok(())
    }

    fn pack_pbo_folders<A: AsRef<Path>>(&self, addons_folder: A, pbo_folders: &HashSet<PathBuf>) -> Result<(), Error>
    {
        for pbo_folder in pbo_folders
        {
            let pbo_folder_name = pbo_folder.file_name().unwrap().to_str().unwrap();

            let pbo_file_name = format!("{pbo_folder_name}.pbo");

            let pbo_file_path = addons_folder.as_ref().join(pbo_file_name);

            self.pbo_packer.pack(pbo_folder, &pbo_file_path).unwrap()
        }

        Ok(())
    }

    fn delete_original_pbo_folders(pbo_folders: &HashSet<PathBuf>) -> Result<(), Error>
    {
        for pbo_folder in pbo_folders
        {
            fs::remove_dir_all(pbo_folder).unwrap();
        }

        Ok(())
    }

    fn sign_packed_pbos<A: AsRef<Path>, B: AsRef<Path>>(&self, addons_folder: A, keys_folder: B) -> Result<HashSet<String>, Error>
    {
        let mut pbo_files = HashSet::<PathBuf>::new();

        for entry in addons_folder.as_ref().read_dir().unwrap()
        {
            let entry = entry.unwrap();

            if entry.path().is_dir()
            {
                continue;
            }

            if entry.path().extension().unwrap().to_str().unwrap() != "pbo"
            {
                continue;
            }

            pbo_files.insert(PathBuf::from(entry.path()));
        }

        let mut pbo_authority_name_pairs = HashSet::<(PathBuf, String)>::new();

        let key_json_handler = KeyJsonHandler::default();

        for pbo_file in pbo_files
        {
            let pbo_file_name = pbo_file.file_stem().unwrap().to_str().unwrap();

            let key_file_name = format!("{pbo_file_name}_key.json");

            let key_file_path = pbo_file.with_file_name(key_file_name);

            if !key_file_path.exists()
            {
                continue;
            }

            let key_json = key_json_handler.read_json(key_file_path).unwrap();

            pbo_authority_name_pairs.insert((pbo_file, key_json.authority_name));
        }

        let mut needed_public_keys = HashSet::<String>::new();

        for (pbo, authority) in pbo_authority_name_pairs
        {
            let private_key_name = format!("{authority}.biprivatekey");
            let public_key_name = format!("{authority}.bikey");

            let private_key_path = keys_folder.as_ref().join(&private_key_name);

            self.pbo_signer.sign(pbo, private_key_path, &addons_folder).unwrap();

            if !needed_public_keys.contains(&public_key_name)
            {
                needed_public_keys.insert(public_key_name);
            }
        }

        Ok(needed_public_keys)
    }

    fn delete_private_keys<A: AsRef<Path>>(keys_folder: A) -> Result<(), Error>
    {
        let mut private_keys = HashSet::<PathBuf>::new();

        let walk = WalkBuilder::new(&keys_folder)
            .standard_filters(false)
            .filter_entry(|entry| {
                if entry.path().is_dir()
                {
                    return true;
                }

                let file_extension = entry.path().extension();

                if file_extension.is_none()
                {
                    return false;
                }

                let file_extension = file_extension.unwrap().to_str();

                return if let Some(file_extension) = file_extension
                {
                    file_extension == "biprivatekey"
                }
                else {
                    false
                };
            })
            .build();

        for entry in walk
        {
            let entry = entry.unwrap();

            if entry.path().is_dir()
            {
                continue;
            }

            private_keys.insert(PathBuf::from(entry.path()));
        }

        for private_key in private_keys
        {
            fs::remove_file(&private_key).unwrap();
        }

        Ok(())
    }

    fn delete_unused_bikeys<A: AsRef<Path>>(keys_folder: A, used_keys: &HashSet<String>) -> Result<(), Error>
    {
        let mut ignore_files = HashSet::<PathBuf>::new();

        let walk = WalkBuilder::new(&keys_folder)
            .standard_filters(false)
            .filter_entry(|entry| {
                if entry.path().is_dir()
                {
                    return true;
                }

                let file_extension = entry.path().extension();

                if file_extension.is_none()
                {
                    return false;
                }

                let file_extension = file_extension.unwrap().to_str();

                return if let Some(file_extension) = file_extension
                {
                    file_extension == "bikey"
                }
                else {
                    false
                };
            })
            .build();

        for entry in walk
        {
            let entry = entry.unwrap();

            if entry.path().is_dir()
            {
                continue;
            }

            let entry_file_name = String::from(entry.file_name().to_str().unwrap());

            if used_keys.contains(&entry_file_name)
            {
                continue;
            }

            ignore_files.insert(PathBuf::from(entry.path()));
        }

        for ignore_file in ignore_files
        {
            fs::remove_file(&ignore_file).unwrap();
        }

        Ok(())
    }

    fn delete_addon_key_json_files<A: AsRef<Path>>(addons_folder: A) -> Result<(), Error>
    {
        let mut key_json_files = HashSet::<PathBuf>::new();

        let walk = WalkBuilder::new(&addons_folder)
            .standard_filters(false)
            .filter_entry(|entry| {
                if entry.path().is_dir()
                {
                    return true;
                }

                let file_name = entry.file_name().to_str();

                return if let Some(file_name) = file_name
                {
                    file_name == "key.json"
                }
                else {
                    false
                };
            })
            .build();

        for entry in walk
        {
            let entry = entry.unwrap();

            if entry.path().is_dir()
            {
                continue;
            }

            key_json_files.insert(PathBuf::from(entry.path()));
        }

        for key_json_file in key_json_files
        {
            fs::remove_file(&key_json_file).unwrap();
        }

        Ok(())
    }

    fn prune_empty_directories<A: AsRef<Path>>(mod_folder: A) -> Result<(), Error>
    {
        remove_empty_subdirs(mod_folder.as_ref()).unwrap();

        Ok(())
    }

    fn copy_files_in_folder<A: AsRef<Path>, B: AsRef<Path>>(source_folder: A, destination_folder: B, recurse: bool) -> Result<(), Error> {
        let source_folder_path: &Path = source_folder.as_ref();
        let destination_folder_path: &Path = destination_folder.as_ref();

        let source_directory = source_folder_path.read_dir()?;

        for item in source_directory {
            let item = item?;

            let metadata = item.metadata()?;

            let next_destination_folder_path = destination_folder_path.join(item.file_name());

            if metadata.is_dir() {
                if recurse {
                    let next_source_folder_path = source_folder_path.join(item.file_name());

                    Self::copy_files_in_folder(next_source_folder_path, next_destination_folder_path, true)?;
                }
            }
            else {
                let item_path = item.path();
                let extension = item_path.extension();

                if let Some(extension) = extension {
                    if extension == "pbo" || extension == "bisign" {
                        continue;
                    }

                    fs::create_dir_all(destination_folder_path)?;

                    fs::copy(item.path(), next_destination_folder_path)?;
                }
            }
        }

        Ok(())
    }

    fn unpack_pbos_in_folder<A: AsRef<Path>, B: AsRef<Path>>(&self, source_folder: A, destination_folder: B, recurse: bool) -> Result<(), Error> {
        let source_folder_path: &Path = source_folder.as_ref();
        let destination_folder_path: &Path = destination_folder.as_ref();

        let source_directory = source_folder_path.read_dir().unwrap();

        for item in source_directory {
            let item = item.unwrap();

            let metadata = item.metadata()?;

            let next_destination_folder_path = destination_folder_path.join(item.file_name());

            if metadata.is_dir() {
                if recurse {
                    let next_source_folder_path = source_folder_path.join(item.file_name());

                    self.unpack_pbos_in_folder(next_source_folder_path, next_destination_folder_path, true).unwrap();
                }
            } else {
                let item_path = item.path();
                let extension = item_path.extension();

                if let Some(extension) = extension {
                    if extension == "pbo" {
                        self.pbo_packer.unpack(item.path(), next_destination_folder_path).unwrap();
                    }
                }
            }
        }

        Ok(())
    }

    fn convert_files<P: AsRef<Path>>(&self, directory: P, recurse: bool) -> Result<(), Error> {
        let directory_path: &Path = directory.as_ref();

        let directory_items = directory_path.read_dir()?;

        directory_items.for_each(|item| {
            let item = item.unwrap();

            let metadata = item.metadata().unwrap();

            if metadata.is_dir() {
                if recurse {
                    self.convert_files(item.path(), true).unwrap();
                }
            }
            else {
                let item_path = item.path();
                let extension = item_path.extension();
                let file_name = item_path.file_stem().unwrap();

                println!("Converting: {:?}", &item_path);

                if let Some(extension) = extension {
                    if extension == "paa" {
                        let output_file_path = item_path.with_extension("png");

                        let result = self.paa_converter.debinarize(&item_path, output_file_path);

                        if let Err(e) = result {
                            eprintln!("Error converting paa: {e}");
                        }
                        else {
                            let remove_result = fs::remove_file(item.path());

                            if let Err(e) = remove_result {
                                eprintln!("Error removing converted paa: {e}");
                            }
                        }
                    }
                    if extension == "bin" && file_name.to_ascii_lowercase() != "texheaders"{
                        let output_file_path = item.path().with_extension("cpp");

                        let result = self.config_converter.debinarize(item.path(), &output_file_path);

                        if let Err(e) = result {
                            eprintln!("Error converting bin: {e}");
                        }
                        else {
                            let remove_result = fs::remove_file(item.path());

                            if let Err(e) = remove_result {
                                eprintln!("Error removing converted bin: {e}");
                            }
                        }
                    }
                    if extension == "rvmat" {
                        let output_file_path = item.path().with_extension("dbrvmat");

                        let result = self.rvmat_converter.debinarize(item.path(), &output_file_path);

                        if let Err(e) = result {
                            eprintln!("Error converting rvmat: {e}");
                        }
                        else {
                            let remove_result = fs::remove_file(item.path());

                            if let Err(e) = remove_result {
                                eprintln!("Error removing converted rvmat: {e}");
                            }
                        }

                        fs::copy(&output_file_path, item.path()).unwrap();

                        fs::remove_file(&output_file_path).unwrap();
                    }
                }
            }
        });

        Ok(())
    }
}