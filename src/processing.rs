use crate::{mangalib, processing::commands::Command};
use std::sync::Arc;

mod commands;
pub mod manga;
pub mod user_list;

#[derive(Clone)]
pub struct Processor<TClient: mangalib::Client> {
    client: Arc<TClient>,
    sender: reqwest::Client,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Manga error: {0}")]
    Manga(#[from] manga::Error),
    #[error("User list error: {0}")]
    UserList(#[from] user_list::Error),
    #[error("Parse command error: {0}")]
    ParseCommand(#[from] commands::ParseError),
    #[error("Client build error: {0}")]
    ClientBuild(reqwest::Error),
}

impl<TClient: mangalib::Client + 'static> Processor<TClient> {
    /// # Errors
    /// - [`Error::ClientBuild`] - Error occurred while building the client.
    pub fn new(client: TClient, sender: Option<reqwest::Client>) -> Result<Self, Error> {
        let sender = if let Some(sender) = sender {
            sender
        } else {
            let reqwest_client = reqwest::ClientBuilder::new()
                .timeout(std::time::Duration::from_mins(2))
                .build();

            match reqwest_client {
                Ok(client) => client,
                Err(err) => return Err(Error::ClientBuild(err)),
            }
        };

        Ok(Self {
            client: Arc::new(client),
            sender,
        })
    }

    /// Processes an incoming scrap request using the configured client and sender.
    ///
    /// # Errors
    /// [`Error::Manga`] - Error occurred while processing manga data.
    /// [`Error::UserList`] - Error occurred while processing user list data.
    /// [`Error::ParseCommand`] - Error occurred while parsing command.
    pub async fn process(&self, semaphore_permits: usize, payload: &str) -> Result<(), Error> {
        let command = commands::parse_command(payload)?;
        match command {
            Command::GetMangaWithChaptersAndImages(params) => {
                manga::handle_full(
                    &params,
                    self.client.clone(),
                    &self.sender,
                    semaphore_permits,
                )
                .await?;
            }
            Command::GetMangaWithOnlyChapters(params) => {
                manga::handle_only_chapters(&params, self.client.clone(), &self.sender).await?;
            }
            Command::GetUserList(params) => user_list::handle(&params).await?,
        }

        Ok(())
    }
}
