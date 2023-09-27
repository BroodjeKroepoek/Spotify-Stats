use std::{error::Error, fs};

use crate::model::{
    raw_streaming_data::RawStreamingData, streaming_data::FoldedStreamingData, Persist,
};

const DATA_FOLDER: &str = "full_data";

#[test]
fn test_isomorphism_raw_internal_streaming_history() -> Result<(), Box<dyn Error>> {
    let initial_entries = RawStreamingData::from_path(DATA_FOLDER)?;
    let initial_json_representation = serde_json::to_string(&initial_entries)?;
    let secondary_entries: RawStreamingData = serde_json::from_str(&initial_json_representation)?;
    assert_eq!(initial_entries, secondary_entries);
    Ok(())
}

#[test]
fn test_isomorphism_raw_external_streaming_history() -> Result<(), Box<dyn Error>> {
    let initial_entries = RawStreamingData::from_path(DATA_FOLDER)?;
    let initial_json_representation = serde_json::to_string(&initial_entries)?;
    let secondary_entries: RawStreamingData = serde_json::from_str(&initial_json_representation)?;
    let secondary_json_representation = serde_json::to_string(&secondary_entries)?;
    assert_eq!(initial_json_representation, secondary_json_representation);
    Ok(())
}

#[test]
fn test_isomorphism_folded_internal_streaming_history() -> Result<(), Box<dyn Error>> {
    let initial_entries = RawStreamingData::from_path(DATA_FOLDER)?;
    let initial_cleaned = FoldedStreamingData::from(initial_entries);
    let initial_json_cleaned_representation = serde_json::to_string(&initial_cleaned)?;
    let secondary_cleaned: FoldedStreamingData =
        serde_json::from_str(&initial_json_cleaned_representation)?;
    assert_eq!(initial_cleaned, secondary_cleaned);
    Ok(())
}

#[test]
fn test_isomorphism_folded_external_streaming_history() -> Result<(), Box<dyn Error>> {
    let initial_entries = RawStreamingData::from_path(DATA_FOLDER)?;
    let initial_cleaned = FoldedStreamingData::from(initial_entries);
    let initial_json_cleaned_representation = serde_json::to_string(&initial_cleaned)?;
    let secondary_cleaned: FoldedStreamingData =
        serde_json::from_str(&initial_json_cleaned_representation)?;
    let secondary_json_cleaned_representation = serde_json::to_string(&secondary_cleaned)?;
    assert_eq!(
        initial_json_cleaned_representation,
        secondary_json_cleaned_representation
    );
    Ok(())
}

#[test]
fn test_persist_folded_streaming_data() -> Result<(), Box<dyn Error>> {
    let initial_entries = RawStreamingData::from_path(DATA_FOLDER)?;
    let initial_cleaned = FoldedStreamingData::from(initial_entries);
    initial_cleaned.save("test.json")?;
    let secondary_cleaned = FoldedStreamingData::load("test.json")?;
    fs::remove_file("test.json")?;
    assert_eq!(initial_cleaned, secondary_cleaned);
    Ok(())
}
