//! This module describes a persistence trait, that allows use to store data between runs of this executable.\
//!
//! This is so we don't have to process the raw Spotify streaming data on every run!
//!
//! # raw_streaming_data
//!
//! Here we describe the front-end format that Spotify uses.
//!
//! # streaming_data
//!
//! Here we describe the back-end format that we use.

use std::{
    collections::BTreeMap,
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

    /// Serializes the implementing object to a compressed byte vector.
    /// Note: allows for extending other datastructures with this one.
    ///
    /// # Returns
    ///
    /// Returns the byte vector containing the serialized object on success or a `Box<dyn Error>` on failure.
    fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        // Serialize the object to a byte vector
        let mut bytes = Vec::new();
        let mut serializer: Serializer<&mut Vec<u8>> = Serializer::new(bytes.as_mut());
        self.serialize(&mut serializer)?;

        // Apply DEFLATE compression to the serialized data
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(&bytes)?;

        Ok(encoder.finish()?)
    }

    // Deserializes an object from a byte slice.
    /// Note: allows for extending other datastructures with this one.
    ///
    /// # Arguments
    ///
    /// - `bytes`: The byte slice containing the serialized object.
    ///
    /// # Returns
    ///
    /// Returns the deserialized object on success or a `Box<dyn Error>` on failure.
    fn from_bytes(compressed_bytes: &[u8]) -> Result<Self, Box<dyn Error>> {
        // Read the compressed bytes
        let mut bytes = Vec::new();
        let mut decoder = DeflateDecoder::new(compressed_bytes);
        decoder.read_to_end(&mut bytes)?;

        // Deserialize the object from the byte slice
        let mut deserializer: Deserializer<ReadReader<&[u8]>> = Deserializer::new(bytes.as_mut());
        Ok(Self::deserialize(&mut deserializer)?)
    }
}

impl<K, V> Persist for BTreeMap<K, V>
where
    K: for<'a> Deserialize<'a> + Serialize + Ord,
    V: for<'a> Deserialize<'a> + Serialize,
{
}
