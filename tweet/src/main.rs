use egg_mode::media::{get_status, media_types, upload_media, ProgressInfo};
use egg_mode::tweet::DraftTweet;
use std::fs;
use std::io::{Read, Write};
use std::time::Duration;
use tokio::time::delay_for;


//This is not an example that can be built with cargo! This is some helper code for the other
//examples so they can load access keys from the same place.

pub struct Config {
    pub token: egg_mode::Token,
    pub user_id: u64,
    pub screen_name: String,
}

impl Config {
    pub async fn load() -> Self {
        let a1 = Config::load_inner().await;
        if let Some(conf) = a1 {
            return conf;
        }

        Config::load_inner().await.unwrap()
    }

    /// This needs to be a separate function so we can retry after creating the
    /// twitter_settings file. Idealy we would recurse, but that requires boxing
    /// the output which doesn't seem worthwhile
    async fn load_inner() -> Option<Self> {
        //IMPORTANT: make an app for yourself at apps.twitter.com and get your
        //key/secret into these files; these examples won't work without them
        /*let consumer_key = include_str!("consumer_key").trim();
        let consumer_secret = include_str!("consumer_secret").trim();*/
        let consumer_key = std::env::var("CONSUMER_KEY").unwrap();
        let consumer_secret = std::env::var("CONSUMER_SECRET").unwrap();

        let con_token = egg_mode::KeyPair::new(consumer_key, consumer_secret);

        let mut config = String::new();
        let user_id: u64;
        let username: String;
        let token: egg_mode::Token;

        //look at all this unwrapping! who told you it was my birthday?
        if let Ok(mut f) = std::fs::File::open("twitter_settings") {
            f.read_to_string(&mut config).unwrap();

            let mut iter = config.split('\n');

            username = iter.next().unwrap().to_string();
            user_id = u64::from_str_radix(&iter.next().unwrap(), 10).unwrap();
            let access_token = egg_mode::KeyPair::new(
                iter.next().unwrap().to_string(),
                iter.next().unwrap().to_string(),
            );
            token = egg_mode::Token::Access {
                consumer: con_token,
                access: access_token,
            };

            if let Err(err) = egg_mode::auth::verify_tokens(&token).await {
                println!("We've hit an error using your old tokens: {:?}", err);
                println!("We'll have to reauthenticate before continuing.");
                std::fs::remove_file("twitter_settings").unwrap();
            } else {
                println!("Welcome back, {}!\n", username);
            }
        } else {
            let request_token = egg_mode::auth::request_token(&con_token, "oob").await.unwrap();

            println!("Go to the following URL, sign in, and give me the PIN that comes back:");
            println!("{}", egg_mode::auth::authorize_url(&request_token));

            let mut pin = String::new();
            std::io::stdin().read_line(&mut pin).unwrap();
            println!("");

            let tok_result = egg_mode::auth::access_token(con_token, &request_token, pin)
                .await
                .unwrap();

            token = tok_result.0;
            user_id = tok_result.1;
            username = tok_result.2;

            match token {
                egg_mode::Token::Access {
                    access: ref access_token,
                    ..
                } => {
                    config.push_str(&username);
                    config.push('\n');
                    config.push_str(&format!("{}", user_id));
                    config.push('\n');
                    config.push_str(&access_token.key);
                    config.push('\n');
                    config.push_str(&access_token.secret);
                }
                _ => unreachable!(),
            }

            let mut f = std::fs::File::create("twitter_settings").unwrap();
            f.write_all(config.as_bytes()).unwrap();

            println!("Welcome, {}, let's get this show on the road!", username);
        }

        //TODO: Is there a better way to query whether a file exists?
        if std::fs::metadata("twitter_settings").is_ok() {
            Some(Config {
                token: token,
                user_id: user_id,
                screen_name: username,
            })
        } else {
            None
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load().await;
    
    let image_name = std::env::args().nth(1).unwrap();

    let data_name = std::env::args().nth(1).unwrap();
    let data = fs::read_to_string(&data_name).unwrap();
    let v: serde_json::Value = serde_json::from_str(&data).unwrap();
    let title = v.get("title").unwrap().as_str().unwrap();
    let author = v.get("author").unwrap().as_str().unwrap();
    let date = v.get("date").unwrap().as_str().unwrap();
    let url = v.get("url").unwrap().as_str().unwrap();

    let tweet = format!("{}\n{} by {} ({})", url, title, author, date);
    let mut tweet = DraftTweet::new(tweet);
    let typ = media_types::image_png();
    let bytes = std::fs::read(image_name)?;
    let handle = upload_media(&bytes, &typ, &config.token).await?;
    tweet.add_media(handle.id.clone());

    for ct in 0..=60u32 {
        match get_status(handle.id.clone(), &config.token).await?.progress {
            None | Some(ProgressInfo::Success) => {
                //println!("\nMedia sucessfully processed");
                break;
            }
            Some(ProgressInfo::Pending(_)) | Some(ProgressInfo::InProgress(_)) => {
                //print!(".");
                //stdout().flush()?;
                delay_for(Duration::from_secs(1)).await;
            }
            Some(ProgressInfo::Failed(err)) => Err(err)?,
        }
        if ct == 60 {
            Err("Error: timeout")?
        }
    }

    tweet.send(&config.token).await?;
    Ok(())
}
