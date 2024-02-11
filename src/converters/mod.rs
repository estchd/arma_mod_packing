use std::error::Error;
use std::path::Path;

pub mod config_converter;
pub mod rvmat_converter;
pub mod paa_converter;

pub trait FileConverter
{
    type BinarizeError: Error;
    type DebinarizeError: Error;

    fn binarize<SourcePath: AsRef<Path>, DestinationPath: AsRef<Path>>(&self, source: SourcePath, destination: DestinationPath) -> Result<(), Self::BinarizeError>;

    fn debinarize<SourcePath: AsRef<Path>, DestinationPath: AsRef<Path>>(&self, source: SourcePath, destination: DestinationPath) -> Result<(), Self::DebinarizeError>;
}