use std::{
    error::Error,
    fs::File,
    io::{Read, Write},
    path::Path,
};

use flate2::{read::DeflateDecoder, write::DeflateEncoder, Compression};
use rmp_serde::{decode::ReadReader, Deserializer, Serializer};
use serde::{Deserialize, Serialize};

pub mod raw_streaming_data;
pub mod streaming_data;

pub trait Persist: Serialize + for<'a> Deserialize<'a> + Sized {
    fn save_to_file<P>(&self, key: P, use_compression: bool) -> Result<(), Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let mut handle = File::create(key)?;
        let mut bytes = Vec::with_capacity(128);
        let mut serializer: Serializer<&mut Vec<u8>> = Serializer::new(bytes.as_mut());
        self.serialize(&mut serializer)?;
        if use_compression {
            let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
            encoder.write_all(&bytes)?;
            handle.write_all(&encoder.finish()?)?;
            Ok(())
        } else {
            handle.write_all(&bytes)?;
            Ok(())
        }
    }

    fn load_from_file<P>(key: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let handle = File::open(key)?;
        let mut decoder = DeflateDecoder::new(handle);
        let mut bytes = Vec::new();
        decoder.read_to_end(&mut bytes)?;
        let mut deserializer: Deserializer<ReadReader<&[u8]>> = Deserializer::new(&bytes);
        Ok(Self::deserialize(&mut deserializer)?)
    }

    fn load_from_reader<R>(reader: R) -> Result<Self, Box<dyn Error>>
    where
        R: Read,
    {
        let mut deserializer = Deserializer::new(reader);
        Ok(Self::deserialize(&mut deserializer)?)
    }

    fn save_to_writer<W>(&self, writer: W) -> Result<(), Box<dyn Error>>
    where
        W: Write,
    {
        let mut serializer = Serializer::new(writer);
        self.serialize(&mut serializer)?;
        Ok(())
    }

    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut out = Vec::with_capacity(128);
        let mut serializer: Serializer<&mut Vec<u8>> = Serializer::new(out.as_mut());
        self.serialize(&mut serializer)?;
        Ok(out)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn Error>> {
        let mut deserializer = Deserializer::new(bytes);
        Ok(Self::deserialize(&mut deserializer)?)
    }
}
