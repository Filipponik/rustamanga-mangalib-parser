use futures::StreamExt;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::mangalib::{MangaPreview, search::get_manga_iter_default_rate_limiter};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to create resource/json/mangalib_manga_list.json file: {0}")]
    FileCreation(std::io::Error),
    #[error("Failed to write to resource/json/mangalib_manga_list.json file: {0}")]
    FileWrite(std::io::Error),
    #[error("Cannot serialize output to json")]
    Serialization(serde_json::Error),
    #[error("Failed to search for manga: {0}")]
    Search(crate::mangalib::search::SearchError),
}

/// # Errors
/// - [`Error::Search`] if the search fails
/// - [`Error::FileCreation`] if the file cannot be created
/// - [`Error::FileWrite`] if the file cannot be written to
/// - [`Error::Serialization`] if the output cannot be serialized to json
pub async fn collect_resource() -> Result<(), Error> {
    let iter = get_manga_iter_default_rate_limiter(30).map_err(Error::Search)?;
    let output = iter.collect::<Vec<MangaPreview>>().await;

    let mut file = File::create("resource/json/mangalib_manga_list.json")
        .await
        .map_err(Error::FileCreation)?;

    file.write_all(
        serde_json::to_string(&output)
            .map_err(Error::Serialization)?
            .as_bytes(),
    )
    .await
    .map_err(Error::FileWrite)?;

    Ok(())
}
