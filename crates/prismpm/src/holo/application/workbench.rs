//! Closed Text/3 metadata projection; no executable application behavior is interpreted here.
use super::*;
use crate::holo::workbench::{WorkbenchApplication, WorkbenchView, PROFILE};

fn source_shape(name: &str) -> Option<&'static [(&'static str, &'static str)]> {
    match name {
        "Choice" => Some(&[("value", "String"), ("label", "String")]),
        "Field" => Some(&[
            ("id", "String"),
            ("label", "String"),
            ("kind", "FieldKind"),
            ("help", "String"),
            ("placeholder", "String"),
            ("initialValue", "String"),
            ("required", "Bool"),
            ("choices", "List Choice"),
        ]),
        "Example" => Some(&[
            ("id", "String"),
            ("label", "String"),
            ("description", "String"),
            ("values", "List String"),
            ("expectedResponse", "String"),
            ("explanation", "String"),
        ]),
        "Operation" => Some(&[
            ("id", "String"),
            ("label", "String"),
            ("summary", "String"),
            ("help", "String"),
            ("requestPrefix", "String"),
            ("fields", "List Field"),
            ("examples", "List Example"),
        ]),
        "Section" => Some(&[
            ("id", "String"),
            ("title", "String"),
            ("paragraphs", "List String"),
        ]),
        "Link" => Some(&[("label", "String"), ("httpsUrl", "String")]),
        "ErrorHint" => Some(&[
            ("code", "String"),
            ("title", "String"),
            ("explanation", "String"),
            ("recovery", "String"),
            ("fields", "List String"),
        ]),
        "StatusCopy" => Some(&[
            ("initializing", "String"),
            ("ready", "String"),
            ("running", "String"),
            ("cancelling", "String"),
            ("cancelled", "String"),
            ("timeout", "String"),
            ("loadError", "String"),
            ("computeError", "String"),
            ("inputError", "String"),
            ("responseError", "String"),
            ("scriptRequired", "String"),
        ]),
        "Design" => Some(&[
            ("palette", "Palette"),
            ("density", "Density"),
            ("width", "Width"),
            ("colorScheme", "ColorScheme"),
        ]),
        "RawEditor" => Some(&[
            ("requestPrefix", "String"),
            ("label", "String"),
            ("help", "String"),
            ("placeholder", "String"),
            ("initialValue", "String"),
            ("submitLabel", "String"),
            ("responseLabel", "String"),
            ("expanded", "Bool"),
        ]),
        "DisplaySpec" => Some(&[
            ("title", "String"),
            ("emptyText", "String"),
            ("rawLabel", "String"),
            ("maximumCells", "UInt32"),
            ("maximumDetails", "UInt32"),
        ]),
        "Execution" => Some(&[
            ("timeoutMilliseconds", "UInt32"),
            ("cancelLabel", "String"),
            ("retryLabel", "String"),
            ("wasmCachePolicy", "WasmCachePolicy"),
        ]),
        "WorkbenchView" => Some(&[
            ("title", "String"),
            ("heading", "String"),
            ("purpose", "String"),
            ("audience", "String"),
            ("guidance", "List String"),
            ("initialOperation", "String"),
            ("initialExample", "String"),
            ("operations", "List Operation"),
            ("sections", "List Section"),
            ("links", "List Link"),
            ("errors", "List ErrorHint"),
            ("statuses", "StatusCopy"),
            ("design", "Design"),
            ("raw", "RawEditor"),
            ("response", "DisplaySpec"),
            ("execution", "Execution"),
            ("labels", "WorkbenchLabels"),
        ]),
        "WorkbenchLabels" => Some(&[
            ("navigation", "String"),
            ("operation", "String"),
            ("examples", "String"),
            ("loadExample", "String"),
            ("compute", "String"),
            ("guide", "String"),
            ("links", "String"),
            ("fields", "String"),
            ("expectedResponse", "String"),
            ("details", "String"),
            ("resultTable", "String"),
            ("row", "String"),
            ("column", "String"),
            ("request", "String"),
            ("reset", "String"),
            ("skipToWorkbench", "String"),
        ]),
        _ => None,
    }
}
fn enum_values(name: &str) -> Option<&'static [&'static str]> {
    match name {
        "FieldKind" => Some(&["Text", "Multiline", "Natural", "Choice"]),
        "Palette" => Some(&["Indigo", "Ocean"]),
        "Density" => Some(&["Comfortable", "Compact"]),
        "Width" => Some(&["Balanced", "Wide"]),
        "ColorScheme" => Some(&["System", "Light", "Dark"]),
        "WasmCachePolicy" => Some(&["NoStore"]),
        _ => None,
    }
}
fn snake(value: &str) -> String {
    let mut result = String::new();
    for ch in value.chars() {
        if ch.is_ascii_uppercase() {
            result.push('_');
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }
    result
}
fn invalid(message: &str) -> PrismError {
    PrismError::new("PP2009", message)
}
fn metadata(
    definitions: &Definitions<'_>,
    module: &str,
    value: &Value,
    kind: &str,
    remaining: &mut usize,
    depth: usize,
) -> Result<Value, PrismError> {
    *remaining = remaining
        .checked_sub(1)
        .ok_or_else(|| invalid("workbench metadata exceeds its node budget"))?;
    if depth > 64 {
        return Err(invalid("workbench metadata exceeds its nesting bound"));
    }
    let (module, value) = evaluated(definitions, module, value)?;
    if let Some(element) = kind.strip_prefix("List ") {
        let mut rows = Vec::new();
        let mut owner = module;
        let mut tail = value;
        loop {
            let (next_owner, next) = evaluated(definitions, owner, tail)?;
            owner = next_owner;
            match next.get("kind").and_then(Value::as_str) {
                Some("nil") => break,
                Some("cons") => {
                    rows.push(metadata(
                        definitions,
                        owner,
                        next.get("head")
                            .ok_or_else(|| invalid("workbench list has no head"))?,
                        element,
                        remaining,
                        depth + 1,
                    )?);
                    tail = next
                        .get("tail")
                        .ok_or_else(|| invalid("workbench list has no tail"))?;
                }
                _ => return Err(invalid("workbench metadata requires a closed list")),
            }
        }
        return Ok(Value::Array(rows));
    }
    let single = BTreeMap::from([("value", (module, value))]);
    match kind {
        "String" => return Ok(Value::String(string(&single, "value")?)),
        "Bool" => return Ok(Value::Bool(boolean(&single, "value")?)),
        "UInt32" => {
            return Ok(Value::from(
                u32::try_from(unsigned(&single, "value", "uint32")?)
                    .map_err(|_| invalid("workbench integer exceeds UInt32"))?,
            ));
        }
        _ => (),
    }
    if let Some(variants) = enum_values(kind) {
        let name = constructor(&single, "value")?;
        let expected = variants
            .iter()
            .find(|variant| name == format!("{kind}.{variant}"));
        if expected.is_none()
            || value.pointer("/constructor/module").and_then(Value::as_str)
                != Some("Foundation.View.Text.V3.Model")
        {
            return Err(invalid(
                "workbench enum requires its exact source-owned constructor",
            ));
        }
        return Ok(Value::String(if kind == "WasmCachePolicy" {
            "no-store".to_owned()
        } else {
            expected.unwrap().to_ascii_lowercase()
        }));
    }
    let shape = source_shape(kind).ok_or_else(|| invalid("workbench metadata type is unknown"))?;
    if value.pointer("/type/module").and_then(Value::as_str)
        != Some("Foundation.View.Text.V3.Model")
        || value.pointer("/type/name").and_then(Value::as_str) != Some(kind)
    {
        return Err(invalid(
            "workbench record requires its exact source-owned type",
        ));
    }
    let fields = record(definitions, module, value)?;
    exact_fields(
        &fields,
        &shape.iter().map(|(name, _)| *name).collect::<Vec<_>>(),
    )?;
    let mut result = serde_json::Map::new();
    for (name, kind) in shape {
        let (owner, source) = field(&fields, name)?;
        result.insert(
            snake(name),
            metadata(definitions, owner, source, kind, remaining, depth + 1)?,
        );
    }
    Ok(Value::Object(result))
}

pub(super) fn project(
    definitions: &Definitions<'_>,
    _module: &str,
    declaration: &Value,
    fields: &BTreeMap<&str, (&str, &Value)>,
) -> Result<WorkbenchApplication, PrismError> {
    if member_name(declaration) != Some("TextApplication")
        || declaration
            .pointer("/result/member/module")
            .and_then(Value::as_str)
            != Some("Foundation.View.Text.V3.Model")
        || string(fields, "profile")? != PROFILE
    {
        return Err(invalid(
            "text/3 requires its exact source-owned Text.V3 application type and profile",
        ));
    }
    exact_fields(
        fields,
        &[
            "profile",
            "name",
            "cargoName",
            "cargoVersion",
            "cargoDescription",
            "cargoRepository",
            "cargoHomepage",
            "libraryRoots",
            "acceptanceVectors",
            "entryRoot",
            "coreContract",
            "requestMaximum",
            "responseMaximum",
            "guestAllocationMaximum",
            "coreMemoryPages",
            "coreStackBytes",
            "capabilitiesEmpty",
            "fatArchive",
            "primaryLayer",
            "viewLayer",
            "view",
        ],
    )?;
    let (owner, value) = field(fields, "view")?;
    let view: WorkbenchView = serde_json::from_value(metadata(
        definitions,
        owner,
        value,
        "WorkbenchView",
        &mut 65_536,
        0,
    )?)
    .map_err(|_| invalid("workbench metadata does not match its closed projection"))?;
    Ok(WorkbenchApplication {
        profile: string(fields, "profile")?,
        name: string(fields, "name")?,
        cargo_name: string(fields, "cargoName")?,
        cargo_version: string(fields, "cargoVersion")?,
        cargo_description: string(fields, "cargoDescription")?,
        cargo_repository: string(fields, "cargoRepository")?,
        cargo_homepage: string(fields, "cargoHomepage")?,
        library_roots: string_list(fields, "libraryRoots")?,
        acceptance_vectors: acceptance_vectors(definitions, fields)?,
        entry_root: string(fields, "entryRoot")?,
        core_contract: string(fields, "coreContract")?,
        request_maximum: u32::try_from(unsigned(fields, "requestMaximum", "uint32")?)
            .map_err(|_| invalid("workbench resource is out of range"))?,
        response_maximum: u32::try_from(unsigned(fields, "responseMaximum", "uint32")?)
            .map_err(|_| invalid("workbench resource is out of range"))?,
        guest_allocation_maximum: u32::try_from(unsigned(
            fields,
            "guestAllocationMaximum",
            "uint32",
        )?)
        .map_err(|_| invalid("workbench resource is out of range"))?,
        core_memory_pages: u32::try_from(unsigned(fields, "coreMemoryPages", "uint32")?)
            .map_err(|_| invalid("workbench resource is out of range"))?,
        core_stack_bytes: u32::try_from(unsigned(fields, "coreStackBytes", "uint32")?)
            .map_err(|_| invalid("workbench resource is out of range"))?,
        capabilities_empty: boolean(fields, "capabilitiesEmpty")?,
        fat_archive: boolean(fields, "fatArchive")?,
        primary_layer: u8::try_from(unsigned(fields, "primaryLayer", "uint8")?)
            .map_err(|_| invalid("workbench resource is out of range"))?,
        view_layer: u8::try_from(unsigned(fields, "viewLayer", "uint8")?)
            .map_err(|_| invalid("workbench resource is out of range"))?,
        view,
    })
}
