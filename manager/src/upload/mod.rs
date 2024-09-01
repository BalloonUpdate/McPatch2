use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::AppContext;

pub fn generate_upload_script(template_ctx: TemplateContext, app_ctx: &AppContext, version_label: &str) -> bool {
    let template_file = app_ctx.working_dir.join(&app_ctx.config.upload_script_template);

    let datetime = chrono::Local::now().format("%Y_%m_%d-%H_%M_%S").to_string();
    let output_file = &app_ctx.config.upload_script_output.replace("{datetime}", &datetime).replace("{label}", version_label);
    let output_file = app_ctx.working_dir.join(output_file);

    let template = match std::fs::read_to_string(template_file) {
        Ok(text) => text,
        Err(_) => return false,
    };

    let mut tt = TinyTemplate::new();
    tt.add_template("upload script", &template).unwrap();

    let rendered = tt.render("upload script", &template_ctx).unwrap();

    std::fs::write(output_file, rendered).unwrap();

    return true;
}

#[derive(Serialize)]
pub struct TemplateContext {
    pub upload_files: Vec<String>,
    pub delete_files: Vec<String>,
}