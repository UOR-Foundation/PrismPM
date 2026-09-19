use super::*;
use serde_json::json;

fn alias(owner: &str, name: &str) -> Value {
    json!({"kind":"call","function":{"module":owner,"name":name},"arguments":[]})
}

fn closed(body: Value) -> Value {
    json!({"kind":"definition","parameters":[],"result":{"kind":"string"},"body":body})
}

#[test]
fn cyclic_metadata_aliases_are_typed_failures() {
    let self_cycle = closed(alias("First", "self"));
    let first = closed(alias("Second", "next"));
    let second = closed(alias("First", "next"));
    let definitions = BTreeMap::from([
        (("First".to_owned(), "self".to_owned()), &self_cycle),
        (("First".to_owned(), "next".to_owned()), &first),
        (("Second".to_owned(), "next".to_owned()), &second),
    ]);
    for request in [alias("First", "self"), alias("First", "next")] {
        assert_eq!(
            evaluated(&definitions, "First", &request).unwrap_err().code,
            "PP2001"
        );
    }
    let record = json!({"kind":"record","fields":[{"field":"name","value":alias("First","self")}]});
    assert_eq!(
        record_with_unevaluated_fields(&definitions, "First", &record, &[])
            .unwrap_err()
            .code,
        "PP2001"
    );
    // Native root lists have their own bounded walker and must stay deferred.
    assert_eq!(
        record_with_unevaluated_fields(&definitions, "First", &record, &["name"]).unwrap()["name"]
            .1,
        &record["fields"][0]["value"]
    );
}

#[test]
fn malformed_alias_owners_never_fall_back_to_the_current_module() {
    let declaration = closed(json!({"kind":"string","value":"valid"}));
    let definitions = BTreeMap::from([(("Here".to_owned(), "value".to_owned()), &declaration)]);
    for owner in [Value::Null, json!(false), json!(7), json!([]), json!({})] {
        let mut request = alias("Here", "value");
        request["function"]["module"] = owner;
        assert_eq!(
            evaluated(&definitions, "Here", &request).unwrap_err().code,
            "PP2001"
        );
    }
    for request in [
        json!({"kind":"call","function":{"name":"value"},"arguments":[{"kind":"string","value":"extra"}]}),
        json!({"kind":"call","function":{"name":"value"},"arguments":null}),
        json!({"kind":"call","function":{"name":7},"arguments":[]}),
        alias("Missing", "value"),
    ] {
        assert_eq!(
            evaluated(&definitions, "Here", &request).unwrap_err().code,
            "PP2001"
        );
    }
}

#[test]
fn aliases_preserve_module_context_and_repeated_independent_fields() {
    let here = closed(json!({"kind":"string","value":"wrong module"}));
    let other = closed(json!({"kind":"string","value":"right module"}));
    let first = closed(json!({"kind":"call","function":{"name":"value"},"arguments":[]}));
    let definitions = BTreeMap::from([
        (("Here".to_owned(), "value".to_owned()), &here),
        (("Other".to_owned(), "value".to_owned()), &other),
        (("Other".to_owned(), "first".to_owned()), &first),
    ]);
    let request = alias("Other", "first");
    let (owner, result) = evaluated(&definitions, "Here", &request).unwrap();
    assert_eq!(owner, "Other");
    assert_eq!(result, &other["body"]);
    let record = json!({"kind":"record","fields":[
        {"field":"name","value":request.clone()},
        {"field":"description","value":request}
    ]});
    let fields = record_with_unevaluated_fields(&definitions, "Here", &record, &[]).unwrap();
    assert_eq!(fields["name"], fields["description"]);
    assert_eq!(fields["name"], ("Other", &other["body"]));
}

#[test]
fn metadata_alias_budget_has_an_exact_iterative_boundary() {
    const LIMIT: usize = 65_536;
    let declarations = (0..=LIMIT)
        .map(|index| {
            if index == LIMIT {
                closed(json!({"kind":"string","value":"leaf"}))
            } else {
                closed(alias("Chain", &format!("item{}", index + 1)))
            }
        })
        .collect::<Vec<_>>();
    let definitions = declarations
        .iter()
        .enumerate()
        .map(|(index, value)| (("Chain".to_owned(), format!("item{index}")), value))
        .collect::<Definitions<'_>>();
    // Each resolved call consumes one step; the terminal literal is not a call.
    let at_limit = alias("Chain", "item1");
    assert_eq!(
        evaluated(&definitions, "Chain", &at_limit).unwrap().1,
        &declarations[LIMIT]["body"]
    );
    let over_limit = alias("Chain", "item0");
    assert_eq!(
        evaluated(&definitions, "Chain", &over_limit)
            .unwrap_err()
            .code,
        "PP2001"
    );
}
