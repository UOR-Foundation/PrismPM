//! Acceptance of a built host program against the modeled exchange corpus.
//!
//! Every acceptance vector is executed through the program's `exchange`
//! command with the vector's transcript as the host script, once over a request
//! file and once over standard input. The response must be byte-identical to the
//! modeled one, the program must exit zero, and every transcript step must be
//! replayed. The record names each declared effect with the number of steps
//! that exercised it, so an effect no vector reaches cannot be reported verified.
//!
//! When the caller links a deterministic binding, every vector is also run
//! through the program's `run` command, which performs the effects through the
//! declared binding with no script: the response must be the modeled one again.

use super::program_name;
use crate::error::PrismError;
use crate::holo::server_application::ServerApplication;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// Schema of the acceptance record.
pub const SCHEMA: &str = "prismpm/server-host-acceptance/1";
/// Transports exercised for every vector, in the order they run.
pub const TRANSPORTS: [&str; 2] = ["file", "stdin"];
/// The transport added when the declared binding serves the effects.
pub const BINDING_TRANSPORT: &str = "binding";

fn disagreement(message: impl Into<String>) -> PrismError {
    PrismError::new("PP6003", message)
}

/// The host script of one vector: its transcript steps, or nothing when the
/// entry performs no effect for it.
///
/// # Errors
/// Returns `PP2012` when a step names an undeclared effect.
pub fn script_bytes(application: &ServerApplication, vector: usize) -> Result<Vec<u8>, PrismError> {
    let mut script = Vec::new();
    for transcript in &application.host.transcripts {
        if usize::try_from(transcript.vector).ok() != Some(vector) {
            continue;
        }
        for step in &transcript.steps {
            let tag = application
                .host
                .effects
                .iter()
                .position(|effect| effect.name == step.effect)
                .ok_or_else(|| {
                    PrismError::new(
                        "PP2012",
                        "a transcript step names an undeclared host effect",
                    )
                })?;
            for chunk in [&step.request, &step.outcome] {
                if chunk.len() > u32::MAX as usize {
                    return Err(PrismError::new("PP2012", "a transcript step is too long"));
                }
            }
            script.extend_from_slice(
                &u32::try_from(tag)
                    .expect("at most 64 effects")
                    .to_be_bytes(),
            );
            for chunk in [&step.request, &step.outcome] {
                script.extend_from_slice(&(chunk.len() as u32).to_be_bytes());
                script.extend_from_slice(chunk);
            }
        }
    }
    Ok(script)
}

/// Steps recorded per declared effect, in declaration order.
fn exercised(application: &ServerApplication) -> Vec<(String, usize)> {
    application
        .host
        .effects
        .iter()
        .map(|effect| {
            let steps = application
                .host
                .transcripts
                .iter()
                .flat_map(|transcript| &transcript.steps)
                .filter(|step| step.effect == effect.name)
                .count();
            (effect.name.clone(), steps)
        })
        .collect()
}

fn run(
    program: &Path,
    arguments: &[&Path],
    stdin: Option<&[u8]>,
) -> Result<(i32, Vec<u8>, String), PrismError> {
    let mut child = Command::new(program)
        .args(arguments)
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| disagreement(format!("{}: {error}", program.display())))?;
    if let (Some(bytes), Some(mut pipe)) = (stdin, child.stdin.take()) {
        // The program may exit before reading all input; its exit status decides.
        let _ = pipe.write_all(bytes);
    }
    let output = child
        .wait_with_output()
        .map_err(|error| disagreement(format!("{}: {error}", program.display())))?;
    Ok((
        output.status.code().unwrap_or(-1),
        output.stdout,
        String::from_utf8_lossy(&output.stderr).into_owned(),
    ))
}

/// Execute every vector against the built `program` and return the record.
/// With `binding`, each vector also runs through the linked binding, which must
/// be deterministic enough to reproduce the modeled response.
///
/// # Errors
/// Returns `PP6003` when the program disagrees with the model for any vector
/// or transport, `PP4002` when the work directory cannot be written and
/// `PP2012` when `binding` is asked for and none is declared.
pub fn execute(
    application: &ServerApplication,
    program: &Path,
    work: &Path,
    binding: bool,
) -> Result<Value, PrismError> {
    if binding && application.bindings.is_empty() {
        return Err(PrismError::new(
            "PP2012",
            "the acceptance asks for a host binding and none is declared",
        ));
    }
    let program_bytes = std::fs::read(program)
        .map_err(|error| PrismError::new("PP5008", format!("{}: {error}", program.display())))?;
    for (index, vector) in application.acceptance_vectors.iter().enumerate() {
        let script_path = work.join(format!("vector-{index}.script"));
        let request_path = work.join(format!("vector-{index}.request"));
        for (path, bytes) in [
            (&script_path, script_bytes(application, index)?),
            (&request_path, vector.request.clone()),
        ] {
            std::fs::write(path, bytes).map_err(|error| {
                PrismError::new("PP4002", format!("{}: {error}", path.display()))
            })?;
        }
        let exchange = Path::new("exchange");
        let dash = Path::new("-");
        for (transport, arguments, stdin) in [
            (
                TRANSPORTS[0],
                vec![exchange, script_path.as_path(), request_path.as_path()],
                None,
            ),
            (
                TRANSPORTS[1],
                vec![exchange, script_path.as_path(), dash],
                Some(vector.request.as_slice()),
            ),
        ] {
            let (code, stdout, stderr) = run(program, &arguments, stdin)?;
            if code != 0 || stdout != vector.response {
                return Err(disagreement(format!(
                    "vector {index} over {transport}: exit {code}, {} response bytes, stderr {stderr:?}",
                    stdout.len()
                )));
            }
        }
        if binding {
            let arguments = [Path::new("run"), request_path.as_path()];
            let (code, stdout, stderr) = run(program, &arguments, None)?;
            if code != 0 || stdout != vector.response {
                return Err(disagreement(format!(
                    "vector {index} over {BINDING_TRANSPORT}: exit {code}, {} response bytes, stderr {stderr:?}",
                    stdout.len()
                )));
            }
        }
    }
    let effects = exercised(application);
    if effects.iter().any(|(_, steps)| *steps == 0) {
        return Err(disagreement(
            "a declared host effect is exercised by no vector",
        ));
    }
    Ok(json!({
        "effects": effects
            .iter()
            .map(|(name, steps)| json!({"name": name, "steps": steps}))
            .collect::<Vec<_>>(),
        "program": program_name(application),
        "program_sha256": format!("{:x}", Sha256::digest(&program_bytes)),
        "protocol": application.host.protocol,
        "schema": SCHEMA,
        "status": "verified",
        "steps": effects.iter().map(|(_, steps)| steps).sum::<usize>(),
        "binding": application.bindings.first().filter(|_| binding).map(|linked| linked.crate_name.as_str()),
        "transports": TRANSPORTS
            .iter()
            .copied()
            .chain(binding.then_some(BINDING_TRANSPORT))
            .collect::<Vec<_>>(),
        "vectors": application.acceptance_vectors.len(),
    }))
}
