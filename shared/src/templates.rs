use tera::Tera;

/// Loads Tera HTML templates.
///
/// ## Remarks
/// Templates are located in the application's working directory at ./www/
/// Wildcards are used to load nested templates, e.g.:
///
/// - ./www/index.html
/// - ./www/labels/label.html
/// - ./www/drawing/drawing.html
pub async fn load_templates() -> anyhow::Result<Tera> {
    let template_path = format!(
        "{}/www/**/*.html",
        std::env::current_dir()?.to_string_lossy()
    );

    Ok(Tera::new(&template_path)?)
}

/// Render a Tera template to and return the resulting HTML.
///
/// ## Parameters
/// - `template_name`: Relative path to a named template loaded into Tera.
/// - `tera`: The Tera renderer where the template may be found.
/// - `context`: Data values to insert into the template's placeholders.
pub fn render(template_name: &str, tera: &Tera, context: &tera::Context) -> anyhow::Result<String> {
    Ok(tera.render(template_name, context)?)
}

/// Render a Tera template to a file.
///
/// ## Parameters
/// - `template_name`: Relative path to a named template loaded into Tera.
/// - `tera`: The Tera renderer where the template may be found.
/// - `context`: Data values to insert into the template's placeholders.
/// - `output_path`: Path to the location where the file should be saved.
pub fn render_to_file(
    template_name: &str,
    tera: &Tera,
    context: &tera::Context,
    output_path: &impl AsRef<std::path::Path>,
) -> anyhow::Result<()> {
    let file = std::fs::File::create(&output_path)?;

    Ok(tera.render_to(template_name, context, file)?)
}
