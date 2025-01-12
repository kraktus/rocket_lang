


use rocket::http::Header;
use rocket::local::asynchronous::Client;

use LangCode::Es;
#[macro_use]
extern crate rocket;

use rocket_lang::{Config, Error, LangCode};

async fn test_config(url: &str, lang: &str, config: Config) {
    let rocket = rocket::build()
        .mount("/", routes![index])
        .attach(config);
    let body = Client::tracked(rocket)
        .await
        .unwrap()
        .get(url)
        .dispatch()
        .await
        .into_string()
        .await
        .unwrap();
    assert!(body == lang);
}

#[get("/<_>/<_>/<_>")]
fn index(lang: LangCode) -> &'static str {
    lang.as_str()
}

#[tokio::test]
async fn url_minus_one() {
    let config = Config::new().url(-1);
    test_config("/index/path/es", "es", config).await;
}

#[tokio::test]
async fn negative_url() {
    let config = Config::new().url(-2);
    test_config("/index/fr/segment", "fr", config).await;
}

#[tokio::test]
async fn positive_url() {
    let config = Config::new().url(0);
    test_config("/de/some/path", "de", config).await;

    let config = Config::new().url(1);
    test_config("/some/pt/path", "pt", config).await;
}

#[tokio::test]
async fn wildcard() {
    let config = Config::new().wildcard(Es);
    test_config("/some/other/path", "es", config).await;
}
#[tokio::test]
async fn test_custom() {
    let config = Config::new().custom(|_req| Ok(LangCode::Om));
    test_config("/some/other/path", "om", config).await;
}

#[tokio::test]
async fn test_failed_custom() {
    let config = Config::new()
        .custom(|_req| Err(Error::NotAcceptable))
        .url(-1);
    test_config("/some/other/es", "es", config).await;
}

#[tokio::test]
async fn accept_header1() {
    let mut config = Config::new();
    config[LangCode::En] = 0.5;
    config[LangCode::De] = 0.5;
    config[LangCode::Es] = 1.0;
    let rocket = rocket::build()
        .mount("/", routes![index])
        .attach(config);
    let client = Client::tracked(rocket)
        .await
        .unwrap();

    let mut req = client.get("/some/path/index.html");
    req.add_header(Header::new("accept-language", "en-US, de;q=0.2"));
    let res = req
        .dispatch()
        .await
        .into_string()
        .await
        .unwrap();
    assert_eq!(res, "en");

    let mut req = client.get("/some/path/index.html");
    req.add_header(Header::new("accept-language", "de, es;q=0.5"));
    let res = req
        .dispatch()
        .await
        .into_string()
        .await
        .unwrap();
    assert_eq!(res, "de");
    let mut req = client.get("/some/path/index.html");
    req.add_header(Header::new("accept-language", "de, es;q=0.6"));
    let res = req
        .dispatch()
        .await
        .into_string()
        .await
        .unwrap();
    assert_eq!(res, "es");
    let mut req = client.get("/some/path/index.html");
    req.add_header(Header::new("accept-language", "de, es;q=0.4"));
    let res = req
        .dispatch()
        .await
        .into_string()
        .await
        .unwrap();
    assert_eq!(res, "de");
}
