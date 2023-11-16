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
    ///
    /// # Returns
    ///
    /// Returns `Result` with `()` on success or a `Box<dyn Error>` on failure.
    fn save_to_file<P>(&self, key: P) -> Result<(), Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        // Serialize the object
        let mut bytes = Vec::with_capacity(128);
        let mut serializer: Serializer<&mut Vec<u8>> = Serializer::new(bytes.as_mut());
        self.serialize(&mut serializer)?;

        // Apply DEFLATE compression
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(&bytes)?;

        // Write the compressed data to the file
        let mut handle = File::create(key)?;
        handle.write_all(&encoder.finish()?)?;
        Ok(())
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
        // Read the compressed data from the file
        let mut handle = File::open(key)?;
        let mut decoder = DeflateDecoder::new(&mut handle);
        let mut bytes = Vec::new();
        decoder.read_to_end(&mut bytes)?;

        // Deserialize the object
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
        // Deserialize the object from the reader
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
        // Serialize the object to the writer
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
        // Serialize the object to a byte vector
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
        // Deserialize the object from the byte slice
        let mut deserializer = Deserializer::new(bytes);
        Ok(Self::deserialize(&mut deserializer)?)
    }
}
