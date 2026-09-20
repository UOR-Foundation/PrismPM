//! Closed source-owned declaration; not an executable browser service.

use super::super::model_document::ApplicationAcceptanceVector;
use serde::{Deserialize, Serialize};

/// Browser orchestration profile, kept distinct from portable applications.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowserApplication {
    /// Exact profile discriminator.
    pub profile: String,
    /// Human-readable package name.
    pub name: String,
    /// Generated Cargo package name.
    pub cargo_name: String,
    /// Exact stable Cargo version.
    pub cargo_version: String,
    /// Bounded package description.
    pub cargo_description: String,
    /// HTTPS source repository metadata.
    pub cargo_repository: String,
    /// HTTPS homepage metadata, not a runtime network permission.
    pub cargo_homepage: String,
    /// Strictly ordered source-owned byte entry points.
    pub library_roots: Vec<String>,
    /// Finite byte vectors, not runtime or service acceptance.
    pub acceptance_vectors: Vec<ApplicationAcceptanceVector>,
    /// Generated application-session entry point.
    pub entry_root: String,
    /// Import-free Hologram Core-Wasm contract.
    pub core_contract: String,
    /// Maximum private application request bytes.
    pub request_maximum: u32,
    /// Maximum private application response bytes.
    pub response_maximum: u32,
    /// Maximum primary input allocation in bytes.
    pub guest_allocation_maximum: u32,
    /// Declared fresh-instance Wasm memory ceiling in pages.
    pub memory_pages: u32,
    /// True for the primary's native Holo capability request only.
    pub capabilities_empty: bool,
    /// Required composed-archive intent.
    pub fat_archive: bool,
    /// Primary layer index, fixed at zero.
    pub primary_layer: u8,
    /// Required browser View layer index, fixed at one.
    pub view_layer: u8,
    /// Exact versioned protocol discriminator.
    pub protocol: String,
    /// Immutable requests, never effective host grants.
    pub requested_effects: Vec<RequestedEffect>,
    /// Mandatory durable single-operation recovery declaration.
    pub durability: Durability,
    /// Source-owned safe presentation declaration.
    pub view: BrowserView,
}

/// Requested resource policy. This is never an effective grant or a credential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestedEffect {
    /// Unique logical resource identifier.
    pub resource: String,
    /// Closed requested primitive and its finite bounds.
    pub adapter: RequestedAdapter,
}

/// Closed primitives admitted by the declaration, not implemented custody APIs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum RequestedAdapter {
    /// Generated import-free guest bound to a source root.
    Guest {
        /// Generated application-session entry point.
        entry_root: String,
        /// Exact versioned protocol discriminator.
        protocol: String,
        /// Maximum guest request bytes.
        input_maximum: u32,
        /// Maximum guest response bytes.
        output_maximum: u32,
        /// Declared fresh-instance Wasm memory ceiling in pages.
        memory_pages: u32,
    },
    /// Bounded secure randomness request.
    Random {
        /// Finite byte bound.
        maximum: u32,
    },
    /// Bounded SHA-256 request.
    Digest {
        /// Finite byte bound.
        maximum: u32,
    },
    /// Signature request using independently possessed private custody.
    Sign {
        /// Logical private custody slot; never a caller-supplied key.
        credential_slot: String,
        /// Exact signature domain-separation context.
        context: String,
        /// Finite byte bound.
        maximum: u32,
    },
    /// Signature verification is not authentication or authorization.
    Verify {
        /// Exact signature domain-separation context.
        context: String,
        /// Finite byte bound.
        maximum: u32,
    },
    /// Bounded local content-addressed storage request.
    Store {
        /// Immutable app-local storage compartment; not organization authority.
        namespace: String,
        /// Maximum immutable object size in bytes.
        max_object_bytes: u32,
        /// Maximum retained immutable object count.
        max_objects: u32,
        /// Maximum retained head count.
        max_heads: u32,
    },
}

/// Mandatory durable single-operation recovery contract; no consensus claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Durability {
    /// Exact versioned protocol discriminator.
    pub protocol: String,
    /// Private journal resource, distinct from every application resource.
    pub resource: String,
    /// Private namespace, distinct from every application storage grant.
    pub namespace: String,
    /// Immutable operation-journal head name.
    pub head: String,
    /// Private payload staging head, distinct from the operation head.
    pub staging_head: String,
    /// Private journal signing resource, never an application signing grant.
    pub signing_resource: String,
    /// Requested custody slot, not a key or account authorization.
    pub credential_slot: String,
    /// Retained operation records; includes reserved terminal successors.
    pub maximum_records: u32,
    /// Generated authenticated replay entry point.
    pub replay_root: String,
    /// One serialized durable operation; not distributed consensus.
    pub max_pending: u32,
}

/// Safe generated presentation declaration, never an HTML or script container.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowserView {
    /// Required browser-specific View selector.
    pub surface: String,
    /// Exact versioned protocol discriminator.
    pub protocol: String,
    /// Plain-text document title.
    pub title: String,
    /// Plain-text primary heading.
    pub heading: String,
    /// Generated bounded presentation entry point.
    pub presentation_root: String,
    /// Finite byte bound.
    pub maximum: u32,
    /// Strictly ordered plain-text labels.
    pub labels: Vec<Label>,
}

/// Bounded plain text selected by a generated presentation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Label {
    /// Stable label identifier.
    pub id: String,
    /// Plain-text content, never interpreted as markup.
    pub text: String,
}
