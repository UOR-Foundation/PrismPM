//! VR-09's conservative production-source guard, including falsification controls.

use std::path::Path;

#[derive(Clone, Copy)]
struct Token<'a> {
    text: &'a str,
    start: usize,
    end: usize,
}

// This scanner only establishes source boundaries; it does not interpret Rust.
// Comments and Rust string/character literals cannot invent module delimiters.
fn tokens(source: &str) -> Result<Vec<Token<'_>>, &'static str> {
    let bytes = source.as_bytes();
    let mut result = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let start = index;
        if bytes[index].is_ascii_whitespace() {
            index += 1;
            continue;
        }
        if bytes[index..].starts_with(b"//") {
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            continue;
        }
        if bytes[index..].starts_with(b"/*") {
            index += 2;
            let mut depth = 1;
            while depth != 0 {
                if index == bytes.len() {
                    return Err("unterminated source comment");
                }
                if bytes[index..].starts_with(b"/*") {
                    depth += 1;
                    index += 2;
                } else if bytes[index..].starts_with(b"*/") {
                    depth -= 1;
                    index += 2;
                } else {
                    index += 1;
                }
            }
            continue;
        }
        // A raw literal may have a byte/C prefix and any number of hash marks.
        let raw = if bytes[index..].starts_with(b"br") || bytes[index..].starts_with(b"cr") {
            Some(index + 2)
        } else if bytes[index] == b'r' {
            Some(index + 1)
        } else {
            None
        };
        let mut raw_end = None;
        if let Some(opening) = raw {
            let mut quote = opening;
            while bytes.get(quote) == Some(&b'#') {
                quote += 1;
            }
            if bytes.get(quote) == Some(&b'"') {
                let hashes = quote - opening;
                index = quote + 1;
                loop {
                    if index >= bytes.len() {
                        return Err("unterminated source string");
                    }
                    if bytes[index] == b'"'
                        && bytes.get(index + 1..index + 1 + hashes)
                            == Some(&bytes[quote - hashes..quote])
                    {
                        raw_end = Some(index + 1 + hashes);
                        break;
                    }
                    index += 1;
                }
            }
        }
        if let Some(end) = raw_end {
            index = end;
        } else if bytes[index] == b'"' {
            index += 1;
            loop {
                match bytes.get(index) {
                    None => return Err("unterminated source string"),
                    Some(b'\\') => index += 2,
                    Some(b'"') => {
                        index += 1;
                        break;
                    }
                    Some(_) => index += 1,
                }
            }
        } else if bytes[index] == b'\'' {
            // Distinguish a character literal from a lifetime such as 'a.
            let next = index + 1;
            if bytes.get(next) == Some(&b'\\') {
                index = next + 2;
                if bytes.get(next + 1) == Some(&b'u') {
                    if bytes.get(index) != Some(&b'{') {
                        return Err("invalid source character escape");
                    }
                    index += 1;
                    while bytes.get(index).is_some_and(u8::is_ascii_hexdigit) {
                        index += 1;
                    }
                    if bytes.get(index) != Some(&b'}') {
                        return Err("invalid source character escape");
                    }
                    index += 1;
                } else if bytes.get(next + 1) == Some(&b'x') {
                    index += 2;
                }
                if bytes.get(index) != Some(&b'\'') {
                    return Err("unterminated source character");
                }
                index += 1;
            } else if let Some(character) = source[next..].chars().next() {
                let close = next + character.len_utf8();
                index = if bytes.get(close) == Some(&b'\'') {
                    close + 1
                } else {
                    next
                };
            } else {
                return Err("unterminated source character");
            }
        } else if bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_' {
            index += 1;
            while bytes
                .get(index)
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
            {
                index += 1;
            }
        } else if bytes[index].is_ascii() {
            index += 1;
        } else {
            return Err("unsupported source token");
        }
        result.push(Token {
            text: &source[start..index],
            start,
            end: index,
        });
    }
    Ok(result)
}

fn production_source(source: &str) -> Result<String, &'static str> {
    if source.len() > 1_048_576 {
        return Err("verification source bound exceeded");
    }
    let tokens = tokens(source)?;
    let mut stack = Vec::new();
    let mut closing = vec![None; tokens.len()];
    let mut depths = Vec::with_capacity(tokens.len());
    for (index, token) in tokens.iter().enumerate() {
        depths.push(stack.len());
        match token.text {
            "(" | "[" | "{" => stack.push(index),
            ")" | "]" | "}" => {
                let open = stack.pop().ok_or("unbalanced source delimiters")?;
                if !matches!(
                    (tokens[open].text, token.text),
                    ("(", ")") | ("[", "]") | ("{", "}")
                ) {
                    return Err("unbalanced source delimiters");
                }
                closing[open] = Some(index);
            }
            _ => {}
        }
    }
    if !stack.is_empty() {
        return Err("unbalanced source delimiters");
    }
    let attribute = ["#", "[", "cfg", "(", "test", ")", "]"];
    let mut modules = Vec::new();
    let mut spans = Vec::new();
    for (index, window) in tokens.windows(attribute.len()).enumerate() {
        if !window.iter().map(|token| token.text).eq(attribute) {
            continue;
        }
        if depths[index] != 0
            || tokens.get(index + 7).map(|token| token.text) != Some("mod")
            || tokens.get(index + 9).map(|token| token.text) != Some("{")
        {
            return Err("invalid top-level test module boundary");
        }
        let name = tokens[index + 8].text;
        if !["portable_oracle_tests", "tests"].contains(&name) || modules.contains(&name) {
            return Err("unexpected or duplicate test module boundary");
        }
        let close = closing[index + 9].ok_or("missing test module closing delimiter")?;
        modules.push(name);
        spans.push((tokens[index].start, tokens[close].end));
    }
    if modules != ["portable_oracle_tests", "tests"] {
        return Err("incomplete test module boundary inventory");
    }
    let mut production = String::new();
    let mut start = 0;
    for (open, close) in spans {
        production.push_str(&source[start..open]);
        start = close;
    }
    production.push_str(&source[start..]);
    Ok(production)
}

fn check(source: &str) -> Result<(), &'static str> {
    let production = production_source(source)?;
    // Preserve the original conservative URL/client ban on all production text,
    // including literal strings. Explicit socket/client names cover non-HTTP IO.
    if production.to_ascii_lowercase().contains("http") {
        return Err("production network reference");
    }
    let forbidden = [
        "reqwest",
        "ureq",
        "hyper",
        "curl",
        "wget",
        "TcpStream",
        "TcpListener",
        "UdpSocket",
        "ToSocketAddrs",
        "TcpSocket",
        "UdpSocketBuilder",
    ];
    if production
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .any(|word| forbidden.contains(&word))
    {
        return Err("production network reference");
    }
    Ok(())
}

fn controls(source: &str) {
    for planted in [
        "fn planted() { reqwest::blocking::get(\"https://github.com/UOR-Foundation/PrismPM\"); }\n",
        "fn planted() { std::net::TcpStream::connect((\"127.0.0.1\", 9)); }\n",
    ] {
        assert_eq!(
            check(&(planted.to_owned() + source)),
            Err("production network reference")
        );
        assert_eq!(
            check(&(source.to_owned() + planted)),
            Err("production network reference")
        );
        let between = source.replacen(
            "pub(crate) fn run(",
            &(planted.to_owned() + "pub(crate) fn run("),
            1,
        );
        assert_ne!(between, source, "middle production insertion occurred");
        assert_eq!(check(&between), Err("production network reference"));
    }
    let without_attribute = source.replacen("#[cfg(test)]\nmod tests {", "mod tests {", 1);
    assert_eq!(
        check(&without_attribute),
        Err("incomplete test module boundary inventory")
    );
    let duplicate = source.to_owned() + "\n#[cfg(test)]\nmod tests {}\n";
    assert_eq!(
        check(&duplicate),
        Err("unexpected or duplicate test module boundary")
    );
    assert_eq!(
        check(&(source.to_owned() + "\n{")),
        Err("unbalanced source delimiters")
    );
    let fake = "const FAKE: &str = r#\"#[cfg(test)]\nmod tests {\"#;\n";
    assert_eq!(check(&(fake.to_owned() + source)), Ok(()));
    assert_eq!(
        check(
            &(fake.to_owned()
                + "fn planted() { std::net::TcpStream::connect((\"127.0.0.1\", 9)); }\n"
                + source)
        ),
        Err("production network reference")
    );
}

pub(super) fn verify(root: &Path) {
    let source = std::fs::read_to_string(root.join("crates/prismpm/src/verification.rs"))
        .expect("actual verification source");
    assert_eq!(
        check(&source),
        Ok(()),
        "offline production verification source"
    );
    controls(&source);
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACTUAL: &str = include_str!("../../../prismpm/src/verification.rs");

    #[test]
    fn actual_production_boundary_accepts_test_urls_and_retains_middle_code() {
        assert!(ACTUAL.contains("https://github.com/"));
        let production = production_source(ACTUAL).unwrap();
        assert!(production.contains("fn package_export_roots("));
        assert!(production.contains("fn run_process("));
        assert!(production.contains("pub(crate) fn run("));
        assert!(!production.contains("mod portable_oracle_tests"));
        assert!(!production.contains("mod tests"));
        assert_eq!(check(ACTUAL), Ok(()));
    }

    #[test]
    fn planted_production_network_and_invalid_boundaries_are_rejected() {
        controls(ACTUAL);
        let marker = "#[cfg(test)]\nmod tests {";
        let moved = ACTUAL.replacen(marker, "#[cfg(test)]\nfn planted() {", 1);
        assert_eq!(check(&moved), Err("invalid top-level test module boundary"));
        let missing_close = ACTUAL.trim_end().strip_suffix('}').unwrap();
        assert_eq!(check(missing_close), Err("unbalanced source delimiters"));
    }
}
