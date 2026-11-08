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
//!
//! A served application is also run as a server: for every served vector the
//! program is started with `serve` on its declared loopback listener, the
//! vector's HTTP request is sent over a TCP connection, and the bytes read until
//! the connection closes must be the vector's response exactly. A fresh program
//! serves each vector, so the host state a vector sees does not depend on the
//! vectors before it.

use super::program_name;
use crate::error::PrismError;
use crate::holo::server_application::ServerApplication;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

/// Schema of the acceptance record.
pub const SCHEMA: &str = "prismpm/server-host-acceptance/1";
/// Transports exercised for every vector, in the order they run.
pub const TRANSPORTS: [&str; 2] = ["file", "stdin"];
/// The transport added when the declared binding serves the effects.
pub const BINDING_TRANSPORT: &str = "binding";
/// The transport added when the application is served over a loopback socket.
pub const SOCKET_TRANSPORT: &str = "socket";
/// How long a started server may take to say where it listens, and a served
/// exchange to complete, whatever the declared read time.
const PATIENCE: Duration = Duration::from_secs(20);

fn disagreement(message: impl Into<String>) -> PrismError {
    PrismError::new("PP6003", message)
}

/// The host script of one vector: its transcript steps, or nothing when the
/// entry performs no effect for it.
///
/// # Errors
/// Returns `PP2013` when a step names an undeclared effect.
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
                        "PP2013",
                        "a transcript step names an undeclared host effect",
                    )
                })?;
            for chunk in [&step.request, &step.outcome] {
                if chunk.len() > u32::MAX as usize {
                    return Err(PrismError::new("PP2013", "a transcript step is too long"));
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

/// A started `serve` program, stopped when dropped.
pub struct Server {
    child: Child,
    /// Where it listens.
    pub address: SocketAddr,
}

impl Server {
    /// Start `program serve` and wait for the line that says where it listens.
    ///
    /// # Errors
    /// Returns `PP6003` when the program cannot be started or does not listen.
    pub fn start(program: &Path) -> Result<Server, PrismError> {
        let mut child = Command::new(program)
            .arg("serve")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| disagreement(format!("{}: {error}", program.display())))?;
        let stdout = child.stdout.take().expect("the output is piped");
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let mut line = String::new();
            let _ = BufReader::new(stdout).read_line(&mut line);
            let _ = sender.send(line);
        });
        let line = receiver.recv_timeout(PATIENCE).unwrap_or_default();
        let address = line
            .strip_prefix("listening ")
            .and_then(|rest| rest.trim_end().parse::<SocketAddr>().ok());
        match address {
            Some(address) => Ok(Server { child, address }),
            None => {
                let _ = child.kill();
                let mut stderr = String::new();
                if let Some(mut pipe) = child.stderr.take() {
                    let _ = pipe.read_to_string(&mut stderr);
                }
                let _ = child.wait();
                Err(disagreement(format!(
                    "serve did not say where it listens: {line:?}, stderr {stderr:?}"
                )))
            }
        }
    }

    /// Send `request` over one connection and read the answer until the server closes it.
    ///
    /// # Errors
    /// Returns `PP6003` when the connection cannot be made.
    pub fn exchange(&self, request: &[u8]) -> Result<Vec<u8>, PrismError> {
        let mut stream = TcpStream::connect_timeout(&self.address, PATIENCE)
            .map_err(|error| disagreement(format!("{}: {error}", self.address)))?;
        stream
            .set_read_timeout(Some(PATIENCE))
            .and_then(|()| stream.set_write_timeout(Some(PATIENCE)))
            .map_err(|error| disagreement(error.to_string()))?;
        // The server may answer and close before it has read everything sent.
        let _ = stream.write_all(request);
        let mut response = Vec::new();
        let _ = stream.read_to_end(&mut response);
        Ok(response)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Run every served vector against the built `program` as a loopback server.
///
/// # Errors
/// Returns `PP6003` when the program does not listen or answers any vector with
/// other bytes than the modeled ones.
pub fn execute_served(
    application: &ServerApplication,
    program: &Path,
) -> Result<usize, PrismError> {
    let Some(serving) = application.serving.first() else {
        return Ok(0);
    };
    for (index, vector) in serving.vectors.iter().enumerate() {
        let server = Server::start(program)?;
        if !server.address.ip().is_loopback() {
            return Err(disagreement(format!(
                "the program listens on {}, which is not the loopback interface",
                server.address
            )));
        }
        let response = server.exchange(&vector.request)?;
        if response != vector.response {
            return Err(disagreement(format!(
                "served vector {index}: {} response bytes differ from the {} modeled ones",
                response.len(),
                vector.response.len()
            )));
        }
    }
    Ok(serving.vectors.len())
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
/// `PP2013` when `binding` is asked for and none is declared.
pub fn execute(
    application: &ServerApplication,
    program: &Path,
    work: &Path,
    binding: bool,
) -> Result<Value, PrismError> {
    if binding && application.bindings.is_empty() {
        return Err(PrismError::new(
            "PP2013",
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
    // An effect that only fills the request frame is exercised by the served vectors.
    let frame = application.frame_effects();
    if effects
        .iter()
        .any(|(name, steps)| *steps == 0 && !frame.contains(name.as_str()))
    {
        return Err(disagreement(
            "a declared host effect is exercised by no vector",
        ));
    }
    let served = execute_served(application, program)?;
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
            .chain((served > 0).then_some(SOCKET_TRANSPORT))
            .collect::<Vec<_>>(),
        "vectors": application.acceptance_vectors.len(),
        "served": served,
    }))
}
