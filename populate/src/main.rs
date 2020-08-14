use kuchiki::traits::*;
use random_number::random;
use std::fs;
use std::io::Read;
use std::path::Path;

fn main() {
    let zip_name = std::env::args().nth(1).unwrap();

    //TODO: figure out total pages
    //let resp = ureq::get("https://www.digitalmzx.com/search.php?browse=ALL")

    let body = loop {
        let max_pages = 37;
        let page = random!(0..=max_pages);
        let page_url = format!("https://www.digitalmzx.com/search.php?browse=ALL&page={}", page);
        println!("Fetching {}", page_url);
        let resp = ureq::get(&page_url)
            .timeout_connect(5000)
            .call();
        assert_eq!(resp.status(), 200);
        let body = resp.into_string().unwrap();
        let document = kuchiki::parse_html().one(&*body);
        let games: Vec<_> = document
            .select("td > a")
            .unwrap()
            .filter_map(|link| link.attributes.borrow().get("href").map(|a| a.to_owned()))
            .filter(|href| href.contains("show.php?"))
            .collect();
        let choice = random!(0..games.len());

        let game_url = format!("https://www.digitalmzx.com/{}", games[choice]);
        println!("Fetching {}", game_url);
        let resp = ureq::get(&game_url)
            .timeout_connect(5000)
            .call();
        assert_eq!(resp.status(), 200);
        let body = resp.into_string().unwrap();

        if body.contains("<td>Game</td>") ||
            body.contains("<td>Short Competition</td>") ||
            body.contains("<td>Competition</td>") ||
            body.contains("<td>Demo/Unfinished</td>") ||
            body.contains("<td>Engine/Resource</td>")
        {
            break body;
        }
    };

    let document = kuchiki::parse_html().one(&*body);
    let download = document
        .select("#downloadList a")
        .unwrap()
        .filter_map(|link| link.attributes.borrow().get("href").map(|a| a.to_owned()))
        .next()
        .unwrap();

    let download_url = format!("https://www.digitalmzx.com/{}", download);
    println!("Fetching {}", download_url);
    let resp = ureq::get(&download_url)
        .timeout_connect(5000)
        .timeout(std::time::Duration::from_secs(60))
        .call();
    assert_eq!(resp.status(), 200);
    let mut reader = resp.into_reader();
    let mut bytes = vec![];
    reader.read_to_end(&mut bytes).unwrap();

    let dir_name = "unzipped";
    let _ = fs::remove_dir_all(dir_name);
    fs::write(Path::new(&zip_name), bytes).unwrap();
    fs::DirBuilder::new()
        .create(dir_name)
        .unwrap();
    assert_eq!(unzip_rs::unzip(&zip_name, dir_name), 0);
}
