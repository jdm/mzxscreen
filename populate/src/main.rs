use kuchiki::traits::*;
use random_number::random;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::Command;

fn main() {
    let zip_name = std::env::args().nth(1).unwrap();
    let json_name = std::env::args().nth(2).unwrap();

    //TODO: figure out total pages
    //let resp = ureq::get("https://www.digitalmzx.com/search.php?browse=ALL")

    let (body, game_url) = loop {
        let max_pages = 36;
        let page = random!(0..=max_pages);
        let page_url = format!(
            "https://www.digitalmzx.com/search.php?browse=ALL&page={}",
            page
        );
        println!("Fetching {}", page_url);
        let resp = ureq::get(&page_url).timeout_connect(5000).call();
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
        let resp = ureq::get(&game_url).timeout_connect(5000).call();
        assert_eq!(resp.status(), 200);
        let body = resp.into_string().unwrap();

        if body.contains("<td>Game</td>")
            || body.contains("<td>Short Competition</td>")
            || body.contains("<td>Competition</td>")
            || body.contains("<td>Demo/Unfinished</td>")
            || body.contains("<td>Engine/Resource</td>")
        {
            break (body, game_url);
        }
    };

    let document = kuchiki::parse_html().one(&*body);

    let mut info = document.select("#showcase table tbody tr td").unwrap();
    let mut name = info.next().unwrap().text_contents();

    let lower_name = name.to_lowercase();
    for special in &[", a", ", an", ", the"] {
        if lower_name.ends_with(special) {
            let end = name.len() - special.len();
            let suffix = &name[end..];
            name = format!("{} {}", suffix.split(" ").nth(1).unwrap(), &name[..end]);
            break;
        }
    }

    let author = info.next().unwrap();
    let author = author.as_node().first_child().unwrap().text_contents();
    let _category = info.next();
    let release = info.next().unwrap().text_contents();

    let json = serde_json::json!({
        "title": name,
        "author": author,
        "date": release.split('-').next().unwrap(),
        "url": game_url,
    });

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
    fs::remove_dir_all(dir_name).unwrap();
    fs::write(Path::new(&zip_name), bytes).unwrap();
    fs::write(
        Path::new(&json_name),
        &serde_json::to_string(&json).unwrap(),
    )
    .unwrap();
    fs::DirBuilder::new().create(dir_name).unwrap();
    let mut f = fs::File::open(&zip_name).expect("no file found");
    let mut bytes = [0; 10];
    f.read(&mut bytes).expect("Couldn't read magic bytes");
    if infer::archive::is_zip(&bytes) {
        let status = Command::new("unzip")
            .args(&[&zip_name, "-d", dir_name])
            .status()
            .unwrap();
        assert!(status.success());
    } else if infer::archive::is_rar(&bytes) {
        let status = Command::new("unrar")
            .args(&["x", &zip_name, dir_name])
            .status()
            .unwrap();
        assert!(status.success());
    } else {
        eprintln!("Couldn't identify archive type.");
    }
}
