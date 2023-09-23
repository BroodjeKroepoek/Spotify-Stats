use std::path::Path;

pub mod raw_streaming_data;
pub mod streaming_data;

trait Persist {
    fn save<P>(&self, key: P) -> Result<(), std::io::Error>
    where
        P: AsRef<Path>;

    fn load<P>(key: P) -> Result<Self, std::io::Error>
    where
        Self: Sized,
        P: AsRef<Path>;
}
