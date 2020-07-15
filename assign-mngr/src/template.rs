//! Global static instance of the template engine [tera](https://github.com/Keats/tera).
use tera::Tera;

lazy_static! {
    /// All templates lazy loaded.
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/*.html") {
            Ok(t) => t,
            Err(e) => {
                log::error!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec!["html", ".sql"]);
        tera
    };
}
