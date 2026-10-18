//! SDK-owned, closed Text/4 workbench projection over the existing byte adapter.

use crate::error::PrismError;
use crate::holo::canonical::encode_value;
use crate::holo::research_workbench::WorkbenchApplication;
use prod_codegen::{
    generate_holoview_bundle, generate_text_view_v4_for_module, GeneratedViewV1, PackageFile,
    TextBrowserAdapterBinding, TextViewV1,
};
use serde_json::json;
use sha2::{Digest, Sha256};

fn html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
fn javascript(value: &serde_json::Value) -> Result<String, PrismError> {
    Ok(serde_json::to_string(value)
        .map_err(|error| PrismError::new("PP5203", error.to_string()))?
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029"))
}
fn records(files: &[PackageFile]) -> Vec<serde_json::Value> {
    files
        .iter()
        .map(|file| json!({"path":file.path,"sha256":format!("{:x}",Sha256::digest(&file.bytes))}))
        .collect()
}

pub(crate) fn generate(
    module: &prod_ir::Module,
    app: &WorkbenchApplication,
    model_id: &str,
    view_id: &str,
    core_sha: &str,
    entry: &str,
) -> Result<GeneratedViewV1, PrismError> {
    crate::holo::research_workbench::validate(app)?;
    // The old adapter remains authoritative for UTF-8 byte copying, checked
    // guest failures, exact limits and generated package dependencies.
    let evaluated = TextViewV1 {
        title: app.view.title.clone(),
        heading: app.view.heading.clone(),
        input_label: app.view.raw.label.clone(),
        submit_label: app.view.raw.submit_label.clone(),
        output_label: app.view.raw.response_label.clone(),
        input_error: app.view.statuses.input_error.clone(),
        response_error: app.view.statuses.response_error.clone(),
        max_input_bytes: app.request_maximum,
        max_output_bytes: app.response_maximum,
        model_id: model_id.to_owned(),
        view_model_id: view_id.to_owned(),
        generated_core_sha256: core_sha.to_owned(),
    };
    let mut generated = generate_text_view_v4_for_module(
        module,
        &evaluated,
        &TextBrowserAdapterBinding {
            package_name: format!("{}-browser", app.cargo_name),
            package_version: app.cargo_version.clone(),
            core_crate_name: app.cargo_name.clone(),
            core_crate_version: app.cargo_version.clone(),
            core_function: entry.to_owned(),
        },
    )
    .map_err(|error| PrismError::new("PP5203", error.to_string()))?;
    let raw = &app.view.raw;
    // Disabled source-owned controls and form-action:none protect drafts even
    // before module execution. Mount transfers the exact pre-init raw draft.
    let index = format!(
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><link rel=\"icon\" href=\"data:,\"><meta http-equiv=\"Content-Security-Policy\" content=\"form-action 'none'\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{}</title><link rel=\"stylesheet\" href=\"app.css?model_id={}\"></head><body><main id=\"workbench-main\" tabindex=\"-1\"><h1>{}</h1><p id=\"status\" role=\"status\" aria-live=\"polite\" aria-atomic=\"true\">{}</p><details id=\"raw-editor\"{}><summary>{}</summary><form id=\"application-form\" novalidate><label for=\"request\">{}</label><textarea id=\"request\" rows=\"5\">{}</textarea><button id=\"submit\" type=\"submit\" disabled>{}</button></form></details><pre id=\"result\"></pre></main><script type=\"module\" src=\"app.js?model_id={}\"></script></body></html>\n",
        html(&app.view.title),
        model_id,
        html(&app.view.heading),
        html(&app.view.statuses.initializing),
        if raw.expanded { " open" } else { "" },
        html(&raw.label),
        html(&app.view.labels.request),
        html(&raw.initial_value),
        html(&raw.submit_label),
        model_id
    );
    let guidance = app
        .view
        .guidance
        .iter()
        .map(|step| format!("<li>{}</li>", html(step)))
        .collect::<String>();
    let mut documentation = String::new();
    for section in &app.view.sections {
        documentation.push_str(&format!(
            "<section id=\"guide-{}\"><h3>{}</h3>",
            section.id,
            html(&section.title)
        ));
        for paragraph in &section.paragraphs {
            documentation.push_str(&format!("<p>{}</p>", html(paragraph)));
        }
        documentation.push_str("</section>");
    }
    let links = app
        .view
        .links
        .iter()
        .map(|link| {
            format!(
                "<a href=\"{}\" rel=\"noreferrer\">{}</a>",
                html(&link.https_url),
                html(&link.label)
            )
        })
        .collect::<String>();
    let index=index.replace("<meta name=\"viewport\"",&format!("<meta name=\"description\" content=\"{}\"><link rel=\"canonical\" href=\"{}\"><meta name=\"viewport\"",html(&app.view.purpose),html(&app.cargo_homepage)))
        .replace("<p id=\"status\"",&format!("<p>{}</p><p>{}</p><ol class=\"guidance\">{}</ol><noscript><p>{}</p></noscript><p id=\"status\"",html(&app.view.purpose),html(&app.view.audience),guidance,html(&app.view.statuses.script_required)))
        .replace("</main>",&format!("<section id=\"guide\"><h2>{}</h2>{}</section><footer id=\"links\"><h2>{}</h2>{}</footer></main>",html(&app.view.labels.guide),documentation,html(&app.view.labels.links),links));
    let view = javascript(
        &serde_json::to_value(&app.view)
            .map_err(|error| PrismError::new("PP5203", error.to_string()))?,
    )?;
    let binding = javascript(&json!(format!("./{}.js", app.cargo_name.replace('-', "_"))))?;
    let assets = |transport: &str| -> Vec<PackageFile> {
        let source = include_str!("research_workbench/runtime.js")
            .replace("__MODEL_ID__", &format!("{model_id:?}"))
            .replace("__INPUT_LIMIT__", &app.request_maximum.to_string())
            .replace("__OUTPUT_LIMIT__", &app.response_maximum.to_string())
            .replace("__BINDING__", &binding)
            .replace("__TRANSPORT__", &format!("{transport:?}"))
            // Source strings are substituted last and never scanned as template syntax.
            .replace("__VIEW__", &view);
        vec![
            PackageFile {
                path: "app.css".into(),
                bytes: include_bytes!("research_workbench/style.css").to_vec(),
            },
            PackageFile {
                path: "app.js".into(),
                bytes: source.into_bytes(),
            },
            PackageFile {
                path: "index.html".into(),
                bytes: index.as_bytes().to_vec(),
            },
        ]
    };
    generated.browser_assets = assets("browser");
    generated.hologram_assets = assets("hologram");
    generated.hologram_bundle = generate_holoview_bundle(&generated.hologram_assets)
        .map_err(|error| PrismError::new("PP5203", error.to_string()))?;
    let manifest = json!({"schema":"prismpm/workbench-view-projection/2","profile":"prism.workbench-view/2","model_id":model_id,"view_model_id":view_id,"generated_core_sha256":core_sha,"max_input_bytes":app.request_maximum,"max_output_bytes":app.response_maximum,"browser_adapter":records(&generated.browser_adapter.files),"hologram_bundle":{"path":generated.hologram_bundle.path,"sha256":format!("{:x}",Sha256::digest(&generated.hologram_bundle.bytes))},"projections":[{"target":"browser-wasm-bindgen","files":records(&generated.browser_assets)},{"target":"hologram-intent-v2","files":records(&generated.hologram_assets)}]});
    crate::contracts::CanonicalDocument::from_value(
        "prismpm/workbench-view-projection/2",
        manifest.clone(),
    )?;
    generated.view_manifest = PackageFile {
        path: "view-manifest.json".into(),
        bytes: encode_value(&manifest)?,
    };
    Ok(generated)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn research_workbench_generator_keys_each_shell_asset_to_its_actual_model() {
        let model: crate::holo::model_document::ModelDocument = serde_json::from_slice(
            include_bytes!("../../../tests/data/research-workbench-model-document.json"),
        )
        .unwrap();
        let Some(crate::holo::model_document::Application::Research(mut app)) = model.application
        else {
            panic!("Text/4 fixture")
        };
        app.view.title = "Literal __MODEL_ID__ __TRANSPORT__ <source>".into();
        let (_, module) = prod_ir::parser::parse_module(
            "(module Fixture (def dispatch ((input Bytes)) Bytes input))",
        )
        .unwrap();
        let identity = "ab".repeat(32);
        let output = generate(
            &module,
            &app,
            &identity,
            &"cd".repeat(32),
            &"ef".repeat(32),
            "dispatch",
        )
        .unwrap();
        for assets in [&output.browser_assets, &output.hologram_assets] {
            let html = String::from_utf8_lossy(
                &assets
                    .iter()
                    .find(|file| file.path == "index.html")
                    .unwrap()
                    .bytes,
            );
            assert!(html.contains(&format!("src=\"app.js?model_id={identity}\"")));
            assert!(html.contains(&format!("href=\"app.css?model_id={identity}\"")));
            assert!(html.contains("Literal __MODEL_ID__ __TRANSPORT__ &lt;source&gt;"));
            assert!(html.contains("<link rel=\"icon\" href=\"data:,\">"));
            let script = String::from_utf8_lossy(
                &assets
                    .iter()
                    .find(|file| file.path == "app.js")
                    .unwrap()
                    .bytes,
            );
            assert!(script.contains("Literal __MODEL_ID__ __TRANSPORT__ \\u003csource\\u003e"));
        }
        let manifest: serde_json::Value =
            serde_json::from_slice(&output.view_manifest.bytes).unwrap();
        assert_eq!(manifest["schema"], "prismpm/workbench-view-projection/2");
        assert_eq!(manifest["projections"][1]["target"], "hologram-intent-v2");
        assert_eq!(manifest["model_id"], identity);
    }
}
