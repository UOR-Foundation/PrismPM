//! Exact artifact names and resource-to-source bindings; never effective grants.

use super::{invalid, json_bytes};
use crate::error::PrismError;
use crate::holo::browser_application::{BrowserApplication, RequestedAdapter};
use crate::library_build::sha256;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Component, Path};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Target {
    pub(super) root: String,
    pub(super) input_maximum: u32,
    pub(super) output_maximum: u32,
    pub(super) memory_pages: u32,
}

impl Target {
    pub(super) fn id(&self) -> Result<String, PrismError> {
        Ok(sha256(&json_bytes(self)?))
    }
    pub(super) fn leaf(&self) -> Result<&str, PrismError> {
        self.root
            .rsplit('.')
            .next()
            .filter(|v| !v.is_empty())
            .ok_or_else(|| invalid("browser root lacks a symbol"))
    }
}

pub(super) struct Plan {
    pub(super) targets: BTreeMap<String, Target>,
    roles: BTreeMap<String, String>,
    model_sha256: String,
    policy: Vec<u8>,
}

pub(super) fn relative(value: &str) -> Result<(), PrismError> {
    if value.is_empty()
        || value.contains('\\')
        || value
            .split('/')
            .any(|v| v.is_empty() || v == "." || v == "..")
        || Path::new(value)
            .components()
            .any(|v| !matches!(v, Component::Normal(_)))
    {
        return Err(invalid(
            "browser artifact path is not a canonical relative path",
        ));
    }
    Ok(())
}

impl Plan {
    pub(super) fn new(application: &BrowserApplication, model: &[u8]) -> Result<Self, PrismError> {
        crate::holo::browser_application::validate(application)?;
        let mut result = Self {
            targets: BTreeMap::new(),
            roles: BTreeMap::new(),
            model_sha256: sha256(model),
            policy: json_bytes(&application.requested_effects)?,
        };
        for (role, root, output) in [
            (
                "primary",
                &application.entry_root,
                application.response_maximum,
            ),
            (
                "replay",
                &application.durability.replay_root,
                application.response_maximum,
            ),
            (
                "presentation",
                &application.view.presentation_root,
                application.view.maximum,
            ),
        ] {
            result.add(
                role.into(),
                Target {
                    root: root.clone(),
                    input_maximum: application.guest_allocation_maximum,
                    output_maximum: output,
                    memory_pages: application.memory_pages,
                },
            )?;
        }
        for request in &application.requested_effects {
            if let RequestedAdapter::Guest {
                entry_root,
                input_maximum,
                output_maximum,
                memory_pages,
                ..
            } = &request.adapter
            {
                result.add(
                    format!("resource:{}", request.resource),
                    Target {
                        root: entry_root.clone(),
                        input_maximum: *input_maximum,
                        output_maximum: *output_maximum,
                        memory_pages: *memory_pages,
                    },
                )?;
            }
        }
        Ok(result)
    }

    fn add(&mut self, role: String, target: Target) -> Result<(), PrismError> {
        let id = target.id()?;
        if self.roles.insert(role, id.clone()).is_some() {
            return Err(invalid("duplicate browser artifact role"));
        }
        if let Some(previous) = self.targets.insert(id, target.clone()) {
            if previous != target {
                return Err(invalid("browser target hash collision"));
            }
        }
        Ok(())
    }

    pub(super) fn binding(&self, files: &BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>, PrismError> {
        let mut rows = Vec::new();
        for (path, bytes) in files {
            relative(path)?;
            if path == "compiler/binding.json" {
                continue;
            }
            rows.push(json!({"path":path,"sha256":sha256(bytes),"byte_length":bytes.len()}));
        }
        json_bytes(
            &json!({"schema":"prismpm/internal-browser-compiler-binding/1", "scope":"unaccepted-compiler-output",
            "model_sha256":self.model_sha256,"policy_sha256":sha256(&self.policy),"roles":self.roles,"targets":self.targets,"files":rows}),
        )
    }

    pub(super) fn validate(&self, files: &BTreeMap<String, Vec<u8>>) -> Result<(), PrismError> {
        if files.get("source/model.json").map(|v| sha256(v)).as_ref() != Some(&self.model_sha256)
            || files.get("compiler/policy.json") != Some(&self.policy)
            || files.get("compiler/binding.json") != Some(&self.binding(files)?)
        {
            return Err(invalid(
                "browser model, requested policy or exact artifact binding differs",
            ));
        }
        // Package files are generated, never caller-named. Closed roles identify
        // each distinct source-root/budget tuple without truncation or coercion.
        for (id, target) in &self.targets {
            let prefix = format!("guests/{id}/");
            let bytes = files
                .get(&format!("{prefix}generation-manifest.json"))
                .ok_or_else(|| invalid("browser generated target metadata is absent"))?;
            let value: Value = serde_json::from_slice(bytes).map_err(|e| invalid(e.to_string()))?;
            if value["entry"] != target.leaf()?
                || value["export"] != "holo_run"
                || value["input_allocation_cap"].as_u64() != Some(u64::from(target.input_maximum))
                || value["output_allocation_cap"].as_u64() != Some(u64::from(target.output_maximum))
                || value["maximum_pages"].as_u64() != Some(u64::from(target.memory_pages))
                || value["input_ir_sha256"]
                    != sha256(
                        files
                            .get("compiler/kernel.ir")
                            .ok_or_else(|| invalid("browser kernel IR is absent"))?,
                    )
                || !files.contains_key(&format!("{prefix}core.wasm"))
            {
                return Err(invalid(
                    "browser generated target lost its exact source root or budget",
                ));
            }
        }
        Ok(())
    }
}

// Consistency alone never authenticates or accepts a retained compiler output.
// The verifier must regenerate the full source/kernel/native/Wasm closure and
// compare every byte, including consistently rehashed substitutions.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "private replay prerequisite; public runtime remains unavailable"
    )
)]
pub(super) fn require_replay(
    expected: &super::Compilation,
    actual: &BTreeMap<String, Vec<u8>>,
) -> Result<(), PrismError> {
    expected.plan.validate(actual)?;
    if &expected.files != actual {
        return Err(invalid(
            "browser compiler output differs from independent source replay",
        ));
    }
    Ok(())
}
