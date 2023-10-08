use std::{error::Error, fs};

use spotify_stats::model::{
    raw_streaming_data::RawStreamingData,
    streaming_data::{CleanedStreamingData, FoldedStreamingData},
    Persist,
};

const DATA_FOLDER: &str = "full_data";

#[test]
fn test_isomorphism_raw_internal_streaming_history() -> Result<(), Box<dyn Error>> {
    let initial_entries = RawStreamingData::from_path(DATA_FOLDER)?;
    let initial_json_representation = postcard::to_stdvec(&initial_entries)?;
    let secondary_entries: RawStreamingData = postcard::from_bytes(&initial_json_representation)?;
    assert_eq!(initial_entries, secondary_entries);
    Ok(())
}

#[test]
fn test_isomorphism_raw_external_streaming_history() -> Result<(), Box<dyn Error>> {
    let initial_entries = RawStreamingData::from_path(DATA_FOLDER)?;
    let initial_json_representation = postcard::to_stdvec(&initial_entries)?;
    let secondary_entries: RawStreamingData = postcard::from_bytes(&initial_json_representation)?;
    let secondary_json_representation = postcard::to_stdvec(&secondary_entries)?;
    assert_eq!(initial_json_representation, secondary_json_representation);
    Ok(())
}

#[test]
fn test_isomorphism_folded_internal_streaming_history() -> Result<(), Box<dyn Error>> {
    let initial_entries = RawStreamingData::from_path(DATA_FOLDER)?;
    let initial_cleaned = FoldedStreamingData::from(initial_entries);
    let initial_json_cleaned_representation = postcard::to_stdvec(&initial_cleaned)?;
    let secondary_cleaned: FoldedStreamingData =
        postcard::from_bytes(&initial_json_cleaned_representation)?;
    assert_eq!(initial_cleaned, secondary_cleaned);
    Ok(())
}

#[test]
fn test_isomorphism_folded_external_streaming_history() -> Result<(), Box<dyn Error>> {
    let initial_entries = RawStreamingData::from_path(DATA_FOLDER)?;
    let initial_cleaned = FoldedStreamingData::from(initial_entries);
    let initial_json_cleaned_representation = postcard::to_stdvec(&initial_cleaned)?;
    let secondary_cleaned: FoldedStreamingData =
        postcard::from_bytes(&initial_json_cleaned_representation)?;
    let secondary_json_cleaned_representation = postcard::to_stdvec(&secondary_cleaned)?;
    assert_eq!(
        initial_json_cleaned_representation,
        secondary_json_cleaned_representation
    );
    Ok(())
}

#[test]
fn test_persist_folded_streaming_data() -> Result<(), Box<dyn Error>> {
    let path = "test_persist_folded_streaming_data.json";
    let initial_entries = RawStreamingData::from_path(DATA_FOLDER)?;
    let initial_cleaned = FoldedStreamingData::from(initial_entries);
    initial_cleaned.save(path)?;
    let secondary_cleaned = FoldedStreamingData::load(path)?;
    fs::remove_file(path)?;
    assert_eq!(initial_cleaned, secondary_cleaned);
    Ok(())
}

#[test]
fn test_isomorphism_cleaned_internal_streaming_history() -> Result<(), Box<dyn Error>> {
    let initial_entries = RawStreamingData::from_path(DATA_FOLDER)?;
    let initial_folded = FoldedStreamingData::from(initial_entries);
    let initial_cleaned = CleanedStreamingData::from(initial_folded);
    let initial_json_cleaned_representation = postcard::to_stdvec(&initial_cleaned)?;
    let secondary_cleaned: CleanedStreamingData =
        postcard::from_bytes(&initial_json_cleaned_representation)?;
    assert_eq!(initial_cleaned, secondary_cleaned);
    Ok(())
}

#[test]
fn test_isomorphism_cleaned_external_streaming_history() -> Result<(), Box<dyn Error>> {
    let initial_entries = RawStreamingData::from_path(DATA_FOLDER)?;
    let initial_folded = FoldedStreamingData::from(initial_entries);
    let initial_cleaned = CleanedStreamingData::from(initial_folded);
    let initial_json_cleaned_representation = postcard::to_stdvec(&initial_cleaned)?;
    let secondary_cleaned: CleanedStreamingData =
        postcard::from_bytes(&initial_json_cleaned_representation)?;
    let secondary_json_cleaned_representation = postcard::to_stdvec(&secondary_cleaned)?;
    assert_eq!(
        initial_json_cleaned_representation,
        secondary_json_cleaned_representation
    );
    Ok(())
}

#[test]
fn test_persist_cleaned_streaming_data() -> Result<(), Box<dyn Error>> {
    let path = "test_persist_cleaned_streaming_data.json";
    let initial_entries = RawStreamingData::from_path(DATA_FOLDER)?;
    let initial_folded = FoldedStreamingData::from(initial_entries);
    let initial_cleaned = CleanedStreamingData::from(initial_folded);
    initial_cleaned.save(path)?;
    let secondary_cleaned = CleanedStreamingData::load(path)?;
    fs::remove_file(path)?;
    assert_eq!(initial_cleaned, secondary_cleaned);
    Ok(())
}
