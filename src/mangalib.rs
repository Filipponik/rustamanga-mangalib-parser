use std::error::Error;

use headless_chrome::Browser;
use headless_chrome::protocol::cdp::Page;

pub async fn get_manga_chapter_images(slug: &str, volume: i32, chapter: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let browser = Browser::default().unwrap();
    let tab = browser.new_tab().unwrap();
    let web_url = format!("https://mangalib.me/{slug}/v{volume}/c{chapter}?page=1");
    println!("{web_url}");
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