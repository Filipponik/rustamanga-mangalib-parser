use mockito::{Matcher, Server};
use rustamanga_mangalib_parser::mangalib::{Client, Error, MangaChapter, http_client::HttpClient};
use utils::load_fixture;

mod utils;

#[tokio::test]
async fn test_get_manga_chapters_positive() {
    // arrange
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/api/manga/naruto/chapters")
        .match_query(Matcher::Any)
        .match_header("Referrer", "test_referrer")
        .match_header("Site-Id", "test_site_id")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(&load_fixture("chapters_response.json").to_string())
        .create_async()
        .await;

    let client = HttpClient::builder()
        .base_url(&server.url())
        .referrer_header("test_referrer")
        .site_id_header("test_site_id")
        .timeout(std::time::Duration::from_secs(2))
        .build();

    // act
    let result = client.get_manga_chapters("naruto").await;

    // assert
    mock.assert();
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(2, result.len());
    assert_eq!(result[0], MangaChapter::new("1", "0"));
    assert_eq!(result[1], MangaChapter::new("1", "1"));
}

#[tokio::test]
async fn test_get_manga_chapters_bad_response() {
    // arrange
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/api/manga/naruto/chapters")
        .match_query(Matcher::Any)
        .match_header("Referrer", "test_referrer")
        .match_header("Site-Id", "test_site_id")
        .with_status(404)
        .with_header("Content-Type", "application/json")
        .with_body("some bad response")
        .create_async()
        .await;

    let client = HttpClient::builder()
        .base_url(&server.url())
        .referrer_header("test_referrer")
        .site_id_header("test_site_id")
        .timeout(std::time::Duration::from_secs(2))
        .build();

    // act
    let result = client.get_manga_chapters("naruto").await;

    // assert
    mock.assert();
    assert!(result.is_err());
    let result = result.unwrap_err();
    if let Error::SerdeParse(_source) = result {
        assert!(true);
    } else {
        panic!("Unexpected error: {:?}", result);
    }
}

#[tokio::test]
async fn test_get_manga_chapters_server_down() {
    // arrange
    let client = HttpClient::builder()
        .base_url("http://localhost:54321")
        .referrer_header("test_referrer")
        .site_id_header("test_site_id")
        .timeout(std::time::Duration::from_millis(10))
        .build();

    // act
    let result = client.get_manga_chapters("naruto").await;

    // assert
    assert!(result.is_err());
    let result = result.unwrap_err();
    if let Error::ReqwestNetwork {
        source: _source,
        url,
    } = result
    {
        assert_eq!("http://localhost:54321/api/manga/naruto/chapters", url);
    } else {
        panic!("Unexpected error: {:?}", result);
    }
}
