use browser_workspace_query_core_probe::{
    queryDecodeProbeBytes, workspaceJournalBytes, workspaceQueryBytes,
};
use std::{error::Error, fs, time::Instant};
fn unhex(value: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if value.len() % 2 != 0 {
        return Err("odd hex".into());
    }
    (0..value.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&value[i..i + 2], 16).map_err(Into::into))
        .collect()
}
fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 1 {
        return Err("closed modeled TSV corpus".into());
    }
    let mut count = 0;
    for line in fs::read_to_string(&args[0])?.lines() {
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 3 {
            return Err("closed vector".into());
        }
        let input = unhex(fields[1])?;
        let expected = unhex(fields[2])?;
        let started = Instant::now();
        for _ in 0..2 {
            let actual = if fields[0].starts_with("Journal") {
                workspaceJournalBytes(input.clone())
            } else if fields[0].starts_with("Decode") {
                Ok(queryDecodeProbeBytes(input.clone()))
            } else {
                workspaceQueryBytes(input.clone())
            }
            .map_err(|e| format!("{}: {e:?}", fields[0]))?;
            assert_eq!(
                actual.len(),
                expected.len(),
                "{} length; prefix {:?}",
                fields[0],
                &actual[..actual.len().min(12)]
            );
            assert_eq!(actual, expected, "{} bytes", fields[0]);
        }
        println!("PASS {} {}ms", fields[0], started.elapsed().as_millis());
        count += 1;
    }
    println!("PASS {count} complete generated command vectors twice");
    Ok(())
}
