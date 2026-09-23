use browser_operation_journal_core_probe::{
    effectWireBytes, fixtureEchoBytes, journalPartitionBytes, journalWireBytes,
};
use std::{error::Error, fs};

fn unhex(text: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if text.len() % 2 != 0 {
        return Err("odd hex".into());
    }
    (0..text.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&text[at..at + 2], 16).map_err(Into::into))
        .collect()
}
fn execute(name: &str, input: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
    let result = if name.starts_with("Guest") {
        Ok(fixtureEchoBytes(input))
    } else if name.starts_with("Effects") {
        effectWireBytes(input)
    } else if name.starts_with("Partition") {
        journalPartitionBytes(input)
    } else {
        journalWireBytes(input)
    };
    result.map_err(|error| format!("{name}: {error:?}").into())
}
fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if let [mode, name, input, expected] = args.as_slice() {
        if mode != "--binary" {
            return Err("closed binary mode".into());
        }
        let input = fs::read(input)?;
        let expected = fs::read(expected)?;
        for _ in 0..2 {
            assert!(
                execute(name, input.clone())? == expected,
                "binary native output mismatch"
            );
        }
        println!("PASS binary complete journal vector twice");
        return Ok(());
    }
    let [file] = args.as_slice() else {
        return Err("one complete corpus".into());
    };
    let mut count = 0;
    for line in fs::read_to_string(file)?.lines() {
        let fields = line.split('\t').collect::<Vec<_>>();
        let [name, input, expected] = fields.as_slice() else {
            return Err("closed corpus".into());
        };
        let input = unhex(input)?;
        let expected = unhex(expected)?;
        for _ in 0..2 {
            assert!(
                execute(name, input.clone())? == expected,
                "{name} native output mismatch"
            );
        }
        println!("PASS {name}");
        count += 1;
    }
    println!("PASS {count} complete journal vectors twice");
    Ok(())
}
