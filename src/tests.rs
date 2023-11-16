use std::{error::Error, fs};

use spotify_stats::model::{
    raw_streaming_data::RawStreamingData,
    streaming_data::{CleanedStreamingData, FoldedStreamingData},
    Persist,
};

const DATA_FOLDER: &str = "full_data";

/// <JSON FOLDER> -> [RawStreamingData] -> <BYTES> -> [RawStreamingData]
///                          |                                |
///                          |________________________________|
#[test]
fn test_isomorphism_raw_internal_streaming_history() -> Result<(), Box<dyn Error>> {
    let initial_entries = RawStreamingData::from_folder_of_json(DATA_FOLDER)?;
    let initial_bytes = initial_entries.to_bytes()?;
    let secondary_entries = RawStreamingData::from_bytes(&initial_bytes)?;
    assert_eq!(initial_entries, secondary_entries);
    Ok(())
}

/// <JSON FOLDER> -> [RawStreamingData] -> <BYTES> -> [RawStreamingData] -> <BYTES>
///                                           |                                |
///                                           |________________________________|
#[test]
fn test_isomorphism_raw_external_streaming_history() -> Result<(), Box<dyn Error>> {
    let initial_entries = RawStreamingData::from_folder_of_json(DATA_FOLDER)?;
    let initial_bytes = initial_entries.to_bytes()?;

    let secondary_entries = RawStreamingData::from_bytes(&initial_bytes)?;
    let secondary_bytes = secondary_entries.to_bytes()?;

    assert_eq!(initial_bytes, secondary_bytes);
    Ok(())
}

/// <JSON FOLDER> -> [RawStreamingData] -> [FoldedStreamingData] -> <BYTES> -> [FoldedStreamingData]
///                                                 |                                   |
///                                                 |___________________________________|
#[test]
fn test_isomorphism_folded_internal_streaming_history() -> Result<(), Box<dyn Error>> {
    let initial_entries = RawStreamingData::from_folder_of_json(DATA_FOLDER)?;
    let initial_folded = FoldedStreamingData::from(initial_entries);
    let initial_bytes = initial_folded.to_bytes()?;

    let secondary_folded = FoldedStreamingData::from_bytes(&initial_bytes)?;
    assert_eq!(initial_folded, secondary_folded);
    Ok(())
}

/// <JSON FOLDER> -> [RawStreamingData] -> [FoldedStreamingData] -> <BYTES> -> [FoldedStreamingData] -> <BYTES>
///                                                                    |                                   |
///                                                                    |___________________________________|
#[test]
fn test_isomorphism_folded_external_streaming_history() -> Result<(), Box<dyn Error>> {
    let initial_entries = RawStreamingData::from_folder_of_json(DATA_FOLDER)?;
    let initial_folded = FoldedStreamingData::from(initial_entries);
    let initial_bytes = initial_folded.to_bytes()?;

    let secondary_folded = FoldedStreamingData::from_bytes(&initial_bytes)?;
    let secondary_bytes = secondary_folded.to_bytes()?;

    assert_eq!(initial_bytes, secondary_bytes);
    Ok(())
}

/// <JSON FOLDER> -> [RawStreamingData] -> [FoldedStreamingData] -> <FILE> -> [FoldedStreamingData]
///                                                  |                                  |
///                                                  |__________________________________|
#[test]
fn test_persist_folded_streaming_data() -> Result<(), Box<dyn Error>> {
    let path = "test_persist_folded_streaming_data.json";
    let initial_entries = RawStreamingData::from_folder_of_json(DATA_FOLDER)?;
    let initial_folded = FoldedStreamingData::from(initial_entries);

    // TODO: Make load determine by itself if the decompression is needed.
    initial_folded.save_to_file(path)?;
    let secondary_folded = FoldedStreamingData::load_from_file(path)?;

    fs::remove_file(path)?;
    assert_eq!(initial_folded, secondary_folded);
    Ok(())
}

/// <JSON FOLDER> -> [RawStreamingData] -> [FoldedStreamingData] -> [CleanedStreamingData] -> <BYTES> -> [CleanedStreamingData]
///                                                                           |                                    |
///                                                                           |____________________________________|
#[test]
fn test_isomorphism_cleaned_internal_streaming_history() -> Result<(), Box<dyn Error>> {
    let initial_raw = RawStreamingData::from_folder_of_json(DATA_FOLDER)?;
    let initial_folded = FoldedStreamingData::from(initial_raw);
    let initial_cleaned = CleanedStreamingData::from(initial_folded);

    let bytes = initial_cleaned.to_bytes()?;
    let secondary_cleaned = CleanedStreamingData::from_bytes(&bytes)?;

    assert_eq!(initial_cleaned, secondary_cleaned);
    Ok(())
}

/// <JSON FOLDER> -> [RawStreamingData] -> [FoldedStreamingData] -> [CleanedStreamingData] -> <BYTES> -> [CleanedStreamingData] -> <BYTES>
///                                                                                              |                                    |
///                                                                                              |____________________________________|
#[test]
fn test_isomorphism_cleaned_external_streaming_history() -> Result<(), Box<dyn Error>> {
    let initial_entries = RawStreamingData::from_folder_of_json(DATA_FOLDER)?;
    let initial_folded = FoldedStreamingData::from(initial_entries);
    let initial_cleaned = CleanedStreamingData::from(initial_folded);

    let initial_bytes = initial_cleaned.to_bytes()?;
    let secondary_cleaned = CleanedStreamingData::from_bytes(&initial_bytes)?;

    let secondary_bytes = secondary_cleaned.to_bytes()?;

    assert_eq!(initial_bytes, secondary_bytes);
    Ok(())
}

/// <JSON FOLDER> -> [RawStreamingData] -> [FoldedStreamingData] -> [CleanedStreamingData] -> <FILE> -> [CleanedStreamingData]
///                                                                           |                                   |
///                                                                           |___________________________________|
#[test]
fn test_persist_cleaned_streaming_data() -> Result<(), Box<dyn Error>> {
    let path = "test_persist_cleaned_streaming_data.json";

    let initial_entries = RawStreamingData::from_folder_of_json(DATA_FOLDER)?;
    let initial_folded = FoldedStreamingData::from(initial_entries);
    let initial_cleaned = CleanedStreamingData::from(initial_folded);

    // TODO: Make load determine by itself if the decompression is needed.
    initial_cleaned.save_to_file(path)?;
    let secondary_cleaned = CleanedStreamingData::load_from_file(path)?;

    fs::remove_file(path)?;
    assert_eq!(initial_cleaned, secondary_cleaned);
    Ok(())
}
