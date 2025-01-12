# Rocket Lang
### [This is still unstable, don't use still]
Rocket lang provides a configurable enum type for multi-language rocket applications. 

# LangCode
A request guard corresponding to the [ISO 639-1](https://es.wikipedia.org/wiki/ISO_639-1) code standard. 
Usage example: 
```rust
#[get("/some-path/")]
fn some_path(lang: LangCode) -> Template {
    // we can now choose which template to display
    // based of the user's language preference
    let path = format!("home/{}", LangCode); 
    Template::render(path, json!({}))
}
```

# Config 
The behavior of the enum can be configured with the `Config` structure, which can be attached to a rocket instance. 
When this is not used, its default behavior is to retrieve the language code from the `Accept-Language` header.

## accept_language
If the preferred method for language resolution is the http accept-language header, the qualities for each language can be set like this:
```rust
let config = Config::new(); 
let config[Es] = 1.0; 
let config[En] = 0.5;
```

# url
The guard can also be configured to extract the language code from a fixed position in the path: 
```rust
/// takes the language code from the last path segment:
let config = Config::new().url(-1); 
```

This way the language code can be retrieved from a positional url segment. 
This also allows other request guards to consume the structure in their API. Most notably, it can be used by foreign structures to return error messages in multiple languages.
```rust
#[get("see-lang/<_>")]
fn see_lang(lang: LangCode) -> &'static str {
    lang.as_str()
}

// here the error message displayed by 
// `Error` will automatically suit the callers configuration. 
#[get("custom-error-message/")]
fn custom_error_message(user: User) -> Error {
    Error::Unauthorized
}

// A possible implementation of `Error`
impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'o> {
        let lang: LangCode = request
            .try_into()
            .map_err(|x: Error| x.status())?;
        let msg = match lang {
            LangCode::En => "Unauthorized",
            LangCode::Es => "No autorizado",
            ...
        }; 
        msg.respond_to(request)
    }
}
```
# custom
If none of the previous approaches suit your needs, you may also use a closure to create a language code from a request: 
```rust
let config = Config::custom(|req: &Request|{
    let lang = from_url(req)?;
    Ok(lang) 
}); 
```




