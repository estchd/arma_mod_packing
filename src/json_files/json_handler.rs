use std::io;
use std::marker::PhantomData;
use std::path::Path;
use serde::{Serialize};
use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JsonWriteError
{
    #[error("IO Error")]
    IO(#[from] io::Error),

    #[error("Serialization Error")]
    Serialization(#[from] serde_json::Error)
}

#[derive(Error, Debug)]
pub enum JsonReadError
{
    #[error("IO Error")]
    IO(#[from] io::Error),

    #[error("Deserialization Error")]
    Deserialization(#[from] serde_json::Error)
}

#[derive(Debug, Clone)]
pub struct JsonHandler<T: Serialize + DeserializeOwned> {
    _marker: PhantomData<T>
}

impl<T: Serialize + DeserializeOwned> Default for JsonHandler<T>
{
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: Serialize + DeserializeOwned> JsonHandler<T>
{
    pub fn write_json<DestinationPath: AsRef<Path>>(&self, value: &T, destination: DestinationPath) -> Result<(), JsonWriteError>
    {
        let file = std::fs::File::create(destination)?;

        serde_json::to_writer_pretty(file, value)?;

        Ok(())
    }

    pub fn read_json<SourcePath: AsRef<Path>>(&self, source: SourcePath) -> Result<T, JsonReadError>
    {
        let file = std::fs::File::open(source)?;

        let value: T = serde_json::from_reader(file)?;

        Ok(value)
    }
}