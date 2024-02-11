use std::error::Error;
use std::path::Path;

pub mod pbo_packer;
pub mod mod_packer;

pub trait Packer
{
    type PackError: Error;
    type UnpackError: Error;

    fn pack<SourcePath: AsRef<Path>, DestPath: AsRef<Path>>(&self, source_folder: SourcePath, destination_file: DestPath) -> Result<(), Self::PackError>;
    fn unpack<SourcePath: AsRef<Path>, DestPath: AsRef<Path>>(&self, source_file: SourcePath, destination_folder: DestPath) -> Result<(), Self::UnpackError>;
}