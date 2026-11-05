//! Project-free, bounded transport over the pinned LexLean authoring SDK.
//! This reads/writes module values, not linked snapshots or proof evidence.
use crate::error::{PrismError, PrismSpan};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{Read, Write};

pub(crate) const DEFAULT_INPUT_BYTES: u64 = 16 * 1024 * 1024;
pub(crate) const DEFAULT_OUTPUT_BYTES: u64 = 64 * 1024 * 1024;
const REQUEST_SCHEMA: &str = crate::contracts::SOURCE_AUTHORING_REQUEST_ID;
const RESULT_SCHEMA: &str = crate::contracts::SOURCE_AUTHORING_RESULT_ID;
const MESSAGE_BYTES: usize = 2048;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Operation {
    Read,
    Write,
    ToNotation,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadRequest {
    schema: String,
    source: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WriteRequest {
    schema: String,
    source: String,
    value: Value,
}
#[derive(Serialize)]
struct ReadResult {
    operation: &'static str,
    schema: &'static str,
    value: Value,
}
#[derive(Serialize)]
struct TextResult {
    operation: &'static str,
    schema: &'static str,
    source: String,
}

fn bounded(message: impl std::fmt::Display) -> String {
    let mut message = message.to_string();
    if message.len() > MESSAGE_BYTES {
        let mut end = MESSAGE_BYTES;
        while !message.is_char_boundary(end) {
            end -= 1;
        }
        message.truncate(end);
        message.push_str(" [truncated]");
    }
    message
}
fn bad_request(message: impl std::fmt::Display) -> PrismError {
    PrismError::new("PP5801", bounded(message))
}
fn addressable(value: u64) -> Result<usize, PrismError> {
    let size =
        usize::try_from(value).map_err(|_| bad_request("source byte limit is not addressable"))?;
    if value == 0 || isize::try_from(size).is_err() || size.checked_add(1).is_none() {
        return Err(bad_request(
            "source byte limits must be positive and addressable",
        ));
    }
    Ok(size)
}
fn source_error(source: &str, error: lexlean::sdk::AuthoringError) -> PrismError {
    let mut result = PrismError::new("PP5802", bounded(&error.message));
    if let Some(offset) = error.offset {
        result = result.with_note(format!("LexLean source UTF-8 byte offset: {offset}"));
        if offset <= source.len() && source.is_char_boundary(offset) {
            let before = &source[..offset];
            let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
            let column = before
                .rsplit('\n')
                .next()
                .map_or(1, |line| line.chars().count() + 1);
            result.primary = Some(PrismSpan {
                path: "stdin-source".to_owned(),
                byte_start: offset,
                byte_end: offset,
                line_start: line,
                line_end: line,
                column_start: column,
                column_end: column,
            });
        }
    }
    result
}
fn read_input(mut input: impl Read, limit: usize) -> Result<Vec<u8>, PrismError> {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let remaining = limit
            .checked_sub(bytes.len())
            .expect("bounded input length");
        let count = buffer.len().min(remaining + 1);
        let size = match input.read(&mut buffer[..count]) {
            Ok(size) => size,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => {
                return Err(PrismError::new(
                    "PP5804",
                    bounded(format!("source stdin read failed: {error}")),
                ))
            }
        };
        if size == 0 {
            return Ok(bytes);
        }
        if size > remaining {
            return Err(PrismError::new(
                "PP5803",
                "source request exceeds --max-input-bytes",
            ));
        }
        bytes
            .try_reserve(size)
            .map_err(|_| PrismError::new("PP5803", "source input allocation refused"))?;
        bytes.extend_from_slice(&buffer[..size]);
    }
}
#[derive(Clone, Copy)]
enum Refusal {
    Limit,
    Allocation,
}
struct Staged {
    bytes: Vec<u8>,
    limit: usize,
    refusal: Option<Refusal>,
}
impl Staged {
    fn reserve(&mut self, additional: usize) -> std::io::Result<()> {
        self.bytes.try_reserve(additional).map_err(|_| {
            self.refusal = Some(Refusal::Allocation);
            std::io::Error::other("source result allocation refused")
        })
    }
}
impl Write for Staged {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            self.refusal = Some(Refusal::Limit);
            return Err(std::io::Error::other("source result byte limit"));
        }
        self.reserve(bytes.len())?;
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn encode(value: &impl Serialize, limit: usize) -> Result<Vec<u8>, PrismError> {
    let mut output = Staged {
        bytes: Vec::new(),
        limit,
        refusal: None,
    };
    // The source value is NOT the typed/canonical model metadata format.
    // Preserve every SDK-accepted JSON Value, including floating-point numbers.
    let encoded = serde_json::to_writer(&mut output, value).map_err(bounded);
    let finished = encoded.and_then(|()| output.write_all(b"\n").map_err(bounded));
    if let Err(message) = finished {
        return Err(match output.refusal {
            Some(Refusal::Limit) => PrismError::new(
                "PP5803",
                "full encoded source result including LF exceeds --max-output-bytes",
            ),
            Some(Refusal::Allocation) => {
                PrismError::new("PP5803", "source result allocation refused")
            }
            None => PrismError::new("PP5801", message),
        });
    }
    Ok(output.bytes)
}

pub(crate) fn prepare(
    operation: Operation,
    input_limit: u64,
    output_limit: u64,
    input: impl Read,
) -> Result<Vec<u8>, PrismError> {
    let input_limit = addressable(input_limit)?;
    let output_limit = addressable(output_limit)?;
    let bytes = read_input(input, input_limit)?;
    match operation {
        Operation::Read | Operation::ToNotation => {
            let request: ReadRequest = serde_json::from_slice(&bytes).map_err(bad_request)?;
            if request.schema != REQUEST_SCHEMA {
                return Err(bad_request("unsupported source-authoring request schema"));
            }
            drop(bytes);
            if matches!(operation, Operation::Read) {
                let mut value = lexlean::sdk::read_module(&request.source)
                    .map_err(|error| source_error(&request.source, error))?;
                value.sort_all_objects();
                encode(
                    &ReadResult {
                        operation: "read",
                        schema: RESULT_SCHEMA,
                        value,
                    },
                    output_limit,
                )
            } else {
                let source = lexlean::sdk::to_notation(&request.source)
                    .map_err(|error| source_error(&request.source, error))?;
                encode(
                    &TextResult {
                        operation: "to-notation",
                        schema: RESULT_SCHEMA,
                        source,
                    },
                    output_limit,
                )
            }
        }
        Operation::Write => {
            let request: WriteRequest = serde_json::from_slice(&bytes).map_err(bad_request)?;
            if request.schema != REQUEST_SCHEMA {
                return Err(bad_request("unsupported source-authoring request schema"));
            }
            drop(bytes);
            let source = lexlean::sdk::write_module(&request.source, &request.value)
                .map_err(|error| source_error(&request.source, error))?;
            encode(
                &TextResult {
                    operation: "write",
                    schema: RESULT_SCHEMA,
                    source,
                },
                output_limit,
            )
        }
    }
}

pub(crate) fn publish(bytes: &[u8], mut output: impl Write) -> Result<(), PrismError> {
    output
        .write_all(bytes)
        .and_then(|()| output.flush())
        .map_err(|error| {
            PrismError::new(
                "PP5804",
                bounded(format!("source stdout write or flush failed: {error}")),
            )
            .with_note("stdout outcome is Unknown; previously written bytes are not rolled back")
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Failed {
        remaining: usize,
        flushed: bool,
    }
    impl Write for Failed {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.remaining == 0 {
                return Err(std::io::Error::other("injected write failure"));
            }
            let count = bytes.len().min(self.remaining);
            self.remaining -= count;
            Ok(count)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.flushed = true;
            Err(std::io::Error::other("injected flush failure"))
        }
    }
    #[test]
    fn stdout_short_write_and_flush_failures_are_unknown_without_retry() {
        let mut partial = Failed {
            remaining: 2,
            flushed: false,
        };
        let error = publish(b"complete result", &mut partial).unwrap_err();
        assert_eq!(error.code, "PP5804");
        assert!(!partial.flushed);
        assert!(error.notes[0].message.contains("Unknown"));
        let mut flush = Failed {
            remaining: 64,
            flushed: false,
        };
        let error = publish(b"complete result", &mut flush).unwrap_err();
        assert_eq!(error.code, "PP5804");
        assert!(flush.flushed);
        assert!(error.notes[0].message.contains("Unknown"));
    }
    #[test]
    fn write_zero_is_a_transport_failure_without_flush_or_success() {
        struct Zero(bool);
        impl Write for Zero {
            fn write(&mut self, _bytes: &[u8]) -> std::io::Result<usize> {
                Ok(0)
            }
            fn flush(&mut self) -> std::io::Result<()> {
                self.0 = true;
                Ok(())
            }
        }
        let mut output = Zero(false);
        let error = publish(b"complete result", &mut output).unwrap_err();
        assert_eq!(error.code, "PP5804");
        assert!(!output.0);
        assert!(error.notes[0].message.contains("Unknown"));
    }
    #[test]
    fn interrupted_input_retries_and_other_input_io_failure_is_typed() {
        struct Interrupted {
            first: bool,
            bytes: std::io::Cursor<Vec<u8>>,
        }
        impl Read for Interrupted {
            fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
                if self.first {
                    self.first = false;
                    Err(std::io::Error::from(std::io::ErrorKind::Interrupted))
                } else {
                    self.bytes.read(output)
                }
            }
        }
        let source = "\\begin{semanticmodule}\n\\semanticdata{42}\n\\end{semanticmodule}";
        let input =
            serde_json::to_vec(&serde_json::json!({"schema":REQUEST_SCHEMA,"source":source}))
                .unwrap();
        let result = prepare(
            Operation::Read,
            4096,
            4096,
            Interrupted {
                first: true,
                bytes: std::io::Cursor::new(input),
            },
        )
        .unwrap();
        let result: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(result["value"], 42);
        struct FailedRead;
        impl Read for FailedRead {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("injected read failure"))
            }
        }
        let error = prepare(Operation::Read, 4096, 4096, FailedRead).unwrap_err();
        assert_eq!(error.code, "PP5804");
    }
    #[test]
    fn full_serialized_envelope_and_lf_are_in_the_output_budget() {
        let value = ReadResult {
            operation: "read",
            schema: RESULT_SCHEMA,
            value: serde_json::json!({"a":null,"b":[],"float":1.25}),
        };
        let bytes = encode(&value, 4096).unwrap();
        assert_eq!(bytes.last(), Some(&b'\n'));
        assert_eq!(encode(&value, bytes.len()).unwrap(), bytes);
        assert_eq!(encode(&value, bytes.len() - 1).unwrap_err().code, "PP5803");
    }
    #[test]
    fn actual_capacity_overflow_is_a_known_resource_refusal() {
        let mut output = Staged {
            bytes: Vec::new(),
            limit: usize::MAX,
            refusal: None,
        };
        assert!(output.reserve(usize::MAX).is_err());
        assert!(matches!(output.refusal, Some(Refusal::Allocation)));
        assert!(matches!(addressable(u64::MAX),Err(error) if error.code=="PP5801"));
        assert_eq!(addressable(0).unwrap_err().code, "PP5801");
    }
}
