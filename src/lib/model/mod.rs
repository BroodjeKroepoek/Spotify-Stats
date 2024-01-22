pub mod compression;
pub mod end_stream;

use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use eyre::{Ok, Result};
use flate2::{
    write::{DeflateDecoder, DeflateEncoder},
    Compression,
};
use rmp_serde::{decode::from_slice, encode::to_vec};
use serde::{Deserialize, Serialize};

pub trait Persist: Serialize + for<'a> Deserialize<'a> {
    fn _impl_serialize(&self) -> Result<Vec<u8>> {
        Ok(to_vec(self)?)
    }

    fn _impl_deserialize(bytes: &[u8]) -> Result<Self> {
        Ok(from_slice(bytes)?)
    }

    fn _impl_compress(bytes: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = DeflateEncoder::new(Vec::with_capacity(128), Compression::best());
        encoder.write_all(bytes)?;
        Ok(encoder.finish()?)
    }

    fn _impl_decompress(compressed_bytes: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = DeflateDecoder::new(Vec::with_capacity(128));
        decoder.write_all(compressed_bytes)?;
        Ok(decoder.finish()?)
    }

    fn to_bytes(&self, compress: bool) -> Result<Vec<u8>> {
        let bytes = self._impl_serialize()?;
        if compress {
            Self::_impl_compress(&bytes)
        } else {
            Ok(bytes)
        }
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if let eyre::Result::Ok(decompressed_bytes) = Self::_impl_decompress(bytes) {
            Self::_impl_deserialize(&decompressed_bytes)
        } else {
            Self::_impl_deserialize(bytes)
        }
    }

    fn save_to_file<P>(&self, path: P, compress: bool) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let maybe_compressed_bytes = self.to_bytes(compress)?;
        let mut file_handle = File::create(path)?;
        file_handle.write_all(&maybe_compressed_bytes)?;
        Ok(())
    }

    fn load_from_file<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut maybe_compressed_bytes = Vec::with_capacity(128);
        let mut handle = File::open(path)?;
        handle.read_to_end(&mut maybe_compressed_bytes)?;
        Self::from_bytes(&maybe_compressed_bytes)
    }
}

impl<T: Serialize + for<'a> Deserialize<'a>> Persist for T {}
