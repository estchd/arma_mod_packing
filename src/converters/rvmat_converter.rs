use std::path::Path;
use crate::converters::config_converter::ArmaToolsConfigConverter;
use crate::converters::FileConverter;

pub trait RVMATConverter : FileConverter {}

impl<P: AsRef<Path>> RVMATConverter for ArmaToolsConfigConverter<P>
{
}