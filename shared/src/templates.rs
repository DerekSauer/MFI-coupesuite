use tera::Tera;

/// Loads Tera HTML temples.
/// Templates are located in the application's working directory at ./www/
/// Wildcards are used to load nested templates, e.g.:
///
/// ./www/index.html
/// ./www/labels/label.html
/// ./www/drawing/drawing.html
pub async fn load_templates() -> anyhow::Result<Tera> {
    let template_path = format!(
        "{}/www/**/*.html",
        std::env::current_dir()?.to_string_lossy()
    );

    Ok(Tera::new(&template_path)?)
}
