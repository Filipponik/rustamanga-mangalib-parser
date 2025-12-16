use futures::StreamExt;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::mangalib::{MangaPreview, search::get_manga_iter};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to create resource/json/mangalib_manga_list.json file: {0}")]
    FileCreation(std::io::Error),
    #[error("Failed to write to resource/json/mangalib_manga_list.json file: {0}")]
    FileWrite(std::io::Error),
    #[error("Cannot serialize output to json")]
    SerializationError(serde_json::Error),
}

/// # Errors
/// - [`Error::FileCreation`] if the file cannot be created
/// - [`Error::FileWrite`] if the file cannot be written to
/// - [`Error::SerializationError`] if the output cannot be serialized to json
pub async fn collect_resource() -> Result<(), Error> {
    let iter = get_manga_iter();
    let output = iter.collect::<Vec<MangaPreview>>().await;

    let mut file = File::create("resource/json/mangalib_manga_list.json")
        .await
        .map_err(Error::FileCreation)?;

    file.write_all(
        serde_json::to_string(&output)
            .map_err(Error::SerializationError)?
            .as_bytes(),
    )
    .await
    .map_err(Error::FileWrite)?;

    Ok(())
}
