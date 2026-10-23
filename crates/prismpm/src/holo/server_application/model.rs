//! Closed source-owned declaration of a server application and its host effects.

use super::super::model_document::ApplicationAcceptanceVector;
use serde::{Deserialize, Serialize};

/// Server application profile: a native LexLean production core that
/// requests a closed, typed set of host effects. Nothing here grants an effect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerApplication {
    /// Exact profile discriminator.
    pub profile: String,
    /// Human-readable package name.
    pub name: String,
    /// Generated Cargo package name of the application core.
    pub cargo_name: String,
    /// Exact stable Cargo version.
    pub cargo_version: String,
    /// Bounded package description.
    pub cargo_description: String,
    /// HTTPS source repository metadata.
    pub cargo_repository: String,
    /// HTTPS homepage metadata, not a runtime network permission.
    pub cargo_homepage: String,
    /// The LexLean production package that is the pure entry of the application.
    pub core: ServerCore,
    /// Finite exchange vectors: one request in, one response out.
    pub acceptance_vectors: Vec<ApplicationAcceptanceVector>,
    /// Maximum request bytes of one exchange.
    pub request_maximum: u32,
    /// Maximum response bytes of one exchange.
    pub response_maximum: u32,
    /// The host binding crate: empty exactly when no effect is declared, else one.
    pub bindings: Vec<ServerBinding>,
    /// The host effect protocol of the core.
    pub host: HostProtocol,
}

/// The core of a server application: a LexLean production root exported as a
/// certified `rust-std` package whose `run` takes the turn as `Bytes` and
/// returns the step as `List<u64>` or fails with `Overflow`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerCore {
    /// Fully qualified Lean name of the production root.
    pub root: String,
    /// Lowercase snake-case crate name of the exported package.
    pub package: String,
    /// Exported package version: three canonical decimals.
    pub version: String,
}

/// The binding crate that implements the generated host trait: a member of the
/// consumer's own Cargo workspace, named by its path, never discovered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerBinding {
    /// Directory of the crate, relative to the workspace root, `/`-separated,
    /// with no `.` or `..` component.
    pub path: String,
    /// Package name in the crate's own manifest.
    pub crate_name: String,
    /// Rust path of the type in that crate that implements the host trait and
    /// `Default`; its first segment is the crate's library name.
    pub host: String,
}

/// The closed host effect protocol and the process surface of its program.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostProtocol {
    /// Exact versioned protocol discriminator.
    pub protocol: String,
    /// Purpose paragraph printed first by `--help`.
    pub description: String,
    /// Exit status of every usage failure.
    pub usage_code: u8,
    /// Exit status of every input or output failure.
    pub io_code: u8,
    /// Exit status of every fault: limits, refusals, host and script failures.
    pub fault_code: u8,
    /// Most effects one exchange may perform; zero exactly when no effect is declared.
    pub effect_budget: u32,
    /// The closed sum of effects; an effect's wire tag is its position.
    pub effects: Vec<HostEffect>,
    /// Effect scripts of acceptance vectors, by vector index.
    pub transcripts: Vec<HostTranscript>,
}

/// Value kinds of effect requests and results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostKind {
    /// Unsigned 32-bit big-endian integer.
    U32,
    /// Unsigned 64-bit big-endian integer.
    U64,
    /// One byte, zero or one.
    Flag,
    /// Opaque bytes with a 32-bit length prefix.
    Bytes,
    /// UTF-8 text with a 32-bit length prefix.
    Text,
}

/// A named, typed field of an effect request or result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostField {
    /// Lowercase snake-case field name.
    pub name: String,
    /// Value kind.
    pub kind: HostKind,
}

/// One declared effect: typed request, typed result, closed failures and bounds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostEffect {
    /// Unique dotted lowercase name, for example `store.get`.
    pub name: String,
    /// Request fields in wire order.
    pub request: Vec<HostField>,
    /// Result fields in wire order.
    pub result: Vec<HostField>,
    /// Closed failure names; a failure's wire index is its position.
    pub failures: Vec<String>,
    /// Largest encoded request payload in bytes.
    pub request_maximum: u32,
    /// Largest encoded outcome (status word, then result or failure) in bytes.
    pub result_maximum: u32,
}

/// The effects the core must request for one acceptance vector, in order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostTranscript {
    /// Index into `acceptance_vectors`.
    pub vector: u32,
    /// The requested effects and the host's outcome for each.
    pub steps: Vec<HostStep>,
}

/// One scripted effect: the expected encoded request and the encoded outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostStep {
    /// Declared effect name.
    pub effect: String,
    /// Encoded request payload the root must produce.
    pub request: Vec<u8>,
    /// Encoded outcome the host returns.
    pub outcome: Vec<u8>,
}

impl HostEffect {
    /// Smallest encoded size of a record of `fields`.
    pub fn minimum_size(fields: &[HostField]) -> u64 {
        fields
            .iter()
            .map(|field| match field.kind {
                HostKind::U32 | HostKind::Bytes | HostKind::Text => 4,
                HostKind::U64 => 8,
                HostKind::Flag => 1,
            })
            .sum()
    }
}

impl ServerApplication {
    /// Longest step the entry may return: the done or perform word, a tag, and
    /// the larger of the response and any effect request.
    pub fn step_maximum(&self) -> u64 {
        let request = self
            .host
            .effects
            .iter()
            .map(|effect| u64::from(effect.request_maximum))
            .max()
            .unwrap_or(0);
        8 + u64::from(self.response_maximum).max(request)
    }

    /// Largest input the core can be given: the request alone without
    /// effects, otherwise the largest turn.
    pub fn input_maximum(&self) -> u64 {
        if self.host.effects.is_empty() {
            u64::from(self.request_maximum)
        } else {
            self.turn_maximum()
        }
    }

    /// Largest turn the core can be given: the request, the effect count
    /// and the most results the budget allows. Meaningful only with effects.
    pub fn turn_maximum(&self) -> u64 {
        let result = self
            .host
            .effects
            .iter()
            .map(|effect| u64::from(effect.result_maximum))
            .max()
            .unwrap_or(0);
        u64::from(self.request_maximum) + 8 + u64::from(self.host.effect_budget) * (8 + result)
    }

    /// Largest replay script of the effect budget: per step, the effect tag, two
    /// lengths, the request and the outcome. Meaningful only with effects.
    pub fn script_maximum(&self) -> u64 {
        let step = self
            .host
            .effects
            .iter()
            .map(|effect| 12 + u64::from(effect.request_maximum) + u64::from(effect.result_maximum))
            .max()
            .unwrap_or(0);
        u64::from(self.host.effect_budget) * step
    }
}
