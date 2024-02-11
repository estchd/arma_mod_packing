use std::path::Path;
use std::io::{Error, ErrorKind};
use std::process::Command;
use crate::json_files::pbo_json::model::pbo_header::PBOHeader;
use crate::json_files::pbo_json::model::pbo_json::PBOJson;
use crate::json_files::pbo_json::pbo_json_handler::PBOJsonHandler;
use crate::packers::Packer;

pub trait PBOPacker : Packer {}

#[derive(Debug)]
pub struct ArmaToolsPBOPacker<P: AsRef<Path>>
{
    pub tool_path: P,
    pub prefix: Option<String>
}

impl<P: AsRef<Path>> PBOPacker for ArmaToolsPBOPacker<P> {}

impl<P: AsRef<Path>> Packer for ArmaToolsPBOPacker<P>
{
    type PackError = Error;
    type UnpackError = Error;

    fn pack<SourcePath: AsRef<Path>, DestPath: AsRef<Path>>(&self, source_folder: SourcePath, destination_file: DestPath) -> Result<(), Self::PackError> {
        let source_path = source_folder.as_ref().canonicalize().unwrap();
        let destination_path = destination_file.as_ref();

        //check_source_and_destination(&source_path, destination_path, true, false)?;

        self.set_pbo_json_prefix(&source_path).unwrap();

        let output = Command::new(self.tool_path.as_ref())
            .current_dir(destination_path.parent().unwrap())
            .arg("pack")
            .arg(source_path.as_os_str())
            .output()?;

        if !output.status.success() {
            let code = output.status.code().unwrap_or(0);

            let stdout = String::from_utf8(output.stdout);
            let stderr = String::from_utf8(output.stderr);

            if let Ok(stdout) = stdout {
                println!("{}", stdout);
            }

            if let Ok(stderr) = stderr {
                eprintln!("{}", stderr);
            }

            println!("Folder path: {:?}", source_path.as_os_str());

            let output = format!("PBO packing failed, error code: {code}");

            return Err(Error::new(ErrorKind::Other, output));
        }

        Ok(())
    }

    fn unpack<SourcePath: AsRef<Path>, DestPath: AsRef<Path>>(&self, source_folder: SourcePath, destination_folder: DestPath) -> Result<(), Self::UnpackError> {
        let source_path = source_folder.as_ref().canonicalize().unwrap();
        let destination_path = destination_folder.as_ref();

        println!("Source Path: {:?}", &source_path);
        println!("Destination Path: {:?}", &destination_path);

        let working_directory = destination_path.parent().unwrap();

        println!("Working Directory: {:?}", working_directory);

        //check_source_and_destination(&source_path, destination_path, false, true)?;

        let output = Command::new(self.tool_path.as_ref())
            .current_dir(working_directory)
            .arg("unpack")
            .arg(&source_path.as_os_str())
            .output().unwrap();

        if !output.status.success() {
            let code = output.status.code().unwrap_or(0);

            let stdout = String::from_utf8(output.stdout.clone());
            let stderr = String::from_utf8(output.stderr.clone());

            if let Ok(stdout) = stdout {
                println!("{}", stdout);
            }
            else {
                let lossy_stdout = String::from_utf8_lossy(&output.stdout);

                eprintln!("Unable to read stdout, cannot convert to utf-8");
                eprintln!("Lossy stdout: {lossy_stdout}")
            }

            if let Ok(stderr) = stderr {
                eprintln!("{}", stderr);
            }
            else {
                let lossy_stderr = String::from_utf8_lossy(&output.stderr);

                eprintln!("Unable to read stderr, cannot convert to utf-8");
                eprintln!("Lossy stderr: {lossy_stderr}")
            }

            println!("PBO path: {:?}", source_path.as_os_str());

            let output = format!("PBO unpacking failed, error code: {code}");

            return Err(Error::new(ErrorKind::Other, output));
        }

        self.set_pbo_json_prefix(destination_folder).unwrap();

        Ok(())
    }
}

impl<P: AsRef<Path>> ArmaToolsPBOPacker<P>
{
    fn set_pbo_json_prefix<DestPath: AsRef<Path>>(&self, destination_folder: DestPath) -> Result<(), Error>
    {
        if let Some(prefix) = &self.prefix
        {
            let pbo_json_path = destination_folder.as_ref().join("pbo.json");

            let handler = PBOJsonHandler::default();

            let mut pbo_json = if pbo_json_path.exists()
            {
                handler.read_json(&pbo_json_path).unwrap()
            }
            else {
                PBOJson{
                    headers: vec![],
                    compress: None,
                }
            };

            let mut prefix_header_set: bool = false;

            for i in 0..pbo_json.headers.len()
            {
                if pbo_json.headers[i].name == "prefix"
                {
                    pbo_json.headers[i].value = prefix.clone();
                    prefix_header_set = true;
                }
            }

            if !prefix_header_set
            {
                let prefix_header = PBOHeader {
                    name: "prefix".to_string(),
                    value: prefix.clone(),
                };

                pbo_json.headers.push(prefix_header);
            }

            handler.write_json(&pbo_json, &pbo_json_path).unwrap();
        }

        Ok(())
    }
}