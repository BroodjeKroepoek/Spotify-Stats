use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

pub mod raw_streaming_data;
pub mod streaming_data;

pub trait Persist {
    type Error;

    fn save<P>(&self, key: P) -> Result<(), Self::Error>
    where
        P: AsRef<Path>,
        Self: Serialize,
        <Self as Persist>::Error: From<serde_json::Error> + From<std::io::Error>,
    {
        Ok(fs::write(key, serde_json::to_string(&self)?)?)
    }

    fn load<P>(key: P) -> Result<Self, Self::Error>
    where
        Self: Sized + for<'a> Deserialize<'a>,
        <Self as Persist>::Error: From<serde_json::Error> + From<std::io::Error>,
        P: AsRef<Path>,
    {
        Ok(serde_json::from_str(&fs::read_to_string(key)?)?)
    }
}
