//! Bounded typed projection; no application execution or effect admission.

use super::{invalid, BrowserApplication, MODULE};
use crate::error::PrismError;
use crate::holo::application::Definitions;
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;

const APPLICATION: &[(&str, &str, &str)] = &[
    ("profile", "profile", "String"),
    ("name", "name", "String"),
    ("cargoName", "cargo_name", "String"),
    ("cargoVersion", "cargo_version", "String"),
    ("cargoDescription", "cargo_description", "String"),
    ("cargoRepository", "cargo_repository", "String"),
    ("cargoHomepage", "cargo_homepage", "String"),
    ("libraryRoots", "library_roots", "ListString"),
    (
        "acceptanceVectors",
        "acceptance_vectors",
        "ListAcceptanceVector",
    ),
    ("entryRoot", "entry_root", "String"),
    ("coreContract", "core_contract", "String"),
    ("requestMaximum", "request_maximum", "UInt32"),
    ("responseMaximum", "response_maximum", "UInt32"),
    (
        "guestAllocationMaximum",
        "guest_allocation_maximum",
        "UInt32",
    ),
    ("memoryPages", "memory_pages", "UInt32"),
    ("capabilitiesEmpty", "capabilities_empty", "Bool"),
    ("fatArchive", "fat_archive", "Bool"),
    ("primaryLayer", "primary_layer", "UInt8"),
    ("viewLayer", "view_layer", "UInt8"),
    ("protocol", "protocol", "String"),
    (
        "requestedEffects",
        "requested_effects",
        "ListRequestedEffect",
    ),
    ("durability", "durability", "Durability"),
    ("view", "view", "BrowserView"),
];

fn fields(kind: &str) -> Option<&'static [(&'static str, &'static str, &'static str)]> {
    Some(match kind {
        "BrowserApplication" => APPLICATION,
        "AcceptanceVector" => &[
            ("request", "request", "Bytes"),
            ("response", "response", "Bytes"),
        ],
        "RequestedEffect" => &[
            ("resource", "resource", "String"),
            ("adapter", "adapter", "RequestedAdapter"),
        ],
        "Durability" => &[
            ("protocol", "protocol", "String"),
            ("resource", "resource", "String"),
            ("head", "head", "String"),
            ("replayRoot", "replay_root", "String"),
            ("maxPending", "max_pending", "UInt32"),
        ],
        "BrowserView" => &[
            ("surface", "surface", "String"),
            ("protocol", "protocol", "String"),
            ("title", "title", "String"),
            ("heading", "heading", "String"),
            ("presentationRoot", "presentation_root", "String"),
            ("maximum", "maximum", "UInt32"),
            ("labels", "labels", "ListLabel"),
        ],
        "Label" => &[("id", "id", "String"), ("text", "text", "String")],
        _ => return None,
    })
}

struct Projector<'a> {
    definitions: &'a Definitions<'a>,
    fuel: usize,
    bytes: usize,
}

impl<'a> Projector<'a> {
    fn step(&mut self) -> Result<(), PrismError> {
        self.fuel = self
            .fuel
            .checked_sub(1)
            .ok_or_else(|| invalid("browser source metadata exceeds its traversal budget"))?;
        Ok(())
    }

    fn resolve(
        &mut self,
        mut owner: &'a str,
        mut value: &'a Value,
    ) -> Result<(&'a str, &'a Value), PrismError> {
        let mut visited = BTreeSet::new();
        loop {
            self.step()?;
            if value["kind"] != "call" {
                return Ok((owner, value));
            }
            if !value["arguments"].as_array().is_some_and(Vec::is_empty) {
                return Err(invalid("browser metadata calls must be closed"));
            }
            let reference = &value["function"];
            let name = reference["name"]
                .as_str()
                .ok_or_else(|| invalid("invalid browser metadata alias"))?;
            owner = match reference.get("module") {
                None => owner,
                Some(v) => v
                    .as_str()
                    .ok_or_else(|| invalid("invalid browser metadata alias owner"))?,
            };
            if !visited.insert((owner, name)) {
                return Err(invalid("cyclic browser metadata alias"));
            }
            let declaration = self
                .definitions
                .get(&(owner.to_owned(), name.to_owned()))
                .ok_or_else(|| invalid("unknown browser metadata alias"))?;
            if !declaration["parameters"]
                .as_array()
                .is_some_and(Vec::is_empty)
            {
                return Err(invalid("browser metadata alias has parameters"));
            }
            value = declaration
                .get("body")
                .ok_or_else(|| invalid("browser metadata alias has no body"))?;
        }
    }

    fn value(&mut self, owner: &'a str, value: &'a Value, kind: &str) -> Result<Value, PrismError> {
        let (owner, value) = self.resolve(owner, value)?;
        if let Some(element) = kind.strip_prefix("List") {
            let maximum = match element {
                "String" => 1024,
                "RequestedEffect" => 64,
                _ => 256,
            };
            let mut result = Vec::new();
            let mut current = value;
            let mut module = owner;
            loop {
                (module, current) = self.resolve(module, current)?;
                if current["kind"] == "nil" {
                    if current.get("element") != Some(&source_type(element)) {
                        return Err(invalid(
                            "browser metadata list has a substituted element type",
                        ));
                    }
                    return Ok(Value::Array(result));
                }
                if current["kind"] != "cons" || result.len() == maximum {
                    return Err(invalid("browser metadata list is malformed or oversized"));
                }
                result.push(
                    self.value(
                        module,
                        current
                            .get("head")
                            .ok_or_else(|| invalid("browser metadata list has no head"))?,
                        element,
                    )?,
                );
                current = current
                    .get("tail")
                    .ok_or_else(|| invalid("browser metadata list has no tail"))?;
            }
        }
        let result = match kind {
            "String" if value["kind"] == "string" => {
                let text = value["value"]
                    .as_str()
                    .ok_or_else(|| invalid("invalid browser metadata string"))?;
                if text.len() > 4096 {
                    return Err(invalid("browser metadata string exceeds its bound"));
                }
                Value::String(text.to_owned())
            }
            "Bool" if value["kind"] == "bool" => Value::Bool(
                value["value"]
                    .as_bool()
                    .ok_or_else(|| invalid("invalid browser metadata Boolean"))?,
            ),
            "UInt32" | "UInt8" if value["kind"] == "integer" => {
                let representation = if kind == "UInt32" { "uint32" } else { "uint8" };
                if value["representation"] != representation {
                    return Err(invalid("browser metadata integer has wrong type"));
                }
                let text = value["value"]
                    .as_str()
                    .ok_or_else(|| invalid("invalid browser metadata integer"))?;
                let number: u32 = text
                    .parse()
                    .map_err(|_| invalid("browser metadata integer out of range"))?;
                if number.to_string() != text || kind == "UInt8" && number > 255 {
                    return Err(invalid("noncanonical browser metadata integer"));
                }
                json!(number)
            }
            "Bytes" if value["kind"] == "bytes" => {
                let hex = value["hex"]
                    .as_str()
                    .ok_or_else(|| invalid("invalid browser vector bytes"))?;
                self.bytes = self.bytes.saturating_add(hex.len() / 2);
                if hex.len() % 2 != 0
                    || hex.len() > 65536
                    || self.bytes > 24576
                    || !hex
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                {
                    return Err(invalid(
                        "browser vector bytes are noncanonical or oversized",
                    ));
                }
                Value::Array(
                    hex.as_bytes()
                        .chunks_exact(2)
                        .map(|pair| {
                            json!(u8::from_str_radix(
                                std::str::from_utf8(pair).expect("ASCII hex"),
                                16
                            )
                            .expect("checked hex"))
                        })
                        .collect(),
                )
            }
            "RequestedAdapter" => self.adapter(owner, value)?,
            _ => {
                let expected =
                    fields(kind).ok_or_else(|| invalid("unknown browser metadata type"))?;
                if value["kind"] != "record"
                    || value["type"] != json!({"module":MODULE,"name":kind})
                {
                    return Err(invalid("browser metadata record type is not source-owned"));
                }
                let rows = value["fields"]
                    .as_array()
                    .ok_or_else(|| invalid("browser metadata record fields are absent"))?;
                if rows.len() != expected.len() {
                    return Err(invalid("browser metadata record is not closed"));
                }
                let mut seen = BTreeSet::new();
                let mut result = Map::new();
                for row in rows {
                    let field = row["field"]
                        .as_str()
                        .ok_or_else(|| invalid("browser metadata field is malformed"))?;
                    let (_, output, ty) = expected
                        .iter()
                        .find(|r| r.0 == field)
                        .ok_or_else(|| invalid("unknown browser metadata field"))?;
                    if !seen.insert(field) {
                        return Err(invalid("duplicate browser metadata field"));
                    }
                    result.insert(
                        (*output).to_owned(),
                        self.value(
                            owner,
                            row.get("value")
                                .ok_or_else(|| invalid("browser metadata field has no value"))?,
                            ty,
                        )?,
                    );
                }
                Value::Object(result)
            }
        };
        Ok(result)
    }

    fn adapter(&mut self, owner: &'a str, value: &'a Value) -> Result<Value, PrismError> {
        if value["kind"] != "constructor"
            || value["constructor"]["module"] != MODULE
            || value
                .get("type_arguments")
                .is_some_and(|v| !v.as_array().is_some_and(Vec::is_empty))
        {
            return Err(invalid(
                "requested adapter is not a source-owned constructor",
            ));
        }
        let (kind, fields): (&str, &[(&str, &str)]) = match value["constructor"]["name"]
            .as_str()
            .and_then(|v| v.strip_prefix("RequestedAdapter."))
        {
            Some("Guest") => (
                "guest",
                &[
                    ("entry_root", "String"),
                    ("protocol", "String"),
                    ("input_maximum", "UInt32"),
                    ("output_maximum", "UInt32"),
                    ("memory_pages", "UInt32"),
                ],
            ),
            Some("Random") => ("random", &[("maximum", "UInt32")]),
            Some("Digest") => ("digest", &[("maximum", "UInt32")]),
            Some("Sign") => (
                "sign",
                &[
                    ("credential_slot", "String"),
                    ("context", "String"),
                    ("maximum", "UInt32"),
                ],
            ),
            Some("Verify") => ("verify", &[("context", "String"), ("maximum", "UInt32")]),
            Some("Store") => (
                "store",
                &[
                    ("namespace", "String"),
                    ("max_object_bytes", "UInt32"),
                    ("max_objects", "UInt32"),
                    ("max_heads", "UInt32"),
                ],
            ),
            _ => return Err(invalid("unknown requested adapter constructor")),
        };
        let args = value["arguments"]
            .as_array()
            .ok_or_else(|| invalid("requested adapter arguments are absent"))?;
        if args.len() != fields.len() {
            return Err(invalid("requested adapter arguments are not closed"));
        }
        let mut result = Map::from_iter([("kind".to_owned(), json!(kind))]);
        for ((name, ty), argument) in fields.iter().zip(args) {
            result.insert((*name).to_owned(), self.value(owner, argument, ty)?);
        }
        Ok(Value::Object(result))
    }
}

fn source_type(kind: &str) -> Value {
    match kind {
        "String" => json!({"kind":"string"}),
        _ => json!({"kind":"named","member":{"module":MODULE,"name":kind},"arguments":[]}),
    }
}

pub(in crate::holo) fn project<'a>(
    definitions: &'a Definitions<'a>,
    owner: &'a str,
    declaration: &'a Value,
) -> Result<BrowserApplication, PrismError> {
    if declaration.get("result") != Some(&source_type("BrowserApplication"))
        || !declaration["parameters"]
            .as_array()
            .is_some_and(Vec::is_empty)
    {
        return Err(invalid(
            "browser application result type must be the exact closed source-owned declaration",
        ));
    }
    let value = Projector {
        definitions,
        fuel: 65536,
        bytes: 0,
    }
    .value(
        owner,
        declaration
            .get("body")
            .ok_or_else(|| invalid("browser application definition has no body"))?,
        "BrowserApplication",
    )?;
    serde_json::from_value(value)
        .map_err(|_| invalid("browser application projection is not a closed declaration"))
}
