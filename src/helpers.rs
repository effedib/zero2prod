use actix_web::{HttpResponse, http::header::LOCATION};
use tera::{Context, Tera};

pub fn init_tera(template_glob: &str) -> Tera {
    let mut tera = Tera::default();

    tera.load_from_glob(template_glob)
        .expect("template folder not found");

    tera
}

pub fn render_html(
    tera: &Tera,
    args: &[(&str, &str)],
    template_name: String,
) -> Result<String, tera::Error> {
    let mut context = Context::new();

    for (key, val) in args {
        context.insert(key.to_string(), val);
    }

    tera.render(&template_name, &context)
}

pub fn error_chain_fmt(
    e: impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

pub fn see_other(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        .finish()
}

pub fn e500<T>(e: T) -> actix_web::Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::ErrorInternalServerError(e)
}

pub fn e400<T>(e: T) -> actix_web::Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::ErrorBadRequest(e)
}
