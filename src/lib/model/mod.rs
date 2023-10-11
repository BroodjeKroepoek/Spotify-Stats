use std::{
    error::Error,
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

pub mod raw_streaming_data;
pub mod streaming_data;

pub trait Persist: Serialize + for<'a> Deserialize<'a> + Sized {
    fn save_to_file<P>(&self, key: P) -> Result<(), Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let handle = File::create(key)?;
        let bufwriter = BufWriter::new(handle);
        let mut serializer = Serializer::new(bufwriter);
        self.serialize(&mut serializer)?;
        Ok(())
    }

    fn load_from_file<P>(key: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let handle = File::open(key)?;
        let bufreader = BufReader::new(handle);
        let mut deserializer = Deserializer::new(bufreader);
        Ok(Self::deserialize(&mut deserializer)?)
    }

    fn load_from_reader<R>(reader: R) -> Result<Self, Box<dyn Error>>
    where
        R: Read,
    {
        let bufreader = BufReader::new(reader);
        let mut deserializer = Deserializer::new(bufreader);
        Ok(Self::deserialize(&mut deserializer)?)
    }

    fn save_to_writer<W>(&self, writer: W) -> Result<(), Box<dyn Error>>
    where
        W: Write,
    {
        let bufwriter = BufWriter::new(writer);
        let mut serializer = Serializer::new(bufwriter);
        self.serialize(&mut serializer)?;
        Ok(())
    }

    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let out = Vec::new();
        let mut bufwriter = BufWriter::new(out);
        let mut serializer = Serializer::new(bufwriter.by_ref());
        self.serialize(&mut serializer)?;
        Ok(bufwriter.into_inner()?)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn Error>> {
        let mut deserializer = Deserializer::new(bytes);
        Ok(Self::deserialize(&mut deserializer)?)
    }
}
