const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36";
const ACCEPT_LANGUAGE: &str = "en-US,en;q=0.9,hi;q=0.8,es;q=0.7,lt;q=0.6";
const PLATFORM: &str = "macOS";
const IMAGE_SERVER_PREFIX: &str = "https://img33.imgslib.link";
const MANGALIB_DEFAULT_BASE_URL: &str = "https://api.mangalib.me";

use crate::mangalib::HeadlessBrowserClient;

#[derive(Default, Debug)]
pub struct Builder {
    user_agent: Option<String>,
    accept_language: Option<String>,
    platform: Option<String>,
    image_server_prefix: Option<String>,
    base_url: Option<String>,
}

impl Builder {
    pub fn user_agent(mut self, user_agent: &str) -> Self {
        self.user_agent = Some(user_agent.to_string());
        self
    }

    pub fn accept_language(mut self, accept_language: &str) -> Self {
        self.accept_language = Some(accept_language.to_string());
        self
    }

    pub fn platform(mut self, platform: &str) -> Self {
        self.platform = Some(platform.to_string());
        self
    }

    pub fn image_server_prefix(mut self, image_server_prefix: &str) -> Self {
        self.image_server_prefix = Some(image_server_prefix.to_string());
        self
    }

    pub fn base_url(mut self, base_url: &str) -> Self {
        self.base_url = Some(base_url.to_string());
        self
    }

    pub fn build(self) -> HeadlessBrowserClient {
        HeadlessBrowserClient {
            user_agent: self.user_agent.unwrap_or_else(|| USER_AGENT.to_string()),
            accept_language: self
                .accept_language
                .unwrap_or_else(|| ACCEPT_LANGUAGE.to_string()),
            platform: self.platform.unwrap_or_else(|| PLATFORM.to_string()),
            image_server_prefix: self
                .image_server_prefix
                .unwrap_or_else(|| IMAGE_SERVER_PREFIX.to_string()),
            base_url: self
                .base_url
                .unwrap_or_else(|| MANGALIB_DEFAULT_BASE_URL.to_string()),
        }
    }
}
