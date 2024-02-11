use std::io::{Error, ErrorKind};
use std::path::Path;
use std::process::Command;
use crate::converters::FileConverter;

pub trait ConfigConverter : FileConverter {}

pub struct ArmaToolsConfigConverter<P: AsRef<Path>>
{
    pub tool_path: P
}

impl<P: AsRef<Path>> ConfigConverter for ArmaToolsConfigConverter<P> {}

impl<P: AsRef<Path>> FileConverter for ArmaToolsConfigConverter<P>
{
    type BinarizeError = Error;
    type DebinarizeError = Error;

    fn binarize<SourcePath: AsRef<Path>, DestinationPath: AsRef<Path>>(&self, source: SourcePath, destination: DestinationPath) -> Result<(), Self::BinarizeError> {
        let cpp_path = source.as_ref().canonicalize().unwrap();
        let bin_path = destination.as_ref();

        let cpp_path_ref = &cpp_path;
        let bin_path_ref = &bin_path;

        println!("Binarizing {cpp_path_ref:?} to {bin_path_ref:?}");

        let output = Command::new("CfgConvert.exe")
            .arg("-bin")
            .arg("-dst")
            .arg(bin_path.as_os_str())
            .arg(cpp_path.as_os_str())
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

            println!("Paa path: {:?}", cpp_path.as_os_str());

            let output = format!("Cpp Binarization Process failed, error code: {code}");

            return Err(Error::new(ErrorKind::Other, output));
        }

        Ok(())
    }

    fn debinarize<SourcePath: AsRef<Path>, DestinationPath: AsRef<Path>>(&self, source: SourcePath, destination: DestinationPath) -> Result<(), Self::DebinarizeError> {
        let bin_path = source.as_ref().canonicalize().unwrap();
        let cpp_path = destination.as_ref();

        let cpp_path_ref = &cpp_path;
        let bin_path_ref = &bin_path;

        println!("Debinarizing {bin_path_ref:?} to {cpp_path_ref:?}");

        let output = Command::new("CfgConvert.exe")
            .arg("-txt")
            .arg("-dst")
            .arg(cpp_path.as_os_str())
            .arg(bin_path.as_os_str())
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

            println!("Paa path: {:?}", cpp_path.as_os_str());

            let output = format!("Bin Debinarization Process failed, error code: {code}");

            return Err(Error::new(ErrorKind::Other, output));
        }

        Ok(())
    }
}