//! Closed list metadata, with iterative, bounded expansion of shared chunks.

use super::super::application::Definitions;
use crate::error::PrismError;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

const MAX_STEPS: usize = 65_536;
const MAX_ROOTS: usize = 1024;

fn malformed() -> PrismError {
    PrismError::new(
        "PP2001",
        "native-library root metadata is not a closed string list",
    )
}

enum Step<'a> {
    Value(&'a str, &'a Value),
    Leave((&'a str, &'a str)),
}

pub(super) fn project(
    definitions: &Definitions<'_>,
    fields: &BTreeMap<&str, (&str, &Value)>,
    name: &str,
) -> Result<Vec<String>, PrismError> {
    let (module, value) = fields.get(name).copied().ok_or_else(malformed)?;
    expand(definitions, module, value, MAX_STEPS)
}

fn expand<'a>(
    definitions: &'a Definitions<'a>,
    module: &'a str,
    value: &'a Value,
    maximum_steps: usize,
) -> Result<Vec<String>, PrismError> {
    let string_type = serde_json::json!({"kind":"string"});
    let list_type = serde_json::json!({"kind":"list","element":{"kind":"string"}});
    let mut pending = vec![Step::Value(module, value)];
    let mut active = BTreeSet::new();
    let mut result = Vec::new();
    let mut remaining = maximum_steps;
    while let Some(step) = pending.pop() {
        let Step::Value(module, value) = step else {
            if let Step::Leave(reference) = step {
                active.remove(&reference);
            }
            continue;
        };
        remaining = remaining.checked_sub(1).ok_or_else(|| {
            PrismError::new(
                "PP2001",
                "native-library root metadata exceeds the projection budget",
            )
        })?;
        match value.get("kind").and_then(Value::as_str) {
            Some("nil") if value.get("element") == Some(&string_type) => {}
            Some("cons") => {
                let head = value.get("head").ok_or_else(malformed)?;
                if head.get("kind").and_then(Value::as_str) != Some("string") {
                    return Err(malformed());
                }
                let name = head
                    .get("value")
                    .and_then(Value::as_str)
                    .ok_or_else(malformed)?;
                if result.len() == MAX_ROOTS || !super::qualified(name) {
                    return Err(super::invalid(
                        "native-library roots exceed their count or identifier bounds",
                    ));
                }
                result.push(name.to_owned());
                pending.push(Step::Value(
                    module,
                    value.get("tail").ok_or_else(malformed)?,
                ));
            }
            Some("primitive")
                if value.get("operation").and_then(Value::as_str) == Some("append")
                    && value.get("result") == Some(&list_type) =>
            {
                let arguments = value
                    .get("arguments")
                    .and_then(Value::as_array)
                    .ok_or_else(malformed)?;
                let [left, right] = arguments.as_slice() else {
                    return Err(malformed());
                };
                pending.push(Step::Value(module, right));
                pending.push(Step::Value(module, left));
            }
            Some("call") => {
                if !value
                    .get("arguments")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
                {
                    return Err(malformed());
                }
                let reference = value.get("function").ok_or_else(malformed)?;
                let name = reference
                    .get("name")
                    .and_then(Value::as_str)
                    .ok_or_else(malformed)?;
                let owner = match reference.get("module") {
                    None => module,
                    Some(value) => value.as_str().ok_or_else(malformed)?,
                };
                let declaration = definitions
                    .get(&(owner.to_owned(), name.to_owned()))
                    .copied()
                    .ok_or_else(malformed)?;
                if declaration.get("result") != Some(&list_type)
                    || !declaration
                        .get("parameters")
                        .and_then(Value::as_array)
                        .is_some_and(Vec::is_empty)
                    || !active.insert((owner, name))
                {
                    return Err(malformed());
                }
                pending.push(Step::Leave((owner, name)));
                pending.push(Step::Value(
                    owner,
                    declaration.get("body").ok_or_else(malformed)?,
                ));
            }
            _ => return Err(malformed()),
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn nil() -> Value {
        json!({"kind":"nil","element":{"kind":"string"}})
    }
    fn list(names: &[&str]) -> Value {
        names.iter().rev().fold(
            nil(),
            |tail, name| json!({"kind":"cons","head":{"kind":"string","value":name},"tail":tail}),
        )
    }
    fn append(left: Value, right: Value) -> Value {
        json!({"kind":"primitive","operation":"append","result":{"kind":"list","element":{"kind":"string"}},"arguments":[left,right]})
    }
    fn call(owner: &str, name: &str) -> Value {
        json!({"kind":"call","function":{"module":owner,"name":name},"arguments":[]})
    }
    fn definition(body: Value) -> Value {
        json!({"kind":"definition","parameters":[],"result":{"kind":"list","element":{"kind":"string"}},"body":body})
    }

    #[test]
    fn shared_closed_chunks_preserve_every_root_and_order() {
        let chunk = definition(list(&["Example.alpha", "Example.beta"]));
        let definitions = BTreeMap::from([(("Other".to_owned(), "chunk".to_owned()), &chunk)]);
        let value = append(
            append(nil(), call("Other", "chunk")),
            append(list(&["Example.gamma"]), call("Other", "chunk")),
        );
        assert_eq!(
            expand(&definitions, "Here", &value, MAX_STEPS).unwrap(),
            [
                "Example.alpha",
                "Example.beta",
                "Example.gamma",
                "Example.alpha",
                "Example.beta"
            ]
        );
        // Duplicate/order rejection belongs to library validation, not a hidden sort/dedup here.
    }

    #[test]
    fn root_calls_reach_the_bounded_walker_without_eager_scalar_evaluation() {
        let title = json!({"kind":"definition","parameters":[],"result":{"kind":"string"},
            "body":{"kind":"string","value":"Example"}});
        let definitions = BTreeMap::from([(("Here".to_owned(), "title".to_owned()), &title)]);
        let value = json!({"kind":"record","fields":[
            {"field":"name","value":call("Here","title")},
            {"field":"exportRoots","value":call("Here","unresolved")}
        ]});
        let fields = super::super::record_with_unevaluated_fields(
            &definitions,
            "Here",
            &value,
            &["exportRoots", "acceptanceRoots"],
        )
        .unwrap();
        assert_eq!(
            fields["name"].1,
            &json!({"kind":"string","value":"Example"})
        );
        assert_eq!(fields["exportRoots"].1, &call("Here", "unresolved"));
        assert_eq!(
            project(&definitions, &fields, "exportRoots")
                .unwrap_err()
                .code
                .as_str(),
            "PP2001"
        );
    }

    #[test]
    fn closed_root_projection_rejects_unresolved_cycles_arguments_and_other_types() {
        let cyclic = definition(call("Here", "cycle"));
        let wrong_type = json!({"kind":"definition","parameters":[],"result":{"kind":"bool"},"body":{"kind":"bool","value":true}});
        let open = json!({"kind":"definition","parameters":[{"name":"x","type":{"kind":"nat"}}],"result":{"kind":"list","element":{"kind":"string"}},"body":nil()});
        let definitions = BTreeMap::from([
            (("Here".to_owned(), "cycle".to_owned()), &cyclic),
            (("Here".to_owned(), "wrong".to_owned()), &wrong_type),
            (("Here".to_owned(), "open".to_owned()), &open),
        ]);
        let mut argument = call("Here", "cycle");
        argument["arguments"] = json!([{"kind":"nat","value":"0"}]);
        let mut malformed_append = append(nil(), nil());
        malformed_append["arguments"] = json!([nil()]);
        let mut wrong_result = append(nil(), nil());
        wrong_result["result"] = json!({"kind":"bytes"});
        for value in [
            call("Here", "cycle"),
            call("Here", "missing"),
            call("Here", "wrong"),
            call("Here", "open"),
            argument,
            malformed_append,
            wrong_result,
            json!({"kind":"nil","element":{"kind":"nat"}}),
            json!({"kind":"cons","head":{"kind":"nat","value":"0"},"tail":nil()}),
            json!({"kind":"var","name":"unbound"}),
        ] {
            assert_eq!(
                expand(&definitions, "Here", &value, MAX_STEPS)
                    .unwrap_err()
                    .code
                    .as_str(),
                "PP2001"
            );
        }
    }

    #[test]
    fn closed_root_projection_enforces_exact_count_and_work_budgets() {
        let empty = Definitions::new();
        let alias = definition(list(&["Example.one"]));
        let definitions = BTreeMap::from([(("Here".to_owned(), "alias".to_owned()), &alias)]);
        let reference = call("Here", "alias");
        assert_eq!(
            expand(&definitions, "Here", &reference, 3).unwrap(),
            ["Example.one"]
        );
        assert!(expand(&definitions, "Here", &reference, 2)
            .unwrap_err()
            .message
            .contains("projection budget"));
        let value = append(list(&["Example.one"]), nil());
        assert_eq!(expand(&empty, "Here", &value, 4).unwrap(), ["Example.one"]);
        assert!(expand(&empty, "Here", &value, 3)
            .unwrap_err()
            .message
            .contains("projection budget"));
        let mut value = list(&["Example.one"]);
        for _ in 0..10 {
            value = append(value.clone(), value);
        }
        assert_eq!(
            expand(&empty, "Here", &value, MAX_STEPS).unwrap().len(),
            1024
        );
        value = append(value, list(&["Example.extra"]));
        assert_eq!(
            expand(&empty, "Here", &value, MAX_STEPS)
                .unwrap_err()
                .code
                .as_str(),
            "PP4004"
        );

        let mut entries = vec![definition(nil())];
        for index in 1..17 {
            let name = format!("chunk{}", index - 1);
            entries.push(definition(append(call("Here", &name), call("Here", &name))));
        }
        let definitions = entries
            .iter()
            .enumerate()
            .map(|(index, value)| (("Here".to_owned(), format!("chunk{index}")), value))
            .collect();
        assert!(
            expand(&definitions, "Here", &call("Here", "chunk16"), MAX_STEPS)
                .unwrap_err()
                .message
                .contains("projection budget")
        );
    }
}
