use std::path::Path;

pub mod raw_streaming_data;
pub mod streaming_data;

pub trait Persist {
    type Error;

    fn save<P>(&self, key: P) -> Result<(), Self::Error>
    where
        P: AsRef<Path>;

    fn load<P>(key: P) -> Result<Self, Self::Error>
    where
        Self: Sized,
        P: AsRef<Path>;
}
