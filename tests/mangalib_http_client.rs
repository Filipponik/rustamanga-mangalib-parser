#![allow(clippy::expect_used)]

use mockito::{Matcher, Server};
use rustamanga_mangalib_parser::mangalib::{Client, Error, MangaChapter, http_client::HttpClient};
use utils::load_fixture;

mod utils;

#[tokio::test]
async fn test_get_chapters_positive() {
    // arrange
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/api/manga/i-alone-level-up/chapters")
        .match_query(Matcher::Any)
        .match_header("Referrer", "test_referrer")
        .match_header("Site-Id", "test_site_id")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            load_fixture("chapters_response.json")
                .expect("Failed to load fixture")
                .to_string(),
        )
        .create_async()
        .await;

    let client = HttpClient::builder()
        .base_url(server.url())
        .referrer_header("test_referrer")
        .site_id_header("test_site_id")
        .timeout(std::time::Duration::from_secs(2))
        .build();

    // act
    let result = client.get_manga_chapters("i-alone-level-up").await;

    // assert
    mock.assert();
    assert!(result.is_ok());
    let result = result.expect("Expected chapters response to succeed");
    assert_eq!(2, result.len());
    assert_eq!(result[0], MangaChapter::new("1", "0"));
    assert_eq!(result[1], MangaChapter::new("1", "1"));
    drop(server);
}

#[tokio::test]
async fn test_get_chapters_bad_response() {
    // arrange
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/api/manga/i-alone-level-up/chapters")
        .match_query(Matcher::Any)
        .match_header("Referrer", "test_referrer")
        .match_header("Site-Id", "test_site_id")
        .with_status(404)
        .with_header("Content-Type", "application/json")
        .with_body("some bad response")
        .create_async()
        .await;

    let client = HttpClient::builder()
        .base_url(server.url())
        .referrer_header("test_referrer")
        .site_id_header("test_site_id")
        .timeout(std::time::Duration::from_secs(2))
        .build();

    // act
    let result = client.get_manga_chapters("i-alone-level-up").await;

    // assert
    mock.assert();
    assert!(result.is_err());
    let result = result.expect_err("Expected serde parse error");

    #[allow(unused_variables)]
    let expected_url = format!("{}/api/manga/i-alone-level-up/chapters", server.url());
    assert!(matches!(
        result,
        Error::ReqwestResponseStatus {
            status: reqwest::StatusCode::NOT_FOUND,
            #[allow(unused_variables)]
            url: expected_url
        }
    ));
    drop(server);
}

#[tokio::test]
async fn test_get_chapters_server_down() {
    // arrange
    let client = HttpClient::builder()
        .base_url("http://localhost:54321")
        .referrer_header("test_referrer")
        .site_id_header("test_site_id")
        .timeout(std::time::Duration::from_millis(10))
        .build();

    // act
    let result = client.get_manga_chapters("i-alone-level-up").await;

    // assert
    assert!(result.is_err());
    let result = result.expect_err("Expected chapters network error");
    assert!(matches!(result, Error::ReqwestNetwork { .. }));
    if let Error::ReqwestNetwork { url, .. } = result {
        assert_eq!(
            "http://localhost:54321/api/manga/i-alone-level-up/chapters",
            url
        );
    }
}

#[tokio::test]
async fn test_get_chapter_images_positive() {
    // arrange
    let mut server = Server::new_async().await;
    let mock = server
        .mock(
            "GET",
            "/api/manga/i-alone-level-up/chapter?number=0&volume=1",
        )
        .match_header("Referrer", "test_referrer")
        .match_header("Site-Id", "test_site_id")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            load_fixture("images_response.json")
                .expect("Failed to load fixture")
                .to_string(),
        )
        .create_async()
        .await;

    let client = HttpClient::builder()
        .base_url(server.url())
        .referrer_header("test_referrer")
        .site_id_header("test_site_id")
        .image_server_prefix("http://localhost:54321")
        .timeout(std::time::Duration::from_secs(2))
        .build();

    // act
    let result = client
        .get_manga_chapter_images("i-alone-level-up", &MangaChapter::new("1", "0"), 1, 1)
        .await;

    // assert
    mock.assert();
    assert!(result.is_ok());
    let result = result.expect("Expected chapter images response to succeed");
    assert_eq!(4, result.len());
    assert_eq!(
        result[0],
        "http://localhost:54321//manga/i-alone-level-up/chapters/214970/1.png"
    );
    assert_eq!(
        result[1],
        "http://localhost:54321//manga/i-alone-level-up/chapters/214970/2.png"
    );
    assert_eq!(
        result[2],
        "http://localhost:54321//manga/i-alone-level-up/chapters/214970/3.png"
    );
    assert_eq!(
        result[3],
        "http://localhost:54321//manga/i-alone-level-up/chapters/214970/end.png"
    );
    drop(server);
}

#[tokio::test]
async fn test_get_chapter_images_bad_response() {
    // arrange
    let mut server = Server::new_async().await;
    let mock = server
        .mock(
            "GET",
            "/api/manga/i-alone-level-up/chapter?number=0&volume=1",
        )
        .match_header("Referrer", "test_referrer")
        .match_header("Site-Id", "test_site_id")
        .with_status(404)
        .with_header("Content-Type", "application/json")
        .with_body("some bad response")
        .create_async()
        .await;

    let client = HttpClient::builder()
        .base_url(server.url())
        .referrer_header("test_referrer")
        .site_id_header("test_site_id")
        .image_server_prefix("http://localhost:54321")
        .timeout(std::time::Duration::from_secs(2))
        .build();

    // act
    let result = client
        .get_manga_chapter_images("i-alone-level-up", &MangaChapter::new("1", "0"), 1, 1)
        .await;

    // assert
    mock.assert();
    assert!(result.is_err());
    let result = result.expect_err("Expected response status error");

    #[allow(unused_variables)]
    let expected_url = format!(
        "{}/api/manga/i-alone-level-up/chapter?number=0&volume=1",
        server.url()
    );
    assert!(matches!(
        result,
        Error::ReqwestResponseStatus {
            status: reqwest::StatusCode::NOT_FOUND,
            #[allow(unused_variables)]
            url: expected_url
        }
    ));
    drop(server);
}

#[tokio::test]
async fn test_get_chapter_images_server_down() {
    // arrange
    let client = HttpClient::builder()
        .base_url("http://localhost:54321")
        .referrer_header("test_referrer")
        .site_id_header("test_site_id")
        .image_server_prefix("http://localhost:54321")
        .timeout(std::time::Duration::from_millis(10))
        .build();

    // act
    let result = client
        .get_manga_chapter_images("i-alone-level-up", &MangaChapter::new("1", "0"), 1, 1)
        .await;

    // assert
    assert!(result.is_err());
    let result = result.expect_err("Expected chapter images network error");
    assert!(matches!(result, Error::ReqwestNetwork { .. }));
    if let Error::ReqwestNetwork { url, .. } = result {
        assert_eq!(
            "http://localhost:54321/api/manga/i-alone-level-up/chapter?number=0&volume=1",
            url
        );
    }
}
