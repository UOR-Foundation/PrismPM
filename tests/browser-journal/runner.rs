use browser_workspace_journal_core_probe::workspaceJournalBytes;
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
fn u24(value: usize) -> [u8; 3] {
    assert!(value < 1 << 24);
    [(value >> 16) as u8, (value >> 8) as u8, value as u8]
}
fn length(bytes: &[u8], at: usize) -> usize {
    usize::from(bytes[at]) * 65536 + usize::from(bytes[at + 1]) * 256 + usize::from(bytes[at + 2])
}
fn replay(path: &str) -> Result<(), Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    let header = lines
        .next()
        .ok_or("target")?
        .split('\t')
        .collect::<Vec<_>>();
    assert_eq!(header.len(), 3);
    assert_eq!(header[0], "Target");
    let target = unhex(header[1])?;
    let expected = unhex(header[2])?;
    let events = lines
        .map(|line| {
            let row = line.split('\t').collect::<Vec<_>>();
            assert_eq!(row.len(), 3);
            Ok((unhex(row[1])?, unhex(row[2])?))
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    assert_eq!(events.len(), 1024);
    assert_eq!(target.len(), 65574);
    assert_eq!(expected.len(), 1100427);
    let started = Instant::now();
    for replay_mode in [false, true] {
        let (mut head, mut state) = (Vec::new(), Vec::new());
        for (index, (object, envelope)) in events.iter().enumerate() {
            assert!(
                started.elapsed().as_secs() < 300,
                "bounded full generated journal replay"
            );
            let mut request = vec![if replay_mode { 3 } else { 1 }];
            if replay_mode {
                request.extend_from_slice(&u24(target.len()));
                request.extend_from_slice(&target);
            }
            request.extend_from_slice(&u24(head.len()));
            request.extend_from_slice(&u24(state.len()));
            request.extend_from_slice(&head);
            request.extend_from_slice(&state);
            request.extend_from_slice(object);
            request.extend_from_slice(envelope);
            let result =
                workspaceJournalBytes(request).map_err(|e| format!("event{index}: {e:?}"))?;
            assert_eq!(
                result[0],
                0,
                "mode={replay_mode} event={index} rejected {:?}",
                &result[..result.len().min(8)]
            );
            let h = length(&result, 1);
            let s = length(&result, 4);
            assert_eq!(result.len(), 7 + h + s);
            head = result[7..7 + h].to_vec();
            state = result[7 + h..].to_vec();
            assert_eq!(head.len(), 38 + (index + 1) * 64);
            assert_eq!(&head[..36], &target[..36]);
            assert_eq!(u16::from_be_bytes([head[36], head[37]]) as usize, index + 1);
            assert_eq!(&head[38..], &target[38..38 + (index + 1) * 64]);
        }
        assert_eq!(head, target);
        assert_eq!(
            state, expected,
            "full maximum state from generated transitions, no snapshot acceptance"
        );
        let mut finish = vec![4];
        finish.extend_from_slice(&u24(target.len()));
        finish.extend_from_slice(&u24(head.len()));
        finish.extend_from_slice(&u24(state.len()));
        finish.extend_from_slice(&target);
        finish.extend_from_slice(&head);
        finish.extend_from_slice(&state);
        let result = workspaceJournalBytes(finish).map_err(|e| format!("finish: {e:?}"))?;
        assert_eq!(result[0], 0);
        assert_eq!(&result[1..], expected);
        println!(
            "PASS full1024 {} exact state/head {}ms",
            if replay_mode { "replay" } else { "append" },
            started.elapsed().as_millis()
        );
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() == 2 && args[0] == "--replay" {
        return replay(&args[1]);
    }
    if args.len() != 1 {
        return Err("modeled TSV corpus".into());
    }
    let text = fs::read_to_string(&args[0])?;
    let mut count = 0;
    for line in text.lines() {
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 3 {
            return Err("closed vector".into());
        }
        let input = unhex(fields[1])?;
        let expected = unhex(fields[2])?;
        let started = Instant::now();
        for _ in 0..2 {
            let actual = workspaceJournalBytes(input.clone())
                .map_err(|e| format!("{}: {e:?}", fields[0]))?;
            assert_eq!(
                actual.len(),
                expected.len(),
                "{} length (actual prefix {:?})",
                fields[0],
                &actual[..actual.len().min(10)]
            );
            assert_eq!(actual, expected, "{} bytes", fields[0]);
            if actual.first() == Some(&0) && matches!(input.first(), Some(1 | 3)) {
                let h = length(&actual, 1);
                let mut request = vec![0];
                request.extend_from_slice(&actual[7..7 + h]);
                let closure = workspaceJournalBytes(request.clone())
                    .map_err(|error| format!("head closure: {error:?}"))?;
                assert_eq!(
                    closure, request,
                    "{} generated candidate-head closure",
                    fields[0]
                );
            }
        }
        println!("PASS {} {}ms", fields[0], started.elapsed().as_millis());
        count += 1;
    }
    println!("PASS {count} complete generated journal vectors twice");
    Ok(())
}
