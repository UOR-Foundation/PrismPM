use browser_workspace_view_core_probe::{
    workspaceInteractionBytes, workspacePresentationBytes, workspaceViewLabelsBytes,
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
        return Err("one corpus".into());
    }
    if args[0] == "--labels" {
        print!("{}", String::from_utf8(workspaceViewLabelsBytes())?);
        return Ok(());
    }
    let mut count = 0;
    for line in fs::read_to_string(&args[0])?.lines() {
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 3 {
            return Err("closed corpus".into());
        }
        let input = unhex(fields[1])?;
        let expected = unhex(fields[2])?;
        let start = Instant::now();
        for _ in 0..2 {
            let actual = workspaceInteractionBytes(input.clone())
                .map_err(|e| format!("{}: {e:?}", fields[0]))?;
            assert_eq!(
                actual.len(),
                expected.len(),
                "{} length; prefix {:?}",
                fields[0],
                &actual[..actual.len().min(12)]
            );
            assert_eq!(actual, expected, "{} bytes", fields[0]);
            if input.first() == Some(&3) && input.len() >= 4 {
                let length =
                    ((input[1] as usize) << 16) | ((input[2] as usize) << 8) | input[3] as usize;
                if input.len() == length + 4 {
                    assert_eq!(
                        workspacePresentationBytes(input[4..].to_vec())
                            .map_err(|e| format!("{e:?}"))?,
                        expected,
                        "{} direct projector",
                        fields[0]
                    );
                }
            }
        }
        println!("PASS {} {}ms", fields[0], start.elapsed().as_millis());
        count += 1;
    }
    println!("PASS {count} complete generated View vectors twice");
    Ok(())
}
