use browser_workspace_envelope_core_probe::{
    decodeWorkspaceEnvelope, encodeWorkspaceEnvelope, workspaceEnvelopeBytes, WorkspaceEnvelope,
};
use std::{error::Error, fs};

fn unhex(value: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if value.len() % 2 != 0 {
        return Err("odd hex".into());
    }
    (0..value.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&value[i..i + 2], 16).map_err(Into::into))
        .collect()
}
fn invoke(bytes: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
    workspaceEnvelopeBytes(bytes).map_err(|e| format!("generated codec: {e:?}").into())
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if let [mode, input] = args.as_slice() {
        if mode != "--query" {
            return Err("unknown mode".into());
        }
        for byte in invoke(unhex(input)?)? {
            print!("{byte:02x}");
        }
        println!();
        return Ok(());
    }
    if args.len() != 1 {
        return Err("corpus path".into());
    }
    let corpus = fs::read_to_string(&args[0])?;
    let mut count = 0;
    let mut valid: Option<WorkspaceEnvelope> = None;
    for line in corpus.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 3 {
            return Err("closed corpus row".into());
        }
        let input = unhex(fields[1])?;
        let expected = unhex(fields[2])?;
        for _ in 0..2 {
            assert_eq!(invoke(input.clone())?, expected, "{}", fields[0]);
        }
        if fields[0] == "RoundTripGenesis" {
            let candidate = decodeWorkspaceEnvelope(input[1..].to_vec())
                .map_err(|e| format!("generated decode: {e:?}"))?
                .ok_or("valid envelope")?;
            let encoded = encodeWorkspaceEnvelope(&candidate)
                .map_err(|e| format!("generated encode: {e:?}"))?
                .ok_or("valid encode")?;
            assert_eq!(encoded, input[1..]);
            for length in 0..encoded.len() {
                assert!(decodeWorkspaceEnvelope(encoded[..length].to_vec())
                    .map_err(|e| format!("truncated decode: {e:?}"))?
                    .is_none());
            }
            valid = Some(candidate);
        }
        println!("PASS {}", fields[0]);
        count += 1;
    }
    assert_eq!(count, 43);
    let valid = valid.ok_or("roundtrip fixture")?;
    for length in [0, 1, 64, 66] {
        let mut invalid = valid.clone();
        invalid.publicKey.resize(length, 4);
        assert!(encodeWorkspaceEnvelope(&invalid)
            .map_err(|e| format!("{e:?}"))?
            .is_none());
    }
    for prefix in [0, 2, 3, 5, 255] {
        let mut invalid = valid.clone();
        invalid.publicKey[0] = prefix;
        assert!(encodeWorkspaceEnvelope(&invalid)
            .map_err(|e| format!("{e:?}"))?
            .is_none());
    }
    for length in [0, 1, 63, 65] {
        let mut invalid = valid.clone();
        invalid.signature.resize(length, 0);
        assert!(encodeWorkspaceEnvelope(&invalid)
            .map_err(|e| format!("{e:?}"))?
            .is_none());
    }
    for bytes in [vec![], vec![0; 133], vec![0; 4231]] {
        let mut invalid = valid.clone();
        invalid.eventBytes = bytes;
        assert!(encodeWorkspaceEnvelope(&invalid)
            .map_err(|e| format!("{e:?}"))?
            .is_none());
    }
    println!("PASS all 43 modeled envelope vectors twice; every genesis prefix rejected; typed encoder rejects malformed fields");
    Ok(())
}
