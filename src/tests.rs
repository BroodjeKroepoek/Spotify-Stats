use std::error::Error;

use crate::model::{raw_streaming_data::RawStreamingData, streaming_data::StreamingData};

#[test]
fn test_isomorphism_raw_internal_streaming_history() -> Result<(), Box<dyn Error>> {
    let initial_entries = RawStreamingData::from_path("data")?;
    let initial_json_representation = serde_json::to_string(&initial_entries)?;
    let secondary_entries: RawStreamingData = serde_json::from_str(&initial_json_representation)?;
    assert_eq!(initial_entries, secondary_entries);
    Ok(())
}

#[test]
fn test_isomorphism_raw_external_streaming_history() -> Result<(), Box<dyn Error>> {
    let initial_entries = RawStreamingData::from_path("data")?;
    let initial_json_representation = serde_json::to_string(&initial_entries)?;
    let secondary_entries: RawStreamingData = serde_json::from_str(&initial_json_representation)?;
    let secondary_json_representation = serde_json::to_string(&secondary_entries)?;
    assert_eq!(initial_json_representation, secondary_json_representation);
    Ok(())
}

#[test]
fn test_isomorphism_internal_streaming_history() -> Result<(), Box<dyn Error>> {
    let initial_entries = RawStreamingData::from_path("data")?;
    let initial_cleaned: StreamingData = StreamingData::from(initial_entries);
    let initial_json_cleaned_representation = serde_json::to_string(&initial_cleaned)?;
    let secondary_cleaned: StreamingData =
        serde_json::from_str(&initial_json_cleaned_representation)?;
    assert_eq!(initial_cleaned, secondary_cleaned);
    Ok(())
}

#[test]
fn test_isomorphism_external_streaming_history() -> Result<(), Box<dyn Error>> {
    let initial_entries = RawStreamingData::from_path("data")?;
    let initial_cleaned: StreamingData = StreamingData::from(initial_entries);
    let initial_json_cleaned_representation = serde_json::to_string(&initial_cleaned)?;
    let secondary_cleaned: StreamingData =
        serde_json::from_str(&initial_json_cleaned_representation)?;
    let secondary_json_cleaned_representation = serde_json::to_string(&secondary_cleaned)?;
    assert_eq!(
        initial_json_cleaned_representation,
        secondary_json_cleaned_representation
    );
    Ok(())
}
