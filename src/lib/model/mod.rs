use std::{error::Error, fs, path::Path};

use postcard::{from_bytes, to_stdvec};
use serde::{Deserialize, Serialize};

pub mod raw_streaming_data;
pub mod streaming_data;

pub trait Persist {
    fn save<P>(&self, key: P) -> Result<(), Box<dyn Error>>
    where
        P: AsRef<Path>,
        Self: Serialize,
    {
        // Ok(fs::write(key, serde_json::to_string(&self)?)?)
        Ok(fs::write(key, to_stdvec(&self)?)?)
    }

    fn load<P>(key: P) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized + for<'a> Deserialize<'a>,
        P: AsRef<Path>,
    {
        // Ok(serde_json::from_str(&fs::read_to_string(key)?)?)
        Ok(from_bytes(&fs::read(key)?)?)
    }
}
