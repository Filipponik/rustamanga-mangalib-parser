use std::error::Error;

use headless_chrome::Browser;
use headless_chrome::protocol::cdp::Page;

pub async fn get_manga_chapter_images(slug: &str, volume: i32, chapter: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let browser = Browser::default().unwrap();
    let tab = browser.new_tab().unwrap();
    let web_url = format!("https://mangalib.me/{slug}/v{volume}/c{chapter}?page=1");
    println!("going to {web_url}");
    tab.set_user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36", Some("en-US,en;q=0.9,hi;q=0.8,es;q=0.7,lt;q=0.6"), Some("macOS")).unwrap();
    tab.navigate_to(&web_url).unwrap().wait_until_navigated().unwrap();
    let reader_element = tab.wait_for_element(".reader-view").unwrap();
    let pages_count = tab.wait_for_element("#reader-pages:last-child :last-child").unwrap().get_attribute_value("value").unwrap().unwrap().parse::<i32>().unwrap();
    let mut urls: Vec<String> = vec![];
    let mut index = 0;
    loop {
        index += 1;
        let jpeg_data = tab.capture_screenshot(
            Page::CaptureScreenshotFormatOption::Jpeg,
            None,
            None,
            true).unwrap();
        std::fs::write(format!("screenshot{index}.jpeg"), jpeg_data).unwrap();
        let selector = &format!(".reader-view__wrap:not(.hidden)[data-p=\"{index}\"] img[src]");
        println!("finding for {selector}");
        let url = tab.wait_for_element(selector).unwrap().get_attribute_value("src").unwrap().unwrap();
        println!("{url}");
        reader_element.click().unwrap();
        urls.push(url);
        if index == pages_count {
            break;
        }
    }

    Ok(urls)
}

#[derive(Debug)]
pub struct MangaPreview {
    manga_type: String,
    name: String,
    url: String,
    slug: String,
    image_url: String,
}

pub async fn search_manga(search_input: &str) -> Result<Vec<MangaPreview>, Box<dyn Error>> {
    let web_url = &format!("https://mangalib.me/manga-list?sort=rate&dir=desc&page=1&name={search_input}");
    let browser = Browser::default().unwrap();
    let tab = browser.new_tab().unwrap();
    println!("going to {web_url}");
    tab.set_user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36", Some("en-US,en;q=0.9,hi;q=0.8,es;q=0.7,lt;q=0.6"), Some("macOS")).unwrap();
    tab.navigate_to(&web_url).unwrap().wait_until_navigated().unwrap();

    tab.wait_for_elements(".media-card")
        .unwrap()
        .iter()
        .map(|element| {
             MangaPreview {
                manga_type: element.find_element(".media-card__subtitle").unwrap().get_inner_text().unwrap(),
                name: element.find_element(".media-card__title.line-clamp").unwrap().get_inner_text().unwrap(),
                url: element.get_attribute_value("href").unwrap().unwrap(),
                slug: element.get_attribute_value("data-media-slug").unwrap().unwrap(),
                image_url: element.get_attribute_value("data-src").unwrap().unwrap(),
            }
        })
        .collect()
}