use crate::*;
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use rocket::Request;
use std::cmp::PartialOrd;

static PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\w{1,3})(?:-\w{1,3})?(?:;q=([\d\.]+))?").unwrap());

fn accept_language<'a>(req: &'a Request<'_>) -> &'a str {
    req.headers()
        .get("Accept-Language")
        .next()
        .unwrap_or("en")
}


fn lang_from_capture(capt: &Captures) -> Option<LangCode> {
    capt.iter()
        .flatten()
        .map(|m| m.as_str())
        .map(|m| m.parse())
        .map(|m| m.ok())
        .nth(1)?
}
fn quality_from_capture(capt: &Captures) -> f32 {
    capt.iter()
        .nth(3)
        .or_else(|| capt.iter().nth(2))
        .flatten()
        .map(|m| m.as_str())
        .map(|m| m.parse())
        .map(|r| r.ok())
        .flatten()
        .unwrap_or(1.0)
}

fn from_regex_capture(cap: Captures) -> Option<(LangCode, f32)> {
    let lang = lang_from_capture(&cap)?;
    let q = quality_from_capture(&cap);
    Some((lang, q))
}

pub(crate) fn languages(text: &'_ str) -> impl Iterator<Item = (LangCode, f32)> + '_ {
    PATTERN
        .captures_iter(text)
        .flat_map(from_regex_capture)
}

fn without_config_from_header(header: &str) -> Result<LangCode, Error> {
    languages(header)
        .max_by(|x, y| x.1.partial_cmp(&y.1).unwrap())
        .ok_or(Error::BadRequest)
        .map(|x| x.0)
}

pub(crate) fn without_config(req: &Request<'_>) -> Result<LangCode, Error> {
    let header = accept_language(req);
    without_config_from_header(header)
}

struct Decider<'a> {
    lang: Option<LangCode>,
    q: Option<f32>,
    config: &'a Config,
}

impl<'a> Decider<'a> {
    fn new(config: &'a Config) -> Self {
        Self {
            lang: None,
            q: None,
            config,
        }
    }
    fn is_none(&self) -> bool {
        self.lang.is_none()
    }
    fn compare(&mut self, lang2: LangCode, qclient2: f32) {
        let lang1 = self.lang.unwrap();
        let qclient1 = self.q.unwrap();
        let qserver1 = self.config[lang1];
        let qserver2 = self.config[lang2];
        if (qserver1 - qserver2) / qserver1 < (qclient2 - qclient1) / qclient2 {
            self.lang = Some(lang2);
            self.q = Some(qclient2)
        }
    }
    fn add_preference(&mut self, lang: LangCode, q: f32) {
        if self.config[lang] == 0.0 || self.config[lang].is_nan() {
            return;
        }
        if self.is_none() {
            self.lang = Some(lang);
            self.q = Some(q);
        } else {
            self.compare(lang, q);
        }
    }
    fn result(&self) -> Result<LangCode, Error> {
        self.lang
            .ok_or(Error::NotAcceptable)
    }
}

// #[throws(Error)]
pub(crate) fn with_config(req: &Request, config: &Config) -> Result<LangCode, Error> {
    let header = accept_language(req);
    let mut decider = Decider::new(config);
    for (lang, q) in languages(header) {
        decider.add_preference(lang, q);
    }
    decider.result()
}

#[test]
fn test_lang_parsing() -> Result<(), Error> {
    assert_eq!(Da, without_config_from_header("da, en-GB;q=0.8, en;q=0.7")?);
    assert_eq!(
        En,
        without_config_from_header("en-US,en;q=0.8,es;q=0.5,es-ES;q=0.3")?
    );
    assert_eq!(En, without_config_from_header("en-US,en;q=0.9")?);
    assert!(if let Err::<LangCode, _>(Error::BadRequest) =
        without_config_from_header("invalid accept language header")
    {
        true
    } else {
        false
    });
    Ok(())
}
