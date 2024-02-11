use std::io::{Error, ErrorKind};
use std::path::Path;
use std::process::Command;
use crate::converters::FileConverter;
use crate::utils::check_source_and_destination;

pub trait PAAConverter : FileConverter {}

pub struct ArmaToolsPAAConverter<P: AsRef<Path>> {
    pub tool_path: P
}

type ArmaToolsPAAConverterError = Error;

impl<P: AsRef<Path>> PAAConverter for ArmaToolsPAAConverter<P> {}

impl<P: AsRef<Path>> FileConverter for ArmaToolsPAAConverter<P>
{
    type BinarizeError = ArmaToolsPAAConverterError;
    type DebinarizeError = ArmaToolsPAAConverterError;

    fn binarize<SourcePath: AsRef<Path>, DestinationPath: AsRef<Path>>(&self, source: SourcePath, destination: DestinationPath) -> Result<(), Self::BinarizeError> {
        self.convert(source, destination)
    }

    fn debinarize<SourcePath: AsRef<Path>, DestinationPath: AsRef<Path>>(&self, source: SourcePath, destination: DestinationPath) -> Result<(), Self::DebinarizeError> {
        self.convert(source, destination)
    }
}

impl<P: AsRef<Path>> ArmaToolsPAAConverter<P>
{
    fn convert<SourcePath: AsRef<Path>, DestinationPath: AsRef<Path>>(&self, source: SourcePath, destination: DestinationPath) -> Result<(), ArmaToolsPAAConverterError> {
        check_source_and_destination(source.as_ref(), destination.as_ref(), false, false)?;

        let paa_path = source.as_ref().canonicalize().unwrap();
        let png_path = destination.as_ref().canonicalize().unwrap();

        let output = Command::new(&self.tool_path.as_ref())
            .arg(paa_path.as_os_str())
            .arg(png_path.as_os_str())
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

            let output = format!("PAA Conversion Process failed, error code: {code}");

            return Err(Error::new(ErrorKind::Other, output));
        }

        Ok(())
    }
}