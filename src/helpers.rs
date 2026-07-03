use tera::{Context, Tera};

pub fn init_tera(template_glob: &str) -> Tera {
    let mut tera = Tera::default();

    tera.load_from_glob(template_glob)
        .expect("template folder not found");

    tera
}

pub fn render_confirmation_email(
    tera: &Tera,
    confirmation_link: &str,
) -> Result<String, tera::Error> {
    let mut context = Context::new();
    context.insert("confirmation_link", confirmation_link);

    tera.render("confirmation.html", &context)
}
