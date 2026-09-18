use browser_workspace_core_probe::{
    decodeAuthenticatedEvent, decodeWorkspaceState, encodeWorkspaceState, reduceAuthenticatedEvent,
    reduceWorkspaceBytes, workspaceSigningPreimage, workspaceStateValid, AuthenticatedEvent,
    WorkspaceAction, WorkspaceState, WorkspaceTransition,
};
use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    time::{Duration, Instant},
};

fn unhex(value: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if value.len() % 2 != 0 {
        return Err("odd hex".into());
    }
    (0..value.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&value[i..i + 2], 16).map_err(Into::into))
        .collect()
}

fn accepted(transition: WorkspaceTransition) -> Result<WorkspaceState, Box<dyn Error>> {
    match transition {
        WorkspaceTransition::Accepted { field_0 } => Ok(field_0),
        WorkspaceTransition::Rejected { field_0 } => {
            Err(format!("replay rejected: {field_0:?}").into())
        }
    }
}

fn parts(input: &[u8]) -> (&[u8], &[u8]) {
    assert_eq!(&input[..4], &[0x50, 0x57, 0x52, 1]);
    let length =
        usize::from(input[4]) * 65536 + usize::from(input[5]) * 256 + usize::from(input[6]);
    (&input[7..7 + length], &input[7 + length..])
}

fn advance(
    state: &mut Option<WorkspaceState>,
    genesis: &AuthenticatedEvent,
    action: WorkspaceAction,
    body: Vec<u8>,
    seen: &[u8],
    started: Instant,
) -> Result<(), Box<dyn Error>> {
    assert!(
        started.elapsed() < Duration::from_secs(120),
        "bounded generated replay"
    );
    let current = state.as_ref().ok_or("replay genesis")?;
    let sequence = current.sequence + 1;
    let offset = usize::try_from(sequence)? * 32;
    let event = AuthenticatedEvent {
        workspace: genesis.workspace.clone(),
        eventId: seen[offset..offset + 32].to_vec(),
        parent: current.head.clone(),
        sequence,
        author: genesis.author.clone(),
        action,
        body,
    };
    *state = Some(accepted(
        reduceAuthenticatedEvent(state.take(), &event)
            .map_err(|error| format!("generated replay: {error:?}"))?,
    )?);
    Ok(())
}

fn replay(corpus: &str) -> Result<(), Box<dyn Error>> {
    let mut vectors = BTreeMap::new();
    for row in corpus.lines() {
        let fields: Vec<_> = row.split('\t').collect();
        if fields.len() != 3 {
            return Err("closed corpus row".into());
        }
        if vectors
            .insert(fields[0], (unhex(fields[1])?, unhex(fields[2])?))
            .is_some()
        {
            return Err("duplicate corpus case".into());
        }
    }
    let (positive, expected) = &vectors["CombinedMaximumStateControlAccepted"];
    let (before_bytes, final_bytes) = parts(positive);
    let before = decodeWorkspaceState(before_bytes.to_vec())
        .map_err(|error| format!("replay prestate: {error:?}"))?
        .ok_or("replay prestate")?;
    let final_event = decodeAuthenticatedEvent(final_bytes.to_vec())
        .map_err(|error| format!("replay final event: {error:?}"))?
        .ok_or("replay final event")?;
    let genesis = decodeAuthenticatedEvent(parts(&vectors["Genesis"].0).1.to_vec())
        .map_err(|error| format!("replay genesis: {error:?}"))?
        .ok_or("replay genesis")?;
    let maximum_message =
        decodeAuthenticatedEvent(parts(&vectors["MaximumMessageAccepted"].0).1.to_vec())
            .map_err(|error| format!("replay body: {error:?}"))?
            .ok_or("replay body")?
            .body;
    assert_eq!(
        (before.sequence, before.messageCount, before.members.len()),
        (1022, 256, 62 * 33)
    );
    assert_eq!(maximum_message.len(), 4096);
    assert_eq!(
        1 + 256 + 62 + 352 * 2,
        1023,
        "explicit prestate event ledger"
    );
    let started = Instant::now();
    let mut state = Some(accepted(
        reduceAuthenticatedEvent(None, &genesis)
            .map_err(|error| format!("replay genesis: {error:?}"))?,
    )?);
    for _ in 0..256 {
        advance(
            &mut state,
            &genesis,
            WorkspaceAction::PostMessage,
            maximum_message.clone(),
            &before.seen,
            started,
        )?;
    }
    for member in before.members.chunks_exact(33) {
        assert_eq!(member[32], 1);
        advance(
            &mut state,
            &genesis,
            WorkspaceAction::GrantContributor,
            member[..32].to_vec(),
            &before.seen,
            started,
        )?;
    }
    for _ in 0..352 {
        advance(
            &mut state,
            &genesis,
            WorkspaceAction::GrantContributor,
            final_event.body.clone(),
            &before.seen,
            started,
        )?;
        advance(
            &mut state,
            &genesis,
            WorkspaceAction::Revoke,
            final_event.body.clone(),
            &before.seen,
            started,
        )?;
    }
    assert_eq!(
        encodeWorkspaceState(state.as_ref().ok_or("replay state")?),
        before_bytes,
        "every prestate byte is reproduced from accepted generated transitions"
    );
    let after = accepted(
        reduceAuthenticatedEvent(state, &final_event)
            .map_err(|error| format!("replay final grant: {error:?}"))?,
    )?;
    assert_eq!(
        (after.sequence, after.messageCount, after.members.len()),
        (1023, 256, 63 * 33)
    );
    let mut response = vec![0];
    response.extend_from_slice(&encodeWorkspaceState(&after));
    assert_eq!(&response, expected);
    assert_eq!(response.len(), 1_100_428);
    let (full, rejection) = &vectors["CombinedMaximumStateRejectedAtEventLimit"];
    assert_eq!(
        parts(full).0,
        &response[1..],
        "cap fixture uses the reached full state"
    );
    assert_eq!(
        &reduceWorkspaceBytes(full.clone()).map_err(|error| format!("replay cap: {error:?}"))?,
        rejection
    );
    assert!(
        started.elapsed() < Duration::from_secs(120),
        "bounded generated replay"
    );
    println!("PASS generated replay: 1024 accepted events, exact prestate, maximum 1100428-byte response, event-cap rejection, {}ms", started.elapsed().as_millis());
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = std::env::args().skip(1);
    let path = arguments.next().ok_or("modeled TSV corpus")?;
    let corpus = fs::read_to_string(path)?;
    if let Some(mode) = arguments.next() {
        if mode != "--replay" || arguments.next().is_some() {
            return Err("unknown mode".into());
        }
        return replay(&corpus);
    }
    let mut count = 0;
    for row in corpus.lines() {
        let fields: Vec<_> = row.split('\t').collect();
        if fields.len() != 3 {
            return Err("closed corpus row".into());
        }
        let input = unhex(fields[1])?;
        let expected = unhex(fields[2])?;
        let started = Instant::now();
        let first =
            reduceWorkspaceBytes(input.clone()).map_err(|e| format!("{}: {e:?}", fields[0]))?;
        assert_eq!(first, expected, "{} expected canonical response", fields[0]);
        if first.first() == Some(&0) {
            let state = decodeWorkspaceState(first[1..].to_vec())
                .map_err(|e| format!("state decode: {e:?}"))?
                .ok_or("accepted state framing")?;
            assert!(
                workspaceStateValid(&state).map_err(|e| format!("state validation: {e:?}"))?,
                "accepted transition preserves invariant"
            );
        }
        if fields[0] == "Genesis" {
            let mut event = decodeAuthenticatedEvent(input[7..].to_vec())
                .map_err(|e| format!("event decode: {e:?}"))?
                .ok_or("genesis event")?;
            let context = b"prismpm/workspace-event/1";
            let mut preimage = b"prismpm/browser-signature/1\0".to_vec();
            preimage.extend_from_slice(&(context.len() as u16).to_be_bytes());
            preimage.extend_from_slice(context);
            preimage.extend_from_slice(&input[7..41]);
            preimage.extend_from_slice(&input[73..]);
            assert_eq!(
                workspaceSigningPreimage(&event),
                preimage,
                "exact host domain and omitted event ID"
            );
            event.eventId[0] ^= 1;
            assert_eq!(
                workspaceSigningPreimage(&event),
                preimage,
                "event digest is not self-hashed"
            );
            event.author[0] ^= 1;
            assert_ne!(
                workspaceSigningPreimage(&event),
                preimage,
                "author is signed"
            );
        }
        let repeated =
            reduceWorkspaceBytes(input).map_err(|e| format!("{} repeated: {e:?}", fields[0]))?;
        assert_eq!(repeated, first, "{} determinism", fields[0]);
        println!("PASS {} {}ms", fields[0], started.elapsed().as_millis());
        count += 1;
    }
    assert_eq!(count, 45, "complete modeled corpus");
    println!("PASS all {count} generated production reducer cases, twice each");
    Ok(())
}
