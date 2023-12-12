//! # Persistence Trait
//!
//! This module defines a trait for persistence, enabling the storage of data between runs of the executable.
//! The primary purpose is to avoid processing raw Spotify streaming data during each execution.
//!
//! ## Front-End Format: `raw_streaming_data`
//!
//! This section describes the front-end format utilized by Spotify.
//!
//! ## Back-End Format: `streaming_data`
//!
//! This section outlines the back-end format employed within this module.
//!
//! ## Usage
//!
//! The trait `Persist` is provided to facilitate the serialization and deserialization of objects, allowing them
//! to be stored and retrieved from files. The trait requires implementing the `Serialize` and `Deserialize`
//! traits from the `serde` crate.
//!
//! # Error Handling
//!
//! Errors during the saving and loading processes are encapsulated in specific error types: `SaveError` and `LoadError`.
//!
//! ## SaveError
//!
//! - `EncodeError`: Occurs when encoding the object fails.
//! - `IOError`: Indicates an I/O error during file operations.
//!
//! ## LoadError
//!
//! - `DecodeError`: Arises when decoding the object from bytes fails.
//! - `IOError`: Denotes an I/O error during file operations.
//!
//! # Methods
//!
//! The `Persist` trait provides the following methods:
//!
//! ## `save_to_file`
//!
//! Saves the implementing object to a specified file path.
//!
//! ## `load_from_file`
//!
//! Loads an object from the specified file path.
//!
//! ## `to_bytes`
//!
//! Serializes the implementing object to a compressed byte vector.
//!
//! ## `from_bytes`
//!
//! Deserializes an object from a byte slice.
//!
//! # Dependencies
//!
//! This module relies on the following external crates:
//! - `flate2` for DEFLATE compression.
//! - `rmp_serde` for MessagePack serialization.
//! - `serde` for serialization and deserialization.
//!
//! Note: Ensure the dependencies are properly added to your project's Cargo.toml file.
//!
//! # License
//!
//! This module is distributed under the terms of the MIT license.
//!
//! ---  
//!
use std::{
    error::Error,
    fmt::Display,
    fs::File,
    io::{Read, Write},
    path::Path,
};

use flate2::{
    write::{DeflateDecoder, DeflateEncoder},
    Compression,
};
use rmp_serde::{decode::from_slice, encode::to_vec};
use serde::{Deserialize, Serialize};

pub mod raw_streaming_data;
pub mod streaming_data;

/// Error type for save operations.
#[derive(Debug)]
pub enum SaveError {
    EncodeError(rmp_serde::encode::Error),
    IOError(std::io::Error),
}

impl Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::EncodeError(err) => write!(f, "encoding error: {err}"),
            SaveError::IOError(err) => write!(f, "io error: {err}"),
        }
    }
}

impl Error for SaveError {}

/// Error type for load operations.
#[derive(Debug)]
pub enum LoadError {
    DecodeError(rmp_serde::decode::Error),
    IOError(std::io::Error),
}

impl Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::DecodeError(err) => write!(f, "decoding error: {err}"),
            LoadError::IOError(err) => write!(f, "io error: {err}"),
        }
    }
}

impl Error for LoadError {}

/// The Persistence trait for serializing and deserializing objects.
pub trait Persist: Serialize + for<'a> Deserialize<'a> + Sized {
    /// Saves the implementing object to a file.
    ///
    /// # Arguments
    ///
    /// - `key`: The path to the file where the object will be saved.
    ///
    /// # Returns
    ///
    /// Returns `Result` with `()` on success or a `SaveError` on failure.
    fn save_to_file<P>(&self, key: P) -> Result<(), SaveError>
    where
        P: AsRef<Path>,
    {
        let compressed_bytes = self.to_bytes()?;

        // Write the compressed data to the file
        let mut handle = File::create(key).map_err(SaveError::IOError)?;
        handle
            .write_all(&compressed_bytes)
            .map_err(SaveError::IOError)?;
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
    /// Returns the loaded object on success or a `LoadError` on failure.
    fn load_from_file<P>(key: P) -> Result<Self, LoadError>
    where
        P: AsRef<Path>,
    {
        // Read the compressed data from the file
        let mut compressed_bytes = Vec::new();
        let mut handle = File::open(key).map_err(LoadError::IOError)?;
        handle
            .read_to_end(&mut compressed_bytes)
            .map_err(LoadError::IOError)?;

        Self::from_bytes(&compressed_bytes)
    }

    /// Serializes the implementing object to a compressed byte vector.
    /// Note: allows for extending other datastructures with this one.
    ///
    /// # Returns
    ///
    /// Returns the byte vector containing the serialized object on success or a `SaveError` on failure.
    fn to_bytes(&self) -> Result<Vec<u8>, SaveError> {
        // Serialize the object
        let bytes = to_vec(self).map_err(SaveError::EncodeError)?;

        // Apply DEFLATE compression
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(&bytes).map_err(SaveError::IOError)?;
        let compressed_bytes = encoder.finish().map_err(SaveError::IOError)?;
        Ok(compressed_bytes)
    }

    // Deserializes an object from a byte slice.
    /// Note: allows for extending other data structures with this one.
    ///
    /// # Arguments
    ///
    /// - `bytes`: The byte slice containing the serialized object.
    ///
    /// # Returns
    ///
    /// Returns the deserialized object on success or a `LoadError` on failure.
    fn from_bytes(compressed_bytes: &[u8]) -> Result<Self, LoadError> {
        let mut decoder = DeflateDecoder::new(Vec::new());
        decoder
            .write_all(compressed_bytes)
            .map_err(LoadError::IOError)?;
        let bytes = decoder.finish().map_err(LoadError::IOError)?;

        // Deserialize the object
        let myself: Self = from_slice(&bytes).map_err(LoadError::DecodeError)?;
        Ok(myself)
    }
}

impl<T: Serialize + for<'a> Deserialize<'a>> Persist for T {}
