#![feature(result_map_or_else)]
extern crate toml;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
use std::{thread, time};
use tokio_core::reactor::Core;

use std::fs;
use std::io::{BufReader, Read};

#[derive(Debug, Deserialize)]
struct Config {
    tk: String,
    ts: String,
    cs: String,
    ck: String,
}

fn read_config() -> Result<Config, failure::Error> {
    let mut f = BufReader::new(fs::File::open("config.toml")?);
    let mut buf = vec![];

    f.read_to_end(&mut buf)?;

    let s = std::str::from_utf8(&buf)?;
    Ok(toml::from_str(s)?)
}

fn main() {
    let config = read_config().unwrap();
    let token = egg_mode::Token::Access {
        consumer: egg_mode::KeyPair::new(config.ck, config.cs),
        access: egg_mode::KeyPair::new(config.tk, config.ts),
    };

    loop {
        main_loop(&token);
        thread::sleep(time::Duration::from_secs(10));
    }
}

fn main_loop(token: &egg_mode::Token) {
    search(&token).map_or_else(
        |e| println!("{:?}", e),
        |ts| {
            for t in ts {
                retweet(&token, &t).map_or_else(|e| println!("{:?}", e), |_| ());
            }
        },
    );
}

fn search(token: &egg_mode::Token) -> Result<Vec<egg_mode::tweet::Tweet>, failure::Error> {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let res = core.run(
        egg_mode::search::search("デート 今日 彼 OR 彼女 OR 彼氏 OR 恋人")
            .result_type(egg_mode::search::ResultType::Recent)
            .count(100)
            .call(&token, &handle),
    )?;
    Ok(res
        .statuses
        .clone()
        .into_iter()
        .filter(tweet_filter)
        .collect())
}

fn retweet(token: &egg_mode::Token, t: &egg_mode::tweet::Tweet) -> Result<(), failure::Error> {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    core.run(egg_mode::tweet::retweet(t.id, &token, &handle))?;
    Ok(())
}

fn tweet_filter(t: &egg_mode::tweet::Tweet) -> bool {
    t.source.name == "Twitter for iPhone"
        && t.current_user_retweet.is_none()
        // && !t.text.contains("http")
        && !t.text.contains("#")
        && !t.text.contains("RT")
}
