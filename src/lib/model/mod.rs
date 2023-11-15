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
    /// Saves the implementing object to a file.
    ///
    /// # Arguments
    ///
    /// - `key`: The path to the file where the object will be saved.
    /// - `use_compression`: A flag indicating whether to apply DEFLATE compression.
    ///
    /// # Returns
    ///
    /// Returns `Result` with `()` on success or a `Box<dyn Error>` on failure.
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

    /// Loads an object from a file.
    ///
    /// # Arguments
    ///
    /// - `key`: The path to the file from which the object will be loaded.
    ///
    /// # Returns
    ///
    /// Returns the loaded object on success or a `Box<dyn Error>` on failure.
    fn load_from_file<P>(key: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let mut handle = File::open(key)?;
        let mut decoder = DeflateDecoder::new(&mut handle);
        let mut bytes = Vec::new();
        let mut buf = Vec::new();
        let bytes = match decoder.read_to_end(&mut bytes) {
            Ok(_) => bytes,
            Err(_) => {
                handle.read_to_end(&mut buf)?;
                buf
            }
        };
        let mut deserializer: Deserializer<ReadReader<&[u8]>> = Deserializer::new(&bytes);
        Ok(Self::deserialize(&mut deserializer)?)
    }

    /// Loads an object from a provided reader.
    ///
    /// # Arguments
    ///
    /// - `reader`: The reader from which the object will be loaded.
    ///
    /// # Returns
    ///
    /// Returns the loaded object on success or a `Box<dyn Error>` on failure.
    fn load_from_reader<R>(reader: R) -> Result<Self, Box<dyn Error>>
    where
        R: Read,
    {
        let mut deserializer = Deserializer::new(reader);
        Ok(Self::deserialize(&mut deserializer)?)
    }

    /// Saves the implementing object to a writer.
    ///
    /// # Arguments
    ///
    /// - `writer`: The writer to which the object will be saved.
    ///
    /// # Returns
    ///
    /// Returns `Result` with `()` on success or a `Box<dyn Error>` on failure.
    fn save_to_writer<W>(&self, writer: W) -> Result<(), Box<dyn Error>>
    where
        W: Write,
    {
        let mut serializer = Serializer::new(writer);
        self.serialize(&mut serializer)?;
        Ok(())
    }

    /// Serializes the implementing object to a byte vector.
    ///
    /// # Returns
    ///
    /// Returns the byte vector containing the serialized object on success or a `Box<dyn Error>` on failure.
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut out = Vec::with_capacity(128);
        let mut serializer: Serializer<&mut Vec<u8>> = Serializer::new(out.as_mut());
        self.serialize(&mut serializer)?;
        Ok(out)
    }

    // Deserializes an object from a byte slice.
    ///
    /// # Arguments
    ///
    /// - `bytes`: The byte slice containing the serialized object.
    ///
    /// # Returns
    ///
    /// Returns the deserialized object on success or a `Box<dyn Error>` on failure.
    fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn Error>> {
        let mut deserializer = Deserializer::new(bytes);
        Ok(Self::deserialize(&mut deserializer)?)
    }
}
