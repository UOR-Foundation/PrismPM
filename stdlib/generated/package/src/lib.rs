#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code, non_snake_case, unused_parens, unused_variables)]
extern crate alloc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputeError {
    AddOverflow,
    MulOverflow,
    ShiftExponentTooLarge,
    ShiftOverflow,
    PowExponentTooLarge,
    PowOverflow,
    OutputTooSmall,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lifecycle {
    pub backup: alloc::string::String,
    pub drift: alloc::string::String,
    pub migration: alloc::string::String,
    pub recovery: alloc::string::String,
    pub retirement: alloc::string::String,
    pub rollback: alloc::string::String,
    pub rollout: alloc::string::String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScalingPolicy {
    pub id: alloc::string::String,
    pub component: alloc::string::String,
    pub trigger: alloc::string::String,
    pub minimum: u32,
    pub maximum: u32,
    pub step: u32,
    pub cooldownSeconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlOrigin {
    Local,
    Inherited { field_0: u64, field_1: alloc::vec::Vec<u8>, field_2: alloc::vec::Vec<u8> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub id: alloc::string::String,
    pub kind: alloc::string::String,
    pub version: alloc::string::String,
    pub artifact: alloc::string::String,
    pub capabilities: alloc::vec::Vec<alloc::string::String>,
    pub command: alloc::vec::Vec<alloc::string::String>,
    pub dependsOn: alloc::vec::Vec<alloc::string::String>,
    pub interfaces: alloc::vec::Vec<alloc::string::String>,
    pub health: alloc::string::String,
    pub liveness: alloc::vec::Vec<alloc::string::String>,
    pub parameters: alloc::vec::Vec<alloc::string::String>,
    pub ports: alloc::vec::Vec<alloc::string::String>,
    pub readiness: alloc::vec::Vec<alloc::string::String>,
    pub resources: crate::ResourceRequirements,
    pub secrets: alloc::vec::Vec<alloc::string::String>,
    pub startup: alloc::vec::Vec<alloc::string::String>,
    pub volumes: alloc::vec::Vec<alloc::string::String>,
    pub isolation: alloc::string::String,
    pub placement: alloc::vec::Vec<alloc::string::String>,
    pub platformRequirements: alloc::vec::Vec<alloc::string::String>,
    pub scalingPolicy: Option<alloc::string::String>,
    pub failurePolicy: alloc::string::String,
    pub retryPolicy: alloc::string::String,
    pub degradationPolicy: alloc::string::String,
    pub idempotency: alloc::string::String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Persistence {
    pub id: alloc::string::String,
    pub owner: alloc::string::String,
    pub schemaArtifact: alloc::string::String,
    pub retention: alloc::string::String,
    pub compatibilityWindow: alloc::string::String,
    pub rpoSeconds: u64,
    pub rtoSeconds: u64,
    pub backup: alloc::string::String,
    pub migrationOrder: alloc::vec::Vec<alloc::string::String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupRecovery {
    pub id: alloc::string::String,
    pub kind: alloc::string::String,
    pub value: alloc::string::String,
    pub dependsOn: alloc::vec::Vec<alloc::string::String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Acceptance {
    pub id: alloc::string::String,
    pub kind: alloc::string::String,
    pub target: alloc::string::String,
    pub component: alloc::string::String,
    pub command: alloc::vec::Vec<alloc::string::String>,
    pub evidence: alloc::string::String,
    pub bounded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Topology {
    pub id: alloc::string::String,
    pub kind: alloc::string::String,
    pub value: alloc::string::String,
    pub owners: alloc::vec::Vec<alloc::string::String>,
    pub dependsOn: alloc::vec::Vec<alloc::string::String>,
    pub network: Option<alloc::string::String>,
    pub port: Option<u16>,
    pub protocol: Option<alloc::string::String>,
    pub publiclyAccessible: bool,
    pub mountPath: Option<alloc::string::String>,
    pub storageBytes: Option<u64>,
    pub storageClass: Option<alloc::string::String>,
    pub capabilities: alloc::vec::Vec<alloc::string::String>,
    pub isolation: alloc::string::String,
    pub placement: Option<alloc::string::String>,
    pub scaleMin: u32,
    pub scaleMax: u32,
    pub platformRequirements: alloc::vec::Vec<alloc::string::String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlObligation {
    pub id: u64,
    pub control: u64,
    pub mode: crate::ControlRequirementMode,
    pub scope: alloc::string::String,
    pub version: alloc::string::String,
    pub subject: alloc::vec::Vec<u8>,
    pub evidenceKind: alloc::string::String,
    pub evidence: alloc::vec::Vec<u8>,
    pub allowedProviders: alloc::vec::Vec<u64>,
    pub residualControls: alloc::vec::Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Control {
    pub id: alloc::string::String,
    pub kind: alloc::string::String,
    pub value: alloc::string::String,
    pub dependsOn: alloc::vec::Vec<alloc::string::String>,
    pub owner: alloc::string::String,
    pub verification: alloc::vec::Vec<alloc::string::String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sli {
    pub id: alloc::string::String,
    pub kind: alloc::string::String,
    pub value: alloc::string::String,
    pub dependsOn: alloc::vec::Vec<alloc::string::String>,
    pub signal: alloc::string::String,
    pub unit: alloc::string::String,
    pub windowSeconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capability {
    pub id: alloc::string::String,
    pub kind: alloc::string::String,
    pub value: alloc::string::String,
    pub dependsOn: alloc::vec::Vec<alloc::string::String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionalCommandView {
    pub title: alloc::string::String,
    pub heading: alloc::string::String,
    pub identityHeading: alloc::string::String,
    pub commandFormHeading: alloc::string::String,
    pub historyHeading: alloc::string::String,
    pub principalLabel: alloc::string::String,
    pub principalDefault: alloc::string::String,
    pub roleLabel: alloc::string::String,
    pub submitterRoleLabel: alloc::string::String,
    pub observerRoleLabel: alloc::string::String,
    pub deniedRoleLabel: alloc::string::String,
    pub authenticateLabel: alloc::string::String,
    pub commandIdLabel: alloc::string::String,
    pub inputALabel: alloc::string::String,
    pub operationLabel: alloc::string::String,
    pub inputBLabel: alloc::string::String,
    pub annotationLabel: alloc::string::String,
    pub submitLabel: alloc::string::String,
    pub refreshLabel: alloc::string::String,
    pub emptyText: alloc::string::String,
    pub loadingText: alloc::string::String,
    pub authenticatedText: alloc::string::String,
    pub accessDeniedText: alloc::string::String,
    pub retryLabel: alloc::string::String,
    pub sequenceHeading: alloc::string::String,
    pub commandColumnHeading: alloc::string::String,
    pub inputSummaryHeading: alloc::string::String,
    pub outcomeHeading: alloc::string::String,
    pub annotationHeading: alloc::string::String,
    pub releasePrefix: alloc::string::String,
    pub historyLoadedText: alloc::string::String,
    pub historyEmptyText: alloc::string::String,
    pub authenticatingText: alloc::string::String,
    pub authenticationFailedText: alloc::string::String,
    pub submittingText: alloc::string::String,
    pub outcomePrefix: alloc::string::String,
    pub commandFailedText: alloc::string::String,
    pub historyFailedText: alloc::string::String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    pub id: alloc::string::String,
    pub kind: alloc::string::String,
    pub value: alloc::string::String,
    pub dependsOn: alloc::vec::Vec<alloc::string::String>,
    pub compatibility: alloc::string::String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlContribution {
    pub obligation: u64,
    pub control: u64,
    pub policy: alloc::vec::Vec<u8>,
    pub scope: alloc::string::String,
    pub version: alloc::string::String,
    pub subject: alloc::vec::Vec<u8>,
    pub evidenceKind: alloc::string::String,
    pub evidence: alloc::vec::Vec<u8>,
    pub origin: crate::ControlOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flow {
    pub id: alloc::string::String,
    pub fromComponent: alloc::string::String,
    pub toComponent: alloc::string::String,
    pub interfaceId: alloc::string::String,
    pub delivery: alloc::string::String,
    pub ordering: alloc::string::String,
    pub idempotency: alloc::string::String,
    pub failurePropagation: alloc::string::String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub id: alloc::string::String,
    pub kind: alloc::string::String,
    pub value: alloc::string::String,
    pub dependsOn: alloc::vec::Vec<alloc::string::String>,
    pub producer: alloc::string::String,
    pub channel: alloc::string::String,
    pub schemaId: alloc::string::String,
    pub owner: alloc::string::String,
    pub delivery: alloc::string::String,
    pub ordering: alloc::string::String,
    pub idempotency: alloc::string::String,
    pub failurePropagation: alloc::string::String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityRequirement {
    pub id: alloc::string::String,
    pub issuerParameter: alloc::string::String,
    pub audiences: alloc::vec::Vec<alloc::string::String>,
    pub subjects: alloc::vec::Vec<alloc::string::String>,
    pub roles: alloc::vec::Vec<alloc::string::String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interface {
    pub id: alloc::string::String,
    pub kind: alloc::string::String,
    pub document: alloc::string::String,
    pub authentication: alloc::string::String,
    pub compatibility: alloc::string::String,
    pub protocol: alloc::string::String,
    pub errors: alloc::vec::Vec<alloc::string::String>,
    pub acceptance: alloc::vec::Vec<alloc::string::String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    pub id: alloc::string::String,
    pub kind: alloc::string::String,
    pub value: alloc::string::String,
    pub dependsOn: alloc::vec::Vec<alloc::string::String>,
    pub fromComponent: alloc::string::String,
    pub toComponent: alloc::string::String,
    pub interfaceId: alloc::string::String,
    pub failurePropagation: alloc::string::String,
    pub timeoutMillis: u64,
    pub retryPolicy: alloc::string::String,
    pub idempotency: alloc::string::String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchitectureBinding {
    pub id: alloc::string::String,
    pub kind: alloc::string::String,
    pub value: alloc::string::String,
    pub dependsOn: alloc::vec::Vec<alloc::string::String>,
    pub owner: alloc::string::String,
    pub viewpoint: alloc::string::String,
    pub verifies: alloc::vec::Vec<alloc::string::String>,
    pub measurement: Option<alloc::string::String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetBinding {
    pub id: alloc::string::String,
    pub kind: alloc::string::String,
    pub apiVersion: alloc::string::String,
    pub adapterDigest: alloc::string::String,
    pub minimumReleaseStatus: alloc::string::String,
    pub capabilities: alloc::vec::Vec<alloc::string::String>,
    pub credentials: Option<alloc::string::String>,
    pub platformRequirements: alloc::vec::Vec<alloc::string::String>,
    pub storageClass: Option<alloc::string::String>,
    pub storageProfile: Option<alloc::string::String>,
    pub ingressClassName: Option<alloc::string::String>,
    pub ingressControllerArtifact: Option<alloc::string::String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceRequirements {
    pub cpuMillis: u32,
    pub memoryBytes: u64,
    pub replicasMin: u32,
    pub replicasMax: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemManifest {
    pub closure: crate::ValidationRelation,
    pub uniqueness: crate::ValidationRelation,
    pub referentialIntegrity: crate::ValidationRelation,
    pub compatibility: crate::ValidationRelation,
    pub capabilitySatisfaction: crate::ValidationRelation,
    pub secretFlow: crate::ValidationRelation,
    pub deploymentOrder: crate::ValidationRelation,
    pub migrationOrder: crate::ValidationRelation,
    pub rollbackSafety: crate::ValidationRelation,
    pub evidenceClosure: crate::ValidationRelation,
    pub licenseClosure: crate::ValidationRelation,
    pub releaseCompleteness: crate::ValidationRelation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemModel {
    pub product: crate::Product,
    pub artifacts: alloc::vec::Vec<crate::Artifact>,
    pub components: alloc::vec::Vec<crate::Component>,
    pub interfaces: alloc::vec::Vec<crate::Interface>,
    pub schemas: alloc::vec::Vec<crate::Schema>,
    pub calls: alloc::vec::Vec<crate::Call>,
    pub events: alloc::vec::Vec<crate::Event>,
    pub flows: alloc::vec::Vec<crate::Flow>,
    pub topology: alloc::vec::Vec<crate::Topology>,
    pub platformRequirements: alloc::vec::Vec<crate::PlatformRequirement>,
    pub scalingPolicies: alloc::vec::Vec<crate::ScalingPolicy>,
    pub storageClasses: alloc::vec::Vec<crate::StorageClass>,
    pub parameters: alloc::vec::Vec<crate::Configuration>,
    pub secretReferences: alloc::vec::Vec<crate::SecretReference>,
    pub identityRequirements: alloc::vec::Vec<crate::IdentityRequirement>,
    pub persistence: alloc::vec::Vec<crate::Persistence>,
    pub migrations: alloc::vec::Vec<crate::Migration>,
    pub backups: alloc::vec::Vec<crate::BackupRecovery>,
    pub observability: crate::Observability,
    pub slis: alloc::vec::Vec<crate::Sli>,
    pub slos: alloc::vec::Vec<crate::Slo>,
    pub alerts: alloc::vec::Vec<crate::Alert>,
    pub controls: alloc::vec::Vec<crate::Control>,
    pub capabilities: alloc::vec::Vec<crate::Capability>,
    pub architecture: alloc::vec::Vec<crate::ArchitectureBinding>,
    pub targets: alloc::vec::Vec<crate::TargetBinding>,
    pub rollouts: alloc::vec::Vec<crate::Rollout>,
    pub rollbacks: alloc::vec::Vec<crate::Rollback>,
    pub drifts: alloc::vec::Vec<crate::Drift>,
    pub retirements: alloc::vec::Vec<crate::Retirement>,
    pub acceptance: alloc::vec::Vec<crate::Acceptance>,
    pub lifecycle: crate::Lifecycle,
    pub applicationProfile: crate::TransactionalCommandServiceProfile,
    pub standards: alloc::vec::Vec<alloc::string::String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slo {
    pub id: alloc::string::String,
    pub kind: alloc::string::String,
    pub value: alloc::string::String,
    pub dependsOn: alloc::vec::Vec<alloc::string::String>,
    pub comparison: alloc::string::String,
    pub threshold: u64,
    pub unit: alloc::string::String,
    pub percentileMillionths: Option<u64>,
    pub windowSeconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationRelation {
    pub bound: u64,
    pub values: alloc::vec::Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rollback {
    pub id: alloc::string::String,
    pub kind: alloc::string::String,
    pub value: alloc::string::String,
    pub dependsOn: alloc::vec::Vec<alloc::string::String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Migration {
    pub id: alloc::string::String,
    pub kind: alloc::string::String,
    pub value: alloc::string::String,
    pub dependsOn: alloc::vec::Vec<alloc::string::String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drift {
    pub id: alloc::string::String,
    pub kind: alloc::string::String,
    pub value: alloc::string::String,
    pub dependsOn: alloc::vec::Vec<alloc::string::String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionalCommandServiceProfile {
    pub contract: alloc::string::String,
    pub applicationErrors: alloc::vec::Vec<crate::ApplicationErrorBinding>,
    pub applicationModelDigest: alloc::string::String,
    pub commandEncoding: alloc::string::String,
    pub responseVersion: alloc::string::String,
    pub optionalAnnotation: alloc::string::String,
    pub coreArtifact: alloc::string::String,
    pub identityAudience: alloc::string::String,
    pub submitterRole: alloc::string::String,
    pub observerRole: alloc::string::String,
    pub eventSource: alloc::string::String,
    pub acceptedEventType: alloc::string::String,
    pub rejectedEventType: alloc::string::String,
    pub databaseName: alloc::string::String,
    pub databaseUser: alloc::string::String,
    pub eventExchange: alloc::string::String,
    pub auditQueue: alloc::string::String,
    pub commandIdPattern: alloc::string::String,
    pub commandPath: alloc::string::String,
    pub tokenPath: alloc::string::String,
    pub commandOperations: alloc::vec::Vec<alloc::string::String>,
    pub annotationMaxScalars: u32,
    pub historyDefaultLimit: u32,
    pub historyMaxLimit: u32,
    pub maxRequestBytes: u64,
    pub availabilityThresholdMillionths: u64,
    pub outboxLagThresholdMillis: u64,
    pub commandIdField: alloc::string::String,
    pub operationField: alloc::string::String,
    pub inputAField: alloc::string::String,
    pub inputBField: alloc::string::String,
    pub annotationField: alloc::string::String,
    pub commandIdColumn: alloc::string::String,
    pub operationColumn: alloc::string::String,
    pub inputAColumn: alloc::string::String,
    pub inputBColumn: alloc::string::String,
    pub annotationColumn: alloc::string::String,
    pub acceptanceErrorOperation: alloc::string::String,
    pub acceptanceErrorInputA: alloc::string::String,
    pub acceptanceErrorInputB: alloc::string::String,
    pub acceptanceExpectedError: alloc::string::String,
    pub acceptanceConflictInputB: alloc::string::String,
    pub acceptanceSuccessOperation: alloc::string::String,
    pub acceptanceSuccessInputA: alloc::string::String,
    pub acceptanceSuccessInputB: alloc::string::String,
    pub acceptanceExpectedResult: alloc::string::String,
    pub view: crate::TransactionalCommandView,
    pub brokerUser: alloc::string::String,
    pub historyTable: alloc::string::String,
    pub outboxTable: alloc::string::String,
    pub auditTable: alloc::string::String,
    pub publicHostnameParameter: alloc::string::String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationErrorBinding {
    pub wireName: alloc::string::String,
    pub modelName: alloc::string::String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Product {
    pub id: alloc::string::String,
    pub owner: alloc::string::String,
    pub version: alloc::string::String,
    pub lifecycle: alloc::string::String,
    pub sourcePolicy: alloc::string::String,
    pub supportedPlatforms: alloc::vec::Vec<alloc::string::String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretReference {
    pub id: alloc::string::String,
    pub providerKey: alloc::string::String,
    pub consumers: alloc::vec::Vec<alloc::string::String>,
    pub rotation: alloc::string::String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ControlRequirementMode {
    LocalRequired = 0,
    InheritedRequired = 1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alert {
    pub id: alloc::string::String,
    pub kind: alloc::string::String,
    pub value: alloc::string::String,
    pub dependsOn: alloc::vec::Vec<alloc::string::String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandardsProfile {
    pub architectureEdition: u64,
    pub applicationSecurityEdition: u64,
    pub controlEdition: u64,
    pub riskEdition: u64,
    pub qualityEdition: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlPolicy {
    pub digest: alloc::vec::Vec<u8>,
    pub obligations: alloc::vec::Vec<crate::ControlObligation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub id: alloc::string::String,
    pub mediaType: alloc::string::String,
    pub digest: alloc::string::String,
    pub licenseExpression: alloc::string::String,
    pub path: alloc::string::String,
    pub role: alloc::string::String,
    pub platformRequirements: alloc::vec::Vec<alloc::string::String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rollout {
    pub id: alloc::string::String,
    pub kind: alloc::string::String,
    pub value: alloc::string::String,
    pub dependsOn: alloc::vec::Vec<alloc::string::String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageClass {
    pub id: alloc::string::String,
    pub accessModes: alloc::vec::Vec<alloc::string::String>,
    pub bindingMode: alloc::string::String,
    pub provisioning: alloc::string::String,
    pub retention: alloc::string::String,
    pub capabilities: alloc::vec::Vec<alloc::string::String>,
    pub platformRequirements: alloc::vec::Vec<alloc::string::String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageOctetConstant {
    pub octets: alloc::vec::Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Retirement {
    pub id: alloc::string::String,
    pub kind: alloc::string::String,
    pub value: alloc::string::String,
    pub dependsOn: alloc::vec::Vec<alloc::string::String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observability {
    pub logs: alloc::vec::Vec<alloc::string::String>,
    pub metrics: alloc::vec::Vec<alloc::string::String>,
    pub traces: alloc::vec::Vec<alloc::string::String>,
    pub alerts: alloc::vec::Vec<alloc::string::String>,
    pub slos: alloc::vec::Vec<alloc::string::String>,
    pub redactedFields: alloc::vec::Vec<alloc::string::String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlSubmission {
    pub contributions: alloc::vec::Vec<crate::ControlContribution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Configuration {
    pub id: alloc::string::String,
    pub valueType: alloc::string::String,
    pub allowed: alloc::vec::Vec<alloc::string::String>,
    pub defaultValue: Option<alloc::string::String>,
    pub required: bool,
    pub mutable: bool,
    pub lateBound: bool,
    pub exposure: alloc::string::String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformRequirement {
    pub id: alloc::string::String,
    pub os: alloc::string::String,
    pub architectures: alloc::vec::Vec<alloc::string::String>,
    pub runtime: alloc::string::String,
    pub runtimeVersion: alloc::string::String,
    pub capabilities: alloc::vec::Vec<alloc::string::String>,
}

pub fn appendBytes(left: alloc::vec::Vec<u8>, right: alloc::vec::Vec<u8>) -> alloc::vec::Vec<u8> {
    { let _x_2 = { let mut __value = left; __value.extend_from_slice(&right); __value }; _x_2 }
}

pub fn byteAt(value: alloc::vec::Vec<u8>, offset: u64) -> Option<u8> {
    __prod_borrowed_byteAt(value.as_ref(), offset)
}

fn __prod_borrowed_byteAt(value: &[u8], offset: u64) -> Option<u8> {
    { let _x_2 = usize::try_from(offset).ok().and_then(|__index| (value).get(__index).cloned()); _x_2 }
}

pub fn byteLength(value: alloc::vec::Vec<u8>) -> u64 {
    __prod_borrowed_byteLength(value.as_ref())
}

fn __prod_borrowed_byteLength(value: &[u8]) -> u64 {
    { let _x_2 = (value).len() as u64; _x_2 }
}

pub fn compareBytes(left: alloc::vec::Vec<u8>, right: alloc::vec::Vec<u8>) -> core::cmp::Ordering {
    __prod_borrowed_compareBytes(left.as_ref(), right.as_ref())
}

fn __prod_borrowed_compareBytes(left: &[u8], right: &[u8]) -> core::cmp::Ordering {
    { let _x_1 = (left).cmp(&right); _x_1 }
}

pub fn sliceBytes(value: alloc::vec::Vec<u8>, start: u64, count: u64) -> Option<alloc::vec::Vec<u8>> {
    { let _x_2 = { let __start = usize::try_from(start).ok(); let __count = usize::try_from(count).ok(); match (__start, __count) { (Some(__start), Some(__count)) => __start.checked_add(__count).and_then(|__end| (value).get(__start..__end).map(|__slice| __slice.to_vec())), _ => None } }; _x_2 }
}

pub fn formatInt64(value: i64) -> alloc::string::String {
    { let _x_4 = alloc::format!("{}", value); _x_4 }
}

pub fn parseInt64(value: alloc::string::String) -> Option<i64> {
    __prod_borrowed_parseInt64(value.as_ref())
}

fn __prod_borrowed_parseInt64(value: &str) -> Option<i64> {
    { let _x_4 = { let __text = value; __text.parse().ok().filter(|__value| alloc::string::ToString::to_string(__value) == __text) }; _x_4 }
}

pub fn portableTrue() -> bool {
    { let _x_1 = true; _x_1 }
}

pub fn applicationSecurityEdition(__prod_self: crate::StandardsProfile) -> u64 {
    { let _x_1 = (__prod_self).applicationSecurityEdition; _x_1 }
}

pub fn architectureEdition(__prod_self: crate::StandardsProfile) -> u64 {
    { let _x_1 = (__prod_self).architectureEdition; _x_1 }
}

pub fn controlEdition(__prod_self: crate::StandardsProfile) -> u64 {
    { let _x_1 = (__prod_self).controlEdition; _x_1 }
}

pub fn qualityEdition(__prod_self: crate::StandardsProfile) -> u64 {
    { let _x_1 = (__prod_self).qualityEdition; _x_1 }
}

pub fn riskEdition(__prod_self: crate::StandardsProfile) -> u64 {
    { let _x_1 = (__prod_self).riskEdition; _x_1 }
}

pub fn allBelow(x_1: u64, x_2: &[u64]) -> bool {
    match x_2 {
        [] => { let _x_30 = true; _x_30 },
        [head_21, tail_22 @ ..] => { let head_21 = head_21.clone(); { let _x_31 = (head_21 < x_1); match _x_31 {
        false => _x_31,
        true => { let _x_34 = allBelow(x_1, &(tail_22)); _x_34 },
    } } },
    }
}

pub fn allConsecutive(x_1: u64, x_2: &[u64]) -> Result<bool, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_45 = true; _x_45 },
        [head_28, tail_29 @ ..] => { let head_28 = head_28.clone(); { let _x_50 = (x_1 == head_28); match _x_50 {
        false => _x_50,
        true => { let _x_54 = 1; { let _x_55 = ((x_1) as u64).checked_add(_x_54).ok_or(crate::ComputeError::AddOverflow)?; { let _x_56 = allConsecutive(_x_55, &(tail_29))?; _x_56 } } },
    } } },
    })
}

pub fn canonicalIndexes(x_1: u64, x_2: u64, output: &mut [u64]) -> Result<usize, crate::ComputeError> {
    match x_2 {
        0 => Ok::<usize, crate::ComputeError>(0),
        _ => { let n_18 = (x_2).saturating_sub(1); { let _x_32 = 1; { let _x_33 = ((x_1) as u64).checked_add(_x_32).ok_or(crate::ComputeError::AddOverflow)?; match (output).split_first_mut() { None => Err(crate::ComputeError::OutputTooSmall), Some((__head0, __rest0)) => { *__head0 = x_1; let __len0 = canonicalIndexes(_x_33, n_18, __rest0)?; Ok(__len0 + 1) } } } } },
    }
}

pub fn validateComponentIndexes(values: &[u64]) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_1 = 0; { let _x_4 = allConsecutive(_x_1, &(values))?; _x_4 } })
}

pub fn validateControlLinks(riskCount: u64, links: &[u64]) -> bool {
    { let _x_1 = allBelow(riskCount, &(links)); _x_1 }
}

pub fn validateEdgeEndpoints(componentCount: u64, endpoints: &[u64]) -> bool {
    { let _x_1 = allBelow(componentCount, &(endpoints)); _x_1 }
}

pub fn validateExactStandardsProfile(profile: crate::StandardsProfile) -> bool {
    { let _x_50 = (profile).architectureEdition; { let _x_51 = 2022; { let _x_54 = (_x_50 == _x_51); match _x_54 {
        false => _x_54,
        true => { let _x_96 = (profile).applicationSecurityEdition; { let _x_97 = 2011; { let _x_98 = (_x_96 == _x_97); match _x_98 {
        false => _x_98,
        true => { let _x_113 = (profile).controlEdition; { let _x_114 = 2017; { let _x_115 = (_x_113 == _x_114); match _x_115 {
        false => _x_115,
        true => { let _x_123 = (profile).riskEdition; { let _x_124 = 2022; { let _x_125 = (_x_123 == _x_124); match _x_125 {
        false => _x_125,
        true => { let _x_129 = (profile).qualityEdition; { let _x_130 = 2023; { let _x_131 = (_x_129 == _x_130); _x_131 } } },
    } } } },
    } } } },
    } } } },
    } } } }
}

pub fn validateFlattenedBounds(bound: u64, indexes: &[u64]) -> bool {
    { let _x_1 = allBelow(bound, &(indexes)); _x_1 }
}

pub fn validateQualityLinks(targetCount: u64, links: &[u64]) -> bool {
    { let _x_1 = allBelow(targetCount, &(links)); _x_1 }
}

pub fn validateRiskLinks(assetOrThreatCount: u64, links: &[u64]) -> bool {
    { let _x_1 = allBelow(assetOrThreatCount, &(links)); _x_1 }
}

pub fn validateViewpointLinks(targetCount: u64, links: &[u64]) -> bool {
    { let _x_1 = allBelow(targetCount, &(links)); _x_1 }
}

pub fn checkedAddInt64(left: i64, right: i64) -> Option<i64> {
    { let _x_1 = (left).checked_add(right); _x_1 }
}

pub fn checkedDivideInt64(left: i64, right: i64) -> Option<i64> {
    { let _x_1 = (left).checked_div(right); _x_1 }
}

pub fn checkedMultiplyInt64(left: i64, right: i64) -> Option<i64> {
    { let _x_1 = (left).checked_mul(right); _x_1 }
}

pub fn checkedNegateInt64(value: i64) -> Option<i64> {
    { let _x_1 = (value).checked_neg(); _x_1 }
}

pub fn checkedSubtractInt64(left: i64, right: i64) -> Option<i64> {
    { let _x_1 = (left).checked_sub(right); _x_1 }
}

pub fn decode(value: alloc::vec::Vec<u8>) -> Option<alloc::string::String> {
    { let _x_1 = alloc::string::String::from_utf8(value).ok(); _x_1 }
}

pub fn encode(value: alloc::string::String) -> alloc::vec::Vec<u8> {
    { let _x_1 = (value).into_bytes(); _x_1 }
}

pub fn coverageAllNatsMember(x_1: &[u64], x_2: &[u64]) -> bool {
    match x_1 {
        [] => { let _x_42 = true; _x_42 },
        [head_30, tail_31 @ ..] => { let head_30 = head_30.clone(); { let _x_46 = coverageNatMember(head_30, &(x_2)); match _x_46 {
        false => _x_46,
        true => { let _x_50 = coverageAllNatsMember(&(tail_31), &(x_2)); _x_50 },
    } } },
    }
}

pub fn coverageAllObligations(x_1: &[crate::ControlObligation], x_2: &[crate::ControlContribution], x_3: &[u8], x_4: &[crate::ControlObligation]) -> bool {
    match x_1 {
        [] => { let _x_56 = true; _x_56 },
        [head_41, tail_42 @ ..] => { let _x_65 = coverageObligationWellFormed(&(head_41), &(x_4)); match _x_65 {
        false => _x_65,
        true => { let _x_72 = coverageContributionPresent(&(head_41), &(x_2), &(x_3), &(x_4)); match _x_72 {
        false => _x_72,
        true => { let _x_76 = coverageAllObligations(&(tail_42), &(x_2), &(x_3), &(x_4)); _x_76 },
    } },
    } },
    }
}

pub fn coverageBindingAbsent(x_1: &crate::ControlObligation, x_2: &[crate::ControlObligation]) -> bool {
    match x_2 {
        [] => { let _x_47 = true; _x_47 },
        [head_31, tail_32 @ ..] => { let _x_57 = coverageSameBinding(&(x_1), &(head_31)); match _x_57 {
        false => { let _x_71 = coverageBindingAbsent(&(x_1), &(tail_32)); _x_71 },
        true => { let _x_67 = false; _x_67 },
    } },
    }
}

pub fn coverageContributionExists(x_1: u64, x_2: &[crate::ControlContribution]) -> bool {
    match x_2 {
        [] => { let _x_31 = false; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = (head_22).obligation; { let _x_33 = (x_1 == _x_32); match _x_33 {
        false => { let _x_35 = coverageContributionExists(x_1, &(tail_23)); _x_35 },
        true => _x_33,
    } } },
    }
}

pub fn coverageContributionMatches(item: &crate::ControlObligation, contribution: &crate::ControlContribution, policy: &[u8], all: &[crate::ControlObligation]) -> bool {
    { let _x_99 = (item).id; { let _x_100 = (contribution).obligation; { let _x_101 = (_x_99 == _x_100); match _x_101 {
        false => _x_101,
        true => { let _x_187 = (item).control; { let _x_188 = (contribution).control; { let _x_189 = (_x_187 == _x_188); match _x_189 {
        false => _x_189,
        true => { let _x_238 = &(contribution).policy; { let _x_239 = coverageDigestEqual(&(policy), &(_x_238)); match _x_239 {
        false => _x_239,
        true => { let _x_281 = &(item).scope; { let _x_282 = &(contribution).scope; { let _x_283 = (_x_281 == _x_282); match _x_283 {
        false => _x_283,
        true => { let _x_316 = &(item).version; { let _x_317 = &(contribution).version; { let _x_318 = (_x_316 == _x_317); match _x_318 {
        false => _x_318,
        true => { let _x_342 = &(item).subject; { let _x_343 = &(contribution).subject; { let _x_344 = coverageDigestEqual(&(_x_342), &(_x_343)); match _x_344 {
        false => _x_344,
        true => { let _x_361 = &(item).evidenceKind; { let _x_362 = &(contribution).evidenceKind; { let _x_363 = (_x_361 == _x_362); match _x_363 {
        false => _x_363,
        true => { let _x_371 = &(item).evidence; { let _x_372 = &(contribution).evidence; { let _x_373 = coverageDigestEqual(&(_x_371), &(_x_372)); match _x_373 {
        false => _x_373,
        true => { let _x_377 = &(contribution).origin; { let _x_378 = coverageOriginValid(&(_x_377), &(item), &(all)); _x_378 } },
    } } } },
    } } } },
    } } } },
    } } } },
    } } } },
    } } },
    } } } },
    } } } }
}

pub fn coverageContributionPresent(x_1: &crate::ControlObligation, x_2: &[crate::ControlContribution], x_3: &[u8], x_4: &[crate::ControlObligation]) -> bool {
    match x_2 {
        [] => { let _x_32 = false; _x_32 },
        [head_23, tail_24 @ ..] => { let _x_33 = coverageContributionMatches(&(x_1), &(head_23), &(x_3), &(x_4)); match _x_33 {
        false => { let _x_35 = coverageContributionPresent(&(x_1), &(tail_24), &(x_3), &(x_4)); _x_35 },
        true => _x_33,
    } },
    }
}

pub fn coverageControlExists(x_1: u64, x_2: &[crate::ControlObligation]) -> bool {
    match x_2 {
        [] => { let _x_31 = false; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = (head_22).control; { let _x_33 = (x_1 == _x_32); match _x_33 {
        false => { let _x_35 = coverageControlExists(x_1, &(tail_23)); _x_35 },
        true => _x_33,
    } } },
    }
}

pub fn coverageDigest(value: &[u8]) -> bool {
    { let _x_2 = (value).len() as u64; { let _x_3 = 32; { let _x_6 = (_x_2 == _x_3); _x_6 } } }
}

pub fn coverageDigestEqual(x_1: &[u8], x_2: &[u8]) -> bool {
    match x_1 {
        [] => match x_2 {
        [] => { let _x_60 = true; _x_60 },
        [head_61, tail_62 @ ..] => { let head_61 = head_61.clone(); { let _x_63 = false; _x_63 } },
    },
        [head_45, tail_46 @ ..] => { let head_45 = head_45.clone(); match x_2 {
        [] => { let _x_72 = false; _x_72 },
        [head_73, tail_74 @ ..] => { let head_73 = head_73.clone(); { let _x_78 = (head_45 == head_73); match _x_78 {
        false => _x_78,
        true => { let _x_81 = coverageDigestEqual(&(tail_46), &(tail_74)); _x_81 },
    } } },
    } },
    }
}

pub fn coverageEarlier(x_1: u64, x_2: u64, x_3: &[crate::ControlObligation]) -> bool {
    match x_3 {
        [] => { let _x_74 = false; _x_74 },
        [head_50, tail_51 @ ..] => { let _x_87 = (head_50).id; { let _x_88 = (x_2 == _x_87); match _x_88 {
        false => { let _x_104 = (head_50).id; { let _x_105 = (x_1 == _x_104); match _x_105 {
        false => { let _x_106 = coverageEarlier(x_1, x_2, &(tail_51)); _x_106 },
        true => _x_105,
    } } },
        true => { let _x_98 = false; _x_98 },
    } } },
    }
}

pub fn coverageLocalRequired(mode: crate::ControlRequirementMode) -> bool {
    match mode {
        crate::ControlRequirementMode::LocalRequired => { let _x_14 = true; _x_14 },
        crate::ControlRequirementMode::InheritedRequired => { let _x_16 = false; _x_16 },
    }
}

pub fn coverageNatMember(x_1: u64, x_2: &[u64]) -> bool {
    match x_2 {
        [] => { let _x_30 = false; _x_30 },
        [head_21, tail_22 @ ..] => { let head_21 = head_21.clone(); { let _x_31 = (x_1 == head_21); match _x_31 {
        false => { let _x_33 = coverageNatMember(x_1, &(tail_22)); _x_33 },
        true => _x_31,
    } } },
    }
}

pub fn coverageNonEmpty(rows: &[crate::ControlObligation]) -> bool {
    match rows {
        [] => { let _x_17 = false; _x_17 },
        [head_11, tail_12 @ ..] => { let _x_18 = true; _x_18 },
    }
}

pub fn coverageNonEmptyString(value: &str) -> bool {
    { let _x_4 = value == ""; match _x_4 {
        false => { let _x_7 = true; _x_7 },
        true => { let _x_8 = false; _x_8 },
    } }
}

pub fn coverageObligationExists(x_1: u64, x_2: &[crate::ControlObligation]) -> bool {
    match x_2 {
        [] => { let _x_31 = false; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = (head_22).id; { let _x_33 = (x_1 == _x_32); match _x_33 {
        false => { let _x_35 = coverageObligationExists(x_1, &(tail_23)); _x_35 },
        true => _x_33,
    } } },
    }
}

pub fn coverageObligationWellFormed(item: &crate::ControlObligation, all: &[crate::ControlObligation]) -> bool {
    { let _x_110 = 0; { let _x_113 = (item).id; { let _x_114 = (_x_110 < _x_113); match _x_114 {
        false => _x_114,
        true => { let _x_209 = 0; { let _x_210 = (item).control; { let _x_211 = (_x_209 < _x_210); match _x_211 {
        false => _x_211,
        true => { let _x_261 = &(item).scope; { let _x_262 = coverageNonEmptyString((_x_261).as_ref()); match _x_262 {
        false => _x_262,
        true => { let _x_306 = &(item).version; { let _x_307 = coverageNonEmptyString((_x_306).as_ref()); match _x_307 {
        false => _x_307,
        true => { let _x_345 = &(item).evidenceKind; { let _x_346 = coverageNonEmptyString((_x_345).as_ref()); match _x_346 {
        false => _x_346,
        true => { let _x_378 = &(item).subject; { let _x_379 = coverageDigest(&(_x_378)); match _x_379 {
        false => _x_379,
        true => { let _x_405 = &(item).evidence; { let _x_406 = coverageDigest(&(_x_405)); match _x_406 {
        false => _x_406,
        true => { let _x_426 = &(item).allowedProviders; { let _x_427 = coverageUniqueNats(&(_x_426)); match _x_427 {
        false => _x_427,
        true => { let _x_441 = &(item).residualControls; { let _x_442 = coverageUniqueNats(&(_x_441)); match _x_442 {
        false => _x_442,
        true => { let _x_450 = &(item).allowedProviders; { let _x_451 = coverageProvidersDeclared(&(_x_450), &(all)); match _x_451 {
        false => _x_451,
        true => { let _x_455 = &(item).residualControls; { let _x_456 = coverageResidualControlsDeclared(&(_x_455), &(all)); _x_456 } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } } },
    } } } }
}

pub fn coverageOriginValid(origin: &crate::ControlOrigin, consumer: &crate::ControlObligation, all: &[crate::ControlObligation]) -> bool {
    match origin {
        crate::ControlOrigin::Local => { let _x_69 = (consumer).mode; { let _x_70 = coverageLocalRequired(_x_69); _x_70 } },
        crate::ControlOrigin::Inherited { field_0: x_42, field_1: x_43, field_2: x_44 } => { let x_42 = x_42.clone(); { let _x_80 = (consumer).mode; { let _x_81 = coverageLocalRequired(_x_80); match _x_81 {
        false => { let _x_102 = &(consumer).allowedProviders; { let _x_103 = coverageNatMember(x_42, &(_x_102)); match _x_103 {
        false => _x_103,
        true => { let _x_104 = coverageProviderPresent(x_42, &(x_43), &(x_44), &(consumer), &(all), &(all)); _x_104 },
    } } },
        true => { let _x_91 = false; _x_91 },
    } } } },
    }
}

pub fn coverageProviderPresent(x_1: u64, x_2: &[u8], x_3: &[u8], x_4: &crate::ControlObligation, x_5: &[crate::ControlObligation], x_6: &[crate::ControlObligation]) -> bool {
    match x_5 {
        [] => { let _x_185 = false; _x_185 },
        [head_143, tail_144 @ ..] => { let _x_249 = (head_143).id; { let _x_250 = (x_1 == _x_249); { let _jp_251 = /* jp "_jp_251" inlined at its jump site */ (); match _x_250 {
        false => { let _y_252 = _x_250; match _y_252.clone() {
        false => { let _x_254 = coverageProviderPresent(x_1, &(x_2), &(x_3), &(x_4), &(tail_144), &(x_6)); _x_254 },
        true => _y_252.clone(),
    } },
        true => { let _x_315 = (x_4).control; { let _x_316 = (head_143).control; { let _x_317 = (_x_315 == _x_316); match _x_317 {
        false => { let _y_252 = _x_317; match _y_252.clone() {
        false => { let _x_254 = coverageProviderPresent(x_1, &(x_2), &(x_3), &(x_4), &(tail_144), &(x_6)); _x_254 },
        true => _y_252.clone(),
    } },
        true => { let _x_370 = &(x_4).scope; { let _x_371 = &(head_143).scope; { let _x_372 = (_x_370 == _x_371); match _x_372 {
        false => { let _y_252 = _x_372; match _y_252.clone() {
        false => { let _x_254 = coverageProviderPresent(x_1, &(x_2), &(x_3), &(x_4), &(tail_144), &(x_6)); _x_254 },
        true => _y_252.clone(),
    } },
        true => { let _x_416 = &(x_4).version; { let _x_417 = &(head_143).version; { let _x_418 = (_x_416 == _x_417); match _x_418 {
        false => { let _y_252 = _x_418; match _y_252.clone() {
        false => { let _x_254 = coverageProviderPresent(x_1, &(x_2), &(x_3), &(x_4), &(tail_144), &(x_6)); _x_254 },
        true => _y_252.clone(),
    } },
        true => { let _x_453 = &(x_4).evidenceKind; { let _x_454 = &(head_143).evidenceKind; { let _x_455 = (_x_453 == _x_454); match _x_455 {
        false => { let _y_252 = _x_455; match _y_252.clone() {
        false => { let _x_254 = coverageProviderPresent(x_1, &(x_2), &(x_3), &(x_4), &(tail_144), &(x_6)); _x_254 },
        true => _y_252.clone(),
    } },
        true => { let _x_482 = &(head_143).subject; { let _x_483 = coverageDigestEqual(&(x_2), &(_x_482)); match _x_483 {
        false => { let _y_252 = _x_483; match _y_252.clone() {
        false => { let _x_254 = coverageProviderPresent(x_1, &(x_2), &(x_3), &(x_4), &(tail_144), &(x_6)); _x_254 },
        true => _y_252.clone(),
    } },
        true => { let _x_504 = &(head_143).evidence; { let _x_505 = coverageDigestEqual(&(x_3), &(_x_504)); match _x_505 {
        false => { let _y_252 = _x_505; match _y_252.clone() {
        false => { let _x_254 = coverageProviderPresent(x_1, &(x_2), &(x_3), &(x_4), &(tail_144), &(x_6)); _x_254 },
        true => _y_252.clone(),
    } },
        true => { let _x_520 = (x_4).id; { let _x_521 = coverageEarlier(x_1, _x_520, &(x_6)); match _x_521 {
        false => { let _y_252 = _x_521; match _y_252.clone() {
        false => { let _x_254 = coverageProviderPresent(x_1, &(x_2), &(x_3), &(x_4), &(tail_144), &(x_6)); _x_254 },
        true => _y_252.clone(),
    } },
        true => { let _x_529 = &(head_143).residualControls; { let _x_530 = &(x_4).residualControls; { let _x_531 = coverageAllNatsMember(&(_x_529), &(_x_530)); match _x_531 {
        false => { let _y_252 = _x_531; match _y_252.clone() {
        false => { let _x_254 = coverageProviderPresent(x_1, &(x_2), &(x_3), &(x_4), &(tail_144), &(x_6)); _x_254 },
        true => _y_252.clone(),
    } },
        true => { let _x_535 = &(head_143).residualControls; { let _x_536 = coverageResidualsPresent(&(_x_535), &(x_4), &(x_6)); { let _y_252 = _x_536; match _y_252.clone() {
        false => { let _x_254 = coverageProviderPresent(x_1, &(x_2), &(x_3), &(x_4), &(tail_144), &(x_6)); _x_254 },
        true => _y_252.clone(),
    } } } },
    } } } },
    } } },
    } } },
    } } },
    } } } },
    } } } },
    } } } },
    } } } },
    } } } },
    }
}

pub fn coverageProvidersDeclared(x_1: &[u64], x_2: &[crate::ControlObligation]) -> bool {
    match x_1 {
        [] => { let _x_42 = true; _x_42 },
        [head_30, tail_31 @ ..] => { let head_30 = head_30.clone(); { let _x_46 = coverageObligationExists(head_30, &(x_2)); match _x_46 {
        false => _x_46,
        true => { let _x_50 = coverageProvidersDeclared(&(tail_31), &(x_2)); _x_50 },
    } } },
    }
}

pub fn coverageResidualControlsDeclared(x_1: &[u64], x_2: &[crate::ControlObligation]) -> bool {
    match x_1 {
        [] => { let _x_42 = true; _x_42 },
        [head_30, tail_31 @ ..] => { let head_30 = head_30.clone(); { let _x_46 = coverageControlExists(head_30, &(x_2)); match _x_46 {
        false => _x_46,
        true => { let _x_50 = coverageResidualControlsDeclared(&(tail_31), &(x_2)); _x_50 },
    } } },
    }
}

pub fn coverageResidualPresent(x_1: u64, x_2: &crate::ControlObligation, x_3: &[crate::ControlObligation], x_4: &[crate::ControlObligation]) -> bool {
    match x_3 {
        [] => { let _x_115 = false; _x_115 },
        [head_88, tail_89 @ ..] => { let _x_146 = (head_88).control; { let _x_147 = (x_1 == _x_146); { let _jp_148 = /* jp "_jp_148" inlined at its jump site */ (); match _x_147 {
        false => { let _y_149 = _x_147; match _y_149.clone() {
        false => { let _x_151 = coverageResidualPresent(x_1, &(x_2), &(tail_89), &(x_4)); _x_151 },
        true => _y_149.clone(),
    } },
        true => { let _x_179 = &(x_2).scope; { let _x_180 = &(head_88).scope; { let _x_181 = (_x_179 == _x_180); match _x_181 {
        false => { let _y_149 = _x_181; match _y_149.clone() {
        false => { let _x_151 = coverageResidualPresent(x_1, &(x_2), &(tail_89), &(x_4)); _x_151 },
        true => _y_149.clone(),
    } },
        true => { let _x_199 = &(x_2).version; { let _x_200 = &(head_88).version; { let _x_201 = (_x_199 == _x_200); match _x_201 {
        false => { let _y_149 = _x_201; match _y_149.clone() {
        false => { let _x_151 = coverageResidualPresent(x_1, &(x_2), &(tail_89), &(x_4)); _x_151 },
        true => _y_149.clone(),
    } },
        true => { let _x_210 = &(x_2).subject; { let _x_211 = &(head_88).subject; { let _x_212 = coverageDigestEqual(&(_x_210), &(_x_211)); match _x_212 {
        false => { let _y_149 = _x_212; match _y_149.clone() {
        false => { let _x_151 = coverageResidualPresent(x_1, &(x_2), &(tail_89), &(x_4)); _x_151 },
        true => _y_149.clone(),
    } },
        true => { let _x_216 = (head_88).id; { let _x_217 = (x_2).id; { let _x_218 = coverageEarlier(_x_216, _x_217, &(x_4)); { let _y_149 = _x_218; match _y_149.clone() {
        false => { let _x_151 = coverageResidualPresent(x_1, &(x_2), &(tail_89), &(x_4)); _x_151 },
        true => _y_149.clone(),
    } } } } },
    } } } },
    } } } },
    } } } },
    } } } },
    }
}

pub fn coverageResidualsPresent(x_1: &[u64], x_2: &crate::ControlObligation, x_3: &[crate::ControlObligation]) -> bool {
    match x_1 {
        [] => { let _x_43 = true; _x_43 },
        [head_31, tail_32 @ ..] => { let head_31 = head_31.clone(); { let _x_47 = coverageResidualPresent(head_31, &(x_2), &(x_3), &(x_3)); match _x_47 {
        false => _x_47,
        true => { let _x_51 = coverageResidualsPresent(&(tail_32), &(x_2), &(x_3)); _x_51 },
    } } },
    }
}

pub fn coverageSameBinding(left: &crate::ControlObligation, right: &crate::ControlObligation) -> bool {
    { let _x_44 = (left).control; { let _x_45 = (right).control; { let _x_46 = (_x_44 == _x_45); match _x_46 {
        false => _x_46,
        true => { let _x_82 = &(left).scope; { let _x_83 = &(right).scope; { let _x_84 = (_x_82 == _x_83); match _x_84 {
        false => _x_84,
        true => { let _x_95 = &(left).version; { let _x_96 = &(right).version; { let _x_97 = (_x_95 == _x_96); match _x_97 {
        false => _x_97,
        true => { let _x_101 = &(left).subject; { let _x_102 = &(right).subject; { let _x_103 = coverageDigestEqual(&(_x_101), &(_x_102)); _x_103 } } },
    } } } },
    } } } },
    } } } }
}

pub fn coverageSubmissionClosed(x_1: &[crate::ControlContribution], x_2: &[crate::ControlObligation]) -> bool {
    match x_1 {
        [] => { let _x_61 = true; _x_61 },
        [head_42, tail_43 @ ..] => { let _x_84 = (head_42).obligation; { let _x_85 = coverageObligationExists(_x_84, &(x_2)); match _x_85 {
        false => _x_85,
        true => { let _x_92 = (head_42).obligation; { let _x_93 = coverageContributionExists(_x_92, &(tail_43)); match _x_93 {
        false => { let _x_107 = coverageSubmissionClosed(&(tail_43), &(x_2)); _x_107 },
        true => { let _x_103 = false; _x_103 },
    } } },
    } } },
    }
}

pub fn coverageUniqueNats(x_1: &[u64]) -> bool {
    match x_1 {
        [] => { let _x_46 = true; _x_46 },
        [head_29, tail_30 @ ..] => { let head_29 = head_29.clone(); { let _x_56 = coverageNatMember(head_29, &(tail_30)); match _x_56 {
        false => { let _x_70 = coverageUniqueNats(&(tail_30)); _x_70 },
        true => { let _x_66 = false; _x_66 },
    } } },
    }
}

pub fn coverageUniqueObligations(x_1: &[crate::ControlObligation]) -> bool {
    match x_1 {
        [] => { let _x_59 = true; _x_59 },
        [head_39, tail_40 @ ..] => { let _x_74 = (head_39).id; { let _x_75 = coverageObligationExists(_x_74, &(tail_40)); match _x_75 {
        false => { let _x_95 = coverageBindingAbsent(&(head_39), &(tail_40)); match _x_95 {
        false => _x_95,
        true => { let _x_96 = coverageUniqueObligations(&(tail_40)); _x_96 },
    } },
        true => { let _x_85 = false; _x_85 },
    } } },
    }
}

pub fn validateControlCoverage(policy: &crate::ControlPolicy, submission: &crate::ControlSubmission) -> bool {
    { let _x_50 = &(policy).digest; { let _x_51 = coverageDigest(&(_x_50)); match _x_51 {
        false => _x_51,
        true => { let _x_93 = &(policy).obligations; { let _x_94 = coverageNonEmpty(&(_x_93)); match _x_94 {
        false => _x_94,
        true => { let _x_111 = &(policy).obligations; { let _x_112 = coverageUniqueObligations(&(_x_111)); match _x_112 {
        false => _x_112,
        true => { let _x_122 = &(submission).contributions; { let _x_123 = &(policy).obligations; { let _x_124 = coverageSubmissionClosed(&(_x_122), &(_x_123)); match _x_124 {
        false => _x_124,
        true => { let _x_128 = &(policy).obligations; { let _x_129 = &(submission).contributions; { let _x_130 = &(policy).digest; { let _x_131 = coverageAllObligations(&(_x_128), &(_x_129), &(_x_130), &(_x_128)); _x_131 } } } },
    } } } },
    } } },
    } } },
    } } }
}

pub fn coverageCaseCombinedDependencyCycle() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsCCLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = alloc::vec![_x_6]; { let _x_31 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_30.clone() }; { let _x_32 = 3; { let _x_35 = crate::ControlRequirementMode::InheritedRequired; { let _x_36 = coverageOctetsBBLength32(); { let _x_37 = &(_x_36).octets; { let _x_38 = coverageOctets03Length32(); { let _x_39 = &(_x_38).octets; { let _x_40 = alloc::vec![_x_3]; { let _x_41 = crate::ControlObligation { id: _x_32, control: _x_6, mode: _x_35, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_37), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_39), allowedProviders: _x_40, residualControls: _x_21.clone() }; { let _x_42 = 4; { let _x_45 = coverageOctets04Length32(); { let _x_46 = &(_x_45).octets; { let _x_47 = alloc::vec![_x_23]; { let _x_48 = crate::ControlObligation { id: _x_42, control: _x_18, mode: _x_35, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_37), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_46), allowedProviders: _x_47, residualControls: _x_30.clone() }; { let _x_50 = alloc::vec![_x_48]; { let _x_51 = { let mut __list = alloc::vec![_x_41]; __list.extend(_x_50); __list }; { let _x_52 = { let mut __list = alloc::vec![_x_31]; __list.extend(_x_51); __list }; { let _x_53 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_52); __list }; { let _x_54 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_53 }; { let _x_55 = crate::ControlOrigin::Local; { let _x_56 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_55.clone() }; { let _x_57 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_55.clone() }; { let _x_58 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_59 = crate::ControlContribution { obligation: _x_32, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_37), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_39), origin: _x_58 }; { let _x_60 = crate::ControlOrigin::Inherited { field_0: _x_23, field_1: alloc::borrow::ToOwned::to_owned(_x_27), field_2: alloc::borrow::ToOwned::to_owned(_x_29) }; { let _x_61 = crate::ControlContribution { obligation: _x_42, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_37), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_46), origin: _x_60 }; { let _x_63 = alloc::vec![_x_61]; { let _x_64 = { let mut __list = alloc::vec![_x_59]; __list.extend(_x_63); __list }; { let _x_65 = { let mut __list = alloc::vec![_x_57]; __list.extend(_x_64); __list }; { let _x_66 = { let mut __list = alloc::vec![_x_56]; __list.extend(_x_65); __list }; { let _x_67 = crate::ControlSubmission { contributions: _x_66 }; { let _x_68 = validateControlCoverage(&(_x_54), &(_x_67)); _x_68 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseContributionOrder() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_45 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_44 }; { let _x_46 = crate::ControlOrigin::Local; { let _x_47 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_46.clone() }; { let _x_48 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_46.clone() }; { let _x_50 = alloc::vec![_x_48]; { let _x_51 = { let mut __list = alloc::vec![_x_47]; __list.extend(_x_50); __list }; { let _x_52 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_51); __list }; { let _x_53 = crate::ControlSubmission { contributions: _x_52 }; { let _x_54 = validateControlCoverage(&(_x_43), &(_x_53)); _x_54 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseDanglingContribution() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Local; { let _x_45 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_44.clone() }; { let _x_46 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_44.clone() }; { let _x_47 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_48 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_47 }; { let _x_49 = 99; { let _x_52 = crate::ControlContribution { obligation: _x_49, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_44.clone() }; { let _x_54 = alloc::vec![_x_52]; { let _x_55 = { let mut __list = alloc::vec![_x_48]; __list.extend(_x_54); __list }; { let _x_56 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_55); __list }; { let _x_57 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_56); __list }; { let _x_58 = crate::ControlSubmission { contributions: _x_57 }; { let _x_59 = validateControlCoverage(&(_x_43), &(_x_58)); _x_59 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseDanglingProviderPermission() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = 99; { let _x_40 = alloc::vec![_x_37]; { let _x_41 = { let mut __list = alloc::vec![_x_3]; __list.extend(_x_40); __list }; { let _x_42 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_41, residualControls: _x_21.clone() }; { let _x_44 = alloc::vec![_x_42]; { let _x_45 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_44); __list }; { let _x_46 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_45); __list }; { let _x_47 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_46 }; { let _x_48 = crate::ControlOrigin::Local; { let _x_49 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_48.clone() }; { let _x_50 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_48.clone() }; { let _x_51 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_52 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_51 }; { let _x_54 = alloc::vec![_x_52]; { let _x_55 = { let mut __list = alloc::vec![_x_50]; __list.extend(_x_54); __list }; { let _x_56 = { let mut __list = alloc::vec![_x_49]; __list.extend(_x_55); __list }; { let _x_57 = crate::ControlSubmission { contributions: _x_56 }; { let _x_58 = validateControlCoverage(&(_x_47), &(_x_57)); _x_58 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseDanglingResidualControl() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 99; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21 }; { let _x_23 = 2; { let _x_26 = 20; { let _x_29 = coverageOctetsBBLength32(); { let _x_30 = &(_x_29).octets; { let _x_31 = coverageOctets02Length32(); { let _x_32 = &(_x_31).octets; { let _x_33 = crate::ControlObligation { id: _x_23, control: _x_26, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_30), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_32), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_34 = 3; { let _x_37 = crate::ControlRequirementMode::InheritedRequired; { let _x_38 = coverageOctets03Length32(); { let _x_39 = &(_x_38).octets; { let _x_40 = alloc::vec![_x_3]; { let _x_41 = alloc::vec![_x_26]; { let _x_42 = crate::ControlObligation { id: _x_34, control: _x_6, mode: _x_37, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_30), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_39), allowedProviders: _x_40, residualControls: _x_41 }; { let _x_44 = alloc::vec![_x_42]; { let _x_45 = { let mut __list = alloc::vec![_x_33]; __list.extend(_x_44); __list }; { let _x_46 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_45); __list }; { let _x_47 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_46 }; { let _x_48 = crate::ControlOrigin::Local; { let _x_49 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_48.clone() }; { let _x_50 = crate::ControlContribution { obligation: _x_23, control: _x_26, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_30), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_32), origin: _x_48.clone() }; { let _x_51 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_52 = crate::ControlContribution { obligation: _x_34, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_30), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_39), origin: _x_51 }; { let _x_54 = alloc::vec![_x_52]; { let _x_55 = { let mut __list = alloc::vec![_x_50]; __list.extend(_x_54); __list }; { let _x_56 = { let mut __list = alloc::vec![_x_49]; __list.extend(_x_55); __list }; { let _x_57 = crate::ControlSubmission { contributions: _x_56 }; { let _x_58 = validateControlCoverage(&(_x_47), &(_x_57)); _x_58 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseDuplicateBinding() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_39 = 4; { let _x_42 = crate::ControlObligation { id: _x_39, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_44 = alloc::vec![_x_42]; { let _x_45 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_44); __list }; { let _x_46 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_45); __list }; { let _x_47 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_46); __list }; { let _x_48 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_47 }; { let _x_49 = crate::ControlOrigin::Local; { let _x_50 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_49.clone() }; { let _x_51 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_49.clone() }; { let _x_52 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_53 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_52 }; { let _x_54 = crate::ControlContribution { obligation: _x_39, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_49.clone() }; { let _x_56 = alloc::vec![_x_54]; { let _x_57 = { let mut __list = alloc::vec![_x_53]; __list.extend(_x_56); __list }; { let _x_58 = { let mut __list = alloc::vec![_x_51]; __list.extend(_x_57); __list }; { let _x_59 = { let mut __list = alloc::vec![_x_50]; __list.extend(_x_58); __list }; { let _x_60 = crate::ControlSubmission { contributions: _x_59 }; { let _x_61 = validateControlCoverage(&(_x_48), &(_x_60)); _x_61 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseDuplicateContribution() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Local; { let _x_45 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_44.clone() }; { let _x_46 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_44.clone() }; { let _x_47 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_48 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_47 }; { let _x_50 = alloc::vec![_x_45.clone()]; { let _x_51 = { let mut __list = alloc::vec![_x_48]; __list.extend(_x_50); __list }; { let _x_52 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_51); __list }; { let _x_53 = { let mut __list = alloc::vec![_x_45.clone()]; __list.extend(_x_52); __list }; { let _x_54 = crate::ControlSubmission { contributions: _x_53 }; { let _x_55 = validateControlCoverage(&(_x_43), &(_x_54)); _x_55 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseDuplicateObligation() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_22.clone()]; { let _x_41 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_41); __list }; { let _x_43 = { let mut __list = alloc::vec![_x_22.clone()]; __list.extend(_x_42); __list }; { let _x_44 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_43 }; { let _x_45 = crate::ControlOrigin::Local; { let _x_46 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_45.clone() }; { let _x_47 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_45.clone() }; { let _x_48 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_49 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_48 }; { let _x_51 = alloc::vec![_x_49]; { let _x_52 = { let mut __list = alloc::vec![_x_47]; __list.extend(_x_51); __list }; { let _x_53 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_52); __list }; { let _x_54 = crate::ControlSubmission { contributions: _x_53 }; { let _x_55 = validateControlCoverage(&(_x_44), &(_x_54)); _x_55 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseDuplicateProviderPermission() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = { let mut __list = alloc::vec![_x_3]; __list.extend(_x_37); __list }; { let _x_39 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_38, residualControls: _x_21.clone() }; { let _x_41 = alloc::vec![_x_39]; { let _x_42 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_41); __list }; { let _x_43 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_42); __list }; { let _x_44 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_43 }; { let _x_45 = crate::ControlOrigin::Local; { let _x_46 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_45.clone() }; { let _x_47 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_45.clone() }; { let _x_48 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_49 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_48 }; { let _x_51 = alloc::vec![_x_49]; { let _x_52 = { let mut __list = alloc::vec![_x_47]; __list.extend(_x_51); __list }; { let _x_53 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_52); __list }; { let _x_54 = crate::ControlSubmission { contributions: _x_53 }; { let _x_55 = validateControlCoverage(&(_x_44), &(_x_54)); _x_55 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseDuplicateResidual() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = { let mut __list = alloc::vec![_x_18]; __list.extend(_x_21.clone()); __list }; { let _x_23 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_22 }; { let _x_24 = 2; { let _x_27 = coverageOctetsBBLength32(); { let _x_28 = &(_x_27).octets; { let _x_29 = coverageOctets02Length32(); { let _x_30 = &(_x_29).octets; { let _x_31 = crate::ControlObligation { id: _x_24, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_30), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_32 = 3; { let _x_35 = crate::ControlRequirementMode::InheritedRequired; { let _x_36 = coverageOctets03Length32(); { let _x_37 = &(_x_36).octets; { let _x_38 = alloc::vec![_x_3]; { let _x_39 = crate::ControlObligation { id: _x_32, control: _x_6, mode: _x_35, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_37), allowedProviders: _x_38, residualControls: _x_21.clone() }; { let _x_41 = alloc::vec![_x_39]; { let _x_42 = { let mut __list = alloc::vec![_x_31]; __list.extend(_x_41); __list }; { let _x_43 = { let mut __list = alloc::vec![_x_23]; __list.extend(_x_42); __list }; { let _x_44 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_43 }; { let _x_45 = crate::ControlOrigin::Local; { let _x_46 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_45.clone() }; { let _x_47 = crate::ControlContribution { obligation: _x_24, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_30), origin: _x_45.clone() }; { let _x_48 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_49 = crate::ControlContribution { obligation: _x_32, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_37), origin: _x_48 }; { let _x_51 = alloc::vec![_x_49]; { let _x_52 = { let mut __list = alloc::vec![_x_47]; __list.extend(_x_51); __list }; { let _x_53 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_52); __list }; { let _x_54 = crate::ControlSubmission { contributions: _x_53 }; { let _x_55 = validateControlCoverage(&(_x_44), &(_x_54)); _x_55 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseEmptyEvidenceKind() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from(""), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_29 = coverageOctets02Length32(); { let _x_30 = &(_x_29).octets; { let _x_31 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_30), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_32 = 3; { let _x_35 = crate::ControlRequirementMode::InheritedRequired; { let _x_36 = coverageOctets03Length32(); { let _x_37 = &(_x_36).octets; { let _x_38 = alloc::vec![_x_3]; { let _x_39 = crate::ControlObligation { id: _x_32, control: _x_6, mode: _x_35, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_37), allowedProviders: _x_38, residualControls: _x_21.clone() }; { let _x_41 = alloc::vec![_x_39]; { let _x_42 = { let mut __list = alloc::vec![_x_31]; __list.extend(_x_41); __list }; { let _x_43 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_42); __list }; { let _x_44 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_43 }; { let _x_45 = crate::ControlOrigin::Local; { let _x_46 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_45.clone() }; { let _x_47 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_30), origin: _x_45.clone() }; { let _x_48 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_49 = crate::ControlContribution { obligation: _x_32, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_37), origin: _x_48 }; { let _x_51 = alloc::vec![_x_49]; { let _x_52 = { let mut __list = alloc::vec![_x_47]; __list.extend(_x_51); __list }; { let _x_53 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_52); __list }; { let _x_54 = crate::ControlSubmission { contributions: _x_53 }; { let _x_55 = validateControlCoverage(&(_x_44), &(_x_54)); _x_55 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseEmptyPolicy() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_4 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: alloc::vec::Vec::new() }; { let _x_6 = crate::ControlSubmission { contributions: alloc::vec::Vec::new() }; { let _x_7 = validateControlCoverage(&(_x_4), &(_x_6)); _x_7 } } } } }
}

pub fn coverageCaseEmptyScope() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from(""), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_27 = coverageOctetsBBLength32(); { let _x_28 = &(_x_27).octets; { let _x_29 = coverageOctets02Length32(); { let _x_30 = &(_x_29).octets; { let _x_31 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_30), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_32 = 3; { let _x_35 = crate::ControlRequirementMode::InheritedRequired; { let _x_36 = coverageOctets03Length32(); { let _x_37 = &(_x_36).octets; { let _x_38 = alloc::vec![_x_3]; { let _x_39 = crate::ControlObligation { id: _x_32, control: _x_6, mode: _x_35, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_37), allowedProviders: _x_38, residualControls: _x_21.clone() }; { let _x_41 = alloc::vec![_x_39]; { let _x_42 = { let mut __list = alloc::vec![_x_31]; __list.extend(_x_41); __list }; { let _x_43 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_42); __list }; { let _x_44 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_43 }; { let _x_45 = crate::ControlOrigin::Local; { let _x_46 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_45.clone() }; { let _x_47 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_30), origin: _x_45.clone() }; { let _x_48 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_49 = crate::ControlContribution { obligation: _x_32, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_37), origin: _x_48 }; { let _x_51 = alloc::vec![_x_49]; { let _x_52 = { let mut __list = alloc::vec![_x_47]; __list.extend(_x_51); __list }; { let _x_53 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_52); __list }; { let _x_54 = crate::ControlSubmission { contributions: _x_53 }; { let _x_55 = validateControlCoverage(&(_x_44), &(_x_54)); _x_55 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseEmptyVersion() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from(""), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_27 = coverageOctetsBBLength32(); { let _x_28 = &(_x_27).octets; { let _x_29 = coverageOctets02Length32(); { let _x_30 = &(_x_29).octets; { let _x_31 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_30), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_32 = 3; { let _x_35 = crate::ControlRequirementMode::InheritedRequired; { let _x_36 = coverageOctets03Length32(); { let _x_37 = &(_x_36).octets; { let _x_38 = alloc::vec![_x_3]; { let _x_39 = crate::ControlObligation { id: _x_32, control: _x_6, mode: _x_35, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_37), allowedProviders: _x_38, residualControls: _x_21.clone() }; { let _x_41 = alloc::vec![_x_39]; { let _x_42 = { let mut __list = alloc::vec![_x_31]; __list.extend(_x_41); __list }; { let _x_43 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_42); __list }; { let _x_44 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_43 }; { let _x_45 = crate::ControlOrigin::Local; { let _x_46 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_45.clone() }; { let _x_47 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_30), origin: _x_45.clone() }; { let _x_48 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_49 = crate::ControlContribution { obligation: _x_32, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_37), origin: _x_48 }; { let _x_51 = alloc::vec![_x_49]; { let _x_52 = { let mut __list = alloc::vec![_x_47]; __list.extend(_x_51); __list }; { let _x_53 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_52); __list }; { let _x_54 = crate::ControlSubmission { contributions: _x_53 }; { let _x_55 = validateControlCoverage(&(_x_44), &(_x_54)); _x_55 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseIndependentOrder() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 2; { let _x_6 = 20; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsBBLength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets02Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_19 = 1; { let _x_22 = 10; { let _x_25 = coverageOctetsAALength32(); { let _x_26 = &(_x_25).octets; { let _x_27 = coverageOctets01Length32(); { let _x_28 = &(_x_27).octets; { let _x_29 = alloc::vec![_x_6]; { let _x_30 = crate::ControlObligation { id: _x_19, control: _x_22, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_26), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_28), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_29.clone() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_19]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_22, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_29.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_18]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Local; { let _x_45 = crate::ControlContribution { obligation: _x_19, control: _x_22, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_26), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_28), origin: _x_44.clone() }; { let _x_46 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_44.clone() }; { let _x_47 = crate::ControlOrigin::Inherited { field_0: _x_19, field_1: alloc::borrow::ToOwned::to_owned(_x_26), field_2: alloc::borrow::ToOwned::to_owned(_x_28) }; { let _x_48 = crate::ControlContribution { obligation: _x_31, control: _x_22, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_47 }; { let _x_50 = alloc::vec![_x_48]; { let _x_51 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_50); __list }; { let _x_52 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_51); __list }; { let _x_53 = crate::ControlSubmission { contributions: _x_52 }; { let _x_54 = validateControlCoverage(&(_x_43), &(_x_53)); _x_54 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseInheritedResidual() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Local; { let _x_45 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_44.clone() }; { let _x_46 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_44.clone() }; { let _x_47 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_48 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_47 }; { let _x_50 = alloc::vec![_x_48]; { let _x_51 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_50); __list }; { let _x_52 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_51); __list }; { let _x_53 = crate::ControlSubmission { contributions: _x_52 }; { let _x_54 = validateControlCoverage(&(_x_43), &(_x_53)); _x_54 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseLocal() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_20 = alloc::vec![_x_18]; { let _x_21 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_20 }; { let _x_22 = crate::ControlOrigin::Local; { let _x_23 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_22 }; { let _x_25 = alloc::vec![_x_23]; { let _x_26 = crate::ControlSubmission { contributions: _x_25 }; { let _x_27 = validateControlCoverage(&(_x_21), &(_x_26)); _x_27 } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseMalformedObligationEvidence() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets00Length1(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = coverageOctets01Length32(); { let _x_45 = &(_x_44).octets; { let _x_46 = crate::ControlOrigin::Local; { let _x_47 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_45), origin: _x_46.clone() }; { let _x_48 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_46.clone() }; { let _x_49 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_45) }; { let _x_50 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_49 }; { let _x_52 = alloc::vec![_x_50]; { let _x_53 = { let mut __list = alloc::vec![_x_48]; __list.extend(_x_52); __list }; { let _x_54 = { let mut __list = alloc::vec![_x_47]; __list.extend(_x_53); __list }; { let _x_55 = crate::ControlSubmission { contributions: _x_54 }; { let _x_56 = validateControlCoverage(&(_x_43), &(_x_55)); _x_56 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseMalformedObligationSubject() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsEmptyLength0(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = coverageOctetsAALength32(); { let _x_45 = &(_x_44).octets; { let _x_46 = crate::ControlOrigin::Local; { let _x_47 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_45), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_46.clone() }; { let _x_48 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_46.clone() }; { let _x_49 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_45), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_50 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_49 }; { let _x_52 = alloc::vec![_x_50]; { let _x_53 = { let mut __list = alloc::vec![_x_48]; __list.extend(_x_52); __list }; { let _x_54 = { let mut __list = alloc::vec![_x_47]; __list.extend(_x_53); __list }; { let _x_55 = crate::ControlSubmission { contributions: _x_54 }; { let _x_56 = validateControlCoverage(&(_x_43), &(_x_55)); _x_56 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseMalformedPolicyDigest() -> bool {
    { let _x_1 = coverageOctets11Length1(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = coverageOctets11Length32(); { let _x_45 = &(_x_44).octets; { let _x_46 = crate::ControlOrigin::Local; { let _x_47 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_45), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_46.clone() }; { let _x_48 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_45), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_46.clone() }; { let _x_49 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_50 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_45), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_49 }; { let _x_52 = alloc::vec![_x_50]; { let _x_53 = { let mut __list = alloc::vec![_x_48]; __list.extend(_x_52); __list }; { let _x_54 = { let mut __list = alloc::vec![_x_47]; __list.extend(_x_53); __list }; { let _x_55 = crate::ControlSubmission { contributions: _x_54 }; { let _x_56 = validateControlCoverage(&(_x_43), &(_x_55)); _x_56 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseMissingContribution() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Local; { let _x_45 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_44.clone() }; { let _x_46 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_44.clone() }; { let _x_48 = alloc::vec![_x_46]; { let _x_49 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_48); __list }; { let _x_50 = crate::ControlSubmission { contributions: _x_49 }; { let _x_51 = validateControlCoverage(&(_x_43), &(_x_50)); _x_51 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseNonTopological() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 3; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::InheritedRequired; { let _x_12 = coverageOctetsBBLength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets03Length32(); { let _x_16 = &(_x_15).octets; { let _x_17 = 1; { let _x_21 = alloc::vec![_x_17]; { let _x_22 = 20; { let _x_25 = alloc::vec![_x_22]; { let _x_26 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: _x_21, residualControls: _x_25.clone() }; { let _x_27 = 2; { let _x_30 = crate::ControlRequirementMode::LocalRequired; { let _x_31 = coverageOctets02Length32(); { let _x_32 = &(_x_31).octets; { let _x_33 = crate::ControlObligation { id: _x_27, control: _x_22, mode: _x_30, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_32), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_34 = coverageOctetsAALength32(); { let _x_35 = &(_x_34).octets; { let _x_36 = coverageOctets01Length32(); { let _x_37 = &(_x_36).octets; { let _x_38 = crate::ControlObligation { id: _x_17, control: _x_6, mode: _x_30, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_35), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_37), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_25.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_33]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_26]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Local; { let _x_45 = crate::ControlContribution { obligation: _x_17, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_35), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_37), origin: _x_44.clone() }; { let _x_46 = crate::ControlContribution { obligation: _x_27, control: _x_22, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_32), origin: _x_44.clone() }; { let _x_47 = crate::ControlOrigin::Inherited { field_0: _x_17, field_1: alloc::borrow::ToOwned::to_owned(_x_35), field_2: alloc::borrow::ToOwned::to_owned(_x_37) }; { let _x_48 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_47 }; { let _x_50 = alloc::vec![_x_48]; { let _x_51 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_50); __list }; { let _x_52 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_51); __list }; { let _x_53 = crate::ControlSubmission { contributions: _x_52 }; { let _x_54 = validateControlCoverage(&(_x_43), &(_x_53)); _x_54 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseProviderWrongControl() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 30; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = 10; { let _x_37 = crate::ControlRequirementMode::InheritedRequired; { let _x_38 = coverageOctets03Length32(); { let _x_39 = &(_x_38).octets; { let _x_40 = alloc::vec![_x_3]; { let _x_41 = crate::ControlObligation { id: _x_31, control: _x_34, mode: _x_37, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_39), allowedProviders: _x_40, residualControls: _x_21.clone() }; { let _x_43 = alloc::vec![_x_41]; { let _x_44 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_43); __list }; { let _x_45 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_44); __list }; { let _x_46 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_45 }; { let _x_47 = crate::ControlOrigin::Local; { let _x_48 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_47.clone() }; { let _x_49 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_47.clone() }; { let _x_50 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_51 = crate::ControlContribution { obligation: _x_31, control: _x_34, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_39), origin: _x_50 }; { let _x_53 = alloc::vec![_x_51]; { let _x_54 = { let mut __list = alloc::vec![_x_49]; __list.extend(_x_53); __list }; { let _x_55 = { let mut __list = alloc::vec![_x_48]; __list.extend(_x_54); __list }; { let _x_56 = crate::ControlSubmission { contributions: _x_55 }; { let _x_57 = validateControlCoverage(&(_x_46), &(_x_56)); _x_57 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseProviderWrongEvidenceKind() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("schema-only"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_29 = coverageOctets02Length32(); { let _x_30 = &(_x_29).octets; { let _x_31 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_30), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_32 = 3; { let _x_35 = crate::ControlRequirementMode::InheritedRequired; { let _x_36 = coverageOctets03Length32(); { let _x_37 = &(_x_36).octets; { let _x_38 = alloc::vec![_x_3]; { let _x_39 = crate::ControlObligation { id: _x_32, control: _x_6, mode: _x_35, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_37), allowedProviders: _x_38, residualControls: _x_21.clone() }; { let _x_41 = alloc::vec![_x_39]; { let _x_42 = { let mut __list = alloc::vec![_x_31]; __list.extend(_x_41); __list }; { let _x_43 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_42); __list }; { let _x_44 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_43 }; { let _x_45 = crate::ControlOrigin::Local; { let _x_46 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("schema-only"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_45.clone() }; { let _x_47 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_30), origin: _x_45.clone() }; { let _x_48 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_49 = crate::ControlContribution { obligation: _x_32, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_37), origin: _x_48 }; { let _x_51 = alloc::vec![_x_49]; { let _x_52 = { let mut __list = alloc::vec![_x_47]; __list.extend(_x_51); __list }; { let _x_53 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_52); __list }; { let _x_54 = crate::ControlSubmission { contributions: _x_53 }; { let _x_55 = validateControlCoverage(&(_x_44), &(_x_54)); _x_55 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseProviderWrongScope() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("other"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_27 = coverageOctetsBBLength32(); { let _x_28 = &(_x_27).octets; { let _x_29 = coverageOctets02Length32(); { let _x_30 = &(_x_29).octets; { let _x_31 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_30), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_32 = 3; { let _x_35 = crate::ControlRequirementMode::InheritedRequired; { let _x_36 = coverageOctets03Length32(); { let _x_37 = &(_x_36).octets; { let _x_38 = alloc::vec![_x_3]; { let _x_39 = crate::ControlObligation { id: _x_32, control: _x_6, mode: _x_35, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_37), allowedProviders: _x_38, residualControls: _x_21.clone() }; { let _x_41 = alloc::vec![_x_39]; { let _x_42 = { let mut __list = alloc::vec![_x_31]; __list.extend(_x_41); __list }; { let _x_43 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_42); __list }; { let _x_44 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_43 }; { let _x_45 = crate::ControlOrigin::Local; { let _x_46 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("other"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_45.clone() }; { let _x_47 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_30), origin: _x_45.clone() }; { let _x_48 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_49 = crate::ControlContribution { obligation: _x_32, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_37), origin: _x_48 }; { let _x_51 = alloc::vec![_x_49]; { let _x_52 = { let mut __list = alloc::vec![_x_47]; __list.extend(_x_51); __list }; { let _x_53 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_52); __list }; { let _x_54 = crate::ControlSubmission { contributions: _x_53 }; { let _x_55 = validateControlCoverage(&(_x_44), &(_x_54)); _x_55 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseProviderWrongVersion() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("2"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_27 = coverageOctetsBBLength32(); { let _x_28 = &(_x_27).octets; { let _x_29 = coverageOctets02Length32(); { let _x_30 = &(_x_29).octets; { let _x_31 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_30), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_32 = 3; { let _x_35 = crate::ControlRequirementMode::InheritedRequired; { let _x_36 = coverageOctets03Length32(); { let _x_37 = &(_x_36).octets; { let _x_38 = alloc::vec![_x_3]; { let _x_39 = crate::ControlObligation { id: _x_32, control: _x_6, mode: _x_35, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_37), allowedProviders: _x_38, residualControls: _x_21.clone() }; { let _x_41 = alloc::vec![_x_39]; { let _x_42 = { let mut __list = alloc::vec![_x_31]; __list.extend(_x_41); __list }; { let _x_43 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_42); __list }; { let _x_44 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_43 }; { let _x_45 = crate::ControlOrigin::Local; { let _x_46 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("2"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_45.clone() }; { let _x_47 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_30), origin: _x_45.clone() }; { let _x_48 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_49 = crate::ControlContribution { obligation: _x_32, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_37), origin: _x_48 }; { let _x_51 = alloc::vec![_x_49]; { let _x_52 = { let mut __list = alloc::vec![_x_47]; __list.extend(_x_51); __list }; { let _x_53 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_52); __list }; { let _x_54 = crate::ControlSubmission { contributions: _x_53 }; { let _x_55 = validateControlCoverage(&(_x_44), &(_x_54)); _x_55 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseRelabeledInheritedAsLocal() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Local; { let _x_45 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_44.clone() }; { let _x_46 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_44.clone() }; { let _x_47 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_44.clone() }; { let _x_49 = alloc::vec![_x_47]; { let _x_50 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_49); __list }; { let _x_51 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_50); __list }; { let _x_52 = crate::ControlSubmission { contributions: _x_51 }; { let _x_53 = validateControlCoverage(&(_x_43), &(_x_52)); _x_53 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseRelabeledLocalAsInherited() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Inherited { field_0: _x_31, field_1: alloc::borrow::ToOwned::to_owned(_x_27), field_2: alloc::borrow::ToOwned::to_owned(_x_36) }; { let _x_45 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_44 }; { let _x_46 = crate::ControlOrigin::Local; { let _x_47 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_46 }; { let _x_48 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_49 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_48 }; { let _x_51 = alloc::vec![_x_49]; { let _x_52 = { let mut __list = alloc::vec![_x_47]; __list.extend(_x_51); __list }; { let _x_53 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_52); __list }; { let _x_54 = crate::ControlSubmission { contributions: _x_53 }; { let _x_55 = validateControlCoverage(&(_x_43), &(_x_54)); _x_55 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseResidualPolicyDropped() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21 }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: alloc::vec::Vec::new() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Local; { let _x_45 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_44.clone() }; { let _x_46 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_44.clone() }; { let _x_47 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_48 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_47 }; { let _x_50 = alloc::vec![_x_48]; { let _x_51 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_50); __list }; { let _x_52 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_51); __list }; { let _x_53 = crate::ControlSubmission { contributions: _x_52 }; { let _x_54 = validateControlCoverage(&(_x_43), &(_x_53)); _x_54 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseResidualWrongScope() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_27 = coverageOctetsBBLength32(); { let _x_28 = &(_x_27).octets; { let _x_29 = coverageOctets02Length32(); { let _x_30 = &(_x_29).octets; { let _x_31 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("other"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_30), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_32 = 3; { let _x_35 = crate::ControlRequirementMode::InheritedRequired; { let _x_36 = coverageOctets03Length32(); { let _x_37 = &(_x_36).octets; { let _x_38 = alloc::vec![_x_3]; { let _x_39 = crate::ControlObligation { id: _x_32, control: _x_6, mode: _x_35, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_37), allowedProviders: _x_38, residualControls: _x_21.clone() }; { let _x_41 = alloc::vec![_x_39]; { let _x_42 = { let mut __list = alloc::vec![_x_31]; __list.extend(_x_41); __list }; { let _x_43 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_42); __list }; { let _x_44 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_43 }; { let _x_45 = crate::ControlOrigin::Local; { let _x_46 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_45.clone() }; { let _x_47 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("other"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_30), origin: _x_45.clone() }; { let _x_48 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_49 = crate::ControlContribution { obligation: _x_32, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_37), origin: _x_48 }; { let _x_51 = alloc::vec![_x_49]; { let _x_52 = { let mut __list = alloc::vec![_x_47]; __list.extend(_x_51); __list }; { let _x_53 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_52); __list }; { let _x_54 = crate::ControlSubmission { contributions: _x_53 }; { let _x_55 = validateControlCoverage(&(_x_44), &(_x_54)); _x_55 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseResidualWrongSubject() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsCCLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctetsBBLength32(); { let _x_36 = &(_x_35).octets; { let _x_37 = coverageOctets03Length32(); { let _x_38 = &(_x_37).octets; { let _x_39 = alloc::vec![_x_3]; { let _x_40 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_36), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_38), allowedProviders: _x_39, residualControls: _x_21.clone() }; { let _x_42 = alloc::vec![_x_40]; { let _x_43 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_42); __list }; { let _x_44 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_43); __list }; { let _x_45 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_44 }; { let _x_46 = crate::ControlOrigin::Local; { let _x_47 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_46.clone() }; { let _x_48 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_46.clone() }; { let _x_49 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_50 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_36), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_38), origin: _x_49 }; { let _x_52 = alloc::vec![_x_50]; { let _x_53 = { let mut __list = alloc::vec![_x_48]; __list.extend(_x_52); __list }; { let _x_54 = { let mut __list = alloc::vec![_x_47]; __list.extend(_x_53); __list }; { let _x_55 = crate::ControlSubmission { contributions: _x_54 }; { let _x_56 = validateControlCoverage(&(_x_45), &(_x_55)); _x_56 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseResidualWrongVersion() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_27 = coverageOctetsBBLength32(); { let _x_28 = &(_x_27).octets; { let _x_29 = coverageOctets02Length32(); { let _x_30 = &(_x_29).octets; { let _x_31 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("2"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_30), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_32 = 3; { let _x_35 = crate::ControlRequirementMode::InheritedRequired; { let _x_36 = coverageOctets03Length32(); { let _x_37 = &(_x_36).octets; { let _x_38 = alloc::vec![_x_3]; { let _x_39 = crate::ControlObligation { id: _x_32, control: _x_6, mode: _x_35, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_37), allowedProviders: _x_38, residualControls: _x_21.clone() }; { let _x_41 = alloc::vec![_x_39]; { let _x_42 = { let mut __list = alloc::vec![_x_31]; __list.extend(_x_41); __list }; { let _x_43 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_42); __list }; { let _x_44 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_43 }; { let _x_45 = crate::ControlOrigin::Local; { let _x_46 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_45.clone() }; { let _x_47 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("2"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_30), origin: _x_45.clone() }; { let _x_48 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_49 = crate::ControlContribution { obligation: _x_32, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_28), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_37), origin: _x_48 }; { let _x_51 = alloc::vec![_x_49]; { let _x_52 = { let mut __list = alloc::vec![_x_47]; __list.extend(_x_51); __list }; { let _x_53 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_52); __list }; { let _x_54 = crate::ControlSubmission { contributions: _x_53 }; { let _x_55 = validateControlCoverage(&(_x_44), &(_x_54)); _x_55 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseSelfCycle() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::InheritedRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = alloc::vec![_x_3]; { let _x_19 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: _x_18, residualControls: alloc::vec::Vec::new() }; { let _x_21 = alloc::vec![_x_19]; { let _x_22 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_21 }; { let _x_23 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_24 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_23 }; { let _x_26 = alloc::vec![_x_24]; { let _x_27 = crate::ControlSubmission { contributions: _x_26 }; { let _x_28 = validateControlCoverage(&(_x_22), &(_x_27)); _x_28 } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseSharedProvider() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37.clone(), residualControls: _x_21.clone() }; { let _x_39 = 4; { let _x_42 = coverageOctetsCCLength32(); { let _x_43 = &(_x_42).octets; { let _x_44 = coverageOctets04Length32(); { let _x_45 = &(_x_44).octets; { let _x_46 = crate::ControlObligation { id: _x_39, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_43), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_45), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_47 = 5; { let _x_50 = coverageOctets05Length32(); { let _x_51 = &(_x_50).octets; { let _x_52 = crate::ControlObligation { id: _x_47, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_43), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_51), allowedProviders: _x_37.clone(), residualControls: _x_21.clone() }; { let _x_54 = alloc::vec![_x_52]; { let _x_55 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_54); __list }; { let _x_56 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_55); __list }; { let _x_57 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_56); __list }; { let _x_58 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_57); __list }; { let _x_59 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_58 }; { let _x_60 = crate::ControlOrigin::Local; { let _x_61 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_60.clone() }; { let _x_62 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_60.clone() }; { let _x_63 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_64 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_63.clone() }; { let _x_65 = crate::ControlContribution { obligation: _x_39, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_43), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_45), origin: _x_60.clone() }; { let _x_66 = crate::ControlContribution { obligation: _x_47, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_43), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_51), origin: _x_63.clone() }; { let _x_68 = alloc::vec![_x_66]; { let _x_69 = { let mut __list = alloc::vec![_x_65]; __list.extend(_x_68); __list }; { let _x_70 = { let mut __list = alloc::vec![_x_64]; __list.extend(_x_69); __list }; { let _x_71 = { let mut __list = alloc::vec![_x_62]; __list.extend(_x_70); __list }; { let _x_72 = { let mut __list = alloc::vec![_x_61]; __list.extend(_x_71); __list }; { let _x_73 = crate::ControlSubmission { contributions: _x_72 }; { let _x_74 = validateControlCoverage(&(_x_59), &(_x_73)); _x_74 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseTransitiveInheritance() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_39 = 4; { let _x_42 = coverageOctetsCCLength32(); { let _x_43 = &(_x_42).octets; { let _x_44 = coverageOctets04Length32(); { let _x_45 = &(_x_44).octets; { let _x_46 = crate::ControlObligation { id: _x_39, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_43), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_45), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_47 = 5; { let _x_50 = coverageOctets05Length32(); { let _x_51 = &(_x_50).octets; { let _x_52 = alloc::vec![_x_31]; { let _x_53 = crate::ControlObligation { id: _x_47, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_43), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_51), allowedProviders: _x_52, residualControls: _x_21.clone() }; { let _x_55 = alloc::vec![_x_53]; { let _x_56 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_55); __list }; { let _x_57 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_56); __list }; { let _x_58 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_57); __list }; { let _x_59 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_58); __list }; { let _x_60 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_59 }; { let _x_61 = crate::ControlOrigin::Local; { let _x_62 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_61.clone() }; { let _x_63 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_61.clone() }; { let _x_64 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_65 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_64 }; { let _x_66 = crate::ControlContribution { obligation: _x_39, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_43), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_45), origin: _x_61.clone() }; { let _x_67 = crate::ControlOrigin::Inherited { field_0: _x_31, field_1: alloc::borrow::ToOwned::to_owned(_x_27), field_2: alloc::borrow::ToOwned::to_owned(_x_36) }; { let _x_68 = crate::ControlContribution { obligation: _x_47, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_43), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_51), origin: _x_67 }; { let _x_70 = alloc::vec![_x_68]; { let _x_71 = { let mut __list = alloc::vec![_x_66]; __list.extend(_x_70); __list }; { let _x_72 = { let mut __list = alloc::vec![_x_65]; __list.extend(_x_71); __list }; { let _x_73 = { let mut __list = alloc::vec![_x_63]; __list.extend(_x_72); __list }; { let _x_74 = { let mut __list = alloc::vec![_x_62]; __list.extend(_x_73); __list }; { let _x_75 = crate::ControlSubmission { contributions: _x_74 }; { let _x_76 = validateControlCoverage(&(_x_60), &(_x_75)); _x_76 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseTwoNodeCycle() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::InheritedRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_17 = 2; { let _x_21 = alloc::vec![_x_17]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: _x_21, residualControls: alloc::vec::Vec::new() }; { let _x_23 = coverageOctetsBBLength32(); { let _x_24 = &(_x_23).octets; { let _x_25 = coverageOctets02Length32(); { let _x_26 = &(_x_25).octets; { let _x_27 = alloc::vec![_x_3]; { let _x_28 = crate::ControlObligation { id: _x_17, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_24), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_26), allowedProviders: _x_27, residualControls: alloc::vec::Vec::new() }; { let _x_30 = alloc::vec![_x_28]; { let _x_31 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_30); __list }; { let _x_32 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_31 }; { let _x_33 = crate::ControlOrigin::Inherited { field_0: _x_17, field_1: alloc::borrow::ToOwned::to_owned(_x_24), field_2: alloc::borrow::ToOwned::to_owned(_x_26) }; { let _x_34 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_33 }; { let _x_35 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_36 = crate::ControlContribution { obligation: _x_17, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_24), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_26), origin: _x_35 }; { let _x_38 = alloc::vec![_x_36]; { let _x_39 = { let mut __list = alloc::vec![_x_34]; __list.extend(_x_38); __list }; { let _x_40 = crate::ControlSubmission { contributions: _x_39 }; { let _x_41 = validateControlCoverage(&(_x_32), &(_x_40)); _x_41 } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseUnauthorizedProvider() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_39 = alloc::vec![_x_37]; { let _x_40 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_39); __list }; { let _x_41 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_40); __list }; { let _x_42 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_41 }; { let _x_43 = crate::ControlOrigin::Local; { let _x_44 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_43.clone() }; { let _x_45 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_43.clone() }; { let _x_46 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_47 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_46 }; { let _x_49 = alloc::vec![_x_47]; { let _x_50 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_49); __list }; { let _x_51 = { let mut __list = alloc::vec![_x_44]; __list.extend(_x_50); __list }; { let _x_52 = crate::ControlSubmission { contributions: _x_51 }; { let _x_53 = validateControlCoverage(&(_x_42), &(_x_52)); _x_53 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseUncoveredProvider() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Local; { let _x_45 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_44 }; { let _x_46 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_47 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_46 }; { let _x_49 = alloc::vec![_x_47]; { let _x_50 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_49); __list }; { let _x_51 = crate::ControlSubmission { contributions: _x_50 }; { let _x_52 = validateControlCoverage(&(_x_43), &(_x_51)); _x_52 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseUncoveredResidual() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Local; { let _x_45 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_44 }; { let _x_46 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_47 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_46 }; { let _x_49 = alloc::vec![_x_47]; { let _x_50 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_49); __list }; { let _x_51 = crate::ControlSubmission { contributions: _x_50 }; { let _x_52 = validateControlCoverage(&(_x_43), &(_x_51)); _x_52 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseWrongControl() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Local; { let _x_45 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_44.clone() }; { let _x_46 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_44.clone() }; { let _x_47 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_48 = crate::ControlContribution { obligation: _x_31, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_47 }; { let _x_50 = alloc::vec![_x_48]; { let _x_51 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_50); __list }; { let _x_52 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_51); __list }; { let _x_53 = crate::ControlSubmission { contributions: _x_52 }; { let _x_54 = validateControlCoverage(&(_x_43), &(_x_53)); _x_54 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseWrongEvidence() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Local; { let _x_45 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_44.clone() }; { let _x_46 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_44.clone() }; { let _x_47 = coverageOctets09Length32(); { let _x_48 = &(_x_47).octets; { let _x_49 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_50 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_48), origin: _x_49 }; { let _x_52 = alloc::vec![_x_50]; { let _x_53 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_52); __list }; { let _x_54 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_53); __list }; { let _x_55 = crate::ControlSubmission { contributions: _x_54 }; { let _x_56 = validateControlCoverage(&(_x_43), &(_x_55)); _x_56 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseWrongEvidenceKind() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Local; { let _x_45 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_44.clone() }; { let _x_46 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_44.clone() }; { let _x_48 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_49 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("schema-only"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_48 }; { let _x_51 = alloc::vec![_x_49]; { let _x_52 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_51); __list }; { let _x_53 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_52); __list }; { let _x_54 = crate::ControlSubmission { contributions: _x_53 }; { let _x_55 = validateControlCoverage(&(_x_43), &(_x_54)); _x_55 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseWrongMalformedEvidence() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Local; { let _x_45 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_44.clone() }; { let _x_46 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_44.clone() }; { let _x_47 = coverageOctetsFFLength1(); { let _x_48 = &(_x_47).octets; { let _x_49 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_50 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_48), origin: _x_49 }; { let _x_52 = alloc::vec![_x_50]; { let _x_53 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_52); __list }; { let _x_54 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_53); __list }; { let _x_55 = crate::ControlSubmission { contributions: _x_54 }; { let _x_56 = validateControlCoverage(&(_x_43), &(_x_55)); _x_56 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseWrongMalformedSubject() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Local; { let _x_45 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_44.clone() }; { let _x_46 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_44.clone() }; { let _x_47 = coverageOctetsAALength1(); { let _x_48 = &(_x_47).octets; { let _x_49 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_50 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_48), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_49 }; { let _x_52 = alloc::vec![_x_50]; { let _x_53 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_52); __list }; { let _x_54 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_53); __list }; { let _x_55 = crate::ControlSubmission { contributions: _x_54 }; { let _x_56 = validateControlCoverage(&(_x_43), &(_x_55)); _x_56 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseWrongPolicy() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Local; { let _x_45 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_44.clone() }; { let _x_46 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_44.clone() }; { let _x_47 = coverageOctets22Length32(); { let _x_48 = &(_x_47).octets; { let _x_49 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_50 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_48), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_49 }; { let _x_52 = alloc::vec![_x_50]; { let _x_53 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_52); __list }; { let _x_54 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_53); __list }; { let _x_55 = crate::ControlSubmission { contributions: _x_54 }; { let _x_56 = validateControlCoverage(&(_x_43), &(_x_55)); _x_56 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseWrongPolicyDigest() -> bool {
    { let _x_1 = coverageOctets22Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = coverageOctets11Length32(); { let _x_45 = &(_x_44).octets; { let _x_46 = crate::ControlOrigin::Local; { let _x_47 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_45), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_46.clone() }; { let _x_48 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_45), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_46.clone() }; { let _x_49 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_50 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_45), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_49 }; { let _x_52 = alloc::vec![_x_50]; { let _x_53 = { let mut __list = alloc::vec![_x_48]; __list.extend(_x_52); __list }; { let _x_54 = { let mut __list = alloc::vec![_x_47]; __list.extend(_x_53); __list }; { let _x_55 = crate::ControlSubmission { contributions: _x_54 }; { let _x_56 = validateControlCoverage(&(_x_43), &(_x_55)); _x_56 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseWrongProvider() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Local; { let _x_45 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_44.clone() }; { let _x_46 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_44.clone() }; { let _x_47 = 99; { let _x_50 = crate::ControlOrigin::Inherited { field_0: _x_47, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_51 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_50 }; { let _x_53 = alloc::vec![_x_51]; { let _x_54 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_53); __list }; { let _x_55 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_54); __list }; { let _x_56 = crate::ControlSubmission { contributions: _x_55 }; { let _x_57 = validateControlCoverage(&(_x_43), &(_x_56)); _x_57 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseWrongProviderEvidence() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Local; { let _x_45 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_44.clone() }; { let _x_46 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_44.clone() }; { let _x_47 = coverageOctets09Length32(); { let _x_48 = &(_x_47).octets; { let _x_49 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_48) }; { let _x_50 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_49 }; { let _x_52 = alloc::vec![_x_50]; { let _x_53 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_52); __list }; { let _x_54 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_53); __list }; { let _x_55 = crate::ControlSubmission { contributions: _x_54 }; { let _x_56 = validateControlCoverage(&(_x_43), &(_x_55)); _x_56 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseWrongProviderSubject() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Local; { let _x_45 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_44.clone() }; { let _x_46 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_44.clone() }; { let _x_47 = coverageOctetsCCLength32(); { let _x_48 = &(_x_47).octets; { let _x_49 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_48), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_50 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_49 }; { let _x_52 = alloc::vec![_x_50]; { let _x_53 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_52); __list }; { let _x_54 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_53); __list }; { let _x_55 = crate::ControlSubmission { contributions: _x_54 }; { let _x_56 = validateControlCoverage(&(_x_43), &(_x_55)); _x_56 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseWrongScope() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Local; { let _x_45 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_44.clone() }; { let _x_46 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_44.clone() }; { let _x_48 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_49 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("other"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_48 }; { let _x_51 = alloc::vec![_x_49]; { let _x_52 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_51); __list }; { let _x_53 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_52); __list }; { let _x_54 = crate::ControlSubmission { contributions: _x_53 }; { let _x_55 = validateControlCoverage(&(_x_43), &(_x_54)); _x_55 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseWrongSubject() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Local; { let _x_45 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_44.clone() }; { let _x_46 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_44.clone() }; { let _x_47 = coverageOctetsCCLength32(); { let _x_48 = &(_x_47).octets; { let _x_49 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_50 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_48), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_49 }; { let _x_52 = alloc::vec![_x_50]; { let _x_53 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_52); __list }; { let _x_54 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_53); __list }; { let _x_55 = crate::ControlSubmission { contributions: _x_54 }; { let _x_56 = validateControlCoverage(&(_x_43), &(_x_55)); _x_56 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseWrongVersion() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; { let _x_44 = crate::ControlOrigin::Local; { let _x_45 = crate::ControlContribution { obligation: _x_3, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_44.clone() }; { let _x_46 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_44.clone() }; { let _x_48 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_49 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("2"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_48 }; { let _x_51 = alloc::vec![_x_49]; { let _x_52 = { let mut __list = alloc::vec![_x_46]; __list.extend(_x_51); __list }; { let _x_53 = { let mut __list = alloc::vec![_x_45]; __list.extend(_x_52); __list }; { let _x_54 = crate::ControlSubmission { contributions: _x_53 }; { let _x_55 = validateControlCoverage(&(_x_43), &(_x_54)); _x_55 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseZeroControl() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 0; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = 10; { let _x_37 = crate::ControlRequirementMode::InheritedRequired; { let _x_38 = coverageOctets03Length32(); { let _x_39 = &(_x_38).octets; { let _x_40 = alloc::vec![_x_3]; { let _x_41 = crate::ControlObligation { id: _x_31, control: _x_34, mode: _x_37, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_39), allowedProviders: _x_40, residualControls: _x_21.clone() }; { let _x_43 = alloc::vec![_x_41]; { let _x_44 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_43); __list }; { let _x_45 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_44); __list }; { let _x_46 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_45 }; { let _x_47 = crate::ControlOrigin::Local; { let _x_48 = crate::ControlContribution { obligation: _x_3, control: _x_34, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_47.clone() }; { let _x_49 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_47.clone() }; { let _x_50 = crate::ControlOrigin::Inherited { field_0: _x_3, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_51 = crate::ControlContribution { obligation: _x_31, control: _x_34, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_39), origin: _x_50 }; { let _x_53 = alloc::vec![_x_51]; { let _x_54 = { let mut __list = alloc::vec![_x_49]; __list.extend(_x_53); __list }; { let _x_55 = { let mut __list = alloc::vec![_x_48]; __list.extend(_x_54); __list }; { let _x_56 = crate::ControlSubmission { contributions: _x_55 }; { let _x_57 = validateControlCoverage(&(_x_46), &(_x_56)); _x_57 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCaseZeroObligation() -> bool {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 0; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = 1; { let _x_40 = alloc::vec![_x_37]; { let _x_41 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_40, residualControls: _x_21.clone() }; { let _x_43 = alloc::vec![_x_41]; { let _x_44 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_43); __list }; { let _x_45 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_44); __list }; { let _x_46 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_45 }; { let _x_47 = crate::ControlOrigin::Local; { let _x_48 = crate::ControlContribution { obligation: _x_37, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), origin: _x_47.clone() }; { let _x_49 = crate::ControlContribution { obligation: _x_23, control: _x_18, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), origin: _x_47.clone() }; { let _x_50 = crate::ControlOrigin::Inherited { field_0: _x_37, field_1: alloc::borrow::ToOwned::to_owned(_x_13), field_2: alloc::borrow::ToOwned::to_owned(_x_16) }; { let _x_51 = crate::ControlContribution { obligation: _x_31, control: _x_6, policy: alloc::borrow::ToOwned::to_owned(_x_2), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), origin: _x_50 }; { let _x_53 = alloc::vec![_x_51]; { let _x_54 = { let mut __list = alloc::vec![_x_49]; __list.extend(_x_53); __list }; { let _x_55 = { let mut __list = alloc::vec![_x_48]; __list.extend(_x_54); __list }; { let _x_56 = crate::ControlSubmission { contributions: _x_55 }; { let _x_57 = validateControlCoverage(&(_x_46), &(_x_56)); _x_57 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageCorpusPassed() -> bool {
    { let _x_532 = coverageCaseInheritedResidual(); match _x_532 {
        false => _x_532,
        true => { let _x_1766 = coverageCaseLocal(); match _x_1766 {
        false => _x_1766,
        true => { let _x_2356 = coverageCaseIndependentOrder(); match _x_2356 {
        false => _x_2356,
        true => { let _x_2941 = coverageCaseContributionOrder(); match _x_2941 {
        false => _x_2941,
        true => { let _x_3521 = coverageCaseTransitiveInheritance(); match _x_3521 {
        false => _x_3521,
        true => { let _x_4096 = coverageCaseSharedProvider(); match _x_4096 {
        false => _x_4096,
        true => { let _x_4659 = coverageCaseEmptyPolicy(); match _x_4659 {
        false => { let _x_17924 = coverageCaseMissingContribution(); match _x_17924 {
        false => { let _x_18064 = coverageCaseUncoveredProvider(); match _x_18064 {
        false => { let _x_18201 = coverageCaseUncoveredResidual(); match _x_18201 {
        false => { let _x_18335 = coverageCaseDuplicateContribution(); match _x_18335 {
        false => { let _x_18466 = coverageCaseDanglingContribution(); match _x_18466 {
        false => { let _x_18594 = coverageCaseDuplicateObligation(); match _x_18594 {
        false => { let _x_18719 = coverageCaseDuplicateBinding(); match _x_18719 {
        false => { let _x_18841 = coverageCaseNonTopological(); match _x_18841 {
        false => { let _x_18960 = coverageCaseMalformedPolicyDigest(); match _x_18960 {
        false => { let _x_19076 = coverageCaseWrongPolicyDigest(); match _x_19076 {
        false => { let _x_19189 = coverageCaseWrongControl(); match _x_19189 {
        false => { let _x_19299 = coverageCaseWrongScope(); match _x_19299 {
        false => { let _x_19406 = coverageCaseWrongVersion(); match _x_19406 {
        false => { let _x_19510 = coverageCaseWrongSubject(); match _x_19510 {
        false => { let _x_19611 = coverageCaseWrongEvidenceKind(); match _x_19611 {
        false => { let _x_19709 = coverageCaseWrongEvidence(); match _x_19709 {
        false => { let _x_19804 = coverageCaseWrongPolicy(); match _x_19804 {
        false => { let _x_19896 = coverageCaseWrongMalformedSubject(); match _x_19896 {
        false => { let _x_19985 = coverageCaseWrongMalformedEvidence(); match _x_19985 {
        false => { let _x_20071 = coverageCaseWrongProvider(); match _x_20071 {
        false => { let _x_20154 = coverageCaseWrongProviderSubject(); match _x_20154 {
        false => { let _x_20234 = coverageCaseWrongProviderEvidence(); match _x_20234 {
        false => { let _x_20311 = coverageCaseUnauthorizedProvider(); match _x_20311 {
        false => { let _x_20385 = coverageCaseDanglingProviderPermission(); match _x_20385 {
        false => { let _x_20456 = coverageCaseDuplicateProviderPermission(); match _x_20456 {
        false => { let _x_20524 = coverageCaseDuplicateResidual(); match _x_20524 {
        false => { let _x_20589 = coverageCaseDanglingResidualControl(); match _x_20589 {
        false => { let _x_20651 = coverageCaseResidualPolicyDropped(); match _x_20651 {
        false => { let _x_20710 = coverageCaseZeroObligation(); match _x_20710 {
        false => { let _x_20766 = coverageCaseZeroControl(); match _x_20766 {
        false => { let _x_20819 = coverageCaseEmptyScope(); match _x_20819 {
        false => { let _x_20869 = coverageCaseEmptyVersion(); match _x_20869 {
        false => { let _x_20916 = coverageCaseEmptyEvidenceKind(); match _x_20916 {
        false => { let _x_20960 = coverageCaseMalformedObligationSubject(); match _x_20960 {
        false => { let _x_21001 = coverageCaseMalformedObligationEvidence(); match _x_21001 {
        false => { let _x_21039 = coverageCaseResidualWrongSubject(); match _x_21039 {
        false => { let _x_21074 = coverageCaseResidualWrongScope(); match _x_21074 {
        false => { let _x_21106 = coverageCaseResidualWrongVersion(); match _x_21106 {
        false => { let _x_21135 = coverageCaseProviderWrongControl(); match _x_21135 {
        false => { let _x_21161 = coverageCaseProviderWrongScope(); match _x_21161 {
        false => { let _x_21184 = coverageCaseProviderWrongVersion(); match _x_21184 {
        false => { let _x_21204 = coverageCaseProviderWrongEvidenceKind(); match _x_21204 {
        false => { let _x_21221 = coverageCaseSelfCycle(); match _x_21221 {
        false => { let _x_21235 = coverageCaseTwoNodeCycle(); match _x_21235 {
        false => { let _x_21246 = coverageCaseCombinedDependencyCycle(); match _x_21246 {
        false => { let _x_21254 = coverageCaseRelabeledInheritedAsLocal(); match _x_21254 {
        false => { let _x_21259 = coverageCaseRelabeledLocalAsInherited(); match _x_21259 {
        false => _x_4096,
        true => _x_21254,
    } },
        true => _x_21246,
    } },
        true => _x_21235,
    } },
        true => _x_21221,
    } },
        true => _x_21204,
    } },
        true => _x_21184,
    } },
        true => _x_21161,
    } },
        true => _x_21135,
    } },
        true => _x_21106,
    } },
        true => _x_21074,
    } },
        true => _x_21039,
    } },
        true => _x_21001,
    } },
        true => _x_20960,
    } },
        true => _x_20916,
    } },
        true => _x_20869,
    } },
        true => _x_20819,
    } },
        true => _x_20766,
    } },
        true => _x_20710,
    } },
        true => _x_20651,
    } },
        true => _x_20589,
    } },
        true => _x_20524,
    } },
        true => _x_20456,
    } },
        true => _x_20385,
    } },
        true => _x_20311,
    } },
        true => _x_20234,
    } },
        true => _x_20154,
    } },
        true => _x_20071,
    } },
        true => _x_19985,
    } },
        true => _x_19896,
    } },
        true => _x_19804,
    } },
        true => _x_19709,
    } },
        true => _x_19611,
    } },
        true => _x_19510,
    } },
        true => _x_19406,
    } },
        true => _x_19299,
    } },
        true => _x_19189,
    } },
        true => _x_19076,
    } },
        true => _x_18960,
    } },
        true => _x_18841,
    } },
        true => _x_18719,
    } },
        true => _x_18594,
    } },
        true => _x_18466,
    } },
        true => _x_18335,
    } },
        true => _x_18201,
    } },
        true => _x_18064,
    } },
        true => _x_17924,
    } },
        true => _x_4659,
    } },
        true => { let _x_4669 = false; _x_4669 },
    } },
    } },
    } },
    } },
    } },
    } },
    } }
}

pub fn coverageOctets00Length1() -> crate::CoverageOctetConstant {
    { let _x_7 = 0; { let _x_5 = alloc::vec![_x_7]; { let _x_6 = crate::CoverageOctetConstant { octets: _x_5 }; _x_6 } } }
}

pub fn coverageOctets01Length32() -> crate::CoverageOctetConstant {
    { let _x_38 = 1; { let _x_5 = alloc::vec![_x_38]; { let _x_6 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_5); __list }; { let _x_7 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_6); __list }; { let _x_8 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_7); __list }; { let _x_9 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_8); __list }; { let _x_10 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_9); __list }; { let _x_11 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_10); __list }; { let _x_12 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_11); __list }; { let _x_13 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_12); __list }; { let _x_14 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_13); __list }; { let _x_15 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_14); __list }; { let _x_16 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_15); __list }; { let _x_17 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_16); __list }; { let _x_18 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_17); __list }; { let _x_19 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_18); __list }; { let _x_20 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_19); __list }; { let _x_21 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_20); __list }; { let _x_22 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_21); __list }; { let _x_23 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_22); __list }; { let _x_24 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_23); __list }; { let _x_25 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_24); __list }; { let _x_26 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_25); __list }; { let _x_27 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_26); __list }; { let _x_28 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_27); __list }; { let _x_29 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_28); __list }; { let _x_30 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_29); __list }; { let _x_31 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_30); __list }; { let _x_32 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_31); __list }; { let _x_33 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_32); __list }; { let _x_34 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_33); __list }; { let _x_35 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_34); __list }; { let _x_36 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_35); __list }; { let _x_37 = crate::CoverageOctetConstant { octets: _x_36 }; _x_37 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageOctets02Length32() -> crate::CoverageOctetConstant {
    { let _x_38 = 2; { let _x_5 = alloc::vec![_x_38]; { let _x_6 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_5); __list }; { let _x_7 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_6); __list }; { let _x_8 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_7); __list }; { let _x_9 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_8); __list }; { let _x_10 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_9); __list }; { let _x_11 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_10); __list }; { let _x_12 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_11); __list }; { let _x_13 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_12); __list }; { let _x_14 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_13); __list }; { let _x_15 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_14); __list }; { let _x_16 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_15); __list }; { let _x_17 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_16); __list }; { let _x_18 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_17); __list }; { let _x_19 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_18); __list }; { let _x_20 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_19); __list }; { let _x_21 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_20); __list }; { let _x_22 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_21); __list }; { let _x_23 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_22); __list }; { let _x_24 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_23); __list }; { let _x_25 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_24); __list }; { let _x_26 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_25); __list }; { let _x_27 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_26); __list }; { let _x_28 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_27); __list }; { let _x_29 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_28); __list }; { let _x_30 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_29); __list }; { let _x_31 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_30); __list }; { let _x_32 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_31); __list }; { let _x_33 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_32); __list }; { let _x_34 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_33); __list }; { let _x_35 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_34); __list }; { let _x_36 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_35); __list }; { let _x_37 = crate::CoverageOctetConstant { octets: _x_36 }; _x_37 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageOctets03Length32() -> crate::CoverageOctetConstant {
    { let _x_38 = 3; { let _x_5 = alloc::vec![_x_38]; { let _x_6 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_5); __list }; { let _x_7 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_6); __list }; { let _x_8 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_7); __list }; { let _x_9 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_8); __list }; { let _x_10 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_9); __list }; { let _x_11 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_10); __list }; { let _x_12 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_11); __list }; { let _x_13 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_12); __list }; { let _x_14 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_13); __list }; { let _x_15 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_14); __list }; { let _x_16 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_15); __list }; { let _x_17 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_16); __list }; { let _x_18 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_17); __list }; { let _x_19 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_18); __list }; { let _x_20 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_19); __list }; { let _x_21 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_20); __list }; { let _x_22 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_21); __list }; { let _x_23 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_22); __list }; { let _x_24 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_23); __list }; { let _x_25 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_24); __list }; { let _x_26 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_25); __list }; { let _x_27 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_26); __list }; { let _x_28 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_27); __list }; { let _x_29 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_28); __list }; { let _x_30 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_29); __list }; { let _x_31 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_30); __list }; { let _x_32 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_31); __list }; { let _x_33 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_32); __list }; { let _x_34 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_33); __list }; { let _x_35 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_34); __list }; { let _x_36 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_35); __list }; { let _x_37 = crate::CoverageOctetConstant { octets: _x_36 }; _x_37 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageOctets04Length32() -> crate::CoverageOctetConstant {
    { let _x_38 = 4; { let _x_5 = alloc::vec![_x_38]; { let _x_6 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_5); __list }; { let _x_7 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_6); __list }; { let _x_8 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_7); __list }; { let _x_9 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_8); __list }; { let _x_10 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_9); __list }; { let _x_11 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_10); __list }; { let _x_12 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_11); __list }; { let _x_13 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_12); __list }; { let _x_14 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_13); __list }; { let _x_15 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_14); __list }; { let _x_16 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_15); __list }; { let _x_17 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_16); __list }; { let _x_18 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_17); __list }; { let _x_19 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_18); __list }; { let _x_20 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_19); __list }; { let _x_21 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_20); __list }; { let _x_22 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_21); __list }; { let _x_23 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_22); __list }; { let _x_24 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_23); __list }; { let _x_25 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_24); __list }; { let _x_26 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_25); __list }; { let _x_27 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_26); __list }; { let _x_28 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_27); __list }; { let _x_29 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_28); __list }; { let _x_30 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_29); __list }; { let _x_31 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_30); __list }; { let _x_32 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_31); __list }; { let _x_33 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_32); __list }; { let _x_34 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_33); __list }; { let _x_35 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_34); __list }; { let _x_36 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_35); __list }; { let _x_37 = crate::CoverageOctetConstant { octets: _x_36 }; _x_37 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageOctets05Length32() -> crate::CoverageOctetConstant {
    { let _x_38 = 5; { let _x_5 = alloc::vec![_x_38]; { let _x_6 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_5); __list }; { let _x_7 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_6); __list }; { let _x_8 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_7); __list }; { let _x_9 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_8); __list }; { let _x_10 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_9); __list }; { let _x_11 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_10); __list }; { let _x_12 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_11); __list }; { let _x_13 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_12); __list }; { let _x_14 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_13); __list }; { let _x_15 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_14); __list }; { let _x_16 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_15); __list }; { let _x_17 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_16); __list }; { let _x_18 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_17); __list }; { let _x_19 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_18); __list }; { let _x_20 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_19); __list }; { let _x_21 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_20); __list }; { let _x_22 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_21); __list }; { let _x_23 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_22); __list }; { let _x_24 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_23); __list }; { let _x_25 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_24); __list }; { let _x_26 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_25); __list }; { let _x_27 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_26); __list }; { let _x_28 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_27); __list }; { let _x_29 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_28); __list }; { let _x_30 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_29); __list }; { let _x_31 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_30); __list }; { let _x_32 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_31); __list }; { let _x_33 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_32); __list }; { let _x_34 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_33); __list }; { let _x_35 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_34); __list }; { let _x_36 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_35); __list }; { let _x_37 = crate::CoverageOctetConstant { octets: _x_36 }; _x_37 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageOctets09Length32() -> crate::CoverageOctetConstant {
    { let _x_38 = 9; { let _x_5 = alloc::vec![_x_38]; { let _x_6 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_5); __list }; { let _x_7 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_6); __list }; { let _x_8 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_7); __list }; { let _x_9 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_8); __list }; { let _x_10 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_9); __list }; { let _x_11 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_10); __list }; { let _x_12 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_11); __list }; { let _x_13 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_12); __list }; { let _x_14 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_13); __list }; { let _x_15 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_14); __list }; { let _x_16 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_15); __list }; { let _x_17 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_16); __list }; { let _x_18 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_17); __list }; { let _x_19 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_18); __list }; { let _x_20 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_19); __list }; { let _x_21 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_20); __list }; { let _x_22 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_21); __list }; { let _x_23 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_22); __list }; { let _x_24 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_23); __list }; { let _x_25 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_24); __list }; { let _x_26 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_25); __list }; { let _x_27 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_26); __list }; { let _x_28 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_27); __list }; { let _x_29 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_28); __list }; { let _x_30 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_29); __list }; { let _x_31 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_30); __list }; { let _x_32 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_31); __list }; { let _x_33 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_32); __list }; { let _x_34 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_33); __list }; { let _x_35 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_34); __list }; { let _x_36 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_35); __list }; { let _x_37 = crate::CoverageOctetConstant { octets: _x_36 }; _x_37 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageOctets11Length1() -> crate::CoverageOctetConstant {
    { let _x_7 = 17; { let _x_5 = alloc::vec![_x_7]; { let _x_6 = crate::CoverageOctetConstant { octets: _x_5 }; _x_6 } } }
}

pub fn coverageOctets11Length32() -> crate::CoverageOctetConstant {
    { let _x_38 = 17; { let _x_5 = alloc::vec![_x_38]; { let _x_6 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_5); __list }; { let _x_7 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_6); __list }; { let _x_8 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_7); __list }; { let _x_9 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_8); __list }; { let _x_10 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_9); __list }; { let _x_11 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_10); __list }; { let _x_12 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_11); __list }; { let _x_13 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_12); __list }; { let _x_14 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_13); __list }; { let _x_15 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_14); __list }; { let _x_16 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_15); __list }; { let _x_17 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_16); __list }; { let _x_18 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_17); __list }; { let _x_19 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_18); __list }; { let _x_20 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_19); __list }; { let _x_21 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_20); __list }; { let _x_22 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_21); __list }; { let _x_23 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_22); __list }; { let _x_24 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_23); __list }; { let _x_25 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_24); __list }; { let _x_26 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_25); __list }; { let _x_27 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_26); __list }; { let _x_28 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_27); __list }; { let _x_29 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_28); __list }; { let _x_30 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_29); __list }; { let _x_31 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_30); __list }; { let _x_32 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_31); __list }; { let _x_33 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_32); __list }; { let _x_34 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_33); __list }; { let _x_35 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_34); __list }; { let _x_36 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_35); __list }; { let _x_37 = crate::CoverageOctetConstant { octets: _x_36 }; _x_37 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageOctets22Length32() -> crate::CoverageOctetConstant {
    { let _x_38 = 34; { let _x_5 = alloc::vec![_x_38]; { let _x_6 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_5); __list }; { let _x_7 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_6); __list }; { let _x_8 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_7); __list }; { let _x_9 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_8); __list }; { let _x_10 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_9); __list }; { let _x_11 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_10); __list }; { let _x_12 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_11); __list }; { let _x_13 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_12); __list }; { let _x_14 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_13); __list }; { let _x_15 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_14); __list }; { let _x_16 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_15); __list }; { let _x_17 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_16); __list }; { let _x_18 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_17); __list }; { let _x_19 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_18); __list }; { let _x_20 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_19); __list }; { let _x_21 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_20); __list }; { let _x_22 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_21); __list }; { let _x_23 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_22); __list }; { let _x_24 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_23); __list }; { let _x_25 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_24); __list }; { let _x_26 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_25); __list }; { let _x_27 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_26); __list }; { let _x_28 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_27); __list }; { let _x_29 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_28); __list }; { let _x_30 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_29); __list }; { let _x_31 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_30); __list }; { let _x_32 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_31); __list }; { let _x_33 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_32); __list }; { let _x_34 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_33); __list }; { let _x_35 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_34); __list }; { let _x_36 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_35); __list }; { let _x_37 = crate::CoverageOctetConstant { octets: _x_36 }; _x_37 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageOctetsAALength1() -> crate::CoverageOctetConstant {
    { let _x_7 = 170; { let _x_5 = alloc::vec![_x_7]; { let _x_6 = crate::CoverageOctetConstant { octets: _x_5 }; _x_6 } } }
}

pub fn coverageOctetsAALength32() -> crate::CoverageOctetConstant {
    { let _x_38 = 170; { let _x_5 = alloc::vec![_x_38]; { let _x_6 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_5); __list }; { let _x_7 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_6); __list }; { let _x_8 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_7); __list }; { let _x_9 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_8); __list }; { let _x_10 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_9); __list }; { let _x_11 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_10); __list }; { let _x_12 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_11); __list }; { let _x_13 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_12); __list }; { let _x_14 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_13); __list }; { let _x_15 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_14); __list }; { let _x_16 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_15); __list }; { let _x_17 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_16); __list }; { let _x_18 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_17); __list }; { let _x_19 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_18); __list }; { let _x_20 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_19); __list }; { let _x_21 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_20); __list }; { let _x_22 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_21); __list }; { let _x_23 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_22); __list }; { let _x_24 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_23); __list }; { let _x_25 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_24); __list }; { let _x_26 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_25); __list }; { let _x_27 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_26); __list }; { let _x_28 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_27); __list }; { let _x_29 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_28); __list }; { let _x_30 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_29); __list }; { let _x_31 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_30); __list }; { let _x_32 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_31); __list }; { let _x_33 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_32); __list }; { let _x_34 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_33); __list }; { let _x_35 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_34); __list }; { let _x_36 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_35); __list }; { let _x_37 = crate::CoverageOctetConstant { octets: _x_36 }; _x_37 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageOctetsBBLength32() -> crate::CoverageOctetConstant {
    { let _x_38 = 187; { let _x_5 = alloc::vec![_x_38]; { let _x_6 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_5); __list }; { let _x_7 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_6); __list }; { let _x_8 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_7); __list }; { let _x_9 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_8); __list }; { let _x_10 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_9); __list }; { let _x_11 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_10); __list }; { let _x_12 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_11); __list }; { let _x_13 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_12); __list }; { let _x_14 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_13); __list }; { let _x_15 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_14); __list }; { let _x_16 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_15); __list }; { let _x_17 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_16); __list }; { let _x_18 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_17); __list }; { let _x_19 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_18); __list }; { let _x_20 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_19); __list }; { let _x_21 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_20); __list }; { let _x_22 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_21); __list }; { let _x_23 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_22); __list }; { let _x_24 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_23); __list }; { let _x_25 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_24); __list }; { let _x_26 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_25); __list }; { let _x_27 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_26); __list }; { let _x_28 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_27); __list }; { let _x_29 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_28); __list }; { let _x_30 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_29); __list }; { let _x_31 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_30); __list }; { let _x_32 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_31); __list }; { let _x_33 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_32); __list }; { let _x_34 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_33); __list }; { let _x_35 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_34); __list }; { let _x_36 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_35); __list }; { let _x_37 = crate::CoverageOctetConstant { octets: _x_36 }; _x_37 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageOctetsCCLength32() -> crate::CoverageOctetConstant {
    { let _x_38 = 204; { let _x_5 = alloc::vec![_x_38]; { let _x_6 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_5); __list }; { let _x_7 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_6); __list }; { let _x_8 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_7); __list }; { let _x_9 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_8); __list }; { let _x_10 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_9); __list }; { let _x_11 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_10); __list }; { let _x_12 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_11); __list }; { let _x_13 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_12); __list }; { let _x_14 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_13); __list }; { let _x_15 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_14); __list }; { let _x_16 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_15); __list }; { let _x_17 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_16); __list }; { let _x_18 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_17); __list }; { let _x_19 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_18); __list }; { let _x_20 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_19); __list }; { let _x_21 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_20); __list }; { let _x_22 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_21); __list }; { let _x_23 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_22); __list }; { let _x_24 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_23); __list }; { let _x_25 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_24); __list }; { let _x_26 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_25); __list }; { let _x_27 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_26); __list }; { let _x_28 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_27); __list }; { let _x_29 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_28); __list }; { let _x_30 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_29); __list }; { let _x_31 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_30); __list }; { let _x_32 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_31); __list }; { let _x_33 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_32); __list }; { let _x_34 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_33); __list }; { let _x_35 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_34); __list }; { let _x_36 = { let mut __list = alloc::vec![_x_38]; __list.extend(_x_35); __list }; { let _x_37 = crate::CoverageOctetConstant { octets: _x_36 }; _x_37 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageOctetsEmptyLength0() -> crate::CoverageOctetConstant {
    { let _x_2 = crate::CoverageOctetConstant { octets: alloc::vec::Vec::new() }; _x_2 }
}

pub fn coverageOctetsFFLength1() -> crate::CoverageOctetConstant {
    { let _x_7 = 255; { let _x_5 = alloc::vec![_x_7]; { let _x_6 = crate::CoverageOctetConstant { octets: _x_5 }; _x_6 } } }
}

pub fn coverageValidPolicy() -> crate::ControlPolicy {
    { let _x_1 = coverageOctets11Length32(); { let _x_2 = &(_x_1).octets; { let _x_3 = 1; { let _x_6 = 10; { let _x_9 = crate::ControlRequirementMode::LocalRequired; { let _x_12 = coverageOctetsAALength32(); { let _x_13 = &(_x_12).octets; { let _x_15 = coverageOctets01Length32(); { let _x_16 = &(_x_15).octets; { let _x_18 = 20; { let _x_21 = alloc::vec![_x_18]; { let _x_22 = crate::ControlObligation { id: _x_3, control: _x_6, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_13), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_16), allowedProviders: alloc::vec::Vec::new(), residualControls: _x_21.clone() }; { let _x_23 = 2; { let _x_26 = coverageOctetsBBLength32(); { let _x_27 = &(_x_26).octets; { let _x_28 = coverageOctets02Length32(); { let _x_29 = &(_x_28).octets; { let _x_30 = crate::ControlObligation { id: _x_23, control: _x_18, mode: _x_9, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_29), allowedProviders: alloc::vec::Vec::new(), residualControls: alloc::vec::Vec::new() }; { let _x_31 = 3; { let _x_34 = crate::ControlRequirementMode::InheritedRequired; { let _x_35 = coverageOctets03Length32(); { let _x_36 = &(_x_35).octets; { let _x_37 = alloc::vec![_x_3]; { let _x_38 = crate::ControlObligation { id: _x_31, control: _x_6, mode: _x_34, scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_27), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_36), allowedProviders: _x_37, residualControls: _x_21.clone() }; { let _x_40 = alloc::vec![_x_38]; { let _x_41 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_41); __list }; { let _x_43 = crate::ControlPolicy { digest: alloc::borrow::ToOwned::to_owned(_x_2), obligations: _x_42 }; _x_43 } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn coverageValidSubmission() -> crate::ControlSubmission {
    { let _x_1 = 1; { let _x_4 = 10; { let _x_7 = coverageOctets11Length32(); { let _x_8 = &(_x_7).octets; { let _x_11 = coverageOctetsAALength32(); { let _x_12 = &(_x_11).octets; { let _x_14 = coverageOctets01Length32(); { let _x_15 = &(_x_14).octets; { let _x_16 = crate::ControlOrigin::Local; { let _x_17 = crate::ControlContribution { obligation: _x_1, control: _x_4, policy: alloc::borrow::ToOwned::to_owned(_x_8), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_12), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_15), origin: _x_16.clone() }; { let _x_18 = 2; { let _x_21 = 20; { let _x_24 = coverageOctetsBBLength32(); { let _x_25 = &(_x_24).octets; { let _x_26 = coverageOctets02Length32(); { let _x_27 = &(_x_26).octets; { let _x_28 = crate::ControlContribution { obligation: _x_18, control: _x_21, policy: alloc::borrow::ToOwned::to_owned(_x_8), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_25), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_27), origin: _x_16.clone() }; { let _x_29 = 3; { let _x_32 = coverageOctets03Length32(); { let _x_33 = &(_x_32).octets; { let _x_34 = crate::ControlOrigin::Inherited { field_0: _x_1, field_1: alloc::borrow::ToOwned::to_owned(_x_12), field_2: alloc::borrow::ToOwned::to_owned(_x_15) }; { let _x_35 = crate::ControlContribution { obligation: _x_29, control: _x_4, policy: alloc::borrow::ToOwned::to_owned(_x_8), scope: alloc::string::String::from("control-scope"), version: alloc::string::String::from("1"), subject: alloc::borrow::ToOwned::to_owned(_x_25), evidenceKind: alloc::string::String::from("accepted-test-evidence"), evidence: alloc::borrow::ToOwned::to_owned(_x_33), origin: _x_34 }; { let _x_37 = alloc::vec![_x_35]; { let _x_38 = { let mut __list = alloc::vec![_x_28]; __list.extend(_x_37); __list }; { let _x_39 = { let mut __list = alloc::vec![_x_17]; __list.extend(_x_38); __list }; { let _x_40 = crate::ControlSubmission { contributions: _x_39 }; _x_40 } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn acceptanceEvidenceClosed(x_1: &[crate::Acceptance], x_2: &[crate::TargetBinding], x_3: &[crate::Component]) -> bool {
    match x_1 {
        [] => { let _x_89 = true; _x_89 },
        [head_67, tail_68 @ ..] => { let _x_117 = (head_67).bounded; match _x_117 {
        false => _x_117,
        true => { let _x_142 = &(head_67).target; { let _x_143 = memberIdTargets((_x_142).as_ref(), &(x_2)); match _x_143 {
        false => _x_143,
        true => { let _x_162 = &(head_67).component; { let _x_163 = memberIdComponents((_x_162).as_ref(), &(x_3)); match _x_163 {
        false => _x_163,
        true => { let _x_176 = &(head_67).evidence; { let _x_177 = nonEmptyString((_x_176).as_ref()); match _x_177 {
        false => _x_177,
        true => { let _x_181 = 0; { let _x_183 = &(head_67).command; { let _x_184 = (_x_183).len() as u64; { let _x_185 = (_x_181 < _x_184); match _x_185 {
        false => _x_185,
        true => { let _x_188 = acceptanceEvidenceClosed(&(tail_68), &(x_2), &(x_3)); _x_188 },
    } } } } },
    } } },
    } } },
    } } },
    } },
    }
}

pub fn acceptanceReferential(x_1: &[crate::Acceptance], x_2: &[crate::TargetBinding], x_3: &[crate::Component]) -> bool {
    match x_1 {
        [] => { let _x_45 = true; _x_45 },
        [head_33, tail_34 @ ..] => { let _x_52 = &(head_33).target; { let _x_53 = memberIdTargets((_x_52).as_ref(), &(x_2)); match _x_53 {
        false => _x_53,
        true => { let _x_57 = &(head_33).component; { let _x_58 = memberIdComponents((_x_57).as_ref(), &(x_3)); match _x_58 {
        false => _x_58,
        true => { let _x_61 = acceptanceReferential(&(tail_34), &(x_2), &(x_3)); _x_61 },
    } } },
    } } },
    }
}

pub fn allIdsMemberAcceptance(x_1: &[alloc::string::String], x_2: &[crate::Acceptance]) -> bool {
    match x_1 {
        [] => { let _x_30 = true; _x_30 },
        [head_21, tail_22 @ ..] => { let _x_31 = memberIdAcceptance((head_21).as_ref(), &(x_2)); match _x_31 {
        false => _x_31,
        true => { let _x_34 = allIdsMemberAcceptance(&(tail_22), &(x_2)); _x_34 },
    } },
    }
}

pub fn allIdsMemberCapabilities(x_1: &[alloc::string::String], x_2: &[crate::Capability]) -> bool {
    match x_1 {
        [] => { let _x_30 = true; _x_30 },
        [head_21, tail_22 @ ..] => { let _x_31 = memberIdCapabilities((head_21).as_ref(), &(x_2)); match _x_31 {
        false => _x_31,
        true => { let _x_34 = allIdsMemberCapabilities(&(tail_22), &(x_2)); _x_34 },
    } },
    }
}

pub fn allIdsMemberComponents(x_1: &[alloc::string::String], x_2: &[crate::Component]) -> bool {
    match x_1 {
        [] => { let _x_30 = true; _x_30 },
        [head_21, tail_22 @ ..] => { let _x_31 = memberIdComponents((head_21).as_ref(), &(x_2)); match _x_31 {
        false => _x_31,
        true => { let _x_34 = allIdsMemberComponents(&(tail_22), &(x_2)); _x_34 },
    } },
    }
}

pub fn allIdsMemberInterfaces(x_1: &[alloc::string::String], x_2: &[crate::Interface]) -> bool {
    match x_1 {
        [] => { let _x_30 = true; _x_30 },
        [head_21, tail_22 @ ..] => { let _x_31 = memberIdInterfaces((head_21).as_ref(), &(x_2)); match _x_31 {
        false => _x_31,
        true => { let _x_34 = allIdsMemberInterfaces(&(tail_22), &(x_2)); _x_34 },
    } },
    }
}

pub fn allIdsMemberMigrations(x_1: &[alloc::string::String], x_2: &[crate::Migration]) -> bool {
    match x_1 {
        [] => { let _x_30 = true; _x_30 },
        [head_21, tail_22 @ ..] => { let _x_31 = memberIdMigrations((head_21).as_ref(), &(x_2)); match _x_31 {
        false => _x_31,
        true => { let _x_34 = allIdsMemberMigrations(&(tail_22), &(x_2)); _x_34 },
    } },
    }
}

pub fn allIdsMemberParameters(x_1: &[alloc::string::String], x_2: &[crate::Configuration]) -> bool {
    match x_1 {
        [] => { let _x_30 = true; _x_30 },
        [head_21, tail_22 @ ..] => { let _x_31 = memberIdParameters((head_21).as_ref(), &(x_2)); match _x_31 {
        false => _x_31,
        true => { let _x_34 = allIdsMemberParameters(&(tail_22), &(x_2)); _x_34 },
    } },
    }
}

pub fn allIdsMemberPlatformRequirements(x_1: &[alloc::string::String], x_2: &[crate::PlatformRequirement]) -> bool {
    match x_1 {
        [] => { let _x_30 = true; _x_30 },
        [head_21, tail_22 @ ..] => { let _x_31 = memberIdPlatformRequirements((head_21).as_ref(), &(x_2)); match _x_31 {
        false => _x_31,
        true => { let _x_34 = allIdsMemberPlatformRequirements(&(tail_22), &(x_2)); _x_34 },
    } },
    }
}

pub fn allIdsMemberSecretReferences(x_1: &[alloc::string::String], x_2: &[crate::SecretReference]) -> bool {
    match x_1 {
        [] => { let _x_30 = true; _x_30 },
        [head_21, tail_22 @ ..] => { let _x_31 = memberIdSecretReferences((head_21).as_ref(), &(x_2)); match _x_31 {
        false => _x_31,
        true => { let _x_34 = allIdsMemberSecretReferences(&(tail_22), &(x_2)); _x_34 },
    } },
    }
}

pub fn allIdsMemberTopology(x_1: &[alloc::string::String], x_2: &[crate::Topology]) -> bool {
    match x_1 {
        [] => { let _x_30 = true; _x_30 },
        [head_21, tail_22 @ ..] => { let _x_31 = memberIdTopology((head_21).as_ref(), &(x_2)); match _x_31 {
        false => _x_31,
        true => { let _x_34 = allIdsMemberTopology(&(tail_22), &(x_2)); _x_34 },
    } },
    }
}

pub fn allStringsModelMember(x_1: &[alloc::string::String], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_30 = true; _x_30 },
        [head_21, tail_22 @ ..] => { let _x_31 = modelHasId((head_21).as_ref(), &(x_2)); match _x_31 {
        false => _x_31,
        true => { let _x_34 = allStringsModelMember(&(tail_22), &(x_2)); _x_34 },
    } },
    }
}

pub fn anyIdsMemberMigrations(x_1: &[alloc::string::String], x_2: &[crate::Migration]) -> bool {
    match x_1 {
        [] => { let _x_30 = false; _x_30 },
        [head_21, tail_22 @ ..] => { let _x_31 = memberIdMigrations((head_21).as_ref(), &(x_2)); match _x_31 {
        false => { let _x_33 = anyIdsMemberMigrations(&(tail_22), &(x_2)); _x_33 },
        true => _x_31,
    } },
    }
}

pub fn anyIdsMemberRollouts(x_1: &[alloc::string::String], x_2: &[crate::Rollout]) -> bool {
    match x_1 {
        [] => { let _x_30 = false; _x_30 },
        [head_21, tail_22 @ ..] => { let _x_31 = memberIdRollouts((head_21).as_ref(), &(x_2)); match _x_31 {
        false => { let _x_33 = anyIdsMemberRollouts(&(tail_22), &(x_2)); _x_33 },
        true => _x_31,
    } },
    }
}

pub fn artifactLicensesClosed(x_1: &[crate::Artifact]) -> bool {
    match x_1 {
        [] => { let _x_32 = true; _x_32 },
        [head_22, tail_23 @ ..] => { let _x_33 = &(head_22).licenseExpression; { let _x_34 = nonEmptyString((_x_33).as_ref()); match _x_34 {
        false => _x_34,
        true => { let _x_37 = artifactLicensesClosed(&(tail_23)); _x_37 },
    } } },
    }
}

pub fn artifactsReferential(x_1: &[crate::Artifact], x_2: &[crate::PlatformRequirement]) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).platformRequirements; { let _x_33 = allIdsMemberPlatformRequirements(&(_x_32), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = artifactsReferential(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn callsReferential(x_1: &[crate::Call], x_2: &[crate::Component], x_3: &[crate::Interface]) -> bool {
    match x_1 {
        [] => { let _x_58 = true; _x_58 },
        [head_43, tail_44 @ ..] => { let _x_71 = &(head_43).fromComponent; { let _x_72 = memberIdComponents((_x_71).as_ref(), &(x_2)); match _x_72 {
        false => _x_72,
        true => { let _x_82 = &(head_43).toComponent; { let _x_83 = memberIdComponents((_x_82).as_ref(), &(x_2)); match _x_83 {
        false => _x_83,
        true => { let _x_87 = &(head_43).interfaceId; { let _x_88 = memberIdInterfaces((_x_87).as_ref(), &(x_3)); match _x_88 {
        false => _x_88,
        true => { let _x_91 = callsReferential(&(tail_44), &(x_2), &(x_3)); _x_91 },
    } } },
    } } },
    } } },
    }
}

pub fn compatibilityValue(value: &str) -> bool {
    { let _x_37 = value == "exact"; match _x_37 {
        false => { let _x_69 = value == "backward"; match _x_69 {
        false => { let _x_81 = value == "forward"; match _x_81 {
        false => { let _x_88 = value == "full"; _x_88 },
        true => _x_81,
    } },
        true => _x_69,
    } },
        true => _x_37,
    } }
}

pub fn componentCapabilitiesSatisfied(x_1: &[crate::Component], x_2: &[crate::Capability]) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).capabilities; { let _x_33 = allIdsMemberCapabilities(&(_x_32), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = componentCapabilitiesSatisfied(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn componentSecretsSatisfied(x_1: &[crate::Component], x_2: &[crate::SecretReference]) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).secrets; { let _x_33 = allIdsMemberSecretReferences(&(_x_32), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = componentSecretsSatisfied(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn componentsHaveArtifacts(x_1: &[crate::Component], x_2: &[crate::Artifact]) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).artifact; { let _x_33 = memberIdArtifacts((_x_32).as_ref(), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = componentsHaveArtifacts(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn componentsReferential(x_1: &[crate::Component], x_2: &[crate::Artifact], x_3: &[crate::Capability], x_4: &[crate::Component], x_5: &[crate::Interface], x_6: &[crate::Configuration], x_7: &[crate::Topology], x_8: &[crate::SecretReference], x_9: &[crate::PlatformRequirement], x_10: &[crate::ScalingPolicy]) -> bool {
    match x_1 {
        [] => { let _x_169 = true; _x_169 },
        [head_130, tail_131 @ ..] => { let _x_230 = &(head_130).artifact; { let _x_231 = memberIdArtifacts((_x_230).as_ref(), &(x_2)); match _x_231 {
        false => _x_231,
        true => { let _x_289 = &(head_130).capabilities; { let _x_290 = allIdsMemberCapabilities(&(_x_289), &(x_3)); match _x_290 {
        false => _x_290,
        true => { let _x_342 = &(head_130).dependsOn; { let _x_343 = allIdsMemberComponents(&(_x_342), &(x_4)); match _x_343 {
        false => _x_343,
        true => { let _x_389 = &(head_130).interfaces; { let _x_390 = allIdsMemberInterfaces(&(_x_389), &(x_5)); match _x_390 {
        false => _x_390,
        true => { let _x_430 = &(head_130).parameters; { let _x_431 = allIdsMemberParameters(&(_x_430), &(x_6)); match _x_431 {
        false => _x_431,
        true => { let _x_465 = &(head_130).ports; { let _x_466 = allIdsMemberTopology(&(_x_465), &(x_7)); match _x_466 {
        false => _x_466,
        true => { let _x_494 = &(head_130).volumes; { let _x_495 = allIdsMemberTopology(&(_x_494), &(x_7)); match _x_495 {
        false => _x_495,
        true => { let _x_517 = &(head_130).placement; { let _x_518 = allIdsMemberTopology(&(_x_517), &(x_7)); match _x_518 {
        false => _x_518,
        true => { let _x_534 = &(head_130).secrets; { let _x_535 = allIdsMemberSecretReferences(&(_x_534), &(x_8)); match _x_535 {
        false => _x_535,
        true => { let _x_545 = &(head_130).platformRequirements; { let _x_546 = allIdsMemberPlatformRequirements(&(_x_545), &(x_9)); match _x_546 {
        false => _x_546,
        true => { let _x_550 = &(head_130).scalingPolicy; { let _x_551 = __prod_borrowed_optionalIdMemberScalingPolicies(&(_x_550), &(x_10)); match _x_551 {
        false => _x_551,
        true => { let _x_554 = componentsReferential(&(tail_131), &(x_2), &(x_3), &(x_4), &(x_5), &(x_6), &(x_7), &(x_8), &(x_9), &(x_10)); _x_554 },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    }
}

pub fn countIdAcceptance(x_1: alloc::string::String, x_2: &[crate::Acceptance]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdAcceptance(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdAcceptance(x_1: &str, x_2: &[crate::Acceptance]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdAcceptance((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdAcceptance((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdAlerts(x_1: alloc::string::String, x_2: &[crate::Alert]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdAlerts(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdAlerts(x_1: &str, x_2: &[crate::Alert]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdAlerts((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdAlerts((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdArchitecture(x_1: alloc::string::String, x_2: &[crate::ArchitectureBinding]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdArchitecture(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdArchitecture(x_1: &str, x_2: &[crate::ArchitectureBinding]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdArchitecture((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdArchitecture((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdArtifacts(x_1: alloc::string::String, x_2: &[crate::Artifact]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdArtifacts(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdArtifacts(x_1: &str, x_2: &[crate::Artifact]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdArtifacts((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdArtifacts((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdBackups(x_1: alloc::string::String, x_2: &[crate::BackupRecovery]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdBackups(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdBackups(x_1: &str, x_2: &[crate::BackupRecovery]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdBackups((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdBackups((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdCalls(x_1: alloc::string::String, x_2: &[crate::Call]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdCalls(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdCalls(x_1: &str, x_2: &[crate::Call]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdCalls((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdCalls((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdCapabilities(x_1: alloc::string::String, x_2: &[crate::Capability]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdCapabilities(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdCapabilities(x_1: &str, x_2: &[crate::Capability]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdCapabilities((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdCapabilities((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdComponents(x_1: alloc::string::String, x_2: &[crate::Component]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdComponents(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdComponents(x_1: &str, x_2: &[crate::Component]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdComponents((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdComponents((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdControls(x_1: alloc::string::String, x_2: &[crate::Control]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdControls(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdControls(x_1: &str, x_2: &[crate::Control]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdControls((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdControls((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdDrifts(x_1: alloc::string::String, x_2: &[crate::Drift]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdDrifts(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdDrifts(x_1: &str, x_2: &[crate::Drift]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdDrifts((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdDrifts((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdEvents(x_1: alloc::string::String, x_2: &[crate::Event]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdEvents(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdEvents(x_1: &str, x_2: &[crate::Event]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdEvents((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdEvents((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdFlows(x_1: alloc::string::String, x_2: &[crate::Flow]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdFlows(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdFlows(x_1: &str, x_2: &[crate::Flow]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdFlows((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdFlows((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdIdentityRequirements(x_1: alloc::string::String, x_2: &[crate::IdentityRequirement]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdIdentityRequirements(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdIdentityRequirements(x_1: &str, x_2: &[crate::IdentityRequirement]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdIdentityRequirements((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdIdentityRequirements((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdInterfaces(x_1: alloc::string::String, x_2: &[crate::Interface]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdInterfaces(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdInterfaces(x_1: &str, x_2: &[crate::Interface]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdInterfaces((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdInterfaces((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdMigrations(x_1: alloc::string::String, x_2: &[crate::Migration]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdMigrations(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdMigrations(x_1: &str, x_2: &[crate::Migration]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdMigrations((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdMigrations((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdParameters(x_1: alloc::string::String, x_2: &[crate::Configuration]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdParameters(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdParameters(x_1: &str, x_2: &[crate::Configuration]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdParameters((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdParameters((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdPersistence(x_1: alloc::string::String, x_2: &[crate::Persistence]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdPersistence(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdPersistence(x_1: &str, x_2: &[crate::Persistence]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdPersistence((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdPersistence((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdPlatformRequirements(x_1: alloc::string::String, x_2: &[crate::PlatformRequirement]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdPlatformRequirements(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdPlatformRequirements(x_1: &str, x_2: &[crate::PlatformRequirement]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdPlatformRequirements((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdPlatformRequirements((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdRetirements(x_1: alloc::string::String, x_2: &[crate::Retirement]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdRetirements(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdRetirements(x_1: &str, x_2: &[crate::Retirement]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdRetirements((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdRetirements((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdRollbacks(x_1: alloc::string::String, x_2: &[crate::Rollback]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdRollbacks(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdRollbacks(x_1: &str, x_2: &[crate::Rollback]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdRollbacks((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdRollbacks((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdRollouts(x_1: alloc::string::String, x_2: &[crate::Rollout]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdRollouts(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdRollouts(x_1: &str, x_2: &[crate::Rollout]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdRollouts((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdRollouts((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdScalingPolicies(x_1: alloc::string::String, x_2: &[crate::ScalingPolicy]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdScalingPolicies(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdScalingPolicies(x_1: &str, x_2: &[crate::ScalingPolicy]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdScalingPolicies((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdScalingPolicies((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdSchemas(x_1: alloc::string::String, x_2: &[crate::Schema]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdSchemas(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdSchemas(x_1: &str, x_2: &[crate::Schema]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdSchemas((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdSchemas((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdSecretReferences(x_1: alloc::string::String, x_2: &[crate::SecretReference]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdSecretReferences(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdSecretReferences(x_1: &str, x_2: &[crate::SecretReference]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdSecretReferences((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdSecretReferences((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdSlis(x_1: alloc::string::String, x_2: &[crate::Sli]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdSlis(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdSlis(x_1: &str, x_2: &[crate::Sli]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdSlis((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdSlis((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdSlos(x_1: alloc::string::String, x_2: &[crate::Slo]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdSlos(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdSlos(x_1: &str, x_2: &[crate::Slo]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdSlos((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdSlos((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdStorageClasses(x_1: alloc::string::String, x_2: &[crate::StorageClass]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdStorageClasses(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdStorageClasses(x_1: &str, x_2: &[crate::StorageClass]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdStorageClasses((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdStorageClasses((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdTargets(x_1: alloc::string::String, x_2: &[crate::TargetBinding]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdTargets(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdTargets(x_1: &str, x_2: &[crate::TargetBinding]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdTargets((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdTargets((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn countIdTopology(x_1: alloc::string::String, x_2: &[crate::Topology]) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_countIdTopology(x_1.as_ref(), x_2)
}

fn __prod_borrowed_countIdTopology(x_1: &str, x_2: &[crate::Topology]) -> Result<u64, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_64 = 0; _x_64 },
        [head_42, tail_43 @ ..] => { let _x_67 = &(head_42).id; { let _x_68 = (x_1 == _x_67); { let _jp_69 = /* jp "_jp_69" inlined at its jump site */ (); match _x_68 {
        false => { let _x_74 = 0; { let _y_70 = _x_74; { let _x_71 = __prod_borrowed_countIdTopology((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
        true => { let _x_76 = 1; { let _y_70 = _x_76; { let _x_71 = __prod_borrowed_countIdTopology((x_1).as_ref(), &(tail_43))?; { let _x_72 = ((_y_70) as u64).checked_add(_x_71).ok_or(crate::ComputeError::AddOverflow)?; _x_72 } } } },
    } } } },
    })
}

pub fn directComponentDependenciesPrecede(x_1: &[alloc::string::String], x_2: &str, x_3: &[crate::Component], x_4: &[u64]) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_32 = true; _x_32 },
        [head_23, tail_24 @ ..] => { let _x_33 = directComponentIndexPrecedes((head_23).as_ref(), (x_2).as_ref(), &(x_3), &(x_4))?; match _x_33 {
        false => _x_33,
        true => { let _x_36 = directComponentDependenciesPrecede(&(tail_24), (x_2).as_ref(), &(x_3), &(x_4))?; _x_36 },
    } },
    })
}

pub fn directComponentIndex(x_1: alloc::string::String, x_2: &[crate::Component], x_3: u64) -> Result<Option<u64>, crate::ComputeError> {
    __prod_borrowed_directComponentIndex(x_1.as_ref(), x_2, x_3)
}

fn __prod_borrowed_directComponentIndex(x_1: &str, x_2: &[crate::Component], x_3: u64) -> Result<Option<u64>, crate::ComputeError> {
    Ok(match x_2 {
        [] => None,
        [head_32, tail_33 @ ..] => { let _x_56 = &(head_32).id; { let _x_57 = (x_1 == _x_56); match _x_57 {
        false => { let _x_62 = 1; { let _x_63 = ((x_3) as u64).checked_add(_x_62).ok_or(crate::ComputeError::AddOverflow)?; { let _x_64 = __prod_borrowed_directComponentIndex((x_1).as_ref(), &(tail_33), _x_63)?; _x_64 } } },
        true => { let _x_61 = Some(x_3); _x_61 },
    } } },
    })
}

pub fn directComponentIndexPrecedes(dependency: &str, current: &str, rows: &[crate::Component], order: &[u64]) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_43 = 0; { let _x_46 = __prod_borrowed_directComponentIndex((dependency).as_ref(), &(rows), _x_43)?; match _x_46 {
        None => { let _x_65 = false; _x_65 },
        Some(val_49) => { let _x_80 = 0; { let _x_81 = __prod_borrowed_directComponentIndex((current).as_ref(), &(rows), _x_80)?; match _x_81 {
        None => { let _x_83 = false; _x_83 },
        Some(val_84) => { let _x_93 = 0; { let _x_94 = natIndex(val_49, &(order), _x_93)?; match _x_94 {
        None => { let _x_96 = false; _x_96 },
        Some(val_97) => { let _x_99 = 0; { let _x_100 = natIndex(val_84, &(order), _x_99)?; match _x_100 {
        None => { let _x_102 = false; _x_102 },
        Some(val_103) => { let _x_104 = (val_97 < val_103); _x_104 },
    } } },
    } } },
    } } },
    } } })
}

pub fn directComponentOrderValid(x_1: &[crate::Component], x_2: &[crate::Component], x_3: &[u64]) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_33 = true; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_34 = &(head_24).dependsOn; { let _x_35 = &(head_24).id; { let _x_36 = directComponentDependenciesPrecede(&(_x_34), (_x_35).as_ref(), &(x_2), &(x_3))?; match _x_36 {
        false => _x_36,
        true => { let _x_39 = directComponentOrderValid(&(tail_25), &(x_2), &(x_3))?; _x_39 },
    } } } },
    })
}

pub fn directMigrationDependenciesPrecede(x_1: &[alloc::string::String], x_2: &str, x_3: &[crate::Migration], x_4: &[u64]) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_56 = true; _x_56 },
        [head_41, tail_42 @ ..] => { let _x_60 = memberIdMigrations((head_41).as_ref(), &(x_3)); match _x_60 {
        false => { let _x_71 = directMigrationDependenciesPrecede(&(tail_42), (x_2).as_ref(), &(x_3), &(x_4))?; _x_71 },
        true => { let _x_70 = directMigrationIndexPrecedes((head_41).as_ref(), (x_2).as_ref(), &(x_3), &(x_4))?; match _x_70 {
        false => _x_70,
        true => { let _x_73 = directMigrationDependenciesPrecede(&(tail_42), (x_2).as_ref(), &(x_3), &(x_4))?; _x_73 },
    } },
    } },
    })
}

pub fn directMigrationIndex(x_1: alloc::string::String, x_2: &[crate::Migration], x_3: u64) -> Result<Option<u64>, crate::ComputeError> {
    __prod_borrowed_directMigrationIndex(x_1.as_ref(), x_2, x_3)
}

fn __prod_borrowed_directMigrationIndex(x_1: &str, x_2: &[crate::Migration], x_3: u64) -> Result<Option<u64>, crate::ComputeError> {
    Ok(match x_2 {
        [] => None,
        [head_32, tail_33 @ ..] => { let _x_56 = &(head_32).id; { let _x_57 = (x_1 == _x_56); match _x_57 {
        false => { let _x_62 = 1; { let _x_63 = ((x_3) as u64).checked_add(_x_62).ok_or(crate::ComputeError::AddOverflow)?; { let _x_64 = __prod_borrowed_directMigrationIndex((x_1).as_ref(), &(tail_33), _x_63)?; _x_64 } } },
        true => { let _x_61 = Some(x_3); _x_61 },
    } } },
    })
}

pub fn directMigrationIndexPrecedes(dependency: &str, current: &str, rows: &[crate::Migration], order: &[u64]) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_43 = 0; { let _x_46 = __prod_borrowed_directMigrationIndex((dependency).as_ref(), &(rows), _x_43)?; match _x_46 {
        None => { let _x_65 = false; _x_65 },
        Some(val_49) => { let _x_80 = 0; { let _x_81 = __prod_borrowed_directMigrationIndex((current).as_ref(), &(rows), _x_80)?; match _x_81 {
        None => { let _x_83 = false; _x_83 },
        Some(val_84) => { let _x_93 = 0; { let _x_94 = natIndex(val_49, &(order), _x_93)?; match _x_94 {
        None => { let _x_96 = false; _x_96 },
        Some(val_97) => { let _x_99 = 0; { let _x_100 = natIndex(val_84, &(order), _x_99)?; match _x_100 {
        None => { let _x_102 = false; _x_102 },
        Some(val_103) => { let _x_104 = (val_97 < val_103); _x_104 },
    } } },
    } } },
    } } },
    } } })
}

pub fn directMigrationOrderValid(x_1: &[crate::Migration], x_2: &[crate::Migration], x_3: &[u64]) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_33 = true; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_34 = &(head_24).dependsOn; { let _x_35 = &(head_24).id; { let _x_36 = directMigrationDependenciesPrecede(&(_x_34), (_x_35).as_ref(), &(x_2), &(x_3))?; match _x_36 {
        false => _x_36,
        true => { let _x_39 = directMigrationOrderValid(&(tail_25), &(x_2), &(x_3))?; _x_39 },
    } } } },
    })
}

pub fn eventsReferential(x_1: &[crate::Event], x_2: &[crate::Component], x_3: &[crate::Interface], x_4: &[crate::Schema]) -> bool {
    match x_1 {
        [] => { let _x_72 = true; _x_72 },
        [head_54, tail_55 @ ..] => { let _x_91 = &(head_54).producer; { let _x_92 = memberIdComponents((_x_91).as_ref(), &(x_2)); match _x_92 {
        false => _x_92,
        true => { let _x_108 = &(head_54).owner; { let _x_109 = memberIdComponents((_x_108).as_ref(), &(x_2)); match _x_109 {
        false => _x_109,
        true => { let _x_119 = &(head_54).channel; { let _x_120 = memberIdInterfaces((_x_119).as_ref(), &(x_3)); match _x_120 {
        false => _x_120,
        true => { let _x_124 = &(head_54).schemaId; { let _x_125 = memberIdSchemas((_x_124).as_ref(), &(x_4)); match _x_125 {
        false => _x_125,
        true => { let _x_128 = eventsReferential(&(tail_55), &(x_2), &(x_3), &(x_4)); _x_128 },
    } } },
    } } },
    } } },
    } } },
    }
}

pub fn flowsReferential(x_1: &[crate::Flow], x_2: &[crate::Component], x_3: &[crate::Interface]) -> bool {
    match x_1 {
        [] => { let _x_58 = true; _x_58 },
        [head_43, tail_44 @ ..] => { let _x_71 = &(head_43).fromComponent; { let _x_72 = memberIdComponents((_x_71).as_ref(), &(x_2)); match _x_72 {
        false => _x_72,
        true => { let _x_82 = &(head_43).toComponent; { let _x_83 = memberIdComponents((_x_82).as_ref(), &(x_2)); match _x_83 {
        false => _x_83,
        true => { let _x_87 = &(head_43).interfaceId; { let _x_88 = memberIdInterfaces((_x_87).as_ref(), &(x_3)); match _x_88 {
        false => _x_88,
        true => { let _x_91 = flowsReferential(&(tail_44), &(x_2), &(x_3)); _x_91 },
    } } },
    } } },
    } } },
    }
}

pub fn identityReferential(x_1: &[crate::IdentityRequirement], x_2: &[crate::Configuration]) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).issuerParameter; { let _x_33 = memberIdParameters((_x_32).as_ref(), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = identityReferential(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn idsGloballyUniqueAcceptance(x_1: &[crate::Acceptance], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueAcceptance(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueAlerts(x_1: &[crate::Alert], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueAlerts(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueArchitecture(x_1: &[crate::ArchitectureBinding], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueArchitecture(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueArtifacts(x_1: &[crate::Artifact], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueArtifacts(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueBackups(x_1: &[crate::BackupRecovery], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueBackups(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueCalls(x_1: &[crate::Call], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueCalls(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueCapabilities(x_1: &[crate::Capability], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueCapabilities(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueComponents(x_1: &[crate::Component], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueComponents(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueControls(x_1: &[crate::Control], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueControls(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueDrifts(x_1: &[crate::Drift], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueDrifts(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueEvents(x_1: &[crate::Event], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueEvents(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueFlows(x_1: &[crate::Flow], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueFlows(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueIdentityRequirements(x_1: &[crate::IdentityRequirement], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueIdentityRequirements(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueInterfaces(x_1: &[crate::Interface], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueInterfaces(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueMigrations(x_1: &[crate::Migration], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueMigrations(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueParameters(x_1: &[crate::Configuration], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueParameters(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniquePersistence(x_1: &[crate::Persistence], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniquePersistence(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniquePlatformRequirements(x_1: &[crate::PlatformRequirement], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniquePlatformRequirements(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueRetirements(x_1: &[crate::Retirement], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueRetirements(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueRollbacks(x_1: &[crate::Rollback], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueRollbacks(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueRollouts(x_1: &[crate::Rollout], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueRollouts(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueScalingPolicies(x_1: &[crate::ScalingPolicy], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueScalingPolicies(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueSchemas(x_1: &[crate::Schema], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueSchemas(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueSecretReferences(x_1: &[crate::SecretReference], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueSecretReferences(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueSlis(x_1: &[crate::Sli], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueSlis(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueSlos(x_1: &[crate::Slo], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueSlos(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueStorageClasses(x_1: &[crate::StorageClass], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueStorageClasses(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueTargets(x_1: &[crate::TargetBinding], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueTargets(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn idsGloballyUniqueTopology(x_1: &[crate::Topology], x_2: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok(match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_26, tail_27 @ ..] => { let _x_37 = &(head_26).id; { let _x_38 = __prod_borrowed_modelIdCount((_x_37).as_ref(), &(x_2))?; { let _x_39 = 1; { let _x_40 = (_x_38 == _x_39); match _x_40 {
        false => _x_40,
        true => { let _x_43 = idsGloballyUniqueTopology(&(tail_27), &(x_2))?; _x_43 },
    } } } } },
    })
}

pub fn interfacesCompatible(x_1: &[crate::Interface]) -> bool {
    match x_1 {
        [] => { let _x_32 = true; _x_32 },
        [head_22, tail_23 @ ..] => { let _x_33 = &(head_22).compatibility; { let _x_34 = compatibilityValue((_x_33).as_ref()); match _x_34 {
        false => _x_34,
        true => { let _x_37 = interfacesCompatible(&(tail_23)); _x_37 },
    } } },
    }
}

pub fn interfacesReferential(x_1: &[crate::Interface], x_2: &[crate::Schema], x_3: &[crate::Acceptance]) -> bool {
    match x_1 {
        [] => { let _x_45 = true; _x_45 },
        [head_33, tail_34 @ ..] => { let _x_52 = &(head_33).document; { let _x_53 = memberIdSchemas((_x_52).as_ref(), &(x_2)); match _x_53 {
        false => _x_53,
        true => { let _x_57 = &(head_33).acceptance; { let _x_58 = allIdsMemberAcceptance(&(_x_57), &(x_3)); match _x_58 {
        false => _x_58,
        true => { let _x_61 = interfacesReferential(&(tail_34), &(x_2), &(x_3)); _x_61 },
    } } },
    } } },
    }
}

pub fn memberIdAcceptance(x_1: &str, x_2: &[crate::Acceptance]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdAcceptance((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdAlerts(x_1: &str, x_2: &[crate::Alert]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdAlerts((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdArchitecture(x_1: &str, x_2: &[crate::ArchitectureBinding]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdArchitecture((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdArtifacts(x_1: &str, x_2: &[crate::Artifact]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdArtifacts((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdBackups(x_1: &str, x_2: &[crate::BackupRecovery]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdBackups((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdCalls(x_1: &str, x_2: &[crate::Call]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdCalls((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdCapabilities(x_1: &str, x_2: &[crate::Capability]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdCapabilities((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdComponents(x_1: &str, x_2: &[crate::Component]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdComponents((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdControls(x_1: &str, x_2: &[crate::Control]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdControls((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdDrifts(x_1: &str, x_2: &[crate::Drift]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdDrifts((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdEvents(x_1: &str, x_2: &[crate::Event]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdEvents((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdFlows(x_1: &str, x_2: &[crate::Flow]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdFlows((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdIdentityRequirements(x_1: &str, x_2: &[crate::IdentityRequirement]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdIdentityRequirements((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdInterfaces(x_1: &str, x_2: &[crate::Interface]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdInterfaces((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdMigrations(x_1: &str, x_2: &[crate::Migration]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdMigrations((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdParameters(x_1: &str, x_2: &[crate::Configuration]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdParameters((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdPersistence(x_1: &str, x_2: &[crate::Persistence]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdPersistence((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdPlatformRequirements(x_1: &str, x_2: &[crate::PlatformRequirement]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdPlatformRequirements((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdRetirements(x_1: &str, x_2: &[crate::Retirement]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdRetirements((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdRollbacks(x_1: &str, x_2: &[crate::Rollback]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdRollbacks((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdRollouts(x_1: &str, x_2: &[crate::Rollout]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdRollouts((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdScalingPolicies(x_1: &str, x_2: &[crate::ScalingPolicy]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdScalingPolicies((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdSchemas(x_1: &str, x_2: &[crate::Schema]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdSchemas((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdSecretReferences(x_1: &str, x_2: &[crate::SecretReference]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdSecretReferences((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdSlis(x_1: &str, x_2: &[crate::Sli]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdSlis((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdSlos(x_1: &str, x_2: &[crate::Slo]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdSlos((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdStorageClasses(x_1: &str, x_2: &[crate::StorageClass]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdStorageClasses((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdTargets(x_1: &str, x_2: &[crate::TargetBinding]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdTargets((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn memberIdTopology(x_1: &str, x_2: &[crate::Topology]) -> bool {
    match x_2 {
        [] => { let _x_33 = false; _x_33 },
        [head_24, tail_25 @ ..] => { let _x_36 = &(head_24).id; { let _x_37 = (x_1 == _x_36); match _x_37 {
        false => { let _x_39 = memberIdTopology((x_1).as_ref(), &(tail_25)); _x_39 },
        true => _x_37,
    } } },
    }
}

pub fn modelEntityCount(model: &crate::SystemModel) -> Result<u64, crate::ComputeError> {
    Ok({ let _x_4 = 1; { let _x_7 = modelEntityCountGroup0(&(model))?; { let _x_8 = modelEntityCountGroup1(&(model))?; { let _x_9 = modelEntityCountGroup2(&(model))?; { let _x_10 = modelEntityCountGroup3(&(model))?; { let _x_11 = modelEntityCountGroup4(&(model))?; { let _x_12 = modelEntityCountGroup5(&(model))?; { let _x_26 = ((_x_11) as u64).checked_add(_x_12).ok_or(crate::ComputeError::AddOverflow)?; { let _x_30 = ((_x_10) as u64).checked_add(_x_26).ok_or(crate::ComputeError::AddOverflow)?; { let _x_34 = ((_x_9) as u64).checked_add(_x_30).ok_or(crate::ComputeError::AddOverflow)?; { let _x_38 = ((_x_8) as u64).checked_add(_x_34).ok_or(crate::ComputeError::AddOverflow)?; { let _x_42 = ((_x_7) as u64).checked_add(_x_38).ok_or(crate::ComputeError::AddOverflow)?; { let _x_46 = ((_x_4) as u64).checked_add(_x_42).ok_or(crate::ComputeError::AddOverflow)?; _x_46 } } } } } } } } } } } } })
}

pub fn modelEntityCountGroup0(model: &crate::SystemModel) -> Result<u64, crate::ComputeError> {
    Ok({ let _x_5 = &(model).acceptance; { let _x_6 = (_x_5).len() as u64; { let _x_8 = &(model).alerts; { let _x_9 = (_x_8).len() as u64; { let _x_11 = &(model).architecture; { let _x_12 = (_x_11).len() as u64; { let _x_14 = &(model).artifacts; { let _x_15 = (_x_14).len() as u64; { let _x_17 = &(model).backups; { let _x_18 = (_x_17).len() as u64; { let _x_29 = ((_x_15) as u64).checked_add(_x_18).ok_or(crate::ComputeError::AddOverflow)?; { let _x_33 = ((_x_12) as u64).checked_add(_x_29).ok_or(crate::ComputeError::AddOverflow)?; { let _x_37 = ((_x_9) as u64).checked_add(_x_33).ok_or(crate::ComputeError::AddOverflow)?; { let _x_41 = ((_x_6) as u64).checked_add(_x_37).ok_or(crate::ComputeError::AddOverflow)?; _x_41 } } } } } } } } } } } } } })
}

pub fn modelEntityCountGroup1(model: &crate::SystemModel) -> Result<u64, crate::ComputeError> {
    Ok({ let _x_5 = &(model).calls; { let _x_6 = (_x_5).len() as u64; { let _x_8 = &(model).capabilities; { let _x_9 = (_x_8).len() as u64; { let _x_11 = &(model).components; { let _x_12 = (_x_11).len() as u64; { let _x_14 = &(model).controls; { let _x_15 = (_x_14).len() as u64; { let _x_17 = &(model).drifts; { let _x_18 = (_x_17).len() as u64; { let _x_29 = ((_x_15) as u64).checked_add(_x_18).ok_or(crate::ComputeError::AddOverflow)?; { let _x_33 = ((_x_12) as u64).checked_add(_x_29).ok_or(crate::ComputeError::AddOverflow)?; { let _x_37 = ((_x_9) as u64).checked_add(_x_33).ok_or(crate::ComputeError::AddOverflow)?; { let _x_41 = ((_x_6) as u64).checked_add(_x_37).ok_or(crate::ComputeError::AddOverflow)?; _x_41 } } } } } } } } } } } } } })
}

pub fn modelEntityCountGroup2(model: &crate::SystemModel) -> Result<u64, crate::ComputeError> {
    Ok({ let _x_5 = &(model).events; { let _x_6 = (_x_5).len() as u64; { let _x_8 = &(model).flows; { let _x_9 = (_x_8).len() as u64; { let _x_11 = &(model).identityRequirements; { let _x_12 = (_x_11).len() as u64; { let _x_14 = &(model).interfaces; { let _x_15 = (_x_14).len() as u64; { let _x_17 = &(model).migrations; { let _x_18 = (_x_17).len() as u64; { let _x_29 = ((_x_15) as u64).checked_add(_x_18).ok_or(crate::ComputeError::AddOverflow)?; { let _x_33 = ((_x_12) as u64).checked_add(_x_29).ok_or(crate::ComputeError::AddOverflow)?; { let _x_37 = ((_x_9) as u64).checked_add(_x_33).ok_or(crate::ComputeError::AddOverflow)?; { let _x_41 = ((_x_6) as u64).checked_add(_x_37).ok_or(crate::ComputeError::AddOverflow)?; _x_41 } } } } } } } } } } } } } })
}

pub fn modelEntityCountGroup3(model: &crate::SystemModel) -> Result<u64, crate::ComputeError> {
    Ok({ let _x_5 = &(model).parameters; { let _x_6 = (_x_5).len() as u64; { let _x_8 = &(model).persistence; { let _x_9 = (_x_8).len() as u64; { let _x_11 = &(model).platformRequirements; { let _x_12 = (_x_11).len() as u64; { let _x_14 = &(model).retirements; { let _x_15 = (_x_14).len() as u64; { let _x_17 = &(model).rollbacks; { let _x_18 = (_x_17).len() as u64; { let _x_29 = ((_x_15) as u64).checked_add(_x_18).ok_or(crate::ComputeError::AddOverflow)?; { let _x_33 = ((_x_12) as u64).checked_add(_x_29).ok_or(crate::ComputeError::AddOverflow)?; { let _x_37 = ((_x_9) as u64).checked_add(_x_33).ok_or(crate::ComputeError::AddOverflow)?; { let _x_41 = ((_x_6) as u64).checked_add(_x_37).ok_or(crate::ComputeError::AddOverflow)?; _x_41 } } } } } } } } } } } } } })
}

pub fn modelEntityCountGroup4(model: &crate::SystemModel) -> Result<u64, crate::ComputeError> {
    Ok({ let _x_5 = &(model).rollouts; { let _x_6 = (_x_5).len() as u64; { let _x_8 = &(model).scalingPolicies; { let _x_9 = (_x_8).len() as u64; { let _x_11 = &(model).schemas; { let _x_12 = (_x_11).len() as u64; { let _x_14 = &(model).secretReferences; { let _x_15 = (_x_14).len() as u64; { let _x_17 = &(model).slis; { let _x_18 = (_x_17).len() as u64; { let _x_29 = ((_x_15) as u64).checked_add(_x_18).ok_or(crate::ComputeError::AddOverflow)?; { let _x_33 = ((_x_12) as u64).checked_add(_x_29).ok_or(crate::ComputeError::AddOverflow)?; { let _x_37 = ((_x_9) as u64).checked_add(_x_33).ok_or(crate::ComputeError::AddOverflow)?; { let _x_41 = ((_x_6) as u64).checked_add(_x_37).ok_or(crate::ComputeError::AddOverflow)?; _x_41 } } } } } } } } } } } } } })
}

pub fn modelEntityCountGroup5(model: &crate::SystemModel) -> Result<u64, crate::ComputeError> {
    Ok({ let _x_5 = &(model).slos; { let _x_6 = (_x_5).len() as u64; { let _x_8 = &(model).storageClasses; { let _x_9 = (_x_8).len() as u64; { let _x_11 = &(model).targets; { let _x_12 = (_x_11).len() as u64; { let _x_14 = &(model).topology; { let _x_15 = (_x_14).len() as u64; { let _x_25 = ((_x_12) as u64).checked_add(_x_15).ok_or(crate::ComputeError::AddOverflow)?; { let _x_29 = ((_x_9) as u64).checked_add(_x_25).ok_or(crate::ComputeError::AddOverflow)?; { let _x_33 = ((_x_6) as u64).checked_add(_x_29).ok_or(crate::ComputeError::AddOverflow)?; _x_33 } } } } } } } } } } })
}

pub fn modelHasId(value: &str, model: &crate::SystemModel) -> bool {
    { let _x_54 = &(model).product; { let _x_55 = &(_x_54).id; { let _x_56 = (value == _x_55); match _x_56 {
        false => { let _x_102 = modelHasIdGroup0((value).as_ref(), &(model)); match _x_102 {
        false => { let _x_121 = modelHasIdGroup1((value).as_ref(), &(model)); match _x_121 {
        false => { let _x_135 = modelHasIdGroup2((value).as_ref(), &(model)); match _x_135 {
        false => { let _x_144 = modelHasIdGroup3((value).as_ref(), &(model)); match _x_144 {
        false => { let _x_148 = modelHasIdGroup4((value).as_ref(), &(model)); match _x_148 {
        false => { let _x_150 = modelHasIdGroup5((value).as_ref(), &(model)); _x_150 },
        true => _x_148,
    } },
        true => _x_144,
    } },
        true => _x_135,
    } },
        true => _x_121,
    } },
        true => _x_102,
    } },
        true => _x_56,
    } } } }
}

pub fn modelHasIdGroup0(value: &str, model: &crate::SystemModel) -> bool {
    { let _x_38 = &(model).acceptance; { let _x_39 = memberIdAcceptance((value).as_ref(), &(_x_38)); match _x_39 {
        false => { let _x_73 = &(model).alerts; { let _x_74 = memberIdAlerts((value).as_ref(), &(_x_73)); match _x_74 {
        false => { let _x_87 = &(model).architecture; { let _x_88 = memberIdArchitecture((value).as_ref(), &(_x_87)); match _x_88 {
        false => { let _x_95 = &(model).artifacts; { let _x_96 = memberIdArtifacts((value).as_ref(), &(_x_95)); match _x_96 {
        false => { let _x_100 = &(model).backups; { let _x_101 = memberIdBackups((value).as_ref(), &(_x_100)); _x_101 } },
        true => _x_96,
    } } },
        true => _x_88,
    } } },
        true => _x_74,
    } } },
        true => _x_39,
    } } }
}

pub fn modelHasIdGroup1(value: &str, model: &crate::SystemModel) -> bool {
    { let _x_38 = &(model).calls; { let _x_39 = memberIdCalls((value).as_ref(), &(_x_38)); match _x_39 {
        false => { let _x_73 = &(model).capabilities; { let _x_74 = memberIdCapabilities((value).as_ref(), &(_x_73)); match _x_74 {
        false => { let _x_87 = &(model).components; { let _x_88 = memberIdComponents((value).as_ref(), &(_x_87)); match _x_88 {
        false => { let _x_95 = &(model).controls; { let _x_96 = memberIdControls((value).as_ref(), &(_x_95)); match _x_96 {
        false => { let _x_100 = &(model).drifts; { let _x_101 = memberIdDrifts((value).as_ref(), &(_x_100)); _x_101 } },
        true => _x_96,
    } } },
        true => _x_88,
    } } },
        true => _x_74,
    } } },
        true => _x_39,
    } } }
}

pub fn modelHasIdGroup2(value: &str, model: &crate::SystemModel) -> bool {
    { let _x_38 = &(model).events; { let _x_39 = memberIdEvents((value).as_ref(), &(_x_38)); match _x_39 {
        false => { let _x_73 = &(model).flows; { let _x_74 = memberIdFlows((value).as_ref(), &(_x_73)); match _x_74 {
        false => { let _x_87 = &(model).identityRequirements; { let _x_88 = memberIdIdentityRequirements((value).as_ref(), &(_x_87)); match _x_88 {
        false => { let _x_95 = &(model).interfaces; { let _x_96 = memberIdInterfaces((value).as_ref(), &(_x_95)); match _x_96 {
        false => { let _x_100 = &(model).migrations; { let _x_101 = memberIdMigrations((value).as_ref(), &(_x_100)); _x_101 } },
        true => _x_96,
    } } },
        true => _x_88,
    } } },
        true => _x_74,
    } } },
        true => _x_39,
    } } }
}

pub fn modelHasIdGroup3(value: &str, model: &crate::SystemModel) -> bool {
    { let _x_38 = &(model).parameters; { let _x_39 = memberIdParameters((value).as_ref(), &(_x_38)); match _x_39 {
        false => { let _x_73 = &(model).persistence; { let _x_74 = memberIdPersistence((value).as_ref(), &(_x_73)); match _x_74 {
        false => { let _x_87 = &(model).platformRequirements; { let _x_88 = memberIdPlatformRequirements((value).as_ref(), &(_x_87)); match _x_88 {
        false => { let _x_95 = &(model).retirements; { let _x_96 = memberIdRetirements((value).as_ref(), &(_x_95)); match _x_96 {
        false => { let _x_100 = &(model).rollbacks; { let _x_101 = memberIdRollbacks((value).as_ref(), &(_x_100)); _x_101 } },
        true => _x_96,
    } } },
        true => _x_88,
    } } },
        true => _x_74,
    } } },
        true => _x_39,
    } } }
}

pub fn modelHasIdGroup4(value: &str, model: &crate::SystemModel) -> bool {
    { let _x_38 = &(model).rollouts; { let _x_39 = memberIdRollouts((value).as_ref(), &(_x_38)); match _x_39 {
        false => { let _x_73 = &(model).scalingPolicies; { let _x_74 = memberIdScalingPolicies((value).as_ref(), &(_x_73)); match _x_74 {
        false => { let _x_87 = &(model).schemas; { let _x_88 = memberIdSchemas((value).as_ref(), &(_x_87)); match _x_88 {
        false => { let _x_95 = &(model).secretReferences; { let _x_96 = memberIdSecretReferences((value).as_ref(), &(_x_95)); match _x_96 {
        false => { let _x_100 = &(model).slis; { let _x_101 = memberIdSlis((value).as_ref(), &(_x_100)); _x_101 } },
        true => _x_96,
    } } },
        true => _x_88,
    } } },
        true => _x_74,
    } } },
        true => _x_39,
    } } }
}

pub fn modelHasIdGroup5(value: &str, model: &crate::SystemModel) -> bool {
    { let _x_28 = &(model).slos; { let _x_29 = memberIdSlos((value).as_ref(), &(_x_28)); match _x_29 {
        false => { let _x_54 = &(model).storageClasses; { let _x_55 = memberIdStorageClasses((value).as_ref(), &(_x_54)); match _x_55 {
        false => { let _x_62 = &(model).targets; { let _x_63 = memberIdTargets((value).as_ref(), &(_x_62)); match _x_63 {
        false => { let _x_67 = &(model).topology; { let _x_68 = memberIdTopology((value).as_ref(), &(_x_67)); _x_68 } },
        true => _x_63,
    } } },
        true => _x_55,
    } } },
        true => _x_29,
    } } }
}

pub fn modelIdCount(value: alloc::string::String, model: &crate::SystemModel) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_modelIdCount(value.as_ref(), model)
}

fn __prod_borrowed_modelIdCount(value: &str, model: &crate::SystemModel) -> Result<u64, crate::ComputeError> {
    Ok({ let _x_16 = &(model).product; { let _x_17 = &(_x_16).id; { let _x_18 = (value == _x_17); { let _jp_36 = /* jp "_jp_36" inlined at its jump site */ (); match _x_18 {
        false => { let _x_76 = 0; { let _y_23 = _x_76; { let _x_24 = __prod_borrowed_modelIdCountGroup0((value).as_ref(), &(model))?; { let _x_25 = __prod_borrowed_modelIdCountGroup1((value).as_ref(), &(model))?; { let _x_26 = __prod_borrowed_modelIdCountGroup2((value).as_ref(), &(model))?; { let _x_27 = __prod_borrowed_modelIdCountGroup3((value).as_ref(), &(model))?; { let _x_28 = __prod_borrowed_modelIdCountGroup4((value).as_ref(), &(model))?; { let _x_29 = __prod_borrowed_modelIdCountGroup5((value).as_ref(), &(model))?; { let _x_51 = ((_x_28) as u64).checked_add(_x_29).ok_or(crate::ComputeError::AddOverflow)?; { let _x_55 = ((_x_27) as u64).checked_add(_x_51).ok_or(crate::ComputeError::AddOverflow)?; { let _x_59 = ((_x_26) as u64).checked_add(_x_55).ok_or(crate::ComputeError::AddOverflow)?; { let _x_63 = ((_x_25) as u64).checked_add(_x_59).ok_or(crate::ComputeError::AddOverflow)?; { let _x_67 = ((_x_24) as u64).checked_add(_x_63).ok_or(crate::ComputeError::AddOverflow)?; { let _x_71 = ((_y_23) as u64).checked_add(_x_67).ok_or(crate::ComputeError::AddOverflow)?; _x_71 } } } } } } } } } } } } } },
        true => { let _x_79 = 1; { let _y_23 = _x_79; { let _x_24 = __prod_borrowed_modelIdCountGroup0((value).as_ref(), &(model))?; { let _x_25 = __prod_borrowed_modelIdCountGroup1((value).as_ref(), &(model))?; { let _x_26 = __prod_borrowed_modelIdCountGroup2((value).as_ref(), &(model))?; { let _x_27 = __prod_borrowed_modelIdCountGroup3((value).as_ref(), &(model))?; { let _x_28 = __prod_borrowed_modelIdCountGroup4((value).as_ref(), &(model))?; { let _x_29 = __prod_borrowed_modelIdCountGroup5((value).as_ref(), &(model))?; { let _x_51 = ((_x_28) as u64).checked_add(_x_29).ok_or(crate::ComputeError::AddOverflow)?; { let _x_55 = ((_x_27) as u64).checked_add(_x_51).ok_or(crate::ComputeError::AddOverflow)?; { let _x_59 = ((_x_26) as u64).checked_add(_x_55).ok_or(crate::ComputeError::AddOverflow)?; { let _x_63 = ((_x_25) as u64).checked_add(_x_59).ok_or(crate::ComputeError::AddOverflow)?; { let _x_67 = ((_x_24) as u64).checked_add(_x_63).ok_or(crate::ComputeError::AddOverflow)?; { let _x_71 = ((_y_23) as u64).checked_add(_x_67).ok_or(crate::ComputeError::AddOverflow)?; _x_71 } } } } } } } } } } } } } },
    } } } } })
}

pub fn modelIdCountGroup0(value: alloc::string::String, model: &crate::SystemModel) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_modelIdCountGroup0(value.as_ref(), model)
}

fn __prod_borrowed_modelIdCountGroup0(value: &str, model: &crate::SystemModel) -> Result<u64, crate::ComputeError> {
    Ok({ let _x_4 = &(model).acceptance; { let _x_5 = __prod_borrowed_countIdAcceptance((value).as_ref(), &(_x_4))?; { let _x_6 = &(model).alerts; { let _x_7 = __prod_borrowed_countIdAlerts((value).as_ref(), &(_x_6))?; { let _x_8 = &(model).architecture; { let _x_9 = __prod_borrowed_countIdArchitecture((value).as_ref(), &(_x_8))?; { let _x_10 = &(model).artifacts; { let _x_11 = __prod_borrowed_countIdArtifacts((value).as_ref(), &(_x_10))?; { let _x_12 = &(model).backups; { let _x_13 = __prod_borrowed_countIdBackups((value).as_ref(), &(_x_12))?; { let _x_24 = ((_x_11) as u64).checked_add(_x_13).ok_or(crate::ComputeError::AddOverflow)?; { let _x_28 = ((_x_9) as u64).checked_add(_x_24).ok_or(crate::ComputeError::AddOverflow)?; { let _x_32 = ((_x_7) as u64).checked_add(_x_28).ok_or(crate::ComputeError::AddOverflow)?; { let _x_36 = ((_x_5) as u64).checked_add(_x_32).ok_or(crate::ComputeError::AddOverflow)?; _x_36 } } } } } } } } } } } } } })
}

pub fn modelIdCountGroup1(value: alloc::string::String, model: &crate::SystemModel) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_modelIdCountGroup1(value.as_ref(), model)
}

fn __prod_borrowed_modelIdCountGroup1(value: &str, model: &crate::SystemModel) -> Result<u64, crate::ComputeError> {
    Ok({ let _x_4 = &(model).calls; { let _x_5 = __prod_borrowed_countIdCalls((value).as_ref(), &(_x_4))?; { let _x_6 = &(model).capabilities; { let _x_7 = __prod_borrowed_countIdCapabilities((value).as_ref(), &(_x_6))?; { let _x_8 = &(model).components; { let _x_9 = __prod_borrowed_countIdComponents((value).as_ref(), &(_x_8))?; { let _x_10 = &(model).controls; { let _x_11 = __prod_borrowed_countIdControls((value).as_ref(), &(_x_10))?; { let _x_12 = &(model).drifts; { let _x_13 = __prod_borrowed_countIdDrifts((value).as_ref(), &(_x_12))?; { let _x_24 = ((_x_11) as u64).checked_add(_x_13).ok_or(crate::ComputeError::AddOverflow)?; { let _x_28 = ((_x_9) as u64).checked_add(_x_24).ok_or(crate::ComputeError::AddOverflow)?; { let _x_32 = ((_x_7) as u64).checked_add(_x_28).ok_or(crate::ComputeError::AddOverflow)?; { let _x_36 = ((_x_5) as u64).checked_add(_x_32).ok_or(crate::ComputeError::AddOverflow)?; _x_36 } } } } } } } } } } } } } })
}

pub fn modelIdCountGroup2(value: alloc::string::String, model: &crate::SystemModel) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_modelIdCountGroup2(value.as_ref(), model)
}

fn __prod_borrowed_modelIdCountGroup2(value: &str, model: &crate::SystemModel) -> Result<u64, crate::ComputeError> {
    Ok({ let _x_4 = &(model).events; { let _x_5 = __prod_borrowed_countIdEvents((value).as_ref(), &(_x_4))?; { let _x_6 = &(model).flows; { let _x_7 = __prod_borrowed_countIdFlows((value).as_ref(), &(_x_6))?; { let _x_8 = &(model).identityRequirements; { let _x_9 = __prod_borrowed_countIdIdentityRequirements((value).as_ref(), &(_x_8))?; { let _x_10 = &(model).interfaces; { let _x_11 = __prod_borrowed_countIdInterfaces((value).as_ref(), &(_x_10))?; { let _x_12 = &(model).migrations; { let _x_13 = __prod_borrowed_countIdMigrations((value).as_ref(), &(_x_12))?; { let _x_24 = ((_x_11) as u64).checked_add(_x_13).ok_or(crate::ComputeError::AddOverflow)?; { let _x_28 = ((_x_9) as u64).checked_add(_x_24).ok_or(crate::ComputeError::AddOverflow)?; { let _x_32 = ((_x_7) as u64).checked_add(_x_28).ok_or(crate::ComputeError::AddOverflow)?; { let _x_36 = ((_x_5) as u64).checked_add(_x_32).ok_or(crate::ComputeError::AddOverflow)?; _x_36 } } } } } } } } } } } } } })
}

pub fn modelIdCountGroup3(value: alloc::string::String, model: &crate::SystemModel) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_modelIdCountGroup3(value.as_ref(), model)
}

fn __prod_borrowed_modelIdCountGroup3(value: &str, model: &crate::SystemModel) -> Result<u64, crate::ComputeError> {
    Ok({ let _x_4 = &(model).parameters; { let _x_5 = __prod_borrowed_countIdParameters((value).as_ref(), &(_x_4))?; { let _x_6 = &(model).persistence; { let _x_7 = __prod_borrowed_countIdPersistence((value).as_ref(), &(_x_6))?; { let _x_8 = &(model).platformRequirements; { let _x_9 = __prod_borrowed_countIdPlatformRequirements((value).as_ref(), &(_x_8))?; { let _x_10 = &(model).retirements; { let _x_11 = __prod_borrowed_countIdRetirements((value).as_ref(), &(_x_10))?; { let _x_12 = &(model).rollbacks; { let _x_13 = __prod_borrowed_countIdRollbacks((value).as_ref(), &(_x_12))?; { let _x_24 = ((_x_11) as u64).checked_add(_x_13).ok_or(crate::ComputeError::AddOverflow)?; { let _x_28 = ((_x_9) as u64).checked_add(_x_24).ok_or(crate::ComputeError::AddOverflow)?; { let _x_32 = ((_x_7) as u64).checked_add(_x_28).ok_or(crate::ComputeError::AddOverflow)?; { let _x_36 = ((_x_5) as u64).checked_add(_x_32).ok_or(crate::ComputeError::AddOverflow)?; _x_36 } } } } } } } } } } } } } })
}

pub fn modelIdCountGroup4(value: alloc::string::String, model: &crate::SystemModel) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_modelIdCountGroup4(value.as_ref(), model)
}

fn __prod_borrowed_modelIdCountGroup4(value: &str, model: &crate::SystemModel) -> Result<u64, crate::ComputeError> {
    Ok({ let _x_4 = &(model).rollouts; { let _x_5 = __prod_borrowed_countIdRollouts((value).as_ref(), &(_x_4))?; { let _x_6 = &(model).scalingPolicies; { let _x_7 = __prod_borrowed_countIdScalingPolicies((value).as_ref(), &(_x_6))?; { let _x_8 = &(model).schemas; { let _x_9 = __prod_borrowed_countIdSchemas((value).as_ref(), &(_x_8))?; { let _x_10 = &(model).secretReferences; { let _x_11 = __prod_borrowed_countIdSecretReferences((value).as_ref(), &(_x_10))?; { let _x_12 = &(model).slis; { let _x_13 = __prod_borrowed_countIdSlis((value).as_ref(), &(_x_12))?; { let _x_24 = ((_x_11) as u64).checked_add(_x_13).ok_or(crate::ComputeError::AddOverflow)?; { let _x_28 = ((_x_9) as u64).checked_add(_x_24).ok_or(crate::ComputeError::AddOverflow)?; { let _x_32 = ((_x_7) as u64).checked_add(_x_28).ok_or(crate::ComputeError::AddOverflow)?; { let _x_36 = ((_x_5) as u64).checked_add(_x_32).ok_or(crate::ComputeError::AddOverflow)?; _x_36 } } } } } } } } } } } } } })
}

pub fn modelIdCountGroup5(value: alloc::string::String, model: &crate::SystemModel) -> Result<u64, crate::ComputeError> {
    __prod_borrowed_modelIdCountGroup5(value.as_ref(), model)
}

fn __prod_borrowed_modelIdCountGroup5(value: &str, model: &crate::SystemModel) -> Result<u64, crate::ComputeError> {
    Ok({ let _x_4 = &(model).slos; { let _x_5 = __prod_borrowed_countIdSlos((value).as_ref(), &(_x_4))?; { let _x_6 = &(model).storageClasses; { let _x_7 = __prod_borrowed_countIdStorageClasses((value).as_ref(), &(_x_6))?; { let _x_8 = &(model).targets; { let _x_9 = __prod_borrowed_countIdTargets((value).as_ref(), &(_x_8))?; { let _x_10 = &(model).topology; { let _x_11 = __prod_borrowed_countIdTopology((value).as_ref(), &(_x_10))?; { let _x_21 = ((_x_9) as u64).checked_add(_x_11).ok_or(crate::ComputeError::AddOverflow)?; { let _x_25 = ((_x_7) as u64).checked_add(_x_21).ok_or(crate::ComputeError::AddOverflow)?; { let _x_29 = ((_x_5) as u64).checked_add(_x_25).ok_or(crate::ComputeError::AddOverflow)?; _x_29 } } } } } } } } } } })
}

pub fn modelIdsGloballyUnique(model: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_52 = &(model).product; { let _x_53 = &(_x_52).id; { let _x_54 = __prod_borrowed_modelIdCount((_x_53).as_ref(), &(model))?; { let _x_55 = 1; { let _x_58 = (_x_54 == _x_55); match _x_58 {
        false => _x_58,
        true => { let _x_105 = modelIdsGloballyUniqueGroup0(&(model))?; match _x_105 {
        false => _x_105,
        true => { let _x_124 = modelIdsGloballyUniqueGroup1(&(model))?; match _x_124 {
        false => _x_124,
        true => { let _x_138 = modelIdsGloballyUniqueGroup2(&(model))?; match _x_138 {
        false => _x_138,
        true => { let _x_147 = modelIdsGloballyUniqueGroup3(&(model))?; match _x_147 {
        false => _x_147,
        true => { let _x_151 = modelIdsGloballyUniqueGroup4(&(model))?; match _x_151 {
        false => _x_151,
        true => { let _x_154 = modelIdsGloballyUniqueGroup5(&(model))?; _x_154 },
    } },
    } },
    } },
    } },
    } },
    } } } } } })
}

pub fn modelIdsGloballyUniqueGroup0(model: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_38 = &(model).acceptance; { let _x_39 = idsGloballyUniqueAcceptance(&(_x_38), &(model))?; match _x_39 {
        false => _x_39,
        true => { let _x_73 = &(model).alerts; { let _x_74 = idsGloballyUniqueAlerts(&(_x_73), &(model))?; match _x_74 {
        false => _x_74,
        true => { let _x_87 = &(model).architecture; { let _x_88 = idsGloballyUniqueArchitecture(&(_x_87), &(model))?; match _x_88 {
        false => _x_88,
        true => { let _x_95 = &(model).artifacts; { let _x_96 = idsGloballyUniqueArtifacts(&(_x_95), &(model))?; match _x_96 {
        false => _x_96,
        true => { let _x_100 = &(model).backups; { let _x_101 = idsGloballyUniqueBackups(&(_x_100), &(model))?; _x_101 } },
    } } },
    } } },
    } } },
    } } })
}

pub fn modelIdsGloballyUniqueGroup1(model: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_38 = &(model).calls; { let _x_39 = idsGloballyUniqueCalls(&(_x_38), &(model))?; match _x_39 {
        false => _x_39,
        true => { let _x_73 = &(model).capabilities; { let _x_74 = idsGloballyUniqueCapabilities(&(_x_73), &(model))?; match _x_74 {
        false => _x_74,
        true => { let _x_87 = &(model).components; { let _x_88 = idsGloballyUniqueComponents(&(_x_87), &(model))?; match _x_88 {
        false => _x_88,
        true => { let _x_95 = &(model).controls; { let _x_96 = idsGloballyUniqueControls(&(_x_95), &(model))?; match _x_96 {
        false => _x_96,
        true => { let _x_100 = &(model).drifts; { let _x_101 = idsGloballyUniqueDrifts(&(_x_100), &(model))?; _x_101 } },
    } } },
    } } },
    } } },
    } } })
}

pub fn modelIdsGloballyUniqueGroup2(model: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_38 = &(model).events; { let _x_39 = idsGloballyUniqueEvents(&(_x_38), &(model))?; match _x_39 {
        false => _x_39,
        true => { let _x_73 = &(model).flows; { let _x_74 = idsGloballyUniqueFlows(&(_x_73), &(model))?; match _x_74 {
        false => _x_74,
        true => { let _x_87 = &(model).identityRequirements; { let _x_88 = idsGloballyUniqueIdentityRequirements(&(_x_87), &(model))?; match _x_88 {
        false => _x_88,
        true => { let _x_95 = &(model).interfaces; { let _x_96 = idsGloballyUniqueInterfaces(&(_x_95), &(model))?; match _x_96 {
        false => _x_96,
        true => { let _x_100 = &(model).migrations; { let _x_101 = idsGloballyUniqueMigrations(&(_x_100), &(model))?; _x_101 } },
    } } },
    } } },
    } } },
    } } })
}

pub fn modelIdsGloballyUniqueGroup3(model: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_38 = &(model).parameters; { let _x_39 = idsGloballyUniqueParameters(&(_x_38), &(model))?; match _x_39 {
        false => _x_39,
        true => { let _x_73 = &(model).persistence; { let _x_74 = idsGloballyUniquePersistence(&(_x_73), &(model))?; match _x_74 {
        false => _x_74,
        true => { let _x_87 = &(model).platformRequirements; { let _x_88 = idsGloballyUniquePlatformRequirements(&(_x_87), &(model))?; match _x_88 {
        false => _x_88,
        true => { let _x_95 = &(model).retirements; { let _x_96 = idsGloballyUniqueRetirements(&(_x_95), &(model))?; match _x_96 {
        false => _x_96,
        true => { let _x_100 = &(model).rollbacks; { let _x_101 = idsGloballyUniqueRollbacks(&(_x_100), &(model))?; _x_101 } },
    } } },
    } } },
    } } },
    } } })
}

pub fn modelIdsGloballyUniqueGroup4(model: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_38 = &(model).rollouts; { let _x_39 = idsGloballyUniqueRollouts(&(_x_38), &(model))?; match _x_39 {
        false => _x_39,
        true => { let _x_73 = &(model).scalingPolicies; { let _x_74 = idsGloballyUniqueScalingPolicies(&(_x_73), &(model))?; match _x_74 {
        false => _x_74,
        true => { let _x_87 = &(model).schemas; { let _x_88 = idsGloballyUniqueSchemas(&(_x_87), &(model))?; match _x_88 {
        false => _x_88,
        true => { let _x_95 = &(model).secretReferences; { let _x_96 = idsGloballyUniqueSecretReferences(&(_x_95), &(model))?; match _x_96 {
        false => _x_96,
        true => { let _x_100 = &(model).slis; { let _x_101 = idsGloballyUniqueSlis(&(_x_100), &(model))?; _x_101 } },
    } } },
    } } },
    } } },
    } } })
}

pub fn modelIdsGloballyUniqueGroup5(model: &crate::SystemModel) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_28 = &(model).slos; { let _x_29 = idsGloballyUniqueSlos(&(_x_28), &(model))?; match _x_29 {
        false => _x_29,
        true => { let _x_54 = &(model).storageClasses; { let _x_55 = idsGloballyUniqueStorageClasses(&(_x_54), &(model))?; match _x_55 {
        false => _x_55,
        true => { let _x_62 = &(model).targets; { let _x_63 = idsGloballyUniqueTargets(&(_x_62), &(model))?; match _x_63 {
        false => _x_63,
        true => { let _x_67 = &(model).topology; { let _x_68 = idsGloballyUniqueTopology(&(_x_67), &(model))?; _x_68 } },
    } } },
    } } },
    } } })
}

pub fn natIndex(x_1: u64, x_2: &[u64], x_3: u64) -> Result<Option<u64>, crate::ComputeError> {
    Ok(match x_2 {
        [] => None,
        [head_29, tail_30 @ ..] => { let head_29 = head_29.clone(); { let _x_51 = (x_1 == head_29); match _x_51 {
        false => { let _x_56 = 1; { let _x_57 = ((x_3) as u64).checked_add(_x_56).ok_or(crate::ComputeError::AddOverflow)?; { let _x_58 = natIndex(x_1, &(tail_30), _x_57)?; _x_58 } } },
        true => { let _x_55 = Some(x_3); _x_55 },
    } } },
    })
}

pub fn natMember(x_1: u64, x_2: &[u64]) -> bool {
    match x_2 {
        [] => { let _x_30 = false; _x_30 },
        [head_21, tail_22 @ ..] => { let head_21 = head_21.clone(); { let _x_31 = (x_1 == head_21); match _x_31 {
        false => { let _x_33 = natMember(x_1, &(tail_22)); _x_33 },
        true => _x_31,
    } } },
    }
}

pub fn nonEmptyString(value: &str) -> bool {
    { let _x_4 = value == ""; match _x_4 {
        false => { let _x_7 = true; _x_7 },
        true => { let _x_8 = false; _x_8 },
    } }
}

pub fn optionalIdMemberArtifacts(value: Option<alloc::string::String>, rows: &[crate::Artifact]) -> bool {
    __prod_borrowed_optionalIdMemberArtifacts(&value, rows)
}

fn __prod_borrowed_optionalIdMemberArtifacts(value: &Option<alloc::string::String>, rows: &[crate::Artifact]) -> bool {
    match value {
        None => { let _x_14 = true; _x_14 },
        Some(val_9) => { let _x_15 = memberIdArtifacts((val_9).as_ref(), &(rows)); _x_15 },
    }
}

pub fn optionalIdMemberScalingPolicies(value: Option<alloc::string::String>, rows: &[crate::ScalingPolicy]) -> bool {
    __prod_borrowed_optionalIdMemberScalingPolicies(&value, rows)
}

fn __prod_borrowed_optionalIdMemberScalingPolicies(value: &Option<alloc::string::String>, rows: &[crate::ScalingPolicy]) -> bool {
    match value {
        None => { let _x_14 = true; _x_14 },
        Some(val_9) => { let _x_15 = memberIdScalingPolicies((val_9).as_ref(), &(rows)); _x_15 },
    }
}

pub fn optionalIdMemberSecretReferences(value: Option<alloc::string::String>, rows: &[crate::SecretReference]) -> bool {
    __prod_borrowed_optionalIdMemberSecretReferences(&value, rows)
}

fn __prod_borrowed_optionalIdMemberSecretReferences(value: &Option<alloc::string::String>, rows: &[crate::SecretReference]) -> bool {
    match value {
        None => { let _x_14 = true; _x_14 },
        Some(val_9) => { let _x_15 = memberIdSecretReferences((val_9).as_ref(), &(rows)); _x_15 },
    }
}

pub fn optionalIdMemberStorageClasses(value: Option<alloc::string::String>, rows: &[crate::StorageClass]) -> bool {
    __prod_borrowed_optionalIdMemberStorageClasses(&value, rows)
}

fn __prod_borrowed_optionalIdMemberStorageClasses(value: &Option<alloc::string::String>, rows: &[crate::StorageClass]) -> bool {
    match value {
        None => { let _x_14 = true; _x_14 },
        Some(val_9) => { let _x_15 = memberIdStorageClasses((val_9).as_ref(), &(rows)); _x_15 },
    }
}

pub fn optionalIdMemberTopology(value: Option<alloc::string::String>, rows: &[crate::Topology]) -> bool {
    __prod_borrowed_optionalIdMemberTopology(&value, rows)
}

fn __prod_borrowed_optionalIdMemberTopology(value: &Option<alloc::string::String>, rows: &[crate::Topology]) -> bool {
    match value {
        None => { let _x_14 = true; _x_14 },
        Some(val_9) => { let _x_15 = memberIdTopology((val_9).as_ref(), &(rows)); _x_15 },
    }
}

pub fn optionalStringModelMember(value: Option<alloc::string::String>, model: &crate::SystemModel) -> bool {
    __prod_borrowed_optionalStringModelMember(&value, model)
}

fn __prod_borrowed_optionalStringModelMember(value: &Option<alloc::string::String>, model: &crate::SystemModel) -> bool {
    match value {
        None => { let _x_14 = true; _x_14 },
        Some(val_9) => { let _x_15 = modelHasId((val_9).as_ref(), &(model)); _x_15 },
    }
}

pub fn persistenceCompatible(x_1: &[crate::Persistence]) -> bool {
    match x_1 {
        [] => { let _x_32 = true; _x_32 },
        [head_22, tail_23 @ ..] => { let _x_33 = &(head_22).compatibilityWindow; { let _x_34 = nonEmptyString((_x_33).as_ref()); match _x_34 {
        false => _x_34,
        true => { let _x_37 = persistenceCompatible(&(tail_23)); _x_37 },
    } } },
    }
}

pub fn persistenceReferential(x_1: &[crate::Persistence], x_2: &[crate::Component], x_3: &[crate::Artifact], x_4: &[crate::BackupRecovery], x_5: &[crate::Migration]) -> bool {
    match x_1 {
        [] => { let _x_73 = true; _x_73 },
        [head_55, tail_56 @ ..] => { let _x_92 = &(head_55).owner; { let _x_93 = memberIdComponents((_x_92).as_ref(), &(x_2)); match _x_93 {
        false => _x_93,
        true => { let _x_109 = &(head_55).schemaArtifact; { let _x_110 = memberIdArtifacts((_x_109).as_ref(), &(x_3)); match _x_110 {
        false => _x_110,
        true => { let _x_120 = &(head_55).backup; { let _x_121 = memberIdBackups((_x_120).as_ref(), &(x_4)); match _x_121 {
        false => _x_121,
        true => { let _x_125 = &(head_55).migrationOrder; { let _x_126 = allIdsMemberMigrations(&(_x_125), &(x_5)); match _x_126 {
        false => _x_126,
        true => { let _x_129 = persistenceReferential(&(tail_56), &(x_2), &(x_3), &(x_4), &(x_5)); _x_129 },
    } } },
    } } },
    } } },
    } } },
    }
}

pub fn platformCapabilitiesSatisfied(x_1: &[crate::PlatformRequirement], x_2: &[crate::Capability]) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).capabilities; { let _x_33 = allIdsMemberCapabilities(&(_x_32), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = platformCapabilitiesSatisfied(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn productRefsClosed(product: &crate::Product, model: &crate::SystemModel) -> bool {
    { let _x_1 = &(product).supportedPlatforms; { let _x_2 = allStringsModelMember(&(_x_1), &(model)); _x_2 } }
}

pub fn refsClosedAcceptance(x_1: &[crate::Acceptance], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_44 = true; _x_44 },
        [head_32, tail_33 @ ..] => { let _x_51 = &(head_32).target; { let _x_52 = modelHasId((_x_51).as_ref(), &(x_2)); match _x_52 {
        false => _x_52,
        true => { let _x_56 = &(head_32).component; { let _x_57 = modelHasId((_x_56).as_ref(), &(x_2)); match _x_57 {
        false => _x_57,
        true => { let _x_60 = refsClosedAcceptance(&(tail_33), &(x_2)); _x_60 },
    } } },
    } } },
    }
}

pub fn refsClosedAlerts(x_1: &[crate::Alert], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).dependsOn; { let _x_33 = allStringsModelMember(&(_x_32), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = refsClosedAlerts(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn refsClosedArchitecture(x_1: &[crate::ArchitectureBinding], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_44 = true; _x_44 },
        [head_32, tail_33 @ ..] => { let _x_51 = &(head_32).dependsOn; { let _x_52 = allStringsModelMember(&(_x_51), &(x_2)); match _x_52 {
        false => _x_52,
        true => { let _x_56 = &(head_32).verifies; { let _x_57 = allStringsModelMember(&(_x_56), &(x_2)); match _x_57 {
        false => _x_57,
        true => { let _x_60 = refsClosedArchitecture(&(tail_33), &(x_2)); _x_60 },
    } } },
    } } },
    }
}

pub fn refsClosedArtifacts(x_1: &[crate::Artifact], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).platformRequirements; { let _x_33 = allStringsModelMember(&(_x_32), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = refsClosedArtifacts(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn refsClosedBackups(x_1: &[crate::BackupRecovery], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).dependsOn; { let _x_33 = allStringsModelMember(&(_x_32), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = refsClosedBackups(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn refsClosedCalls(x_1: &[crate::Call], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_70 = true; _x_70 },
        [head_52, tail_53 @ ..] => { let _x_89 = &(head_52).fromComponent; { let _x_90 = modelHasId((_x_89).as_ref(), &(x_2)); match _x_90 {
        false => _x_90,
        true => { let _x_106 = &(head_52).toComponent; { let _x_107 = modelHasId((_x_106).as_ref(), &(x_2)); match _x_107 {
        false => _x_107,
        true => { let _x_117 = &(head_52).interfaceId; { let _x_118 = modelHasId((_x_117).as_ref(), &(x_2)); match _x_118 {
        false => _x_118,
        true => { let _x_122 = &(head_52).dependsOn; { let _x_123 = allStringsModelMember(&(_x_122), &(x_2)); match _x_123 {
        false => _x_123,
        true => { let _x_126 = refsClosedCalls(&(tail_53), &(x_2)); _x_126 },
    } } },
    } } },
    } } },
    } } },
    }
}

pub fn refsClosedCapabilities(x_1: &[crate::Capability], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).dependsOn; { let _x_33 = allStringsModelMember(&(_x_32), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = refsClosedCapabilities(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn refsClosedComponents(x_1: &[crate::Component], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_161 = true; _x_161 },
        [head_122, tail_123 @ ..] => { let _x_222 = &(head_122).artifact; { let _x_223 = modelHasId((_x_222).as_ref(), &(x_2)); match _x_223 {
        false => _x_223,
        true => { let _x_281 = &(head_122).capabilities; { let _x_282 = allStringsModelMember(&(_x_281), &(x_2)); match _x_282 {
        false => _x_282,
        true => { let _x_334 = &(head_122).dependsOn; { let _x_335 = allStringsModelMember(&(_x_334), &(x_2)); match _x_335 {
        false => _x_335,
        true => { let _x_381 = &(head_122).interfaces; { let _x_382 = allStringsModelMember(&(_x_381), &(x_2)); match _x_382 {
        false => _x_382,
        true => { let _x_422 = &(head_122).parameters; { let _x_423 = allStringsModelMember(&(_x_422), &(x_2)); match _x_423 {
        false => _x_423,
        true => { let _x_457 = &(head_122).ports; { let _x_458 = allStringsModelMember(&(_x_457), &(x_2)); match _x_458 {
        false => _x_458,
        true => { let _x_486 = &(head_122).secrets; { let _x_487 = allStringsModelMember(&(_x_486), &(x_2)); match _x_487 {
        false => _x_487,
        true => { let _x_509 = &(head_122).volumes; { let _x_510 = allStringsModelMember(&(_x_509), &(x_2)); match _x_510 {
        false => _x_510,
        true => { let _x_526 = &(head_122).placement; { let _x_527 = allStringsModelMember(&(_x_526), &(x_2)); match _x_527 {
        false => _x_527,
        true => { let _x_537 = &(head_122).platformRequirements; { let _x_538 = allStringsModelMember(&(_x_537), &(x_2)); match _x_538 {
        false => _x_538,
        true => { let _x_542 = &(head_122).scalingPolicy; { let _x_543 = __prod_borrowed_optionalStringModelMember(&(_x_542), &(x_2)); match _x_543 {
        false => _x_543,
        true => { let _x_546 = refsClosedComponents(&(tail_123), &(x_2)); _x_546 },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    }
}

pub fn refsClosedControls(x_1: &[crate::Control], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_44 = true; _x_44 },
        [head_32, tail_33 @ ..] => { let _x_51 = &(head_32).dependsOn; { let _x_52 = allStringsModelMember(&(_x_51), &(x_2)); match _x_52 {
        false => _x_52,
        true => { let _x_56 = &(head_32).verification; { let _x_57 = allStringsModelMember(&(_x_56), &(x_2)); match _x_57 {
        false => _x_57,
        true => { let _x_60 = refsClosedControls(&(tail_33), &(x_2)); _x_60 },
    } } },
    } } },
    }
}

pub fn refsClosedDrifts(x_1: &[crate::Drift], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).dependsOn; { let _x_33 = allStringsModelMember(&(_x_32), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = refsClosedDrifts(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn refsClosedEvents(x_1: &[crate::Event], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_83 = true; _x_83 },
        [head_62, tail_63 @ ..] => { let _x_108 = &(head_62).producer; { let _x_109 = modelHasId((_x_108).as_ref(), &(x_2)); match _x_109 {
        false => _x_109,
        true => { let _x_131 = &(head_62).owner; { let _x_132 = modelHasId((_x_131).as_ref(), &(x_2)); match _x_132 {
        false => _x_132,
        true => { let _x_148 = &(head_62).channel; { let _x_149 = modelHasId((_x_148).as_ref(), &(x_2)); match _x_149 {
        false => _x_149,
        true => { let _x_159 = &(head_62).schemaId; { let _x_160 = modelHasId((_x_159).as_ref(), &(x_2)); match _x_160 {
        false => _x_160,
        true => { let _x_164 = &(head_62).dependsOn; { let _x_165 = allStringsModelMember(&(_x_164), &(x_2)); match _x_165 {
        false => _x_165,
        true => { let _x_168 = refsClosedEvents(&(tail_63), &(x_2)); _x_168 },
    } } },
    } } },
    } } },
    } } },
    } } },
    }
}

pub fn refsClosedFlows(x_1: &[crate::Flow], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_57 = true; _x_57 },
        [head_42, tail_43 @ ..] => { let _x_70 = &(head_42).fromComponent; { let _x_71 = modelHasId((_x_70).as_ref(), &(x_2)); match _x_71 {
        false => _x_71,
        true => { let _x_81 = &(head_42).toComponent; { let _x_82 = modelHasId((_x_81).as_ref(), &(x_2)); match _x_82 {
        false => _x_82,
        true => { let _x_86 = &(head_42).interfaceId; { let _x_87 = modelHasId((_x_86).as_ref(), &(x_2)); match _x_87 {
        false => _x_87,
        true => { let _x_90 = refsClosedFlows(&(tail_43), &(x_2)); _x_90 },
    } } },
    } } },
    } } },
    }
}

pub fn refsClosedIdentityRequirements(x_1: &[crate::IdentityRequirement], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).issuerParameter; { let _x_33 = modelHasId((_x_32).as_ref(), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = refsClosedIdentityRequirements(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn refsClosedInterfaces(x_1: &[crate::Interface], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_44 = true; _x_44 },
        [head_32, tail_33 @ ..] => { let _x_51 = &(head_32).document; { let _x_52 = modelHasId((_x_51).as_ref(), &(x_2)); match _x_52 {
        false => _x_52,
        true => { let _x_56 = &(head_32).acceptance; { let _x_57 = allStringsModelMember(&(_x_56), &(x_2)); match _x_57 {
        false => _x_57,
        true => { let _x_60 = refsClosedInterfaces(&(tail_33), &(x_2)); _x_60 },
    } } },
    } } },
    }
}

pub fn refsClosedMigrations(x_1: &[crate::Migration], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).dependsOn; { let _x_33 = allStringsModelMember(&(_x_32), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = refsClosedMigrations(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn refsClosedParameters(x_1: &[crate::Configuration], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_16 = true; _x_16 },
        [head_11, tail_12 @ ..] => { let _x_17 = refsClosedParameters(&(tail_12), &(x_2)); _x_17 },
    }
}

pub fn refsClosedPersistence(x_1: &[crate::Persistence], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_70 = true; _x_70 },
        [head_52, tail_53 @ ..] => { let _x_89 = &(head_52).owner; { let _x_90 = modelHasId((_x_89).as_ref(), &(x_2)); match _x_90 {
        false => _x_90,
        true => { let _x_106 = &(head_52).schemaArtifact; { let _x_107 = modelHasId((_x_106).as_ref(), &(x_2)); match _x_107 {
        false => _x_107,
        true => { let _x_117 = &(head_52).backup; { let _x_118 = modelHasId((_x_117).as_ref(), &(x_2)); match _x_118 {
        false => _x_118,
        true => { let _x_122 = &(head_52).migrationOrder; { let _x_123 = allStringsModelMember(&(_x_122), &(x_2)); match _x_123 {
        false => _x_123,
        true => { let _x_126 = refsClosedPersistence(&(tail_53), &(x_2)); _x_126 },
    } } },
    } } },
    } } },
    } } },
    }
}

pub fn refsClosedPlatformRequirements(x_1: &[crate::PlatformRequirement], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).capabilities; { let _x_33 = allStringsModelMember(&(_x_32), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = refsClosedPlatformRequirements(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn refsClosedRetirements(x_1: &[crate::Retirement], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).dependsOn; { let _x_33 = allStringsModelMember(&(_x_32), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = refsClosedRetirements(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn refsClosedRollbacks(x_1: &[crate::Rollback], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).dependsOn; { let _x_33 = allStringsModelMember(&(_x_32), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = refsClosedRollbacks(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn refsClosedRollouts(x_1: &[crate::Rollout], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).dependsOn; { let _x_33 = allStringsModelMember(&(_x_32), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = refsClosedRollouts(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn refsClosedScalingPolicies(x_1: &[crate::ScalingPolicy], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).component; { let _x_33 = modelHasId((_x_32).as_ref(), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = refsClosedScalingPolicies(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn refsClosedSchemas(x_1: &[crate::Schema], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).dependsOn; { let _x_33 = allStringsModelMember(&(_x_32), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = refsClosedSchemas(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn refsClosedSecretReferences(x_1: &[crate::SecretReference], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).consumers; { let _x_33 = allStringsModelMember(&(_x_32), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = refsClosedSecretReferences(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn refsClosedSlis(x_1: &[crate::Sli], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).dependsOn; { let _x_33 = allStringsModelMember(&(_x_32), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = refsClosedSlis(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn refsClosedSlos(x_1: &[crate::Slo], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).dependsOn; { let _x_33 = allStringsModelMember(&(_x_32), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = refsClosedSlos(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn refsClosedStorageClasses(x_1: &[crate::StorageClass], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_44 = true; _x_44 },
        [head_32, tail_33 @ ..] => { let _x_51 = &(head_32).capabilities; { let _x_52 = allStringsModelMember(&(_x_51), &(x_2)); match _x_52 {
        false => _x_52,
        true => { let _x_56 = &(head_32).platformRequirements; { let _x_57 = allStringsModelMember(&(_x_56), &(x_2)); match _x_57 {
        false => _x_57,
        true => { let _x_60 = refsClosedStorageClasses(&(tail_33), &(x_2)); _x_60 },
    } } },
    } } },
    }
}

pub fn refsClosedTargets(x_1: &[crate::TargetBinding], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_83 = true; _x_83 },
        [head_62, tail_63 @ ..] => { let _x_108 = &(head_62).capabilities; { let _x_109 = allStringsModelMember(&(_x_108), &(x_2)); match _x_109 {
        false => _x_109,
        true => { let _x_131 = &(head_62).platformRequirements; { let _x_132 = allStringsModelMember(&(_x_131), &(x_2)); match _x_132 {
        false => _x_132,
        true => { let _x_148 = &(head_62).credentials; { let _x_149 = __prod_borrowed_optionalStringModelMember(&(_x_148), &(x_2)); match _x_149 {
        false => _x_149,
        true => { let _x_159 = &(head_62).storageClass; { let _x_160 = __prod_borrowed_optionalStringModelMember(&(_x_159), &(x_2)); match _x_160 {
        false => _x_160,
        true => { let _x_164 = &(head_62).ingressControllerArtifact; { let _x_165 = __prod_borrowed_optionalStringModelMember(&(_x_164), &(x_2)); match _x_165 {
        false => _x_165,
        true => { let _x_168 = refsClosedTargets(&(tail_63), &(x_2)); _x_168 },
    } } },
    } } },
    } } },
    } } },
    } } },
    }
}

pub fn refsClosedTopology(x_1: &[crate::Topology], x_2: &crate::SystemModel) -> bool {
    match x_1 {
        [] => { let _x_109 = true; _x_109 },
        [head_82, tail_83 @ ..] => { let _x_146 = &(head_82).owners; { let _x_147 = allStringsModelMember(&(_x_146), &(x_2)); match _x_147 {
        false => _x_147,
        true => { let _x_181 = &(head_82).dependsOn; { let _x_182 = allStringsModelMember(&(_x_181), &(x_2)); match _x_182 {
        false => _x_182,
        true => { let _x_210 = &(head_82).capabilities; { let _x_211 = allStringsModelMember(&(_x_210), &(x_2)); match _x_211 {
        false => _x_211,
        true => { let _x_233 = &(head_82).platformRequirements; { let _x_234 = allStringsModelMember(&(_x_233), &(x_2)); match _x_234 {
        false => _x_234,
        true => { let _x_250 = &(head_82).network; { let _x_251 = __prod_borrowed_optionalStringModelMember(&(_x_250), &(x_2)); match _x_251 {
        false => _x_251,
        true => { let _x_261 = &(head_82).storageClass; { let _x_262 = __prod_borrowed_optionalStringModelMember(&(_x_261), &(x_2)); match _x_262 {
        false => _x_262,
        true => { let _x_266 = &(head_82).placement; { let _x_267 = __prod_borrowed_optionalStringModelMember(&(_x_266), &(x_2)); match _x_267 {
        false => _x_267,
        true => { let _x_270 = refsClosedTopology(&(tail_83), &(x_2)); _x_270 },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    }
}

pub fn rollbackRowsSafe(x_1: &[crate::Rollback], x_2: &[crate::Migration], x_3: &[crate::Rollout]) -> bool {
    match x_1 {
        [] => { let _x_71 = true; _x_71 },
        [head_53, tail_54 @ ..] => { let _x_90 = &(head_53).kind; { let _x_91 = nonEmptyString((_x_90).as_ref()); match _x_91 {
        false => _x_91,
        true => { let _x_107 = &(head_53).value; { let _x_108 = nonEmptyString((_x_107).as_ref()); match _x_108 {
        false => _x_108,
        true => { let _x_118 = &(head_53).dependsOn; { let _x_119 = anyIdsMemberMigrations(&(_x_118), &(x_2)); match _x_119 {
        false => _x_119,
        true => { let _x_123 = &(head_53).dependsOn; { let _x_124 = anyIdsMemberRollouts(&(_x_123), &(x_3)); match _x_124 {
        false => _x_124,
        true => { let _x_127 = rollbackRowsSafe(&(tail_54), &(x_2), &(x_3)); _x_127 },
    } } },
    } } },
    } } },
    } } },
    }
}

pub fn scalingReferential(x_1: &[crate::ScalingPolicy], x_2: &[crate::Component]) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).component; { let _x_33 = memberIdComponents((_x_32).as_ref(), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = scalingReferential(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn schemasCompatible(x_1: &[crate::Schema]) -> bool {
    match x_1 {
        [] => { let _x_32 = true; _x_32 },
        [head_22, tail_23 @ ..] => { let _x_33 = &(head_22).compatibility; { let _x_34 = compatibilityValue((_x_33).as_ref()); match _x_34 {
        false => _x_34,
        true => { let _x_37 = schemasCompatible(&(tail_23)); _x_37 },
    } } },
    }
}

pub fn secretConsumersSatisfied(x_1: &[crate::SecretReference], x_2: &[crate::Component]) -> bool {
    match x_1 {
        [] => { let _x_57 = true; _x_57 },
        [head_42, tail_43 @ ..] => { let _x_70 = &(head_42).consumers; { let _x_71 = allIdsMemberComponents(&(_x_70), &(x_2)); match _x_71 {
        false => _x_71,
        true => { let _x_81 = &(head_42).providerKey; { let _x_82 = nonEmptyString((_x_81).as_ref()); match _x_82 {
        false => _x_82,
        true => { let _x_86 = &(head_42).rotation; { let _x_87 = nonEmptyString((_x_86).as_ref()); match _x_87 {
        false => _x_87,
        true => { let _x_90 = secretConsumersSatisfied(&(tail_43), &(x_2)); _x_90 },
    } } },
    } } },
    } } },
    }
}

pub fn storageCapabilitiesSatisfied(x_1: &[crate::StorageClass], x_2: &[crate::Capability]) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).capabilities; { let _x_33 = allIdsMemberCapabilities(&(_x_32), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = storageCapabilitiesSatisfied(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn storageReferential(x_1: &[crate::StorageClass], x_2: &[crate::Capability], x_3: &[crate::PlatformRequirement]) -> bool {
    match x_1 {
        [] => { let _x_45 = true; _x_45 },
        [head_33, tail_34 @ ..] => { let _x_52 = &(head_33).capabilities; { let _x_53 = allIdsMemberCapabilities(&(_x_52), &(x_2)); match _x_53 {
        false => _x_53,
        true => { let _x_57 = &(head_33).platformRequirements; { let _x_58 = allIdsMemberPlatformRequirements(&(_x_57), &(x_3)); match _x_58 {
        false => _x_58,
        true => { let _x_61 = storageReferential(&(tail_34), &(x_2), &(x_3)); _x_61 },
    } } },
    } } },
    }
}

pub fn targetCapabilitiesSatisfied(x_1: &[crate::TargetBinding], x_2: &[crate::Capability]) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).capabilities; { let _x_33 = allIdsMemberCapabilities(&(_x_32), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = targetCapabilitiesSatisfied(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn targetsReferential(x_1: &[crate::TargetBinding], x_2: &[crate::Capability], x_3: &[crate::PlatformRequirement], x_4: &[crate::StorageClass], x_5: &[crate::SecretReference], x_6: &[crate::Artifact]) -> bool {
    match x_1 {
        [] => { let _x_87 = true; _x_87 },
        [head_66, tail_67 @ ..] => { let _x_112 = &(head_66).capabilities; { let _x_113 = allIdsMemberCapabilities(&(_x_112), &(x_2)); match _x_113 {
        false => _x_113,
        true => { let _x_135 = &(head_66).platformRequirements; { let _x_136 = allIdsMemberPlatformRequirements(&(_x_135), &(x_3)); match _x_136 {
        false => _x_136,
        true => { let _x_152 = &(head_66).storageClass; { let _x_153 = __prod_borrowed_optionalIdMemberStorageClasses(&(_x_152), &(x_4)); match _x_153 {
        false => _x_153,
        true => { let _x_163 = &(head_66).credentials; { let _x_164 = __prod_borrowed_optionalIdMemberSecretReferences(&(_x_163), &(x_5)); match _x_164 {
        false => _x_164,
        true => { let _x_168 = &(head_66).ingressControllerArtifact; { let _x_169 = __prod_borrowed_optionalIdMemberArtifacts(&(_x_168), &(x_6)); match _x_169 {
        false => _x_169,
        true => { let _x_172 = targetsReferential(&(tail_67), &(x_2), &(x_3), &(x_4), &(x_5), &(x_6)); _x_172 },
    } } },
    } } },
    } } },
    } } },
    } } },
    }
}

pub fn topologyCapabilitiesSatisfied(x_1: &[crate::Topology], x_2: &[crate::Capability]) -> bool {
    match x_1 {
        [] => { let _x_31 = true; _x_31 },
        [head_22, tail_23 @ ..] => { let _x_32 = &(head_22).capabilities; { let _x_33 = allIdsMemberCapabilities(&(_x_32), &(x_2)); match _x_33 {
        false => _x_33,
        true => { let _x_36 = topologyCapabilitiesSatisfied(&(tail_23), &(x_2)); _x_36 },
    } } },
    }
}

pub fn topologyReferential(x_1: &[crate::Topology], x_2: &[crate::Component], x_3: &[crate::Capability], x_4: &[crate::PlatformRequirement], x_5: &[crate::StorageClass], x_6: &[crate::Topology]) -> bool {
    match x_1 {
        [] => { let _x_100 = true; _x_100 },
        [head_76, tail_77 @ ..] => { let _x_131 = &(head_76).owners; { let _x_132 = allIdsMemberComponents(&(_x_131), &(x_2)); match _x_132 {
        false => _x_132,
        true => { let _x_160 = &(head_76).capabilities; { let _x_161 = allIdsMemberCapabilities(&(_x_160), &(x_3)); match _x_161 {
        false => _x_161,
        true => { let _x_183 = &(head_76).platformRequirements; { let _x_184 = allIdsMemberPlatformRequirements(&(_x_183), &(x_4)); match _x_184 {
        false => _x_184,
        true => { let _x_200 = &(head_76).storageClass; { let _x_201 = __prod_borrowed_optionalIdMemberStorageClasses(&(_x_200), &(x_5)); match _x_201 {
        false => _x_201,
        true => { let _x_211 = &(head_76).network; { let _x_212 = __prod_borrowed_optionalIdMemberTopology(&(_x_211), &(x_6)); match _x_212 {
        false => _x_212,
        true => { let _x_216 = &(head_76).placement; { let _x_217 = __prod_borrowed_optionalIdMemberTopology(&(_x_216), &(x_6)); match _x_217 {
        false => _x_217,
        true => { let _x_220 = topologyReferential(&(tail_77), &(x_2), &(x_3), &(x_4), &(x_5), &(x_6)); _x_220 },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    }
}

pub fn uniqueNats(x_1: &[u64]) -> bool {
    match x_1 {
        [] => { let _x_36 = true; _x_36 },
        [head_22, tail_23 @ ..] => { let head_22 = head_22.clone(); { let _x_43 = natMember(head_22, &(tail_23)); match _x_43 {
        false => { let _x_55 = uniqueNats(&(tail_23)); _x_55 },
        true => { let _x_53 = false; _x_53 },
    } } },
    }
}

pub fn validateManifest(model: &crate::SystemModel, manifest: &crate::SystemManifest) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_97 = validateModelClosure(&(model), &(manifest))?; match _x_97 {
        false => _x_97,
        true => { let _x_183 = validateModelUniqueness(&(model), &(manifest))?; match _x_183 {
        false => _x_183,
        true => { let _x_227 = validateModelReferentialIntegrity(&(model), &(manifest))?; match _x_227 {
        false => _x_227,
        true => { let _x_266 = validateModelCompatibility(&(model), &(manifest)); match _x_266 {
        false => _x_266,
        true => { let _x_300 = validateModelCapabilitySatisfaction(&(model), &(manifest)); match _x_300 {
        false => _x_300,
        true => { let _x_329 = validateModelSecretFlow(&(model), &(manifest)); match _x_329 {
        false => _x_329,
        true => { let _x_353 = validateModelDeploymentOrder(&(model), &(manifest))?; match _x_353 {
        false => _x_353,
        true => { let _x_372 = validateModelMigrationOrder(&(model), &(manifest))?; match _x_372 {
        false => _x_372,
        true => { let _x_386 = validateModelRollbackSafety(&(model), &(manifest))?; match _x_386 {
        false => _x_386,
        true => { let _x_395 = validateModelEvidenceClosure(&(model), &(manifest))?; match _x_395 {
        false => _x_395,
        true => { let _x_399 = validateModelLicenseClosure(&(model), &(manifest)); match _x_399 {
        false => _x_399,
        true => { let _x_402 = validateModelReleaseCompleteness(&(model), &(manifest))?; _x_402 },
    } },
    } },
    } },
    } },
    } },
    } },
    } },
    } },
    } },
    } },
    } })
}

pub fn validateModelCapabilitySatisfaction(model: &crate::SystemModel, manifest: &crate::SystemManifest) -> bool {
    { let _x_68 = &(model).components; { let _x_69 = &(model).capabilities; { let _x_70 = componentCapabilitiesSatisfied(&(_x_68), &(_x_69)); match _x_70 {
        false => _x_70,
        true => { let _x_131 = &(model).platformRequirements; { let _x_132 = &(model).capabilities; { let _x_133 = platformCapabilitiesSatisfied(&(_x_131), &(_x_132)); match _x_133 {
        false => _x_133,
        true => { let _x_166 = &(model).storageClasses; { let _x_167 = &(model).capabilities; { let _x_168 = storageCapabilitiesSatisfied(&(_x_166), &(_x_167)); match _x_168 {
        false => _x_168,
        true => { let _x_194 = &(model).targets; { let _x_195 = &(model).capabilities; { let _x_196 = targetCapabilitiesSatisfied(&(_x_194), &(_x_195)); match _x_196 {
        false => _x_196,
        true => { let _x_215 = &(model).topology; { let _x_216 = &(model).capabilities; { let _x_217 = topologyCapabilitiesSatisfied(&(_x_215), &(_x_216)); match _x_217 {
        false => _x_217,
        true => { let _x_226 = &(manifest).capabilitySatisfaction; { let _x_227 = (_x_226).bound; { let _x_229 = &(model).capabilities; { let _x_230 = (_x_229).len() as u64; { let _x_231 = (_x_227 == _x_230); match _x_231 {
        false => _x_231,
        true => { let _x_235 = &(manifest).capabilitySatisfaction; { let _x_236 = (_x_235).bound; { let _x_237 = &(_x_235).values; { let _x_238 = validateCapabilitySatisfaction(_x_236, &(_x_237)); _x_238 } } } },
    } } } } } },
    } } } },
    } } } },
    } } } },
    } } } },
    } } } }
}

pub fn validateModelClosure(model: &crate::SystemModel, manifest: &crate::SystemManifest) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_321 = &(model).secretReferences; { let _x_322 = refsClosedSecretReferences(&(_x_321), &(model)); match _x_322 {
        false => _x_322,
        true => { let _x_612 = modelIdsGloballyUnique(&(model))?; match _x_612 {
        false => _x_612,
        true => { let _x_797 = &(model).product; { let _x_798 = productRefsClosed(&(_x_797), &(model)); match _x_798 {
        false => _x_798,
        true => { let _x_977 = &(model).acceptance; { let _x_978 = refsClosedAcceptance(&(_x_977), &(model)); match _x_978 {
        false => _x_978,
        true => { let _x_1151 = &(model).alerts; { let _x_1152 = refsClosedAlerts(&(_x_1151), &(model)); match _x_1152 {
        false => _x_1152,
        true => { let _x_1319 = &(model).architecture; { let _x_1320 = refsClosedArchitecture(&(_x_1319), &(model)); match _x_1320 {
        false => _x_1320,
        true => { let _x_1481 = &(model).artifacts; { let _x_1482 = refsClosedArtifacts(&(_x_1481), &(model)); match _x_1482 {
        false => _x_1482,
        true => { let _x_1637 = &(model).backups; { let _x_1638 = refsClosedBackups(&(_x_1637), &(model)); match _x_1638 {
        false => _x_1638,
        true => { let _x_1787 = &(model).calls; { let _x_1788 = refsClosedCalls(&(_x_1787), &(model)); match _x_1788 {
        false => _x_1788,
        true => { let _x_1931 = &(model).capabilities; { let _x_1932 = refsClosedCapabilities(&(_x_1931), &(model)); match _x_1932 {
        false => _x_1932,
        true => { let _x_2069 = &(model).components; { let _x_2070 = refsClosedComponents(&(_x_2069), &(model)); match _x_2070 {
        false => _x_2070,
        true => { let _x_2201 = &(model).controls; { let _x_2202 = refsClosedControls(&(_x_2201), &(model)); match _x_2202 {
        false => _x_2202,
        true => { let _x_2327 = &(model).drifts; { let _x_2328 = refsClosedDrifts(&(_x_2327), &(model)); match _x_2328 {
        false => _x_2328,
        true => { let _x_2447 = &(model).events; { let _x_2448 = refsClosedEvents(&(_x_2447), &(model)); match _x_2448 {
        false => _x_2448,
        true => { let _x_2561 = &(model).flows; { let _x_2562 = refsClosedFlows(&(_x_2561), &(model)); match _x_2562 {
        false => _x_2562,
        true => { let _x_2669 = &(model).identityRequirements; { let _x_2670 = refsClosedIdentityRequirements(&(_x_2669), &(model)); match _x_2670 {
        false => _x_2670,
        true => { let _x_2771 = &(model).interfaces; { let _x_2772 = refsClosedInterfaces(&(_x_2771), &(model)); match _x_2772 {
        false => _x_2772,
        true => { let _x_2867 = &(model).migrations; { let _x_2868 = refsClosedMigrations(&(_x_2867), &(model)); match _x_2868 {
        false => _x_2868,
        true => { let _x_2957 = &(model).parameters; { let _x_2958 = refsClosedParameters(&(_x_2957), &(model)); match _x_2958 {
        false => _x_2958,
        true => { let _x_3041 = &(model).platformRequirements; { let _x_3042 = refsClosedPlatformRequirements(&(_x_3041), &(model)); match _x_3042 {
        false => _x_3042,
        true => { let _x_3119 = &(model).persistence; { let _x_3120 = refsClosedPersistence(&(_x_3119), &(model)); match _x_3120 {
        false => _x_3120,
        true => { let _x_3191 = &(model).retirements; { let _x_3192 = refsClosedRetirements(&(_x_3191), &(model)); match _x_3192 {
        false => _x_3192,
        true => { let _x_3257 = &(model).rollbacks; { let _x_3258 = refsClosedRollbacks(&(_x_3257), &(model)); match _x_3258 {
        false => _x_3258,
        true => { let _x_3317 = &(model).rollouts; { let _x_3318 = refsClosedRollouts(&(_x_3317), &(model)); match _x_3318 {
        false => _x_3318,
        true => { let _x_3371 = &(model).scalingPolicies; { let _x_3372 = refsClosedScalingPolicies(&(_x_3371), &(model)); match _x_3372 {
        false => _x_3372,
        true => { let _x_3419 = &(model).schemas; { let _x_3420 = refsClosedSchemas(&(_x_3419), &(model)); match _x_3420 {
        false => _x_3420,
        true => { let _x_3461 = &(model).slis; { let _x_3462 = refsClosedSlis(&(_x_3461), &(model)); match _x_3462 {
        false => _x_3462,
        true => { let _x_3497 = &(model).slos; { let _x_3498 = refsClosedSlos(&(_x_3497), &(model)); match _x_3498 {
        false => _x_3498,
        true => { let _x_3527 = &(model).targets; { let _x_3528 = refsClosedTargets(&(_x_3527), &(model)); match _x_3528 {
        false => _x_3528,
        true => { let _x_3551 = &(model).topology; { let _x_3552 = refsClosedTopology(&(_x_3551), &(model)); match _x_3552 {
        false => _x_3552,
        true => { let _x_3569 = &(model).storageClasses; { let _x_3570 = refsClosedStorageClasses(&(_x_3569), &(model)); match _x_3570 {
        false => _x_3570,
        true => { let _x_3579 = &(manifest).closure; { let _x_3580 = (_x_3579).bound; { let _x_3581 = modelEntityCount(&(model))?; { let _x_3582 = (_x_3580 == _x_3581); match _x_3582 {
        false => _x_3582,
        true => { let _x_3586 = &(manifest).closure; { let _x_3587 = (_x_3586).bound; { let _x_3588 = &(_x_3586).values; { let _x_3589 = validateClosure(_x_3587, &(_x_3588)); _x_3589 } } } },
    } } } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } } },
    } },
    } } })
}

pub fn validateModelCompatibility(model: &crate::SystemModel, manifest: &crate::SystemManifest) -> bool {
    { let _x_60 = &(model).schemas; { let _x_61 = schemasCompatible(&(_x_60)); match _x_61 {
        false => _x_61,
        true => { let _x_115 = &(model).interfaces; { let _x_116 = interfacesCompatible(&(_x_115)); match _x_116 {
        false => _x_116,
        true => { let _x_145 = &(model).persistence; { let _x_146 = persistenceCompatible(&(_x_145)); match _x_146 {
        false => _x_146,
        true => { let _x_167 = &(manifest).compatibility; { let _x_168 = (_x_167).bound; { let _x_169 = 4; { let _x_170 = (_x_168 == _x_169); match _x_170 {
        false => _x_170,
        true => { let _x_180 = &(manifest).compatibility; { let _x_181 = &(_x_180).values; { let _x_182 = (_x_181).len() as u64; { let _x_184 = &(model).interfaces; { let _x_185 = (_x_184).len() as u64; { let _x_186 = (_x_182 == _x_185); match _x_186 {
        false => _x_186,
        true => { let _x_190 = &(manifest).compatibility; { let _x_191 = (_x_190).bound; { let _x_192 = &(_x_190).values; { let _x_193 = validateCompatibility(_x_191, &(_x_192)); _x_193 } } } },
    } } } } } } },
    } } } } },
    } } },
    } } },
    } } }
}

pub fn validateModelDeploymentOrder(model: &crate::SystemModel, manifest: &crate::SystemManifest) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_51 = &(manifest).deploymentOrder; { let _x_52 = (_x_51).bound; { let _x_54 = &(model).components; { let _x_55 = (_x_54).len() as u64; { let _x_56 = (_x_52 == _x_55); match _x_56 {
        false => _x_56,
        true => { let _x_98 = &(manifest).deploymentOrder; { let _x_99 = &(_x_98).values; { let _x_100 = (_x_99).len() as u64; { let _x_102 = &(model).components; { let _x_103 = (_x_102).len() as u64; { let _x_104 = (_x_100 == _x_103); match _x_104 {
        false => _x_104,
        true => { let _x_123 = &(manifest).deploymentOrder; { let _x_124 = &(_x_123).values; { let _x_125 = uniqueNats(&(_x_124)); match _x_125 {
        false => _x_125,
        true => { let _x_135 = &(model).components; { let _x_136 = (_x_135).len() as u64; { let _x_137 = &(manifest).deploymentOrder; { let _x_138 = &(_x_137).values; { let _x_139 = relationAllBelow(_x_136, &(_x_138)); match _x_139 {
        false => _x_139,
        true => { let _x_143 = &(model).components; { let _x_144 = &(manifest).deploymentOrder; { let _x_145 = &(_x_144).values; { let _x_146 = directComponentOrderValid(&(_x_143), &(_x_143), &(_x_145))?; _x_146 } } } },
    } } } } } },
    } } } },
    } } } } } } },
    } } } } } })
}

pub fn validateModelEvidenceClosure(model: &crate::SystemModel, manifest: &crate::SystemManifest) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_22 = &(model).acceptance; { let _x_23 = &(model).targets; { let _x_24 = &(model).components; { let _x_25 = acceptanceEvidenceClosed(&(_x_22), &(_x_23), &(_x_24)); match _x_25 {
        false => _x_25,
        true => { let _x_43 = &(manifest).evidenceClosure; { let _x_44 = (_x_43).bound; { let _x_45 = modelEntityCount(&(model))?; { let _x_46 = (_x_44 == _x_45); match _x_46 {
        false => _x_46,
        true => { let _x_50 = &(manifest).evidenceClosure; { let _x_51 = (_x_50).bound; { let _x_52 = &(_x_50).values; { let _x_53 = validateEvidenceClosure(_x_51, &(_x_52)); _x_53 } } } },
    } } } } },
    } } } } })
}

pub fn validateModelLicenseClosure(model: &crate::SystemModel, manifest: &crate::SystemManifest) -> bool {
    { let _x_49 = 0; { let _x_53 = &(model).artifacts; { let _x_54 = (_x_53).len() as u64; { let _x_55 = (_x_49 < _x_54); match _x_55 {
        false => _x_55,
        true => { let _x_100 = &(model).artifacts; { let _x_101 = artifactLicensesClosed(&(_x_100)); match _x_101 {
        false => _x_101,
        true => { let _x_121 = &(manifest).licenseClosure; { let _x_122 = (_x_121).bound; { let _x_123 = 257; { let _x_124 = (_x_122 == _x_123); match _x_124 {
        false => _x_124,
        true => { let _x_133 = &(manifest).licenseClosure; { let _x_134 = &(_x_133).values; { let _x_135 = (_x_134).len() as u64; { let _x_137 = &(model).artifacts; { let _x_138 = (_x_137).len() as u64; { let _x_139 = (_x_135 == _x_138); match _x_139 {
        false => _x_139,
        true => { let _x_143 = &(manifest).licenseClosure; { let _x_144 = &(_x_143).values; { let _x_145 = validateLicenseClosure(&(_x_144)); _x_145 } } },
    } } } } } } },
    } } } } },
    } } },
    } } } } }
}

pub fn validateModelMigrationOrder(model: &crate::SystemModel, manifest: &crate::SystemManifest) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_51 = &(manifest).migrationOrder; { let _x_52 = (_x_51).bound; { let _x_54 = &(model).migrations; { let _x_55 = (_x_54).len() as u64; { let _x_56 = (_x_52 == _x_55); match _x_56 {
        false => _x_56,
        true => { let _x_98 = &(manifest).migrationOrder; { let _x_99 = &(_x_98).values; { let _x_100 = (_x_99).len() as u64; { let _x_102 = &(model).migrations; { let _x_103 = (_x_102).len() as u64; { let _x_104 = (_x_100 == _x_103); match _x_104 {
        false => _x_104,
        true => { let _x_123 = &(manifest).migrationOrder; { let _x_124 = &(_x_123).values; { let _x_125 = uniqueNats(&(_x_124)); match _x_125 {
        false => _x_125,
        true => { let _x_135 = &(model).migrations; { let _x_136 = (_x_135).len() as u64; { let _x_137 = &(manifest).migrationOrder; { let _x_138 = &(_x_137).values; { let _x_139 = relationAllBelow(_x_136, &(_x_138)); match _x_139 {
        false => _x_139,
        true => { let _x_143 = &(model).migrations; { let _x_144 = &(manifest).migrationOrder; { let _x_145 = &(_x_144).values; { let _x_146 = directMigrationOrderValid(&(_x_143), &(_x_143), &(_x_145))?; _x_146 } } } },
    } } } } } },
    } } } },
    } } } } } } },
    } } } } } })
}

pub fn validateModelReferentialIntegrity(model: &crate::SystemModel, manifest: &crate::SystemManifest) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_189 = &(model).product; { let _x_190 = &(_x_189).supportedPlatforms; { let _x_191 = &(model).platformRequirements; { let _x_192 = allIdsMemberPlatformRequirements(&(_x_190), &(_x_191)); match _x_192 {
        false => _x_192,
        true => { let _x_365 = &(model).artifacts; { let _x_366 = &(model).platformRequirements; { let _x_367 = artifactsReferential(&(_x_365), &(_x_366)); match _x_367 {
        false => _x_367,
        true => { let _x_478 = &(model).components; { let _x_479 = &(model).artifacts; { let _x_480 = &(model).capabilities; { let _x_481 = &(model).interfaces; { let _x_482 = &(model).parameters; { let _x_483 = &(model).topology; { let _x_484 = &(model).secretReferences; { let _x_485 = &(model).platformRequirements; { let _x_486 = &(model).scalingPolicies; { let _x_487 = componentsReferential(&(_x_478), &(_x_479), &(_x_480), &(_x_478), &(_x_481), &(_x_482), &(_x_483), &(_x_484), &(_x_485), &(_x_486)); match _x_487 {
        false => _x_487,
        true => { let _x_590 = &(model).interfaces; { let _x_591 = &(model).schemas; { let _x_592 = &(model).acceptance; { let _x_593 = interfacesReferential(&(_x_590), &(_x_591), &(_x_592)); match _x_593 {
        false => _x_593,
        true => { let _x_688 = &(model).calls; { let _x_689 = &(model).components; { let _x_690 = &(model).interfaces; { let _x_691 = callsReferential(&(_x_688), &(_x_689), &(_x_690)); match _x_691 {
        false => _x_691,
        true => { let _x_777 = &(model).events; { let _x_778 = &(model).components; { let _x_779 = &(model).interfaces; { let _x_780 = &(model).schemas; { let _x_781 = eventsReferential(&(_x_777), &(_x_778), &(_x_779), &(_x_780)); match _x_781 {
        false => _x_781,
        true => { let _x_859 = &(model).flows; { let _x_860 = &(model).components; { let _x_861 = &(model).interfaces; { let _x_862 = flowsReferential(&(_x_859), &(_x_860), &(_x_861)); match _x_862 {
        false => _x_862,
        true => { let _x_933 = &(model).identityRequirements; { let _x_934 = &(model).parameters; { let _x_935 = identityReferential(&(_x_933), &(_x_934)); match _x_935 {
        false => _x_935,
        true => { let _x_996 = &(model).persistence; { let _x_997 = &(model).components; { let _x_998 = &(model).artifacts; { let _x_999 = &(model).backups; { let _x_1000 = &(model).migrations; { let _x_1001 = persistenceReferential(&(_x_996), &(_x_997), &(_x_998), &(_x_999), &(_x_1000)); match _x_1001 {
        false => _x_1001,
        true => { let _x_1055 = &(model).scalingPolicies; { let _x_1056 = &(model).components; { let _x_1057 = scalingReferential(&(_x_1055), &(_x_1056)); match _x_1057 {
        false => _x_1057,
        true => { let _x_1101 = &(model).topology; { let _x_1102 = &(model).components; { let _x_1103 = &(model).capabilities; { let _x_1104 = &(model).platformRequirements; { let _x_1105 = &(model).storageClasses; { let _x_1106 = topologyReferential(&(_x_1101), &(_x_1102), &(_x_1103), &(_x_1104), &(_x_1105), &(_x_1101)); match _x_1106 {
        false => _x_1106,
        true => { let _x_1142 = &(model).storageClasses; { let _x_1143 = &(model).capabilities; { let _x_1144 = &(model).platformRequirements; { let _x_1145 = storageReferential(&(_x_1142), &(_x_1143), &(_x_1144)); match _x_1145 {
        false => _x_1145,
        true => { let _x_1170 = &(model).targets; { let _x_1171 = &(model).capabilities; { let _x_1172 = &(model).platformRequirements; { let _x_1173 = &(model).storageClasses; { let _x_1174 = &(model).secretReferences; { let _x_1175 = &(model).artifacts; { let _x_1176 = targetsReferential(&(_x_1170), &(_x_1171), &(_x_1172), &(_x_1173), &(_x_1174), &(_x_1175)); match _x_1176 {
        false => _x_1176,
        true => { let _x_1193 = &(model).acceptance; { let _x_1194 = &(model).targets; { let _x_1195 = &(model).components; { let _x_1196 = acceptanceReferential(&(_x_1193), &(_x_1194), &(_x_1195)); match _x_1196 {
        false => _x_1196,
        true => { let _x_1205 = &(manifest).referentialIntegrity; { let _x_1206 = (_x_1205).bound; { let _x_1207 = modelEntityCount(&(model))?; { let _x_1208 = (_x_1206 == _x_1207); match _x_1208 {
        false => _x_1208,
        true => { let _x_1212 = &(manifest).referentialIntegrity; { let _x_1213 = (_x_1212).bound; { let _x_1214 = &(_x_1212).values; { let _x_1215 = validateReferentialIntegrity(_x_1213, &(_x_1214)); _x_1215 } } } },
    } } } } },
    } } } } },
    } } } } } } } },
    } } } } },
    } } } } } } },
    } } } },
    } } } } } } },
    } } } },
    } } } } },
    } } } } } },
    } } } } },
    } } } } },
    } } } } } } } } } } },
    } } } },
    } } } } })
}

pub fn validateModelReleaseCompleteness(model: &crate::SystemModel, manifest: &crate::SystemManifest) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_93 = 0; { let _x_97 = &(model).artifacts; { let _x_98 = (_x_97).len() as u64; { let _x_99 = (_x_93 < _x_98); match _x_99 {
        false => _x_99,
        true => { let _x_181 = 0; { let _x_183 = &(model).components; { let _x_184 = (_x_183).len() as u64; { let _x_185 = (_x_181 < _x_184); match _x_185 {
        false => _x_185,
        true => { let _x_230 = 0; { let _x_232 = &(model).product; { let _x_233 = &(_x_232).supportedPlatforms; { let _x_234 = (_x_233).len() as u64; { let _x_235 = (_x_230 < _x_234); match _x_235 {
        false => _x_235,
        true => { let _x_272 = &(model).product; { let _x_273 = &(_x_272).supportedPlatforms; { let _x_274 = &(model).platformRequirements; { let _x_275 = allIdsMemberPlatformRequirements(&(_x_273), &(_x_274)); match _x_275 {
        false => _x_275,
        true => { let _x_305 = &(model).components; { let _x_306 = &(model).artifacts; { let _x_307 = componentsHaveArtifacts(&(_x_305), &(_x_306)); match _x_307 {
        false => _x_307,
        true => { let _x_327 = &(manifest).releaseCompleteness; { let _x_328 = (_x_327).bound; { let _x_330 = &(model).artifacts; { let _x_331 = (_x_330).len() as u64; { let _x_332 = (_x_328 == _x_331); match _x_332 {
        false => _x_332,
        true => { let _x_341 = &(manifest).releaseCompleteness; { let _x_342 = &(_x_341).values; { let _x_343 = (_x_342).len() as u64; { let _x_345 = &(model).artifacts; { let _x_346 = (_x_345).len() as u64; { let _x_347 = (_x_343 == _x_346); match _x_347 {
        false => _x_347,
        true => { let _x_351 = &(manifest).releaseCompleteness; { let _x_352 = &(_x_351).values; { let _x_353 = validateReleaseCompleteness(&(_x_352))?; _x_353 } } },
    } } } } } } },
    } } } } } },
    } } } },
    } } } } },
    } } } } } },
    } } } } },
    } } } } })
}

pub fn validateModelRollbackSafety(model: &crate::SystemModel, manifest: &crate::SystemManifest) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_22 = &(model).rollbacks; { let _x_23 = &(model).migrations; { let _x_24 = &(model).rollouts; { let _x_25 = rollbackRowsSafe(&(_x_22), &(_x_23), &(_x_24)); match _x_25 {
        false => _x_25,
        true => { let _x_43 = &(manifest).rollbackSafety; { let _x_44 = (_x_43).bound; { let _x_45 = modelEntityCount(&(model))?; { let _x_46 = (_x_44 == _x_45); match _x_46 {
        false => _x_46,
        true => { let _x_50 = &(manifest).rollbackSafety; { let _x_51 = (_x_50).bound; { let _x_52 = &(_x_50).values; { let _x_53 = validateRollbackSafety(_x_51, &(_x_52)); _x_53 } } } },
    } } } } },
    } } } } })
}

pub fn validateModelSecretFlow(model: &crate::SystemModel, manifest: &crate::SystemManifest) -> bool {
    { let _x_35 = &(model).components; { let _x_36 = &(model).secretReferences; { let _x_37 = componentSecretsSatisfied(&(_x_35), &(_x_36)); match _x_37 {
        false => _x_37,
        true => { let _x_68 = &(model).secretReferences; { let _x_69 = &(model).components; { let _x_70 = secretConsumersSatisfied(&(_x_68), &(_x_69)); match _x_70 {
        false => _x_70,
        true => { let _x_79 = &(manifest).secretFlow; { let _x_80 = (_x_79).bound; { let _x_82 = &(model).components; { let _x_83 = (_x_82).len() as u64; { let _x_84 = (_x_80 == _x_83); match _x_84 {
        false => _x_84,
        true => { let _x_88 = &(manifest).secretFlow; { let _x_89 = (_x_88).bound; { let _x_90 = &(_x_88).values; { let _x_91 = validateSecretFlow(_x_89, &(_x_90)); _x_91 } } } },
    } } } } } },
    } } } },
    } } } }
}

pub fn validateModelUniqueness(model: &crate::SystemModel, manifest: &crate::SystemManifest) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_23 = modelIdsGloballyUnique(&(model))?; match _x_23 {
        false => _x_23,
        true => { let _x_41 = &(manifest).uniqueness; { let _x_42 = &(_x_41).values; { let _x_43 = (_x_42).len() as u64; { let _x_44 = modelEntityCount(&(model))?; { let _x_45 = (_x_43 == _x_44); match _x_45 {
        false => _x_45,
        true => { let _x_49 = &(manifest).uniqueness; { let _x_50 = &(_x_49).values; { let _x_51 = validateUniqueness(&(_x_50))?; _x_51 } } },
    } } } } } },
    } })
}

pub fn corpusInvalidCapabilitySatisfactionModel() -> crate::SystemModel {
    { let _x_8 = alloc::vec![alloc::string::String::from("platform")]; { let _x_9 = crate::Product { id: alloc::string::String::from("product"), owner: alloc::string::String::from("PrismPM"), version: alloc::string::String::from("1"), lifecycle: alloc::string::String::from("development"), sourcePolicy: alloc::string::String::from("closed"), supportedPlatforms: _x_8.clone() }; { let _x_16 = crate::Artifact { id: alloc::string::String::from("artifact"), mediaType: alloc::string::String::from("application/octet-stream"), digest: alloc::string::String::from("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), licenseExpression: alloc::string::String::from("MIT"), path: alloc::string::String::from("artifact.bin"), role: alloc::string::String::from("runtime"), platformRequirements: _x_8.clone() }; { let _x_18 = alloc::vec![_x_16]; { let _x_22 = alloc::vec![alloc::string::String::from("missing")]; { let _x_24 = alloc::vec![alloc::string::String::from("run")]; { let _x_26 = alloc::vec![alloc::string::String::from("interface")]; { let _x_138 = 1; { let _x_140 = 1; { let _x_33 = crate::ResourceRequirements { cpuMillis: _x_138, memoryBytes: _x_140, replicasMin: _x_138, replicasMax: _x_138 }; { let _x_35 = alloc::vec![alloc::string::String::from("secret")]; { let _x_42 = crate::Component { id: alloc::string::String::from("component"), kind: alloc::string::String::from("service"), version: alloc::string::String::from("1"), artifact: alloc::string::String::from("artifact"), capabilities: _x_22, command: _x_24, dependsOn: alloc::vec::Vec::new(), interfaces: _x_26, health: alloc::string::String::from("healthy"), liveness: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), ports: alloc::vec::Vec::new(), readiness: alloc::vec::Vec::new(), resources: _x_33, secrets: _x_35, startup: alloc::vec::Vec::new(), volumes: alloc::vec::Vec::new(), isolation: alloc::string::String::from("value"), placement: alloc::vec::Vec::new(), platformRequirements: _x_8.clone(), scalingPolicy: None, failurePolicy: alloc::string::String::from("fail-closed"), retryPolicy: alloc::string::String::from("bounded"), degradationPolicy: alloc::string::String::from("unavailable"), idempotency: alloc::string::String::from("keyed") }; { let _x_44 = alloc::vec![_x_42]; { let _x_51 = alloc::vec![alloc::string::String::from("acceptance")]; { let _x_52 = crate::Interface { id: alloc::string::String::from("interface"), kind: alloc::string::String::from("http"), document: alloc::string::String::from("schema"), authentication: alloc::string::String::from("oidc"), compatibility: alloc::string::String::from("exact"), protocol: alloc::string::String::from("https"), errors: alloc::vec::Vec::new(), acceptance: _x_51 }; { let _x_54 = alloc::vec![_x_52]; { let _x_57 = crate::Schema { id: alloc::string::String::from("schema"), kind: alloc::string::String::from("json-schema"), value: alloc::string::String::from("{}"), dependsOn: alloc::vec::Vec::new(), compatibility: alloc::string::String::from("exact") }; { let _x_59 = alloc::vec![_x_57]; { let _x_66 = alloc::vec![alloc::string::String::from("amd64")]; { let _x_70 = alloc::vec![alloc::string::String::from("capability")]; { let _x_71 = crate::PlatformRequirement { id: alloc::string::String::from("platform"), os: alloc::string::String::from("linux"), architectures: _x_66, runtime: alloc::string::String::from("oci"), runtimeVersion: alloc::string::String::from("1.1"), capabilities: _x_70.clone() }; { let _x_73 = alloc::vec![_x_71]; { let _x_78 = alloc::vec![alloc::string::String::from("component")]; { let _x_80 = crate::SecretReference { id: alloc::string::String::from("secret"), providerKey: alloc::string::String::from("secret://provider/key"), consumers: _x_78, rotation: alloc::string::String::from("automatic") }; { let _x_82 = alloc::vec![_x_80]; { let _x_88 = crate::Migration { id: alloc::string::String::from("migration"), kind: alloc::string::String::from("initialize"), value: alloc::string::String::from("SELECT 1;"), dependsOn: alloc::vec::Vec::new() }; { let _x_90 = alloc::vec![_x_88]; { let _x_92 = crate::Observability { logs: alloc::vec::Vec::new(), metrics: alloc::vec::Vec::new(), traces: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), redactedFields: alloc::vec::Vec::new() }; { let _x_98 = crate::Capability { id: alloc::string::String::from("capability"), kind: alloc::string::String::from("runtime"), value: alloc::string::String::from("present"), dependsOn: alloc::vec::Vec::new() }; { let _x_100 = alloc::vec![_x_98]; { let _x_105 = Some(alloc::string::String::from("secret")); { let _x_106 = Some(alloc::string::String::from("artifact")); { let _x_107 = crate::TargetBinding { id: alloc::string::String::from("target"), kind: alloc::string::String::from("compose"), apiVersion: alloc::string::String::from("1"), adapterDigest: alloc::string::String::from("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"), minimumReleaseStatus: alloc::string::String::from("development"), capabilities: _x_70.clone(), credentials: _x_105, platformRequirements: _x_8.clone(), storageClass: None, storageProfile: None, ingressClassName: None, ingressControllerArtifact: _x_106 }; { let _x_109 = alloc::vec![_x_107]; { let _x_113 = alloc::vec![alloc::string::String::from("migration")]; { let _x_114 = crate::Rollout { id: alloc::string::String::from("rollout"), kind: alloc::string::String::from("rolling"), value: alloc::string::String::from("one-at-a-time"), dependsOn: _x_113 }; { let _x_116 = alloc::vec![_x_114]; { let _x_118 = alloc::vec![alloc::string::String::from("rollout")]; { let _x_119 = { let mut __list = alloc::vec![alloc::string::String::from("migration")]; __list.extend(_x_118); __list }; { let _x_120 = crate::Rollback { id: alloc::string::String::from("rollback"), kind: alloc::string::String::from("automatic"), value: alloc::string::String::from("rollback"), dependsOn: _x_119 }; { let _x_122 = alloc::vec![_x_120]; { let _x_127 = alloc::vec![alloc::string::String::from("probe")]; { let _x_129 = true; { let _x_130 = crate::Acceptance { id: alloc::string::String::from("acceptance"), kind: alloc::string::String::from("positive"), target: alloc::string::String::from("target"), component: alloc::string::String::from("component"), command: _x_127, evidence: alloc::string::String::from("evidence"), bounded: _x_129 }; { let _x_132 = alloc::vec![_x_130]; { let _x_133 = crate::Lifecycle { backup: alloc::string::String::from("value"), drift: alloc::string::String::from("value"), migration: alloc::string::String::from("value"), recovery: alloc::string::String::from("value"), retirement: alloc::string::String::from("value"), rollback: alloc::string::String::from("value"), rollout: alloc::string::String::from("value") }; { let _x_135 = crate::TransactionalCommandView { title: alloc::string::String::from("value"), heading: alloc::string::String::from("value"), identityHeading: alloc::string::String::from("value"), commandFormHeading: alloc::string::String::from("value"), historyHeading: alloc::string::String::from("value"), principalLabel: alloc::string::String::from("value"), principalDefault: alloc::string::String::from("value"), roleLabel: alloc::string::String::from("value"), submitterRoleLabel: alloc::string::String::from("value"), observerRoleLabel: alloc::string::String::from("value"), deniedRoleLabel: alloc::string::String::from("value"), authenticateLabel: alloc::string::String::from("value"), commandIdLabel: alloc::string::String::from("value"), inputALabel: alloc::string::String::from("value"), operationLabel: alloc::string::String::from("value"), inputBLabel: alloc::string::String::from("value"), annotationLabel: alloc::string::String::from("value"), submitLabel: alloc::string::String::from("value"), refreshLabel: alloc::string::String::from("value"), emptyText: alloc::string::String::from("value"), loadingText: alloc::string::String::from("value"), authenticatedText: alloc::string::String::from("value"), accessDeniedText: alloc::string::String::from("value"), retryLabel: alloc::string::String::from("value"), sequenceHeading: alloc::string::String::from("value"), commandColumnHeading: alloc::string::String::from("value"), inputSummaryHeading: alloc::string::String::from("value"), outcomeHeading: alloc::string::String::from("value"), annotationHeading: alloc::string::String::from("value"), releasePrefix: alloc::string::String::from("value"), historyLoadedText: alloc::string::String::from("value"), historyEmptyText: alloc::string::String::from("value"), authenticatingText: alloc::string::String::from("value"), authenticationFailedText: alloc::string::String::from("value"), submittingText: alloc::string::String::from("value"), outcomePrefix: alloc::string::String::from("value"), commandFailedText: alloc::string::String::from("value"), historyFailedText: alloc::string::String::from("value") }; { let _x_136 = crate::TransactionalCommandServiceProfile { contract: alloc::string::String::from("value"), applicationErrors: alloc::vec::Vec::new(), applicationModelDigest: alloc::string::String::from("value"), commandEncoding: alloc::string::String::from("value"), responseVersion: alloc::string::String::from("value"), optionalAnnotation: alloc::string::String::from("value"), coreArtifact: alloc::string::String::from("value"), identityAudience: alloc::string::String::from("value"), submitterRole: alloc::string::String::from("value"), observerRole: alloc::string::String::from("value"), eventSource: alloc::string::String::from("value"), acceptedEventType: alloc::string::String::from("value"), rejectedEventType: alloc::string::String::from("value"), databaseName: alloc::string::String::from("value"), databaseUser: alloc::string::String::from("value"), eventExchange: alloc::string::String::from("value"), auditQueue: alloc::string::String::from("value"), commandIdPattern: alloc::string::String::from("value"), commandPath: alloc::string::String::from("value"), tokenPath: alloc::string::String::from("value"), commandOperations: alloc::vec::Vec::new(), annotationMaxScalars: _x_138, historyDefaultLimit: _x_138, historyMaxLimit: _x_138, maxRequestBytes: _x_140, availabilityThresholdMillionths: _x_140, outboxLagThresholdMillis: _x_140, commandIdField: alloc::string::String::from("value"), operationField: alloc::string::String::from("value"), inputAField: alloc::string::String::from("value"), inputBField: alloc::string::String::from("value"), annotationField: alloc::string::String::from("value"), commandIdColumn: alloc::string::String::from("value"), operationColumn: alloc::string::String::from("value"), inputAColumn: alloc::string::String::from("value"), inputBColumn: alloc::string::String::from("value"), annotationColumn: alloc::string::String::from("value"), acceptanceErrorOperation: alloc::string::String::from("value"), acceptanceErrorInputA: alloc::string::String::from("value"), acceptanceErrorInputB: alloc::string::String::from("value"), acceptanceExpectedError: alloc::string::String::from("value"), acceptanceConflictInputB: alloc::string::String::from("value"), acceptanceSuccessOperation: alloc::string::String::from("value"), acceptanceSuccessInputA: alloc::string::String::from("value"), acceptanceSuccessInputB: alloc::string::String::from("value"), acceptanceExpectedResult: alloc::string::String::from("value"), view: _x_135, brokerUser: alloc::string::String::from("value"), historyTable: alloc::string::String::from("value"), outboxTable: alloc::string::String::from("value"), auditTable: alloc::string::String::from("value"), publicHostnameParameter: alloc::string::String::from("value") }; { let _x_137 = crate::SystemModel { product: _x_9, artifacts: _x_18, components: _x_44, interfaces: _x_54, schemas: _x_59, calls: alloc::vec::Vec::new(), events: alloc::vec::Vec::new(), flows: alloc::vec::Vec::new(), topology: alloc::vec::Vec::new(), platformRequirements: _x_73, scalingPolicies: alloc::vec::Vec::new(), storageClasses: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), secretReferences: _x_82, identityRequirements: alloc::vec::Vec::new(), persistence: alloc::vec::Vec::new(), migrations: _x_90, backups: alloc::vec::Vec::new(), observability: _x_92, slis: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), controls: alloc::vec::Vec::new(), capabilities: _x_100, architecture: alloc::vec::Vec::new(), targets: _x_109, rollouts: _x_116, rollbacks: _x_122, drifts: alloc::vec::Vec::new(), retirements: alloc::vec::Vec::new(), acceptance: _x_132, lifecycle: _x_133, applicationProfile: _x_136, standards: alloc::vec::Vec::new() }; _x_137 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn corpusInvalidClosureModel() -> crate::SystemModel {
    { let _x_8 = alloc::vec![alloc::string::String::from("platform")]; { let _x_9 = crate::Product { id: alloc::string::String::from("product"), owner: alloc::string::String::from("PrismPM"), version: alloc::string::String::from("1"), lifecycle: alloc::string::String::from("development"), sourcePolicy: alloc::string::String::from("closed"), supportedPlatforms: _x_8.clone() }; { let _x_17 = alloc::vec![alloc::string::String::from("missing")]; { let _x_18 = crate::Artifact { id: alloc::string::String::from("artifact"), mediaType: alloc::string::String::from("application/octet-stream"), digest: alloc::string::String::from("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), licenseExpression: alloc::string::String::from("MIT"), path: alloc::string::String::from("artifact.bin"), role: alloc::string::String::from("runtime"), platformRequirements: _x_17 }; { let _x_20 = alloc::vec![_x_18]; { let _x_24 = alloc::vec![alloc::string::String::from("capability")]; { let _x_26 = alloc::vec![alloc::string::String::from("run")]; { let _x_28 = alloc::vec![alloc::string::String::from("interface")]; { let _x_138 = 1; { let _x_140 = 1; { let _x_35 = crate::ResourceRequirements { cpuMillis: _x_138, memoryBytes: _x_140, replicasMin: _x_138, replicasMax: _x_138 }; { let _x_37 = alloc::vec![alloc::string::String::from("secret")]; { let _x_44 = crate::Component { id: alloc::string::String::from("component"), kind: alloc::string::String::from("service"), version: alloc::string::String::from("1"), artifact: alloc::string::String::from("artifact"), capabilities: _x_24.clone(), command: _x_26, dependsOn: alloc::vec::Vec::new(), interfaces: _x_28, health: alloc::string::String::from("healthy"), liveness: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), ports: alloc::vec::Vec::new(), readiness: alloc::vec::Vec::new(), resources: _x_35, secrets: _x_37, startup: alloc::vec::Vec::new(), volumes: alloc::vec::Vec::new(), isolation: alloc::string::String::from("value"), placement: alloc::vec::Vec::new(), platformRequirements: _x_8.clone(), scalingPolicy: None, failurePolicy: alloc::string::String::from("fail-closed"), retryPolicy: alloc::string::String::from("bounded"), degradationPolicy: alloc::string::String::from("unavailable"), idempotency: alloc::string::String::from("keyed") }; { let _x_46 = alloc::vec![_x_44]; { let _x_53 = alloc::vec![alloc::string::String::from("acceptance")]; { let _x_54 = crate::Interface { id: alloc::string::String::from("interface"), kind: alloc::string::String::from("http"), document: alloc::string::String::from("schema"), authentication: alloc::string::String::from("oidc"), compatibility: alloc::string::String::from("exact"), protocol: alloc::string::String::from("https"), errors: alloc::vec::Vec::new(), acceptance: _x_53 }; { let _x_56 = alloc::vec![_x_54]; { let _x_59 = crate::Schema { id: alloc::string::String::from("schema"), kind: alloc::string::String::from("json-schema"), value: alloc::string::String::from("{}"), dependsOn: alloc::vec::Vec::new(), compatibility: alloc::string::String::from("exact") }; { let _x_61 = alloc::vec![_x_59]; { let _x_68 = alloc::vec![alloc::string::String::from("amd64")]; { let _x_71 = crate::PlatformRequirement { id: alloc::string::String::from("platform"), os: alloc::string::String::from("linux"), architectures: _x_68, runtime: alloc::string::String::from("oci"), runtimeVersion: alloc::string::String::from("1.1"), capabilities: _x_24.clone() }; { let _x_73 = alloc::vec![_x_71]; { let _x_78 = alloc::vec![alloc::string::String::from("component")]; { let _x_80 = crate::SecretReference { id: alloc::string::String::from("secret"), providerKey: alloc::string::String::from("secret://provider/key"), consumers: _x_78, rotation: alloc::string::String::from("automatic") }; { let _x_82 = alloc::vec![_x_80]; { let _x_88 = crate::Migration { id: alloc::string::String::from("migration"), kind: alloc::string::String::from("initialize"), value: alloc::string::String::from("SELECT 1;"), dependsOn: alloc::vec::Vec::new() }; { let _x_90 = alloc::vec![_x_88]; { let _x_92 = crate::Observability { logs: alloc::vec::Vec::new(), metrics: alloc::vec::Vec::new(), traces: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), redactedFields: alloc::vec::Vec::new() }; { let _x_98 = crate::Capability { id: alloc::string::String::from("capability"), kind: alloc::string::String::from("runtime"), value: alloc::string::String::from("present"), dependsOn: alloc::vec::Vec::new() }; { let _x_100 = alloc::vec![_x_98]; { let _x_105 = Some(alloc::string::String::from("secret")); { let _x_106 = Some(alloc::string::String::from("artifact")); { let _x_107 = crate::TargetBinding { id: alloc::string::String::from("target"), kind: alloc::string::String::from("compose"), apiVersion: alloc::string::String::from("1"), adapterDigest: alloc::string::String::from("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"), minimumReleaseStatus: alloc::string::String::from("development"), capabilities: _x_24.clone(), credentials: _x_105, platformRequirements: _x_8.clone(), storageClass: None, storageProfile: None, ingressClassName: None, ingressControllerArtifact: _x_106 }; { let _x_109 = alloc::vec![_x_107]; { let _x_113 = alloc::vec![alloc::string::String::from("migration")]; { let _x_114 = crate::Rollout { id: alloc::string::String::from("rollout"), kind: alloc::string::String::from("rolling"), value: alloc::string::String::from("one-at-a-time"), dependsOn: _x_113 }; { let _x_116 = alloc::vec![_x_114]; { let _x_118 = alloc::vec![alloc::string::String::from("rollout")]; { let _x_119 = { let mut __list = alloc::vec![alloc::string::String::from("migration")]; __list.extend(_x_118); __list }; { let _x_120 = crate::Rollback { id: alloc::string::String::from("rollback"), kind: alloc::string::String::from("automatic"), value: alloc::string::String::from("rollback"), dependsOn: _x_119 }; { let _x_122 = alloc::vec![_x_120]; { let _x_127 = alloc::vec![alloc::string::String::from("probe")]; { let _x_129 = true; { let _x_130 = crate::Acceptance { id: alloc::string::String::from("acceptance"), kind: alloc::string::String::from("positive"), target: alloc::string::String::from("target"), component: alloc::string::String::from("component"), command: _x_127, evidence: alloc::string::String::from("evidence"), bounded: _x_129 }; { let _x_132 = alloc::vec![_x_130]; { let _x_133 = crate::Lifecycle { backup: alloc::string::String::from("value"), drift: alloc::string::String::from("value"), migration: alloc::string::String::from("value"), recovery: alloc::string::String::from("value"), retirement: alloc::string::String::from("value"), rollback: alloc::string::String::from("value"), rollout: alloc::string::String::from("value") }; { let _x_135 = crate::TransactionalCommandView { title: alloc::string::String::from("value"), heading: alloc::string::String::from("value"), identityHeading: alloc::string::String::from("value"), commandFormHeading: alloc::string::String::from("value"), historyHeading: alloc::string::String::from("value"), principalLabel: alloc::string::String::from("value"), principalDefault: alloc::string::String::from("value"), roleLabel: alloc::string::String::from("value"), submitterRoleLabel: alloc::string::String::from("value"), observerRoleLabel: alloc::string::String::from("value"), deniedRoleLabel: alloc::string::String::from("value"), authenticateLabel: alloc::string::String::from("value"), commandIdLabel: alloc::string::String::from("value"), inputALabel: alloc::string::String::from("value"), operationLabel: alloc::string::String::from("value"), inputBLabel: alloc::string::String::from("value"), annotationLabel: alloc::string::String::from("value"), submitLabel: alloc::string::String::from("value"), refreshLabel: alloc::string::String::from("value"), emptyText: alloc::string::String::from("value"), loadingText: alloc::string::String::from("value"), authenticatedText: alloc::string::String::from("value"), accessDeniedText: alloc::string::String::from("value"), retryLabel: alloc::string::String::from("value"), sequenceHeading: alloc::string::String::from("value"), commandColumnHeading: alloc::string::String::from("value"), inputSummaryHeading: alloc::string::String::from("value"), outcomeHeading: alloc::string::String::from("value"), annotationHeading: alloc::string::String::from("value"), releasePrefix: alloc::string::String::from("value"), historyLoadedText: alloc::string::String::from("value"), historyEmptyText: alloc::string::String::from("value"), authenticatingText: alloc::string::String::from("value"), authenticationFailedText: alloc::string::String::from("value"), submittingText: alloc::string::String::from("value"), outcomePrefix: alloc::string::String::from("value"), commandFailedText: alloc::string::String::from("value"), historyFailedText: alloc::string::String::from("value") }; { let _x_136 = crate::TransactionalCommandServiceProfile { contract: alloc::string::String::from("value"), applicationErrors: alloc::vec::Vec::new(), applicationModelDigest: alloc::string::String::from("value"), commandEncoding: alloc::string::String::from("value"), responseVersion: alloc::string::String::from("value"), optionalAnnotation: alloc::string::String::from("value"), coreArtifact: alloc::string::String::from("value"), identityAudience: alloc::string::String::from("value"), submitterRole: alloc::string::String::from("value"), observerRole: alloc::string::String::from("value"), eventSource: alloc::string::String::from("value"), acceptedEventType: alloc::string::String::from("value"), rejectedEventType: alloc::string::String::from("value"), databaseName: alloc::string::String::from("value"), databaseUser: alloc::string::String::from("value"), eventExchange: alloc::string::String::from("value"), auditQueue: alloc::string::String::from("value"), commandIdPattern: alloc::string::String::from("value"), commandPath: alloc::string::String::from("value"), tokenPath: alloc::string::String::from("value"), commandOperations: alloc::vec::Vec::new(), annotationMaxScalars: _x_138, historyDefaultLimit: _x_138, historyMaxLimit: _x_138, maxRequestBytes: _x_140, availabilityThresholdMillionths: _x_140, outboxLagThresholdMillis: _x_140, commandIdField: alloc::string::String::from("value"), operationField: alloc::string::String::from("value"), inputAField: alloc::string::String::from("value"), inputBField: alloc::string::String::from("value"), annotationField: alloc::string::String::from("value"), commandIdColumn: alloc::string::String::from("value"), operationColumn: alloc::string::String::from("value"), inputAColumn: alloc::string::String::from("value"), inputBColumn: alloc::string::String::from("value"), annotationColumn: alloc::string::String::from("value"), acceptanceErrorOperation: alloc::string::String::from("value"), acceptanceErrorInputA: alloc::string::String::from("value"), acceptanceErrorInputB: alloc::string::String::from("value"), acceptanceExpectedError: alloc::string::String::from("value"), acceptanceConflictInputB: alloc::string::String::from("value"), acceptanceSuccessOperation: alloc::string::String::from("value"), acceptanceSuccessInputA: alloc::string::String::from("value"), acceptanceSuccessInputB: alloc::string::String::from("value"), acceptanceExpectedResult: alloc::string::String::from("value"), view: _x_135, brokerUser: alloc::string::String::from("value"), historyTable: alloc::string::String::from("value"), outboxTable: alloc::string::String::from("value"), auditTable: alloc::string::String::from("value"), publicHostnameParameter: alloc::string::String::from("value") }; { let _x_137 = crate::SystemModel { product: _x_9, artifacts: _x_20, components: _x_46, interfaces: _x_56, schemas: _x_61, calls: alloc::vec::Vec::new(), events: alloc::vec::Vec::new(), flows: alloc::vec::Vec::new(), topology: alloc::vec::Vec::new(), platformRequirements: _x_73, scalingPolicies: alloc::vec::Vec::new(), storageClasses: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), secretReferences: _x_82, identityRequirements: alloc::vec::Vec::new(), persistence: alloc::vec::Vec::new(), migrations: _x_90, backups: alloc::vec::Vec::new(), observability: _x_92, slis: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), controls: alloc::vec::Vec::new(), capabilities: _x_100, architecture: alloc::vec::Vec::new(), targets: _x_109, rollouts: _x_116, rollbacks: _x_122, drifts: alloc::vec::Vec::new(), retirements: alloc::vec::Vec::new(), acceptance: _x_132, lifecycle: _x_133, applicationProfile: _x_136, standards: alloc::vec::Vec::new() }; _x_137 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn corpusInvalidCompatibilityModel() -> crate::SystemModel {
    { let _x_8 = alloc::vec![alloc::string::String::from("platform")]; { let _x_9 = crate::Product { id: alloc::string::String::from("product"), owner: alloc::string::String::from("PrismPM"), version: alloc::string::String::from("1"), lifecycle: alloc::string::String::from("development"), sourcePolicy: alloc::string::String::from("closed"), supportedPlatforms: _x_8.clone() }; { let _x_16 = crate::Artifact { id: alloc::string::String::from("artifact"), mediaType: alloc::string::String::from("application/octet-stream"), digest: alloc::string::String::from("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), licenseExpression: alloc::string::String::from("MIT"), path: alloc::string::String::from("artifact.bin"), role: alloc::string::String::from("runtime"), platformRequirements: _x_8.clone() }; { let _x_18 = alloc::vec![_x_16]; { let _x_22 = alloc::vec![alloc::string::String::from("capability")]; { let _x_24 = alloc::vec![alloc::string::String::from("run")]; { let _x_26 = alloc::vec![alloc::string::String::from("interface")]; { let _x_137 = 1; { let _x_139 = 1; { let _x_33 = crate::ResourceRequirements { cpuMillis: _x_137, memoryBytes: _x_139, replicasMin: _x_137, replicasMax: _x_137 }; { let _x_35 = alloc::vec![alloc::string::String::from("secret")]; { let _x_42 = crate::Component { id: alloc::string::String::from("component"), kind: alloc::string::String::from("service"), version: alloc::string::String::from("1"), artifact: alloc::string::String::from("artifact"), capabilities: _x_22.clone(), command: _x_24, dependsOn: alloc::vec::Vec::new(), interfaces: _x_26, health: alloc::string::String::from("healthy"), liveness: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), ports: alloc::vec::Vec::new(), readiness: alloc::vec::Vec::new(), resources: _x_33, secrets: _x_35, startup: alloc::vec::Vec::new(), volumes: alloc::vec::Vec::new(), isolation: alloc::string::String::from("value"), placement: alloc::vec::Vec::new(), platformRequirements: _x_8.clone(), scalingPolicy: None, failurePolicy: alloc::string::String::from("fail-closed"), retryPolicy: alloc::string::String::from("bounded"), degradationPolicy: alloc::string::String::from("unavailable"), idempotency: alloc::string::String::from("keyed") }; { let _x_44 = alloc::vec![_x_42]; { let _x_51 = alloc::vec![alloc::string::String::from("acceptance")]; { let _x_52 = crate::Interface { id: alloc::string::String::from("interface"), kind: alloc::string::String::from("http"), document: alloc::string::String::from("schema"), authentication: alloc::string::String::from("oidc"), compatibility: alloc::string::String::from("unknown"), protocol: alloc::string::String::from("https"), errors: alloc::vec::Vec::new(), acceptance: _x_51 }; { let _x_54 = alloc::vec![_x_52]; { let _x_58 = crate::Schema { id: alloc::string::String::from("schema"), kind: alloc::string::String::from("json-schema"), value: alloc::string::String::from("{}"), dependsOn: alloc::vec::Vec::new(), compatibility: alloc::string::String::from("exact") }; { let _x_60 = alloc::vec![_x_58]; { let _x_67 = alloc::vec![alloc::string::String::from("amd64")]; { let _x_70 = crate::PlatformRequirement { id: alloc::string::String::from("platform"), os: alloc::string::String::from("linux"), architectures: _x_67, runtime: alloc::string::String::from("oci"), runtimeVersion: alloc::string::String::from("1.1"), capabilities: _x_22.clone() }; { let _x_72 = alloc::vec![_x_70]; { let _x_77 = alloc::vec![alloc::string::String::from("component")]; { let _x_79 = crate::SecretReference { id: alloc::string::String::from("secret"), providerKey: alloc::string::String::from("secret://provider/key"), consumers: _x_77, rotation: alloc::string::String::from("automatic") }; { let _x_81 = alloc::vec![_x_79]; { let _x_87 = crate::Migration { id: alloc::string::String::from("migration"), kind: alloc::string::String::from("initialize"), value: alloc::string::String::from("SELECT 1;"), dependsOn: alloc::vec::Vec::new() }; { let _x_89 = alloc::vec![_x_87]; { let _x_91 = crate::Observability { logs: alloc::vec::Vec::new(), metrics: alloc::vec::Vec::new(), traces: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), redactedFields: alloc::vec::Vec::new() }; { let _x_97 = crate::Capability { id: alloc::string::String::from("capability"), kind: alloc::string::String::from("runtime"), value: alloc::string::String::from("present"), dependsOn: alloc::vec::Vec::new() }; { let _x_99 = alloc::vec![_x_97]; { let _x_104 = Some(alloc::string::String::from("secret")); { let _x_105 = Some(alloc::string::String::from("artifact")); { let _x_106 = crate::TargetBinding { id: alloc::string::String::from("target"), kind: alloc::string::String::from("compose"), apiVersion: alloc::string::String::from("1"), adapterDigest: alloc::string::String::from("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"), minimumReleaseStatus: alloc::string::String::from("development"), capabilities: _x_22.clone(), credentials: _x_104, platformRequirements: _x_8.clone(), storageClass: None, storageProfile: None, ingressClassName: None, ingressControllerArtifact: _x_105 }; { let _x_108 = alloc::vec![_x_106]; { let _x_112 = alloc::vec![alloc::string::String::from("migration")]; { let _x_113 = crate::Rollout { id: alloc::string::String::from("rollout"), kind: alloc::string::String::from("rolling"), value: alloc::string::String::from("one-at-a-time"), dependsOn: _x_112 }; { let _x_115 = alloc::vec![_x_113]; { let _x_117 = alloc::vec![alloc::string::String::from("rollout")]; { let _x_118 = { let mut __list = alloc::vec![alloc::string::String::from("migration")]; __list.extend(_x_117); __list }; { let _x_119 = crate::Rollback { id: alloc::string::String::from("rollback"), kind: alloc::string::String::from("automatic"), value: alloc::string::String::from("rollback"), dependsOn: _x_118 }; { let _x_121 = alloc::vec![_x_119]; { let _x_126 = alloc::vec![alloc::string::String::from("probe")]; { let _x_128 = true; { let _x_129 = crate::Acceptance { id: alloc::string::String::from("acceptance"), kind: alloc::string::String::from("positive"), target: alloc::string::String::from("target"), component: alloc::string::String::from("component"), command: _x_126, evidence: alloc::string::String::from("evidence"), bounded: _x_128 }; { let _x_131 = alloc::vec![_x_129]; { let _x_132 = crate::Lifecycle { backup: alloc::string::String::from("value"), drift: alloc::string::String::from("value"), migration: alloc::string::String::from("value"), recovery: alloc::string::String::from("value"), retirement: alloc::string::String::from("value"), rollback: alloc::string::String::from("value"), rollout: alloc::string::String::from("value") }; { let _x_134 = crate::TransactionalCommandView { title: alloc::string::String::from("value"), heading: alloc::string::String::from("value"), identityHeading: alloc::string::String::from("value"), commandFormHeading: alloc::string::String::from("value"), historyHeading: alloc::string::String::from("value"), principalLabel: alloc::string::String::from("value"), principalDefault: alloc::string::String::from("value"), roleLabel: alloc::string::String::from("value"), submitterRoleLabel: alloc::string::String::from("value"), observerRoleLabel: alloc::string::String::from("value"), deniedRoleLabel: alloc::string::String::from("value"), authenticateLabel: alloc::string::String::from("value"), commandIdLabel: alloc::string::String::from("value"), inputALabel: alloc::string::String::from("value"), operationLabel: alloc::string::String::from("value"), inputBLabel: alloc::string::String::from("value"), annotationLabel: alloc::string::String::from("value"), submitLabel: alloc::string::String::from("value"), refreshLabel: alloc::string::String::from("value"), emptyText: alloc::string::String::from("value"), loadingText: alloc::string::String::from("value"), authenticatedText: alloc::string::String::from("value"), accessDeniedText: alloc::string::String::from("value"), retryLabel: alloc::string::String::from("value"), sequenceHeading: alloc::string::String::from("value"), commandColumnHeading: alloc::string::String::from("value"), inputSummaryHeading: alloc::string::String::from("value"), outcomeHeading: alloc::string::String::from("value"), annotationHeading: alloc::string::String::from("value"), releasePrefix: alloc::string::String::from("value"), historyLoadedText: alloc::string::String::from("value"), historyEmptyText: alloc::string::String::from("value"), authenticatingText: alloc::string::String::from("value"), authenticationFailedText: alloc::string::String::from("value"), submittingText: alloc::string::String::from("value"), outcomePrefix: alloc::string::String::from("value"), commandFailedText: alloc::string::String::from("value"), historyFailedText: alloc::string::String::from("value") }; { let _x_135 = crate::TransactionalCommandServiceProfile { contract: alloc::string::String::from("value"), applicationErrors: alloc::vec::Vec::new(), applicationModelDigest: alloc::string::String::from("value"), commandEncoding: alloc::string::String::from("value"), responseVersion: alloc::string::String::from("value"), optionalAnnotation: alloc::string::String::from("value"), coreArtifact: alloc::string::String::from("value"), identityAudience: alloc::string::String::from("value"), submitterRole: alloc::string::String::from("value"), observerRole: alloc::string::String::from("value"), eventSource: alloc::string::String::from("value"), acceptedEventType: alloc::string::String::from("value"), rejectedEventType: alloc::string::String::from("value"), databaseName: alloc::string::String::from("value"), databaseUser: alloc::string::String::from("value"), eventExchange: alloc::string::String::from("value"), auditQueue: alloc::string::String::from("value"), commandIdPattern: alloc::string::String::from("value"), commandPath: alloc::string::String::from("value"), tokenPath: alloc::string::String::from("value"), commandOperations: alloc::vec::Vec::new(), annotationMaxScalars: _x_137, historyDefaultLimit: _x_137, historyMaxLimit: _x_137, maxRequestBytes: _x_139, availabilityThresholdMillionths: _x_139, outboxLagThresholdMillis: _x_139, commandIdField: alloc::string::String::from("value"), operationField: alloc::string::String::from("value"), inputAField: alloc::string::String::from("value"), inputBField: alloc::string::String::from("value"), annotationField: alloc::string::String::from("value"), commandIdColumn: alloc::string::String::from("value"), operationColumn: alloc::string::String::from("value"), inputAColumn: alloc::string::String::from("value"), inputBColumn: alloc::string::String::from("value"), annotationColumn: alloc::string::String::from("value"), acceptanceErrorOperation: alloc::string::String::from("value"), acceptanceErrorInputA: alloc::string::String::from("value"), acceptanceErrorInputB: alloc::string::String::from("value"), acceptanceExpectedError: alloc::string::String::from("value"), acceptanceConflictInputB: alloc::string::String::from("value"), acceptanceSuccessOperation: alloc::string::String::from("value"), acceptanceSuccessInputA: alloc::string::String::from("value"), acceptanceSuccessInputB: alloc::string::String::from("value"), acceptanceExpectedResult: alloc::string::String::from("value"), view: _x_134, brokerUser: alloc::string::String::from("value"), historyTable: alloc::string::String::from("value"), outboxTable: alloc::string::String::from("value"), auditTable: alloc::string::String::from("value"), publicHostnameParameter: alloc::string::String::from("value") }; { let _x_136 = crate::SystemModel { product: _x_9, artifacts: _x_18, components: _x_44, interfaces: _x_54, schemas: _x_60, calls: alloc::vec::Vec::new(), events: alloc::vec::Vec::new(), flows: alloc::vec::Vec::new(), topology: alloc::vec::Vec::new(), platformRequirements: _x_72, scalingPolicies: alloc::vec::Vec::new(), storageClasses: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), secretReferences: _x_81, identityRequirements: alloc::vec::Vec::new(), persistence: alloc::vec::Vec::new(), migrations: _x_89, backups: alloc::vec::Vec::new(), observability: _x_91, slis: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), controls: alloc::vec::Vec::new(), capabilities: _x_99, architecture: alloc::vec::Vec::new(), targets: _x_108, rollouts: _x_115, rollbacks: _x_121, drifts: alloc::vec::Vec::new(), retirements: alloc::vec::Vec::new(), acceptance: _x_131, lifecycle: _x_132, applicationProfile: _x_135, standards: alloc::vec::Vec::new() }; _x_136 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn corpusInvalidDeploymentOrderModel() -> crate::SystemModel {
    { let _x_8 = alloc::vec![alloc::string::String::from("platform")]; { let _x_9 = crate::Product { id: alloc::string::String::from("product"), owner: alloc::string::String::from("PrismPM"), version: alloc::string::String::from("1"), lifecycle: alloc::string::String::from("development"), sourcePolicy: alloc::string::String::from("closed"), supportedPlatforms: _x_8.clone() }; { let _x_16 = crate::Artifact { id: alloc::string::String::from("artifact"), mediaType: alloc::string::String::from("application/octet-stream"), digest: alloc::string::String::from("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), licenseExpression: alloc::string::String::from("MIT"), path: alloc::string::String::from("artifact.bin"), role: alloc::string::String::from("runtime"), platformRequirements: _x_8.clone() }; { let _x_18 = alloc::vec![_x_16]; { let _x_22 = alloc::vec![alloc::string::String::from("capability")]; { let _x_24 = alloc::vec![alloc::string::String::from("run")]; { let _x_25 = alloc::vec![alloc::string::String::from("component")]; { let _x_27 = alloc::vec![alloc::string::String::from("interface")]; { let _x_136 = 1; { let _x_138 = 1; { let _x_34 = crate::ResourceRequirements { cpuMillis: _x_136, memoryBytes: _x_138, replicasMin: _x_136, replicasMax: _x_136 }; { let _x_36 = alloc::vec![alloc::string::String::from("secret")]; { let _x_43 = crate::Component { id: alloc::string::String::from("component"), kind: alloc::string::String::from("service"), version: alloc::string::String::from("1"), artifact: alloc::string::String::from("artifact"), capabilities: _x_22.clone(), command: _x_24, dependsOn: _x_25.clone(), interfaces: _x_27, health: alloc::string::String::from("healthy"), liveness: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), ports: alloc::vec::Vec::new(), readiness: alloc::vec::Vec::new(), resources: _x_34, secrets: _x_36, startup: alloc::vec::Vec::new(), volumes: alloc::vec::Vec::new(), isolation: alloc::string::String::from("value"), placement: alloc::vec::Vec::new(), platformRequirements: _x_8.clone(), scalingPolicy: None, failurePolicy: alloc::string::String::from("fail-closed"), retryPolicy: alloc::string::String::from("bounded"), degradationPolicy: alloc::string::String::from("unavailable"), idempotency: alloc::string::String::from("keyed") }; { let _x_45 = alloc::vec![_x_43]; { let _x_52 = alloc::vec![alloc::string::String::from("acceptance")]; { let _x_53 = crate::Interface { id: alloc::string::String::from("interface"), kind: alloc::string::String::from("http"), document: alloc::string::String::from("schema"), authentication: alloc::string::String::from("oidc"), compatibility: alloc::string::String::from("exact"), protocol: alloc::string::String::from("https"), errors: alloc::vec::Vec::new(), acceptance: _x_52 }; { let _x_55 = alloc::vec![_x_53]; { let _x_58 = crate::Schema { id: alloc::string::String::from("schema"), kind: alloc::string::String::from("json-schema"), value: alloc::string::String::from("{}"), dependsOn: alloc::vec::Vec::new(), compatibility: alloc::string::String::from("exact") }; { let _x_60 = alloc::vec![_x_58]; { let _x_67 = alloc::vec![alloc::string::String::from("amd64")]; { let _x_70 = crate::PlatformRequirement { id: alloc::string::String::from("platform"), os: alloc::string::String::from("linux"), architectures: _x_67, runtime: alloc::string::String::from("oci"), runtimeVersion: alloc::string::String::from("1.1"), capabilities: _x_22.clone() }; { let _x_72 = alloc::vec![_x_70]; { let _x_78 = crate::SecretReference { id: alloc::string::String::from("secret"), providerKey: alloc::string::String::from("secret://provider/key"), consumers: _x_25.clone(), rotation: alloc::string::String::from("automatic") }; { let _x_80 = alloc::vec![_x_78]; { let _x_86 = crate::Migration { id: alloc::string::String::from("migration"), kind: alloc::string::String::from("initialize"), value: alloc::string::String::from("SELECT 1;"), dependsOn: alloc::vec::Vec::new() }; { let _x_88 = alloc::vec![_x_86]; { let _x_90 = crate::Observability { logs: alloc::vec::Vec::new(), metrics: alloc::vec::Vec::new(), traces: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), redactedFields: alloc::vec::Vec::new() }; { let _x_96 = crate::Capability { id: alloc::string::String::from("capability"), kind: alloc::string::String::from("runtime"), value: alloc::string::String::from("present"), dependsOn: alloc::vec::Vec::new() }; { let _x_98 = alloc::vec![_x_96]; { let _x_103 = Some(alloc::string::String::from("secret")); { let _x_104 = Some(alloc::string::String::from("artifact")); { let _x_105 = crate::TargetBinding { id: alloc::string::String::from("target"), kind: alloc::string::String::from("compose"), apiVersion: alloc::string::String::from("1"), adapterDigest: alloc::string::String::from("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"), minimumReleaseStatus: alloc::string::String::from("development"), capabilities: _x_22.clone(), credentials: _x_103, platformRequirements: _x_8.clone(), storageClass: None, storageProfile: None, ingressClassName: None, ingressControllerArtifact: _x_104 }; { let _x_107 = alloc::vec![_x_105]; { let _x_111 = alloc::vec![alloc::string::String::from("migration")]; { let _x_112 = crate::Rollout { id: alloc::string::String::from("rollout"), kind: alloc::string::String::from("rolling"), value: alloc::string::String::from("one-at-a-time"), dependsOn: _x_111 }; { let _x_114 = alloc::vec![_x_112]; { let _x_116 = alloc::vec![alloc::string::String::from("rollout")]; { let _x_117 = { let mut __list = alloc::vec![alloc::string::String::from("migration")]; __list.extend(_x_116); __list }; { let _x_118 = crate::Rollback { id: alloc::string::String::from("rollback"), kind: alloc::string::String::from("automatic"), value: alloc::string::String::from("rollback"), dependsOn: _x_117 }; { let _x_120 = alloc::vec![_x_118]; { let _x_125 = alloc::vec![alloc::string::String::from("probe")]; { let _x_127 = true; { let _x_128 = crate::Acceptance { id: alloc::string::String::from("acceptance"), kind: alloc::string::String::from("positive"), target: alloc::string::String::from("target"), component: alloc::string::String::from("component"), command: _x_125, evidence: alloc::string::String::from("evidence"), bounded: _x_127 }; { let _x_130 = alloc::vec![_x_128]; { let _x_131 = crate::Lifecycle { backup: alloc::string::String::from("value"), drift: alloc::string::String::from("value"), migration: alloc::string::String::from("value"), recovery: alloc::string::String::from("value"), retirement: alloc::string::String::from("value"), rollback: alloc::string::String::from("value"), rollout: alloc::string::String::from("value") }; { let _x_133 = crate::TransactionalCommandView { title: alloc::string::String::from("value"), heading: alloc::string::String::from("value"), identityHeading: alloc::string::String::from("value"), commandFormHeading: alloc::string::String::from("value"), historyHeading: alloc::string::String::from("value"), principalLabel: alloc::string::String::from("value"), principalDefault: alloc::string::String::from("value"), roleLabel: alloc::string::String::from("value"), submitterRoleLabel: alloc::string::String::from("value"), observerRoleLabel: alloc::string::String::from("value"), deniedRoleLabel: alloc::string::String::from("value"), authenticateLabel: alloc::string::String::from("value"), commandIdLabel: alloc::string::String::from("value"), inputALabel: alloc::string::String::from("value"), operationLabel: alloc::string::String::from("value"), inputBLabel: alloc::string::String::from("value"), annotationLabel: alloc::string::String::from("value"), submitLabel: alloc::string::String::from("value"), refreshLabel: alloc::string::String::from("value"), emptyText: alloc::string::String::from("value"), loadingText: alloc::string::String::from("value"), authenticatedText: alloc::string::String::from("value"), accessDeniedText: alloc::string::String::from("value"), retryLabel: alloc::string::String::from("value"), sequenceHeading: alloc::string::String::from("value"), commandColumnHeading: alloc::string::String::from("value"), inputSummaryHeading: alloc::string::String::from("value"), outcomeHeading: alloc::string::String::from("value"), annotationHeading: alloc::string::String::from("value"), releasePrefix: alloc::string::String::from("value"), historyLoadedText: alloc::string::String::from("value"), historyEmptyText: alloc::string::String::from("value"), authenticatingText: alloc::string::String::from("value"), authenticationFailedText: alloc::string::String::from("value"), submittingText: alloc::string::String::from("value"), outcomePrefix: alloc::string::String::from("value"), commandFailedText: alloc::string::String::from("value"), historyFailedText: alloc::string::String::from("value") }; { let _x_134 = crate::TransactionalCommandServiceProfile { contract: alloc::string::String::from("value"), applicationErrors: alloc::vec::Vec::new(), applicationModelDigest: alloc::string::String::from("value"), commandEncoding: alloc::string::String::from("value"), responseVersion: alloc::string::String::from("value"), optionalAnnotation: alloc::string::String::from("value"), coreArtifact: alloc::string::String::from("value"), identityAudience: alloc::string::String::from("value"), submitterRole: alloc::string::String::from("value"), observerRole: alloc::string::String::from("value"), eventSource: alloc::string::String::from("value"), acceptedEventType: alloc::string::String::from("value"), rejectedEventType: alloc::string::String::from("value"), databaseName: alloc::string::String::from("value"), databaseUser: alloc::string::String::from("value"), eventExchange: alloc::string::String::from("value"), auditQueue: alloc::string::String::from("value"), commandIdPattern: alloc::string::String::from("value"), commandPath: alloc::string::String::from("value"), tokenPath: alloc::string::String::from("value"), commandOperations: alloc::vec::Vec::new(), annotationMaxScalars: _x_136, historyDefaultLimit: _x_136, historyMaxLimit: _x_136, maxRequestBytes: _x_138, availabilityThresholdMillionths: _x_138, outboxLagThresholdMillis: _x_138, commandIdField: alloc::string::String::from("value"), operationField: alloc::string::String::from("value"), inputAField: alloc::string::String::from("value"), inputBField: alloc::string::String::from("value"), annotationField: alloc::string::String::from("value"), commandIdColumn: alloc::string::String::from("value"), operationColumn: alloc::string::String::from("value"), inputAColumn: alloc::string::String::from("value"), inputBColumn: alloc::string::String::from("value"), annotationColumn: alloc::string::String::from("value"), acceptanceErrorOperation: alloc::string::String::from("value"), acceptanceErrorInputA: alloc::string::String::from("value"), acceptanceErrorInputB: alloc::string::String::from("value"), acceptanceExpectedError: alloc::string::String::from("value"), acceptanceConflictInputB: alloc::string::String::from("value"), acceptanceSuccessOperation: alloc::string::String::from("value"), acceptanceSuccessInputA: alloc::string::String::from("value"), acceptanceSuccessInputB: alloc::string::String::from("value"), acceptanceExpectedResult: alloc::string::String::from("value"), view: _x_133, brokerUser: alloc::string::String::from("value"), historyTable: alloc::string::String::from("value"), outboxTable: alloc::string::String::from("value"), auditTable: alloc::string::String::from("value"), publicHostnameParameter: alloc::string::String::from("value") }; { let _x_135 = crate::SystemModel { product: _x_9, artifacts: _x_18, components: _x_45, interfaces: _x_55, schemas: _x_60, calls: alloc::vec::Vec::new(), events: alloc::vec::Vec::new(), flows: alloc::vec::Vec::new(), topology: alloc::vec::Vec::new(), platformRequirements: _x_72, scalingPolicies: alloc::vec::Vec::new(), storageClasses: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), secretReferences: _x_80, identityRequirements: alloc::vec::Vec::new(), persistence: alloc::vec::Vec::new(), migrations: _x_88, backups: alloc::vec::Vec::new(), observability: _x_90, slis: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), controls: alloc::vec::Vec::new(), capabilities: _x_98, architecture: alloc::vec::Vec::new(), targets: _x_107, rollouts: _x_114, rollbacks: _x_120, drifts: alloc::vec::Vec::new(), retirements: alloc::vec::Vec::new(), acceptance: _x_130, lifecycle: _x_131, applicationProfile: _x_134, standards: alloc::vec::Vec::new() }; _x_135 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn corpusInvalidEvidenceClosureModel() -> crate::SystemModel {
    { let _x_8 = alloc::vec![alloc::string::String::from("platform")]; { let _x_9 = crate::Product { id: alloc::string::String::from("product"), owner: alloc::string::String::from("PrismPM"), version: alloc::string::String::from("1"), lifecycle: alloc::string::String::from("development"), sourcePolicy: alloc::string::String::from("closed"), supportedPlatforms: _x_8.clone() }; { let _x_16 = crate::Artifact { id: alloc::string::String::from("artifact"), mediaType: alloc::string::String::from("application/octet-stream"), digest: alloc::string::String::from("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), licenseExpression: alloc::string::String::from("MIT"), path: alloc::string::String::from("artifact.bin"), role: alloc::string::String::from("runtime"), platformRequirements: _x_8.clone() }; { let _x_18 = alloc::vec![_x_16]; { let _x_22 = alloc::vec![alloc::string::String::from("capability")]; { let _x_24 = alloc::vec![alloc::string::String::from("run")]; { let _x_26 = alloc::vec![alloc::string::String::from("interface")]; { let _x_136 = 1; { let _x_138 = 1; { let _x_33 = crate::ResourceRequirements { cpuMillis: _x_136, memoryBytes: _x_138, replicasMin: _x_136, replicasMax: _x_136 }; { let _x_35 = alloc::vec![alloc::string::String::from("secret")]; { let _x_42 = crate::Component { id: alloc::string::String::from("component"), kind: alloc::string::String::from("service"), version: alloc::string::String::from("1"), artifact: alloc::string::String::from("artifact"), capabilities: _x_22.clone(), command: _x_24, dependsOn: alloc::vec::Vec::new(), interfaces: _x_26, health: alloc::string::String::from("healthy"), liveness: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), ports: alloc::vec::Vec::new(), readiness: alloc::vec::Vec::new(), resources: _x_33, secrets: _x_35, startup: alloc::vec::Vec::new(), volumes: alloc::vec::Vec::new(), isolation: alloc::string::String::from("value"), placement: alloc::vec::Vec::new(), platformRequirements: _x_8.clone(), scalingPolicy: None, failurePolicy: alloc::string::String::from("fail-closed"), retryPolicy: alloc::string::String::from("bounded"), degradationPolicy: alloc::string::String::from("unavailable"), idempotency: alloc::string::String::from("keyed") }; { let _x_44 = alloc::vec![_x_42]; { let _x_51 = alloc::vec![alloc::string::String::from("acceptance")]; { let _x_52 = crate::Interface { id: alloc::string::String::from("interface"), kind: alloc::string::String::from("http"), document: alloc::string::String::from("schema"), authentication: alloc::string::String::from("oidc"), compatibility: alloc::string::String::from("exact"), protocol: alloc::string::String::from("https"), errors: alloc::vec::Vec::new(), acceptance: _x_51 }; { let _x_54 = alloc::vec![_x_52]; { let _x_57 = crate::Schema { id: alloc::string::String::from("schema"), kind: alloc::string::String::from("json-schema"), value: alloc::string::String::from("{}"), dependsOn: alloc::vec::Vec::new(), compatibility: alloc::string::String::from("exact") }; { let _x_59 = alloc::vec![_x_57]; { let _x_66 = alloc::vec![alloc::string::String::from("amd64")]; { let _x_69 = crate::PlatformRequirement { id: alloc::string::String::from("platform"), os: alloc::string::String::from("linux"), architectures: _x_66, runtime: alloc::string::String::from("oci"), runtimeVersion: alloc::string::String::from("1.1"), capabilities: _x_22.clone() }; { let _x_71 = alloc::vec![_x_69]; { let _x_76 = alloc::vec![alloc::string::String::from("component")]; { let _x_78 = crate::SecretReference { id: alloc::string::String::from("secret"), providerKey: alloc::string::String::from("secret://provider/key"), consumers: _x_76, rotation: alloc::string::String::from("automatic") }; { let _x_80 = alloc::vec![_x_78]; { let _x_86 = crate::Migration { id: alloc::string::String::from("migration"), kind: alloc::string::String::from("initialize"), value: alloc::string::String::from("SELECT 1;"), dependsOn: alloc::vec::Vec::new() }; { let _x_88 = alloc::vec![_x_86]; { let _x_90 = crate::Observability { logs: alloc::vec::Vec::new(), metrics: alloc::vec::Vec::new(), traces: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), redactedFields: alloc::vec::Vec::new() }; { let _x_96 = crate::Capability { id: alloc::string::String::from("capability"), kind: alloc::string::String::from("runtime"), value: alloc::string::String::from("present"), dependsOn: alloc::vec::Vec::new() }; { let _x_98 = alloc::vec![_x_96]; { let _x_103 = Some(alloc::string::String::from("secret")); { let _x_104 = Some(alloc::string::String::from("artifact")); { let _x_105 = crate::TargetBinding { id: alloc::string::String::from("target"), kind: alloc::string::String::from("compose"), apiVersion: alloc::string::String::from("1"), adapterDigest: alloc::string::String::from("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"), minimumReleaseStatus: alloc::string::String::from("development"), capabilities: _x_22.clone(), credentials: _x_103, platformRequirements: _x_8.clone(), storageClass: None, storageProfile: None, ingressClassName: None, ingressControllerArtifact: _x_104 }; { let _x_107 = alloc::vec![_x_105]; { let _x_111 = alloc::vec![alloc::string::String::from("migration")]; { let _x_112 = crate::Rollout { id: alloc::string::String::from("rollout"), kind: alloc::string::String::from("rolling"), value: alloc::string::String::from("one-at-a-time"), dependsOn: _x_111 }; { let _x_114 = alloc::vec![_x_112]; { let _x_116 = alloc::vec![alloc::string::String::from("rollout")]; { let _x_117 = { let mut __list = alloc::vec![alloc::string::String::from("migration")]; __list.extend(_x_116); __list }; { let _x_118 = crate::Rollback { id: alloc::string::String::from("rollback"), kind: alloc::string::String::from("automatic"), value: alloc::string::String::from("rollback"), dependsOn: _x_117 }; { let _x_120 = alloc::vec![_x_118]; { let _x_125 = alloc::vec![alloc::string::String::from("probe")]; { let _x_127 = false; { let _x_128 = crate::Acceptance { id: alloc::string::String::from("acceptance"), kind: alloc::string::String::from("positive"), target: alloc::string::String::from("target"), component: alloc::string::String::from("component"), command: _x_125, evidence: alloc::string::String::from("evidence"), bounded: _x_127 }; { let _x_130 = alloc::vec![_x_128]; { let _x_131 = crate::Lifecycle { backup: alloc::string::String::from("value"), drift: alloc::string::String::from("value"), migration: alloc::string::String::from("value"), recovery: alloc::string::String::from("value"), retirement: alloc::string::String::from("value"), rollback: alloc::string::String::from("value"), rollout: alloc::string::String::from("value") }; { let _x_133 = crate::TransactionalCommandView { title: alloc::string::String::from("value"), heading: alloc::string::String::from("value"), identityHeading: alloc::string::String::from("value"), commandFormHeading: alloc::string::String::from("value"), historyHeading: alloc::string::String::from("value"), principalLabel: alloc::string::String::from("value"), principalDefault: alloc::string::String::from("value"), roleLabel: alloc::string::String::from("value"), submitterRoleLabel: alloc::string::String::from("value"), observerRoleLabel: alloc::string::String::from("value"), deniedRoleLabel: alloc::string::String::from("value"), authenticateLabel: alloc::string::String::from("value"), commandIdLabel: alloc::string::String::from("value"), inputALabel: alloc::string::String::from("value"), operationLabel: alloc::string::String::from("value"), inputBLabel: alloc::string::String::from("value"), annotationLabel: alloc::string::String::from("value"), submitLabel: alloc::string::String::from("value"), refreshLabel: alloc::string::String::from("value"), emptyText: alloc::string::String::from("value"), loadingText: alloc::string::String::from("value"), authenticatedText: alloc::string::String::from("value"), accessDeniedText: alloc::string::String::from("value"), retryLabel: alloc::string::String::from("value"), sequenceHeading: alloc::string::String::from("value"), commandColumnHeading: alloc::string::String::from("value"), inputSummaryHeading: alloc::string::String::from("value"), outcomeHeading: alloc::string::String::from("value"), annotationHeading: alloc::string::String::from("value"), releasePrefix: alloc::string::String::from("value"), historyLoadedText: alloc::string::String::from("value"), historyEmptyText: alloc::string::String::from("value"), authenticatingText: alloc::string::String::from("value"), authenticationFailedText: alloc::string::String::from("value"), submittingText: alloc::string::String::from("value"), outcomePrefix: alloc::string::String::from("value"), commandFailedText: alloc::string::String::from("value"), historyFailedText: alloc::string::String::from("value") }; { let _x_134 = crate::TransactionalCommandServiceProfile { contract: alloc::string::String::from("value"), applicationErrors: alloc::vec::Vec::new(), applicationModelDigest: alloc::string::String::from("value"), commandEncoding: alloc::string::String::from("value"), responseVersion: alloc::string::String::from("value"), optionalAnnotation: alloc::string::String::from("value"), coreArtifact: alloc::string::String::from("value"), identityAudience: alloc::string::String::from("value"), submitterRole: alloc::string::String::from("value"), observerRole: alloc::string::String::from("value"), eventSource: alloc::string::String::from("value"), acceptedEventType: alloc::string::String::from("value"), rejectedEventType: alloc::string::String::from("value"), databaseName: alloc::string::String::from("value"), databaseUser: alloc::string::String::from("value"), eventExchange: alloc::string::String::from("value"), auditQueue: alloc::string::String::from("value"), commandIdPattern: alloc::string::String::from("value"), commandPath: alloc::string::String::from("value"), tokenPath: alloc::string::String::from("value"), commandOperations: alloc::vec::Vec::new(), annotationMaxScalars: _x_136, historyDefaultLimit: _x_136, historyMaxLimit: _x_136, maxRequestBytes: _x_138, availabilityThresholdMillionths: _x_138, outboxLagThresholdMillis: _x_138, commandIdField: alloc::string::String::from("value"), operationField: alloc::string::String::from("value"), inputAField: alloc::string::String::from("value"), inputBField: alloc::string::String::from("value"), annotationField: alloc::string::String::from("value"), commandIdColumn: alloc::string::String::from("value"), operationColumn: alloc::string::String::from("value"), inputAColumn: alloc::string::String::from("value"), inputBColumn: alloc::string::String::from("value"), annotationColumn: alloc::string::String::from("value"), acceptanceErrorOperation: alloc::string::String::from("value"), acceptanceErrorInputA: alloc::string::String::from("value"), acceptanceErrorInputB: alloc::string::String::from("value"), acceptanceExpectedError: alloc::string::String::from("value"), acceptanceConflictInputB: alloc::string::String::from("value"), acceptanceSuccessOperation: alloc::string::String::from("value"), acceptanceSuccessInputA: alloc::string::String::from("value"), acceptanceSuccessInputB: alloc::string::String::from("value"), acceptanceExpectedResult: alloc::string::String::from("value"), view: _x_133, brokerUser: alloc::string::String::from("value"), historyTable: alloc::string::String::from("value"), outboxTable: alloc::string::String::from("value"), auditTable: alloc::string::String::from("value"), publicHostnameParameter: alloc::string::String::from("value") }; { let _x_135 = crate::SystemModel { product: _x_9, artifacts: _x_18, components: _x_44, interfaces: _x_54, schemas: _x_59, calls: alloc::vec::Vec::new(), events: alloc::vec::Vec::new(), flows: alloc::vec::Vec::new(), topology: alloc::vec::Vec::new(), platformRequirements: _x_71, scalingPolicies: alloc::vec::Vec::new(), storageClasses: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), secretReferences: _x_80, identityRequirements: alloc::vec::Vec::new(), persistence: alloc::vec::Vec::new(), migrations: _x_88, backups: alloc::vec::Vec::new(), observability: _x_90, slis: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), controls: alloc::vec::Vec::new(), capabilities: _x_98, architecture: alloc::vec::Vec::new(), targets: _x_107, rollouts: _x_114, rollbacks: _x_120, drifts: alloc::vec::Vec::new(), retirements: alloc::vec::Vec::new(), acceptance: _x_130, lifecycle: _x_131, applicationProfile: _x_134, standards: alloc::vec::Vec::new() }; _x_135 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn corpusInvalidLicenseClosureModel() -> crate::SystemModel {
    { let _x_8 = alloc::vec![alloc::string::String::from("platform")]; { let _x_9 = crate::Product { id: alloc::string::String::from("product"), owner: alloc::string::String::from("PrismPM"), version: alloc::string::String::from("1"), lifecycle: alloc::string::String::from("development"), sourcePolicy: alloc::string::String::from("closed"), supportedPlatforms: _x_8.clone() }; { let _x_16 = crate::Artifact { id: alloc::string::String::from("artifact"), mediaType: alloc::string::String::from("application/octet-stream"), digest: alloc::string::String::from("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), licenseExpression: alloc::string::String::from(""), path: alloc::string::String::from("artifact.bin"), role: alloc::string::String::from("runtime"), platformRequirements: _x_8.clone() }; { let _x_18 = alloc::vec![_x_16]; { let _x_22 = alloc::vec![alloc::string::String::from("capability")]; { let _x_24 = alloc::vec![alloc::string::String::from("run")]; { let _x_26 = alloc::vec![alloc::string::String::from("interface")]; { let _x_136 = 1; { let _x_138 = 1; { let _x_33 = crate::ResourceRequirements { cpuMillis: _x_136, memoryBytes: _x_138, replicasMin: _x_136, replicasMax: _x_136 }; { let _x_35 = alloc::vec![alloc::string::String::from("secret")]; { let _x_42 = crate::Component { id: alloc::string::String::from("component"), kind: alloc::string::String::from("service"), version: alloc::string::String::from("1"), artifact: alloc::string::String::from("artifact"), capabilities: _x_22.clone(), command: _x_24, dependsOn: alloc::vec::Vec::new(), interfaces: _x_26, health: alloc::string::String::from("healthy"), liveness: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), ports: alloc::vec::Vec::new(), readiness: alloc::vec::Vec::new(), resources: _x_33, secrets: _x_35, startup: alloc::vec::Vec::new(), volumes: alloc::vec::Vec::new(), isolation: alloc::string::String::from("value"), placement: alloc::vec::Vec::new(), platformRequirements: _x_8.clone(), scalingPolicy: None, failurePolicy: alloc::string::String::from("fail-closed"), retryPolicy: alloc::string::String::from("bounded"), degradationPolicy: alloc::string::String::from("unavailable"), idempotency: alloc::string::String::from("keyed") }; { let _x_44 = alloc::vec![_x_42]; { let _x_51 = alloc::vec![alloc::string::String::from("acceptance")]; { let _x_52 = crate::Interface { id: alloc::string::String::from("interface"), kind: alloc::string::String::from("http"), document: alloc::string::String::from("schema"), authentication: alloc::string::String::from("oidc"), compatibility: alloc::string::String::from("exact"), protocol: alloc::string::String::from("https"), errors: alloc::vec::Vec::new(), acceptance: _x_51 }; { let _x_54 = alloc::vec![_x_52]; { let _x_57 = crate::Schema { id: alloc::string::String::from("schema"), kind: alloc::string::String::from("json-schema"), value: alloc::string::String::from("{}"), dependsOn: alloc::vec::Vec::new(), compatibility: alloc::string::String::from("exact") }; { let _x_59 = alloc::vec![_x_57]; { let _x_66 = alloc::vec![alloc::string::String::from("amd64")]; { let _x_69 = crate::PlatformRequirement { id: alloc::string::String::from("platform"), os: alloc::string::String::from("linux"), architectures: _x_66, runtime: alloc::string::String::from("oci"), runtimeVersion: alloc::string::String::from("1.1"), capabilities: _x_22.clone() }; { let _x_71 = alloc::vec![_x_69]; { let _x_76 = alloc::vec![alloc::string::String::from("component")]; { let _x_78 = crate::SecretReference { id: alloc::string::String::from("secret"), providerKey: alloc::string::String::from("secret://provider/key"), consumers: _x_76, rotation: alloc::string::String::from("automatic") }; { let _x_80 = alloc::vec![_x_78]; { let _x_86 = crate::Migration { id: alloc::string::String::from("migration"), kind: alloc::string::String::from("initialize"), value: alloc::string::String::from("SELECT 1;"), dependsOn: alloc::vec::Vec::new() }; { let _x_88 = alloc::vec![_x_86]; { let _x_90 = crate::Observability { logs: alloc::vec::Vec::new(), metrics: alloc::vec::Vec::new(), traces: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), redactedFields: alloc::vec::Vec::new() }; { let _x_96 = crate::Capability { id: alloc::string::String::from("capability"), kind: alloc::string::String::from("runtime"), value: alloc::string::String::from("present"), dependsOn: alloc::vec::Vec::new() }; { let _x_98 = alloc::vec![_x_96]; { let _x_103 = Some(alloc::string::String::from("secret")); { let _x_104 = Some(alloc::string::String::from("artifact")); { let _x_105 = crate::TargetBinding { id: alloc::string::String::from("target"), kind: alloc::string::String::from("compose"), apiVersion: alloc::string::String::from("1"), adapterDigest: alloc::string::String::from("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"), minimumReleaseStatus: alloc::string::String::from("development"), capabilities: _x_22.clone(), credentials: _x_103, platformRequirements: _x_8.clone(), storageClass: None, storageProfile: None, ingressClassName: None, ingressControllerArtifact: _x_104 }; { let _x_107 = alloc::vec![_x_105]; { let _x_111 = alloc::vec![alloc::string::String::from("migration")]; { let _x_112 = crate::Rollout { id: alloc::string::String::from("rollout"), kind: alloc::string::String::from("rolling"), value: alloc::string::String::from("one-at-a-time"), dependsOn: _x_111 }; { let _x_114 = alloc::vec![_x_112]; { let _x_116 = alloc::vec![alloc::string::String::from("rollout")]; { let _x_117 = { let mut __list = alloc::vec![alloc::string::String::from("migration")]; __list.extend(_x_116); __list }; { let _x_118 = crate::Rollback { id: alloc::string::String::from("rollback"), kind: alloc::string::String::from("automatic"), value: alloc::string::String::from("rollback"), dependsOn: _x_117 }; { let _x_120 = alloc::vec![_x_118]; { let _x_125 = alloc::vec![alloc::string::String::from("probe")]; { let _x_127 = true; { let _x_128 = crate::Acceptance { id: alloc::string::String::from("acceptance"), kind: alloc::string::String::from("positive"), target: alloc::string::String::from("target"), component: alloc::string::String::from("component"), command: _x_125, evidence: alloc::string::String::from("evidence"), bounded: _x_127 }; { let _x_130 = alloc::vec![_x_128]; { let _x_131 = crate::Lifecycle { backup: alloc::string::String::from("value"), drift: alloc::string::String::from("value"), migration: alloc::string::String::from("value"), recovery: alloc::string::String::from("value"), retirement: alloc::string::String::from("value"), rollback: alloc::string::String::from("value"), rollout: alloc::string::String::from("value") }; { let _x_133 = crate::TransactionalCommandView { title: alloc::string::String::from("value"), heading: alloc::string::String::from("value"), identityHeading: alloc::string::String::from("value"), commandFormHeading: alloc::string::String::from("value"), historyHeading: alloc::string::String::from("value"), principalLabel: alloc::string::String::from("value"), principalDefault: alloc::string::String::from("value"), roleLabel: alloc::string::String::from("value"), submitterRoleLabel: alloc::string::String::from("value"), observerRoleLabel: alloc::string::String::from("value"), deniedRoleLabel: alloc::string::String::from("value"), authenticateLabel: alloc::string::String::from("value"), commandIdLabel: alloc::string::String::from("value"), inputALabel: alloc::string::String::from("value"), operationLabel: alloc::string::String::from("value"), inputBLabel: alloc::string::String::from("value"), annotationLabel: alloc::string::String::from("value"), submitLabel: alloc::string::String::from("value"), refreshLabel: alloc::string::String::from("value"), emptyText: alloc::string::String::from("value"), loadingText: alloc::string::String::from("value"), authenticatedText: alloc::string::String::from("value"), accessDeniedText: alloc::string::String::from("value"), retryLabel: alloc::string::String::from("value"), sequenceHeading: alloc::string::String::from("value"), commandColumnHeading: alloc::string::String::from("value"), inputSummaryHeading: alloc::string::String::from("value"), outcomeHeading: alloc::string::String::from("value"), annotationHeading: alloc::string::String::from("value"), releasePrefix: alloc::string::String::from("value"), historyLoadedText: alloc::string::String::from("value"), historyEmptyText: alloc::string::String::from("value"), authenticatingText: alloc::string::String::from("value"), authenticationFailedText: alloc::string::String::from("value"), submittingText: alloc::string::String::from("value"), outcomePrefix: alloc::string::String::from("value"), commandFailedText: alloc::string::String::from("value"), historyFailedText: alloc::string::String::from("value") }; { let _x_134 = crate::TransactionalCommandServiceProfile { contract: alloc::string::String::from("value"), applicationErrors: alloc::vec::Vec::new(), applicationModelDigest: alloc::string::String::from("value"), commandEncoding: alloc::string::String::from("value"), responseVersion: alloc::string::String::from("value"), optionalAnnotation: alloc::string::String::from("value"), coreArtifact: alloc::string::String::from("value"), identityAudience: alloc::string::String::from("value"), submitterRole: alloc::string::String::from("value"), observerRole: alloc::string::String::from("value"), eventSource: alloc::string::String::from("value"), acceptedEventType: alloc::string::String::from("value"), rejectedEventType: alloc::string::String::from("value"), databaseName: alloc::string::String::from("value"), databaseUser: alloc::string::String::from("value"), eventExchange: alloc::string::String::from("value"), auditQueue: alloc::string::String::from("value"), commandIdPattern: alloc::string::String::from("value"), commandPath: alloc::string::String::from("value"), tokenPath: alloc::string::String::from("value"), commandOperations: alloc::vec::Vec::new(), annotationMaxScalars: _x_136, historyDefaultLimit: _x_136, historyMaxLimit: _x_136, maxRequestBytes: _x_138, availabilityThresholdMillionths: _x_138, outboxLagThresholdMillis: _x_138, commandIdField: alloc::string::String::from("value"), operationField: alloc::string::String::from("value"), inputAField: alloc::string::String::from("value"), inputBField: alloc::string::String::from("value"), annotationField: alloc::string::String::from("value"), commandIdColumn: alloc::string::String::from("value"), operationColumn: alloc::string::String::from("value"), inputAColumn: alloc::string::String::from("value"), inputBColumn: alloc::string::String::from("value"), annotationColumn: alloc::string::String::from("value"), acceptanceErrorOperation: alloc::string::String::from("value"), acceptanceErrorInputA: alloc::string::String::from("value"), acceptanceErrorInputB: alloc::string::String::from("value"), acceptanceExpectedError: alloc::string::String::from("value"), acceptanceConflictInputB: alloc::string::String::from("value"), acceptanceSuccessOperation: alloc::string::String::from("value"), acceptanceSuccessInputA: alloc::string::String::from("value"), acceptanceSuccessInputB: alloc::string::String::from("value"), acceptanceExpectedResult: alloc::string::String::from("value"), view: _x_133, brokerUser: alloc::string::String::from("value"), historyTable: alloc::string::String::from("value"), outboxTable: alloc::string::String::from("value"), auditTable: alloc::string::String::from("value"), publicHostnameParameter: alloc::string::String::from("value") }; { let _x_135 = crate::SystemModel { product: _x_9, artifacts: _x_18, components: _x_44, interfaces: _x_54, schemas: _x_59, calls: alloc::vec::Vec::new(), events: alloc::vec::Vec::new(), flows: alloc::vec::Vec::new(), topology: alloc::vec::Vec::new(), platformRequirements: _x_71, scalingPolicies: alloc::vec::Vec::new(), storageClasses: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), secretReferences: _x_80, identityRequirements: alloc::vec::Vec::new(), persistence: alloc::vec::Vec::new(), migrations: _x_88, backups: alloc::vec::Vec::new(), observability: _x_90, slis: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), controls: alloc::vec::Vec::new(), capabilities: _x_98, architecture: alloc::vec::Vec::new(), targets: _x_107, rollouts: _x_114, rollbacks: _x_120, drifts: alloc::vec::Vec::new(), retirements: alloc::vec::Vec::new(), acceptance: _x_130, lifecycle: _x_131, applicationProfile: _x_134, standards: alloc::vec::Vec::new() }; _x_135 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn corpusInvalidMigrationOrderModel() -> crate::SystemModel {
    { let _x_8 = alloc::vec![alloc::string::String::from("platform")]; { let _x_9 = crate::Product { id: alloc::string::String::from("product"), owner: alloc::string::String::from("PrismPM"), version: alloc::string::String::from("1"), lifecycle: alloc::string::String::from("development"), sourcePolicy: alloc::string::String::from("closed"), supportedPlatforms: _x_8.clone() }; { let _x_16 = crate::Artifact { id: alloc::string::String::from("artifact"), mediaType: alloc::string::String::from("application/octet-stream"), digest: alloc::string::String::from("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), licenseExpression: alloc::string::String::from("MIT"), path: alloc::string::String::from("artifact.bin"), role: alloc::string::String::from("runtime"), platformRequirements: _x_8.clone() }; { let _x_18 = alloc::vec![_x_16]; { let _x_22 = alloc::vec![alloc::string::String::from("capability")]; { let _x_24 = alloc::vec![alloc::string::String::from("run")]; { let _x_26 = alloc::vec![alloc::string::String::from("interface")]; { let _x_136 = 1; { let _x_138 = 1; { let _x_33 = crate::ResourceRequirements { cpuMillis: _x_136, memoryBytes: _x_138, replicasMin: _x_136, replicasMax: _x_136 }; { let _x_35 = alloc::vec![alloc::string::String::from("secret")]; { let _x_42 = crate::Component { id: alloc::string::String::from("component"), kind: alloc::string::String::from("service"), version: alloc::string::String::from("1"), artifact: alloc::string::String::from("artifact"), capabilities: _x_22.clone(), command: _x_24, dependsOn: alloc::vec::Vec::new(), interfaces: _x_26, health: alloc::string::String::from("healthy"), liveness: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), ports: alloc::vec::Vec::new(), readiness: alloc::vec::Vec::new(), resources: _x_33, secrets: _x_35, startup: alloc::vec::Vec::new(), volumes: alloc::vec::Vec::new(), isolation: alloc::string::String::from("value"), placement: alloc::vec::Vec::new(), platformRequirements: _x_8.clone(), scalingPolicy: None, failurePolicy: alloc::string::String::from("fail-closed"), retryPolicy: alloc::string::String::from("bounded"), degradationPolicy: alloc::string::String::from("unavailable"), idempotency: alloc::string::String::from("keyed") }; { let _x_44 = alloc::vec![_x_42]; { let _x_51 = alloc::vec![alloc::string::String::from("acceptance")]; { let _x_52 = crate::Interface { id: alloc::string::String::from("interface"), kind: alloc::string::String::from("http"), document: alloc::string::String::from("schema"), authentication: alloc::string::String::from("oidc"), compatibility: alloc::string::String::from("exact"), protocol: alloc::string::String::from("https"), errors: alloc::vec::Vec::new(), acceptance: _x_51 }; { let _x_54 = alloc::vec![_x_52]; { let _x_57 = crate::Schema { id: alloc::string::String::from("schema"), kind: alloc::string::String::from("json-schema"), value: alloc::string::String::from("{}"), dependsOn: alloc::vec::Vec::new(), compatibility: alloc::string::String::from("exact") }; { let _x_59 = alloc::vec![_x_57]; { let _x_66 = alloc::vec![alloc::string::String::from("amd64")]; { let _x_69 = crate::PlatformRequirement { id: alloc::string::String::from("platform"), os: alloc::string::String::from("linux"), architectures: _x_66, runtime: alloc::string::String::from("oci"), runtimeVersion: alloc::string::String::from("1.1"), capabilities: _x_22.clone() }; { let _x_71 = alloc::vec![_x_69]; { let _x_76 = alloc::vec![alloc::string::String::from("component")]; { let _x_78 = crate::SecretReference { id: alloc::string::String::from("secret"), providerKey: alloc::string::String::from("secret://provider/key"), consumers: _x_76, rotation: alloc::string::String::from("automatic") }; { let _x_80 = alloc::vec![_x_78]; { let _x_86 = alloc::vec![alloc::string::String::from("migration")]; { let _x_87 = crate::Migration { id: alloc::string::String::from("migration"), kind: alloc::string::String::from("initialize"), value: alloc::string::String::from("SELECT 1;"), dependsOn: _x_86.clone() }; { let _x_89 = alloc::vec![_x_87]; { let _x_91 = crate::Observability { logs: alloc::vec::Vec::new(), metrics: alloc::vec::Vec::new(), traces: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), redactedFields: alloc::vec::Vec::new() }; { let _x_97 = crate::Capability { id: alloc::string::String::from("capability"), kind: alloc::string::String::from("runtime"), value: alloc::string::String::from("present"), dependsOn: alloc::vec::Vec::new() }; { let _x_99 = alloc::vec![_x_97]; { let _x_104 = Some(alloc::string::String::from("secret")); { let _x_105 = Some(alloc::string::String::from("artifact")); { let _x_106 = crate::TargetBinding { id: alloc::string::String::from("target"), kind: alloc::string::String::from("compose"), apiVersion: alloc::string::String::from("1"), adapterDigest: alloc::string::String::from("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"), minimumReleaseStatus: alloc::string::String::from("development"), capabilities: _x_22.clone(), credentials: _x_104, platformRequirements: _x_8.clone(), storageClass: None, storageProfile: None, ingressClassName: None, ingressControllerArtifact: _x_105 }; { let _x_108 = alloc::vec![_x_106]; { let _x_112 = crate::Rollout { id: alloc::string::String::from("rollout"), kind: alloc::string::String::from("rolling"), value: alloc::string::String::from("one-at-a-time"), dependsOn: _x_86.clone() }; { let _x_114 = alloc::vec![_x_112]; { let _x_116 = alloc::vec![alloc::string::String::from("rollout")]; { let _x_117 = { let mut __list = alloc::vec![alloc::string::String::from("migration")]; __list.extend(_x_116); __list }; { let _x_118 = crate::Rollback { id: alloc::string::String::from("rollback"), kind: alloc::string::String::from("automatic"), value: alloc::string::String::from("rollback"), dependsOn: _x_117 }; { let _x_120 = alloc::vec![_x_118]; { let _x_125 = alloc::vec![alloc::string::String::from("probe")]; { let _x_127 = true; { let _x_128 = crate::Acceptance { id: alloc::string::String::from("acceptance"), kind: alloc::string::String::from("positive"), target: alloc::string::String::from("target"), component: alloc::string::String::from("component"), command: _x_125, evidence: alloc::string::String::from("evidence"), bounded: _x_127 }; { let _x_130 = alloc::vec![_x_128]; { let _x_131 = crate::Lifecycle { backup: alloc::string::String::from("value"), drift: alloc::string::String::from("value"), migration: alloc::string::String::from("value"), recovery: alloc::string::String::from("value"), retirement: alloc::string::String::from("value"), rollback: alloc::string::String::from("value"), rollout: alloc::string::String::from("value") }; { let _x_133 = crate::TransactionalCommandView { title: alloc::string::String::from("value"), heading: alloc::string::String::from("value"), identityHeading: alloc::string::String::from("value"), commandFormHeading: alloc::string::String::from("value"), historyHeading: alloc::string::String::from("value"), principalLabel: alloc::string::String::from("value"), principalDefault: alloc::string::String::from("value"), roleLabel: alloc::string::String::from("value"), submitterRoleLabel: alloc::string::String::from("value"), observerRoleLabel: alloc::string::String::from("value"), deniedRoleLabel: alloc::string::String::from("value"), authenticateLabel: alloc::string::String::from("value"), commandIdLabel: alloc::string::String::from("value"), inputALabel: alloc::string::String::from("value"), operationLabel: alloc::string::String::from("value"), inputBLabel: alloc::string::String::from("value"), annotationLabel: alloc::string::String::from("value"), submitLabel: alloc::string::String::from("value"), refreshLabel: alloc::string::String::from("value"), emptyText: alloc::string::String::from("value"), loadingText: alloc::string::String::from("value"), authenticatedText: alloc::string::String::from("value"), accessDeniedText: alloc::string::String::from("value"), retryLabel: alloc::string::String::from("value"), sequenceHeading: alloc::string::String::from("value"), commandColumnHeading: alloc::string::String::from("value"), inputSummaryHeading: alloc::string::String::from("value"), outcomeHeading: alloc::string::String::from("value"), annotationHeading: alloc::string::String::from("value"), releasePrefix: alloc::string::String::from("value"), historyLoadedText: alloc::string::String::from("value"), historyEmptyText: alloc::string::String::from("value"), authenticatingText: alloc::string::String::from("value"), authenticationFailedText: alloc::string::String::from("value"), submittingText: alloc::string::String::from("value"), outcomePrefix: alloc::string::String::from("value"), commandFailedText: alloc::string::String::from("value"), historyFailedText: alloc::string::String::from("value") }; { let _x_134 = crate::TransactionalCommandServiceProfile { contract: alloc::string::String::from("value"), applicationErrors: alloc::vec::Vec::new(), applicationModelDigest: alloc::string::String::from("value"), commandEncoding: alloc::string::String::from("value"), responseVersion: alloc::string::String::from("value"), optionalAnnotation: alloc::string::String::from("value"), coreArtifact: alloc::string::String::from("value"), identityAudience: alloc::string::String::from("value"), submitterRole: alloc::string::String::from("value"), observerRole: alloc::string::String::from("value"), eventSource: alloc::string::String::from("value"), acceptedEventType: alloc::string::String::from("value"), rejectedEventType: alloc::string::String::from("value"), databaseName: alloc::string::String::from("value"), databaseUser: alloc::string::String::from("value"), eventExchange: alloc::string::String::from("value"), auditQueue: alloc::string::String::from("value"), commandIdPattern: alloc::string::String::from("value"), commandPath: alloc::string::String::from("value"), tokenPath: alloc::string::String::from("value"), commandOperations: alloc::vec::Vec::new(), annotationMaxScalars: _x_136, historyDefaultLimit: _x_136, historyMaxLimit: _x_136, maxRequestBytes: _x_138, availabilityThresholdMillionths: _x_138, outboxLagThresholdMillis: _x_138, commandIdField: alloc::string::String::from("value"), operationField: alloc::string::String::from("value"), inputAField: alloc::string::String::from("value"), inputBField: alloc::string::String::from("value"), annotationField: alloc::string::String::from("value"), commandIdColumn: alloc::string::String::from("value"), operationColumn: alloc::string::String::from("value"), inputAColumn: alloc::string::String::from("value"), inputBColumn: alloc::string::String::from("value"), annotationColumn: alloc::string::String::from("value"), acceptanceErrorOperation: alloc::string::String::from("value"), acceptanceErrorInputA: alloc::string::String::from("value"), acceptanceErrorInputB: alloc::string::String::from("value"), acceptanceExpectedError: alloc::string::String::from("value"), acceptanceConflictInputB: alloc::string::String::from("value"), acceptanceSuccessOperation: alloc::string::String::from("value"), acceptanceSuccessInputA: alloc::string::String::from("value"), acceptanceSuccessInputB: alloc::string::String::from("value"), acceptanceExpectedResult: alloc::string::String::from("value"), view: _x_133, brokerUser: alloc::string::String::from("value"), historyTable: alloc::string::String::from("value"), outboxTable: alloc::string::String::from("value"), auditTable: alloc::string::String::from("value"), publicHostnameParameter: alloc::string::String::from("value") }; { let _x_135 = crate::SystemModel { product: _x_9, artifacts: _x_18, components: _x_44, interfaces: _x_54, schemas: _x_59, calls: alloc::vec::Vec::new(), events: alloc::vec::Vec::new(), flows: alloc::vec::Vec::new(), topology: alloc::vec::Vec::new(), platformRequirements: _x_71, scalingPolicies: alloc::vec::Vec::new(), storageClasses: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), secretReferences: _x_80, identityRequirements: alloc::vec::Vec::new(), persistence: alloc::vec::Vec::new(), migrations: _x_89, backups: alloc::vec::Vec::new(), observability: _x_91, slis: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), controls: alloc::vec::Vec::new(), capabilities: _x_99, architecture: alloc::vec::Vec::new(), targets: _x_108, rollouts: _x_114, rollbacks: _x_120, drifts: alloc::vec::Vec::new(), retirements: alloc::vec::Vec::new(), acceptance: _x_130, lifecycle: _x_131, applicationProfile: _x_134, standards: alloc::vec::Vec::new() }; _x_135 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn corpusInvalidReferentialIntegrityModel() -> crate::SystemModel {
    { let _x_8 = alloc::vec![alloc::string::String::from("platform")]; { let _x_9 = crate::Product { id: alloc::string::String::from("product"), owner: alloc::string::String::from("PrismPM"), version: alloc::string::String::from("1"), lifecycle: alloc::string::String::from("development"), sourcePolicy: alloc::string::String::from("closed"), supportedPlatforms: _x_8.clone() }; { let _x_16 = crate::Artifact { id: alloc::string::String::from("artifact"), mediaType: alloc::string::String::from("application/octet-stream"), digest: alloc::string::String::from("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), licenseExpression: alloc::string::String::from("MIT"), path: alloc::string::String::from("artifact.bin"), role: alloc::string::String::from("runtime"), platformRequirements: _x_8.clone() }; { let _x_18 = alloc::vec![_x_16]; { let _x_22 = alloc::vec![alloc::string::String::from("capability")]; { let _x_24 = alloc::vec![alloc::string::String::from("run")]; { let _x_26 = alloc::vec![alloc::string::String::from("interface")]; { let _x_137 = 1; { let _x_139 = 1; { let _x_33 = crate::ResourceRequirements { cpuMillis: _x_137, memoryBytes: _x_139, replicasMin: _x_137, replicasMax: _x_137 }; { let _x_35 = alloc::vec![alloc::string::String::from("secret")]; { let _x_42 = crate::Component { id: alloc::string::String::from("component"), kind: alloc::string::String::from("service"), version: alloc::string::String::from("1"), artifact: alloc::string::String::from("artifact"), capabilities: _x_22.clone(), command: _x_24, dependsOn: alloc::vec::Vec::new(), interfaces: _x_26, health: alloc::string::String::from("healthy"), liveness: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), ports: alloc::vec::Vec::new(), readiness: alloc::vec::Vec::new(), resources: _x_33, secrets: _x_35, startup: alloc::vec::Vec::new(), volumes: alloc::vec::Vec::new(), isolation: alloc::string::String::from("value"), placement: alloc::vec::Vec::new(), platformRequirements: _x_8.clone(), scalingPolicy: None, failurePolicy: alloc::string::String::from("fail-closed"), retryPolicy: alloc::string::String::from("bounded"), degradationPolicy: alloc::string::String::from("unavailable"), idempotency: alloc::string::String::from("keyed") }; { let _x_44 = alloc::vec![_x_42]; { let _x_51 = alloc::vec![alloc::string::String::from("acceptance")]; { let _x_52 = crate::Interface { id: alloc::string::String::from("interface"), kind: alloc::string::String::from("http"), document: alloc::string::String::from("missing"), authentication: alloc::string::String::from("oidc"), compatibility: alloc::string::String::from("exact"), protocol: alloc::string::String::from("https"), errors: alloc::vec::Vec::new(), acceptance: _x_51 }; { let _x_54 = alloc::vec![_x_52]; { let _x_58 = crate::Schema { id: alloc::string::String::from("schema"), kind: alloc::string::String::from("json-schema"), value: alloc::string::String::from("{}"), dependsOn: alloc::vec::Vec::new(), compatibility: alloc::string::String::from("exact") }; { let _x_60 = alloc::vec![_x_58]; { let _x_67 = alloc::vec![alloc::string::String::from("amd64")]; { let _x_70 = crate::PlatformRequirement { id: alloc::string::String::from("platform"), os: alloc::string::String::from("linux"), architectures: _x_67, runtime: alloc::string::String::from("oci"), runtimeVersion: alloc::string::String::from("1.1"), capabilities: _x_22.clone() }; { let _x_72 = alloc::vec![_x_70]; { let _x_77 = alloc::vec![alloc::string::String::from("component")]; { let _x_79 = crate::SecretReference { id: alloc::string::String::from("secret"), providerKey: alloc::string::String::from("secret://provider/key"), consumers: _x_77, rotation: alloc::string::String::from("automatic") }; { let _x_81 = alloc::vec![_x_79]; { let _x_87 = crate::Migration { id: alloc::string::String::from("migration"), kind: alloc::string::String::from("initialize"), value: alloc::string::String::from("SELECT 1;"), dependsOn: alloc::vec::Vec::new() }; { let _x_89 = alloc::vec![_x_87]; { let _x_91 = crate::Observability { logs: alloc::vec::Vec::new(), metrics: alloc::vec::Vec::new(), traces: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), redactedFields: alloc::vec::Vec::new() }; { let _x_97 = crate::Capability { id: alloc::string::String::from("capability"), kind: alloc::string::String::from("runtime"), value: alloc::string::String::from("present"), dependsOn: alloc::vec::Vec::new() }; { let _x_99 = alloc::vec![_x_97]; { let _x_104 = Some(alloc::string::String::from("secret")); { let _x_105 = Some(alloc::string::String::from("artifact")); { let _x_106 = crate::TargetBinding { id: alloc::string::String::from("target"), kind: alloc::string::String::from("compose"), apiVersion: alloc::string::String::from("1"), adapterDigest: alloc::string::String::from("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"), minimumReleaseStatus: alloc::string::String::from("development"), capabilities: _x_22.clone(), credentials: _x_104, platformRequirements: _x_8.clone(), storageClass: None, storageProfile: None, ingressClassName: None, ingressControllerArtifact: _x_105 }; { let _x_108 = alloc::vec![_x_106]; { let _x_112 = alloc::vec![alloc::string::String::from("migration")]; { let _x_113 = crate::Rollout { id: alloc::string::String::from("rollout"), kind: alloc::string::String::from("rolling"), value: alloc::string::String::from("one-at-a-time"), dependsOn: _x_112 }; { let _x_115 = alloc::vec![_x_113]; { let _x_117 = alloc::vec![alloc::string::String::from("rollout")]; { let _x_118 = { let mut __list = alloc::vec![alloc::string::String::from("migration")]; __list.extend(_x_117); __list }; { let _x_119 = crate::Rollback { id: alloc::string::String::from("rollback"), kind: alloc::string::String::from("automatic"), value: alloc::string::String::from("rollback"), dependsOn: _x_118 }; { let _x_121 = alloc::vec![_x_119]; { let _x_126 = alloc::vec![alloc::string::String::from("probe")]; { let _x_128 = true; { let _x_129 = crate::Acceptance { id: alloc::string::String::from("acceptance"), kind: alloc::string::String::from("positive"), target: alloc::string::String::from("target"), component: alloc::string::String::from("component"), command: _x_126, evidence: alloc::string::String::from("evidence"), bounded: _x_128 }; { let _x_131 = alloc::vec![_x_129]; { let _x_132 = crate::Lifecycle { backup: alloc::string::String::from("value"), drift: alloc::string::String::from("value"), migration: alloc::string::String::from("value"), recovery: alloc::string::String::from("value"), retirement: alloc::string::String::from("value"), rollback: alloc::string::String::from("value"), rollout: alloc::string::String::from("value") }; { let _x_134 = crate::TransactionalCommandView { title: alloc::string::String::from("value"), heading: alloc::string::String::from("value"), identityHeading: alloc::string::String::from("value"), commandFormHeading: alloc::string::String::from("value"), historyHeading: alloc::string::String::from("value"), principalLabel: alloc::string::String::from("value"), principalDefault: alloc::string::String::from("value"), roleLabel: alloc::string::String::from("value"), submitterRoleLabel: alloc::string::String::from("value"), observerRoleLabel: alloc::string::String::from("value"), deniedRoleLabel: alloc::string::String::from("value"), authenticateLabel: alloc::string::String::from("value"), commandIdLabel: alloc::string::String::from("value"), inputALabel: alloc::string::String::from("value"), operationLabel: alloc::string::String::from("value"), inputBLabel: alloc::string::String::from("value"), annotationLabel: alloc::string::String::from("value"), submitLabel: alloc::string::String::from("value"), refreshLabel: alloc::string::String::from("value"), emptyText: alloc::string::String::from("value"), loadingText: alloc::string::String::from("value"), authenticatedText: alloc::string::String::from("value"), accessDeniedText: alloc::string::String::from("value"), retryLabel: alloc::string::String::from("value"), sequenceHeading: alloc::string::String::from("value"), commandColumnHeading: alloc::string::String::from("value"), inputSummaryHeading: alloc::string::String::from("value"), outcomeHeading: alloc::string::String::from("value"), annotationHeading: alloc::string::String::from("value"), releasePrefix: alloc::string::String::from("value"), historyLoadedText: alloc::string::String::from("value"), historyEmptyText: alloc::string::String::from("value"), authenticatingText: alloc::string::String::from("value"), authenticationFailedText: alloc::string::String::from("value"), submittingText: alloc::string::String::from("value"), outcomePrefix: alloc::string::String::from("value"), commandFailedText: alloc::string::String::from("value"), historyFailedText: alloc::string::String::from("value") }; { let _x_135 = crate::TransactionalCommandServiceProfile { contract: alloc::string::String::from("value"), applicationErrors: alloc::vec::Vec::new(), applicationModelDigest: alloc::string::String::from("value"), commandEncoding: alloc::string::String::from("value"), responseVersion: alloc::string::String::from("value"), optionalAnnotation: alloc::string::String::from("value"), coreArtifact: alloc::string::String::from("value"), identityAudience: alloc::string::String::from("value"), submitterRole: alloc::string::String::from("value"), observerRole: alloc::string::String::from("value"), eventSource: alloc::string::String::from("value"), acceptedEventType: alloc::string::String::from("value"), rejectedEventType: alloc::string::String::from("value"), databaseName: alloc::string::String::from("value"), databaseUser: alloc::string::String::from("value"), eventExchange: alloc::string::String::from("value"), auditQueue: alloc::string::String::from("value"), commandIdPattern: alloc::string::String::from("value"), commandPath: alloc::string::String::from("value"), tokenPath: alloc::string::String::from("value"), commandOperations: alloc::vec::Vec::new(), annotationMaxScalars: _x_137, historyDefaultLimit: _x_137, historyMaxLimit: _x_137, maxRequestBytes: _x_139, availabilityThresholdMillionths: _x_139, outboxLagThresholdMillis: _x_139, commandIdField: alloc::string::String::from("value"), operationField: alloc::string::String::from("value"), inputAField: alloc::string::String::from("value"), inputBField: alloc::string::String::from("value"), annotationField: alloc::string::String::from("value"), commandIdColumn: alloc::string::String::from("value"), operationColumn: alloc::string::String::from("value"), inputAColumn: alloc::string::String::from("value"), inputBColumn: alloc::string::String::from("value"), annotationColumn: alloc::string::String::from("value"), acceptanceErrorOperation: alloc::string::String::from("value"), acceptanceErrorInputA: alloc::string::String::from("value"), acceptanceErrorInputB: alloc::string::String::from("value"), acceptanceExpectedError: alloc::string::String::from("value"), acceptanceConflictInputB: alloc::string::String::from("value"), acceptanceSuccessOperation: alloc::string::String::from("value"), acceptanceSuccessInputA: alloc::string::String::from("value"), acceptanceSuccessInputB: alloc::string::String::from("value"), acceptanceExpectedResult: alloc::string::String::from("value"), view: _x_134, brokerUser: alloc::string::String::from("value"), historyTable: alloc::string::String::from("value"), outboxTable: alloc::string::String::from("value"), auditTable: alloc::string::String::from("value"), publicHostnameParameter: alloc::string::String::from("value") }; { let _x_136 = crate::SystemModel { product: _x_9, artifacts: _x_18, components: _x_44, interfaces: _x_54, schemas: _x_60, calls: alloc::vec::Vec::new(), events: alloc::vec::Vec::new(), flows: alloc::vec::Vec::new(), topology: alloc::vec::Vec::new(), platformRequirements: _x_72, scalingPolicies: alloc::vec::Vec::new(), storageClasses: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), secretReferences: _x_81, identityRequirements: alloc::vec::Vec::new(), persistence: alloc::vec::Vec::new(), migrations: _x_89, backups: alloc::vec::Vec::new(), observability: _x_91, slis: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), controls: alloc::vec::Vec::new(), capabilities: _x_99, architecture: alloc::vec::Vec::new(), targets: _x_108, rollouts: _x_115, rollbacks: _x_121, drifts: alloc::vec::Vec::new(), retirements: alloc::vec::Vec::new(), acceptance: _x_131, lifecycle: _x_132, applicationProfile: _x_135, standards: alloc::vec::Vec::new() }; _x_136 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn corpusInvalidReleaseCompletenessModel() -> crate::SystemModel {
    { let _x_7 = crate::Product { id: alloc::string::String::from("product"), owner: alloc::string::String::from("PrismPM"), version: alloc::string::String::from("1"), lifecycle: alloc::string::String::from("development"), sourcePolicy: alloc::string::String::from("closed"), supportedPlatforms: alloc::vec::Vec::new() }; { let _x_15 = alloc::vec![alloc::string::String::from("platform")]; { let _x_16 = crate::Artifact { id: alloc::string::String::from("artifact"), mediaType: alloc::string::String::from("application/octet-stream"), digest: alloc::string::String::from("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), licenseExpression: alloc::string::String::from("MIT"), path: alloc::string::String::from("artifact.bin"), role: alloc::string::String::from("runtime"), platformRequirements: _x_15.clone() }; { let _x_18 = alloc::vec![_x_16]; { let _x_22 = alloc::vec![alloc::string::String::from("capability")]; { let _x_24 = alloc::vec![alloc::string::String::from("run")]; { let _x_26 = alloc::vec![alloc::string::String::from("interface")]; { let _x_136 = 1; { let _x_138 = 1; { let _x_33 = crate::ResourceRequirements { cpuMillis: _x_136, memoryBytes: _x_138, replicasMin: _x_136, replicasMax: _x_136 }; { let _x_35 = alloc::vec![alloc::string::String::from("secret")]; { let _x_42 = crate::Component { id: alloc::string::String::from("component"), kind: alloc::string::String::from("service"), version: alloc::string::String::from("1"), artifact: alloc::string::String::from("artifact"), capabilities: _x_22.clone(), command: _x_24, dependsOn: alloc::vec::Vec::new(), interfaces: _x_26, health: alloc::string::String::from("healthy"), liveness: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), ports: alloc::vec::Vec::new(), readiness: alloc::vec::Vec::new(), resources: _x_33, secrets: _x_35, startup: alloc::vec::Vec::new(), volumes: alloc::vec::Vec::new(), isolation: alloc::string::String::from("value"), placement: alloc::vec::Vec::new(), platformRequirements: _x_15.clone(), scalingPolicy: None, failurePolicy: alloc::string::String::from("fail-closed"), retryPolicy: alloc::string::String::from("bounded"), degradationPolicy: alloc::string::String::from("unavailable"), idempotency: alloc::string::String::from("keyed") }; { let _x_44 = alloc::vec![_x_42]; { let _x_51 = alloc::vec![alloc::string::String::from("acceptance")]; { let _x_52 = crate::Interface { id: alloc::string::String::from("interface"), kind: alloc::string::String::from("http"), document: alloc::string::String::from("schema"), authentication: alloc::string::String::from("oidc"), compatibility: alloc::string::String::from("exact"), protocol: alloc::string::String::from("https"), errors: alloc::vec::Vec::new(), acceptance: _x_51 }; { let _x_54 = alloc::vec![_x_52]; { let _x_57 = crate::Schema { id: alloc::string::String::from("schema"), kind: alloc::string::String::from("json-schema"), value: alloc::string::String::from("{}"), dependsOn: alloc::vec::Vec::new(), compatibility: alloc::string::String::from("exact") }; { let _x_59 = alloc::vec![_x_57]; { let _x_66 = alloc::vec![alloc::string::String::from("amd64")]; { let _x_69 = crate::PlatformRequirement { id: alloc::string::String::from("platform"), os: alloc::string::String::from("linux"), architectures: _x_66, runtime: alloc::string::String::from("oci"), runtimeVersion: alloc::string::String::from("1.1"), capabilities: _x_22.clone() }; { let _x_71 = alloc::vec![_x_69]; { let _x_76 = alloc::vec![alloc::string::String::from("component")]; { let _x_78 = crate::SecretReference { id: alloc::string::String::from("secret"), providerKey: alloc::string::String::from("secret://provider/key"), consumers: _x_76, rotation: alloc::string::String::from("automatic") }; { let _x_80 = alloc::vec![_x_78]; { let _x_86 = crate::Migration { id: alloc::string::String::from("migration"), kind: alloc::string::String::from("initialize"), value: alloc::string::String::from("SELECT 1;"), dependsOn: alloc::vec::Vec::new() }; { let _x_88 = alloc::vec![_x_86]; { let _x_90 = crate::Observability { logs: alloc::vec::Vec::new(), metrics: alloc::vec::Vec::new(), traces: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), redactedFields: alloc::vec::Vec::new() }; { let _x_96 = crate::Capability { id: alloc::string::String::from("capability"), kind: alloc::string::String::from("runtime"), value: alloc::string::String::from("present"), dependsOn: alloc::vec::Vec::new() }; { let _x_98 = alloc::vec![_x_96]; { let _x_103 = Some(alloc::string::String::from("secret")); { let _x_104 = Some(alloc::string::String::from("artifact")); { let _x_105 = crate::TargetBinding { id: alloc::string::String::from("target"), kind: alloc::string::String::from("compose"), apiVersion: alloc::string::String::from("1"), adapterDigest: alloc::string::String::from("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"), minimumReleaseStatus: alloc::string::String::from("development"), capabilities: _x_22.clone(), credentials: _x_103, platformRequirements: _x_15.clone(), storageClass: None, storageProfile: None, ingressClassName: None, ingressControllerArtifact: _x_104 }; { let _x_107 = alloc::vec![_x_105]; { let _x_111 = alloc::vec![alloc::string::String::from("migration")]; { let _x_112 = crate::Rollout { id: alloc::string::String::from("rollout"), kind: alloc::string::String::from("rolling"), value: alloc::string::String::from("one-at-a-time"), dependsOn: _x_111 }; { let _x_114 = alloc::vec![_x_112]; { let _x_116 = alloc::vec![alloc::string::String::from("rollout")]; { let _x_117 = { let mut __list = alloc::vec![alloc::string::String::from("migration")]; __list.extend(_x_116); __list }; { let _x_118 = crate::Rollback { id: alloc::string::String::from("rollback"), kind: alloc::string::String::from("automatic"), value: alloc::string::String::from("rollback"), dependsOn: _x_117 }; { let _x_120 = alloc::vec![_x_118]; { let _x_125 = alloc::vec![alloc::string::String::from("probe")]; { let _x_127 = true; { let _x_128 = crate::Acceptance { id: alloc::string::String::from("acceptance"), kind: alloc::string::String::from("positive"), target: alloc::string::String::from("target"), component: alloc::string::String::from("component"), command: _x_125, evidence: alloc::string::String::from("evidence"), bounded: _x_127 }; { let _x_130 = alloc::vec![_x_128]; { let _x_131 = crate::Lifecycle { backup: alloc::string::String::from("value"), drift: alloc::string::String::from("value"), migration: alloc::string::String::from("value"), recovery: alloc::string::String::from("value"), retirement: alloc::string::String::from("value"), rollback: alloc::string::String::from("value"), rollout: alloc::string::String::from("value") }; { let _x_133 = crate::TransactionalCommandView { title: alloc::string::String::from("value"), heading: alloc::string::String::from("value"), identityHeading: alloc::string::String::from("value"), commandFormHeading: alloc::string::String::from("value"), historyHeading: alloc::string::String::from("value"), principalLabel: alloc::string::String::from("value"), principalDefault: alloc::string::String::from("value"), roleLabel: alloc::string::String::from("value"), submitterRoleLabel: alloc::string::String::from("value"), observerRoleLabel: alloc::string::String::from("value"), deniedRoleLabel: alloc::string::String::from("value"), authenticateLabel: alloc::string::String::from("value"), commandIdLabel: alloc::string::String::from("value"), inputALabel: alloc::string::String::from("value"), operationLabel: alloc::string::String::from("value"), inputBLabel: alloc::string::String::from("value"), annotationLabel: alloc::string::String::from("value"), submitLabel: alloc::string::String::from("value"), refreshLabel: alloc::string::String::from("value"), emptyText: alloc::string::String::from("value"), loadingText: alloc::string::String::from("value"), authenticatedText: alloc::string::String::from("value"), accessDeniedText: alloc::string::String::from("value"), retryLabel: alloc::string::String::from("value"), sequenceHeading: alloc::string::String::from("value"), commandColumnHeading: alloc::string::String::from("value"), inputSummaryHeading: alloc::string::String::from("value"), outcomeHeading: alloc::string::String::from("value"), annotationHeading: alloc::string::String::from("value"), releasePrefix: alloc::string::String::from("value"), historyLoadedText: alloc::string::String::from("value"), historyEmptyText: alloc::string::String::from("value"), authenticatingText: alloc::string::String::from("value"), authenticationFailedText: alloc::string::String::from("value"), submittingText: alloc::string::String::from("value"), outcomePrefix: alloc::string::String::from("value"), commandFailedText: alloc::string::String::from("value"), historyFailedText: alloc::string::String::from("value") }; { let _x_134 = crate::TransactionalCommandServiceProfile { contract: alloc::string::String::from("value"), applicationErrors: alloc::vec::Vec::new(), applicationModelDigest: alloc::string::String::from("value"), commandEncoding: alloc::string::String::from("value"), responseVersion: alloc::string::String::from("value"), optionalAnnotation: alloc::string::String::from("value"), coreArtifact: alloc::string::String::from("value"), identityAudience: alloc::string::String::from("value"), submitterRole: alloc::string::String::from("value"), observerRole: alloc::string::String::from("value"), eventSource: alloc::string::String::from("value"), acceptedEventType: alloc::string::String::from("value"), rejectedEventType: alloc::string::String::from("value"), databaseName: alloc::string::String::from("value"), databaseUser: alloc::string::String::from("value"), eventExchange: alloc::string::String::from("value"), auditQueue: alloc::string::String::from("value"), commandIdPattern: alloc::string::String::from("value"), commandPath: alloc::string::String::from("value"), tokenPath: alloc::string::String::from("value"), commandOperations: alloc::vec::Vec::new(), annotationMaxScalars: _x_136, historyDefaultLimit: _x_136, historyMaxLimit: _x_136, maxRequestBytes: _x_138, availabilityThresholdMillionths: _x_138, outboxLagThresholdMillis: _x_138, commandIdField: alloc::string::String::from("value"), operationField: alloc::string::String::from("value"), inputAField: alloc::string::String::from("value"), inputBField: alloc::string::String::from("value"), annotationField: alloc::string::String::from("value"), commandIdColumn: alloc::string::String::from("value"), operationColumn: alloc::string::String::from("value"), inputAColumn: alloc::string::String::from("value"), inputBColumn: alloc::string::String::from("value"), annotationColumn: alloc::string::String::from("value"), acceptanceErrorOperation: alloc::string::String::from("value"), acceptanceErrorInputA: alloc::string::String::from("value"), acceptanceErrorInputB: alloc::string::String::from("value"), acceptanceExpectedError: alloc::string::String::from("value"), acceptanceConflictInputB: alloc::string::String::from("value"), acceptanceSuccessOperation: alloc::string::String::from("value"), acceptanceSuccessInputA: alloc::string::String::from("value"), acceptanceSuccessInputB: alloc::string::String::from("value"), acceptanceExpectedResult: alloc::string::String::from("value"), view: _x_133, brokerUser: alloc::string::String::from("value"), historyTable: alloc::string::String::from("value"), outboxTable: alloc::string::String::from("value"), auditTable: alloc::string::String::from("value"), publicHostnameParameter: alloc::string::String::from("value") }; { let _x_135 = crate::SystemModel { product: _x_7, artifacts: _x_18, components: _x_44, interfaces: _x_54, schemas: _x_59, calls: alloc::vec::Vec::new(), events: alloc::vec::Vec::new(), flows: alloc::vec::Vec::new(), topology: alloc::vec::Vec::new(), platformRequirements: _x_71, scalingPolicies: alloc::vec::Vec::new(), storageClasses: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), secretReferences: _x_80, identityRequirements: alloc::vec::Vec::new(), persistence: alloc::vec::Vec::new(), migrations: _x_88, backups: alloc::vec::Vec::new(), observability: _x_90, slis: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), controls: alloc::vec::Vec::new(), capabilities: _x_98, architecture: alloc::vec::Vec::new(), targets: _x_107, rollouts: _x_114, rollbacks: _x_120, drifts: alloc::vec::Vec::new(), retirements: alloc::vec::Vec::new(), acceptance: _x_130, lifecycle: _x_131, applicationProfile: _x_134, standards: alloc::vec::Vec::new() }; _x_135 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn corpusInvalidRollbackSafetyModel() -> crate::SystemModel {
    { let _x_8 = alloc::vec![alloc::string::String::from("platform")]; { let _x_9 = crate::Product { id: alloc::string::String::from("product"), owner: alloc::string::String::from("PrismPM"), version: alloc::string::String::from("1"), lifecycle: alloc::string::String::from("development"), sourcePolicy: alloc::string::String::from("closed"), supportedPlatforms: _x_8.clone() }; { let _x_16 = crate::Artifact { id: alloc::string::String::from("artifact"), mediaType: alloc::string::String::from("application/octet-stream"), digest: alloc::string::String::from("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), licenseExpression: alloc::string::String::from("MIT"), path: alloc::string::String::from("artifact.bin"), role: alloc::string::String::from("runtime"), platformRequirements: _x_8.clone() }; { let _x_18 = alloc::vec![_x_16]; { let _x_22 = alloc::vec![alloc::string::String::from("capability")]; { let _x_24 = alloc::vec![alloc::string::String::from("run")]; { let _x_26 = alloc::vec![alloc::string::String::from("interface")]; { let _x_137 = 1; { let _x_139 = 1; { let _x_33 = crate::ResourceRequirements { cpuMillis: _x_137, memoryBytes: _x_139, replicasMin: _x_137, replicasMax: _x_137 }; { let _x_35 = alloc::vec![alloc::string::String::from("secret")]; { let _x_42 = crate::Component { id: alloc::string::String::from("component"), kind: alloc::string::String::from("service"), version: alloc::string::String::from("1"), artifact: alloc::string::String::from("artifact"), capabilities: _x_22.clone(), command: _x_24, dependsOn: alloc::vec::Vec::new(), interfaces: _x_26, health: alloc::string::String::from("healthy"), liveness: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), ports: alloc::vec::Vec::new(), readiness: alloc::vec::Vec::new(), resources: _x_33, secrets: _x_35, startup: alloc::vec::Vec::new(), volumes: alloc::vec::Vec::new(), isolation: alloc::string::String::from("value"), placement: alloc::vec::Vec::new(), platformRequirements: _x_8.clone(), scalingPolicy: None, failurePolicy: alloc::string::String::from("fail-closed"), retryPolicy: alloc::string::String::from("bounded"), degradationPolicy: alloc::string::String::from("unavailable"), idempotency: alloc::string::String::from("keyed") }; { let _x_44 = alloc::vec![_x_42]; { let _x_51 = alloc::vec![alloc::string::String::from("acceptance")]; { let _x_52 = crate::Interface { id: alloc::string::String::from("interface"), kind: alloc::string::String::from("http"), document: alloc::string::String::from("schema"), authentication: alloc::string::String::from("oidc"), compatibility: alloc::string::String::from("exact"), protocol: alloc::string::String::from("https"), errors: alloc::vec::Vec::new(), acceptance: _x_51 }; { let _x_54 = alloc::vec![_x_52]; { let _x_57 = crate::Schema { id: alloc::string::String::from("schema"), kind: alloc::string::String::from("json-schema"), value: alloc::string::String::from("{}"), dependsOn: alloc::vec::Vec::new(), compatibility: alloc::string::String::from("exact") }; { let _x_59 = alloc::vec![_x_57]; { let _x_66 = alloc::vec![alloc::string::String::from("amd64")]; { let _x_69 = crate::PlatformRequirement { id: alloc::string::String::from("platform"), os: alloc::string::String::from("linux"), architectures: _x_66, runtime: alloc::string::String::from("oci"), runtimeVersion: alloc::string::String::from("1.1"), capabilities: _x_22.clone() }; { let _x_71 = alloc::vec![_x_69]; { let _x_76 = alloc::vec![alloc::string::String::from("component")]; { let _x_78 = crate::SecretReference { id: alloc::string::String::from("secret"), providerKey: alloc::string::String::from("secret://provider/key"), consumers: _x_76, rotation: alloc::string::String::from("automatic") }; { let _x_80 = alloc::vec![_x_78]; { let _x_86 = crate::Migration { id: alloc::string::String::from("migration"), kind: alloc::string::String::from("initialize"), value: alloc::string::String::from("SELECT 1;"), dependsOn: alloc::vec::Vec::new() }; { let _x_88 = alloc::vec![_x_86]; { let _x_90 = crate::Observability { logs: alloc::vec::Vec::new(), metrics: alloc::vec::Vec::new(), traces: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), redactedFields: alloc::vec::Vec::new() }; { let _x_96 = crate::Capability { id: alloc::string::String::from("capability"), kind: alloc::string::String::from("runtime"), value: alloc::string::String::from("present"), dependsOn: alloc::vec::Vec::new() }; { let _x_98 = alloc::vec![_x_96]; { let _x_103 = Some(alloc::string::String::from("secret")); { let _x_104 = Some(alloc::string::String::from("artifact")); { let _x_105 = crate::TargetBinding { id: alloc::string::String::from("target"), kind: alloc::string::String::from("compose"), apiVersion: alloc::string::String::from("1"), adapterDigest: alloc::string::String::from("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"), minimumReleaseStatus: alloc::string::String::from("development"), capabilities: _x_22.clone(), credentials: _x_103, platformRequirements: _x_8.clone(), storageClass: None, storageProfile: None, ingressClassName: None, ingressControllerArtifact: _x_104 }; { let _x_107 = alloc::vec![_x_105]; { let _x_111 = alloc::vec![alloc::string::String::from("migration")]; { let _x_112 = crate::Rollout { id: alloc::string::String::from("rollout"), kind: alloc::string::String::from("rolling"), value: alloc::string::String::from("one-at-a-time"), dependsOn: _x_111 }; { let _x_114 = alloc::vec![_x_112]; { let _x_117 = alloc::vec![alloc::string::String::from("rollout")]; { let _x_118 = { let mut __list = alloc::vec![alloc::string::String::from("migration")]; __list.extend(_x_117); __list }; { let _x_119 = crate::Rollback { id: alloc::string::String::from("rollback"), kind: alloc::string::String::from(""), value: alloc::string::String::from("rollback"), dependsOn: _x_118 }; { let _x_121 = alloc::vec![_x_119]; { let _x_126 = alloc::vec![alloc::string::String::from("probe")]; { let _x_128 = true; { let _x_129 = crate::Acceptance { id: alloc::string::String::from("acceptance"), kind: alloc::string::String::from("positive"), target: alloc::string::String::from("target"), component: alloc::string::String::from("component"), command: _x_126, evidence: alloc::string::String::from("evidence"), bounded: _x_128 }; { let _x_131 = alloc::vec![_x_129]; { let _x_132 = crate::Lifecycle { backup: alloc::string::String::from("value"), drift: alloc::string::String::from("value"), migration: alloc::string::String::from("value"), recovery: alloc::string::String::from("value"), retirement: alloc::string::String::from("value"), rollback: alloc::string::String::from("value"), rollout: alloc::string::String::from("value") }; { let _x_134 = crate::TransactionalCommandView { title: alloc::string::String::from("value"), heading: alloc::string::String::from("value"), identityHeading: alloc::string::String::from("value"), commandFormHeading: alloc::string::String::from("value"), historyHeading: alloc::string::String::from("value"), principalLabel: alloc::string::String::from("value"), principalDefault: alloc::string::String::from("value"), roleLabel: alloc::string::String::from("value"), submitterRoleLabel: alloc::string::String::from("value"), observerRoleLabel: alloc::string::String::from("value"), deniedRoleLabel: alloc::string::String::from("value"), authenticateLabel: alloc::string::String::from("value"), commandIdLabel: alloc::string::String::from("value"), inputALabel: alloc::string::String::from("value"), operationLabel: alloc::string::String::from("value"), inputBLabel: alloc::string::String::from("value"), annotationLabel: alloc::string::String::from("value"), submitLabel: alloc::string::String::from("value"), refreshLabel: alloc::string::String::from("value"), emptyText: alloc::string::String::from("value"), loadingText: alloc::string::String::from("value"), authenticatedText: alloc::string::String::from("value"), accessDeniedText: alloc::string::String::from("value"), retryLabel: alloc::string::String::from("value"), sequenceHeading: alloc::string::String::from("value"), commandColumnHeading: alloc::string::String::from("value"), inputSummaryHeading: alloc::string::String::from("value"), outcomeHeading: alloc::string::String::from("value"), annotationHeading: alloc::string::String::from("value"), releasePrefix: alloc::string::String::from("value"), historyLoadedText: alloc::string::String::from("value"), historyEmptyText: alloc::string::String::from("value"), authenticatingText: alloc::string::String::from("value"), authenticationFailedText: alloc::string::String::from("value"), submittingText: alloc::string::String::from("value"), outcomePrefix: alloc::string::String::from("value"), commandFailedText: alloc::string::String::from("value"), historyFailedText: alloc::string::String::from("value") }; { let _x_135 = crate::TransactionalCommandServiceProfile { contract: alloc::string::String::from("value"), applicationErrors: alloc::vec::Vec::new(), applicationModelDigest: alloc::string::String::from("value"), commandEncoding: alloc::string::String::from("value"), responseVersion: alloc::string::String::from("value"), optionalAnnotation: alloc::string::String::from("value"), coreArtifact: alloc::string::String::from("value"), identityAudience: alloc::string::String::from("value"), submitterRole: alloc::string::String::from("value"), observerRole: alloc::string::String::from("value"), eventSource: alloc::string::String::from("value"), acceptedEventType: alloc::string::String::from("value"), rejectedEventType: alloc::string::String::from("value"), databaseName: alloc::string::String::from("value"), databaseUser: alloc::string::String::from("value"), eventExchange: alloc::string::String::from("value"), auditQueue: alloc::string::String::from("value"), commandIdPattern: alloc::string::String::from("value"), commandPath: alloc::string::String::from("value"), tokenPath: alloc::string::String::from("value"), commandOperations: alloc::vec::Vec::new(), annotationMaxScalars: _x_137, historyDefaultLimit: _x_137, historyMaxLimit: _x_137, maxRequestBytes: _x_139, availabilityThresholdMillionths: _x_139, outboxLagThresholdMillis: _x_139, commandIdField: alloc::string::String::from("value"), operationField: alloc::string::String::from("value"), inputAField: alloc::string::String::from("value"), inputBField: alloc::string::String::from("value"), annotationField: alloc::string::String::from("value"), commandIdColumn: alloc::string::String::from("value"), operationColumn: alloc::string::String::from("value"), inputAColumn: alloc::string::String::from("value"), inputBColumn: alloc::string::String::from("value"), annotationColumn: alloc::string::String::from("value"), acceptanceErrorOperation: alloc::string::String::from("value"), acceptanceErrorInputA: alloc::string::String::from("value"), acceptanceErrorInputB: alloc::string::String::from("value"), acceptanceExpectedError: alloc::string::String::from("value"), acceptanceConflictInputB: alloc::string::String::from("value"), acceptanceSuccessOperation: alloc::string::String::from("value"), acceptanceSuccessInputA: alloc::string::String::from("value"), acceptanceSuccessInputB: alloc::string::String::from("value"), acceptanceExpectedResult: alloc::string::String::from("value"), view: _x_134, brokerUser: alloc::string::String::from("value"), historyTable: alloc::string::String::from("value"), outboxTable: alloc::string::String::from("value"), auditTable: alloc::string::String::from("value"), publicHostnameParameter: alloc::string::String::from("value") }; { let _x_136 = crate::SystemModel { product: _x_9, artifacts: _x_18, components: _x_44, interfaces: _x_54, schemas: _x_59, calls: alloc::vec::Vec::new(), events: alloc::vec::Vec::new(), flows: alloc::vec::Vec::new(), topology: alloc::vec::Vec::new(), platformRequirements: _x_71, scalingPolicies: alloc::vec::Vec::new(), storageClasses: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), secretReferences: _x_80, identityRequirements: alloc::vec::Vec::new(), persistence: alloc::vec::Vec::new(), migrations: _x_88, backups: alloc::vec::Vec::new(), observability: _x_90, slis: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), controls: alloc::vec::Vec::new(), capabilities: _x_98, architecture: alloc::vec::Vec::new(), targets: _x_107, rollouts: _x_114, rollbacks: _x_121, drifts: alloc::vec::Vec::new(), retirements: alloc::vec::Vec::new(), acceptance: _x_131, lifecycle: _x_132, applicationProfile: _x_135, standards: alloc::vec::Vec::new() }; _x_136 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn corpusInvalidSecretFlowModel() -> crate::SystemModel {
    { let _x_8 = alloc::vec![alloc::string::String::from("platform")]; { let _x_9 = crate::Product { id: alloc::string::String::from("product"), owner: alloc::string::String::from("PrismPM"), version: alloc::string::String::from("1"), lifecycle: alloc::string::String::from("development"), sourcePolicy: alloc::string::String::from("closed"), supportedPlatforms: _x_8.clone() }; { let _x_16 = crate::Artifact { id: alloc::string::String::from("artifact"), mediaType: alloc::string::String::from("application/octet-stream"), digest: alloc::string::String::from("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), licenseExpression: alloc::string::String::from("MIT"), path: alloc::string::String::from("artifact.bin"), role: alloc::string::String::from("runtime"), platformRequirements: _x_8.clone() }; { let _x_18 = alloc::vec![_x_16]; { let _x_22 = alloc::vec![alloc::string::String::from("capability")]; { let _x_24 = alloc::vec![alloc::string::String::from("run")]; { let _x_26 = alloc::vec![alloc::string::String::from("interface")]; { let _x_137 = 1; { let _x_139 = 1; { let _x_33 = crate::ResourceRequirements { cpuMillis: _x_137, memoryBytes: _x_139, replicasMin: _x_137, replicasMax: _x_137 }; { let _x_35 = alloc::vec![alloc::string::String::from("secret")]; { let _x_42 = crate::Component { id: alloc::string::String::from("component"), kind: alloc::string::String::from("service"), version: alloc::string::String::from("1"), artifact: alloc::string::String::from("artifact"), capabilities: _x_22.clone(), command: _x_24, dependsOn: alloc::vec::Vec::new(), interfaces: _x_26, health: alloc::string::String::from("healthy"), liveness: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), ports: alloc::vec::Vec::new(), readiness: alloc::vec::Vec::new(), resources: _x_33, secrets: _x_35, startup: alloc::vec::Vec::new(), volumes: alloc::vec::Vec::new(), isolation: alloc::string::String::from("value"), placement: alloc::vec::Vec::new(), platformRequirements: _x_8.clone(), scalingPolicy: None, failurePolicy: alloc::string::String::from("fail-closed"), retryPolicy: alloc::string::String::from("bounded"), degradationPolicy: alloc::string::String::from("unavailable"), idempotency: alloc::string::String::from("keyed") }; { let _x_44 = alloc::vec![_x_42]; { let _x_51 = alloc::vec![alloc::string::String::from("acceptance")]; { let _x_52 = crate::Interface { id: alloc::string::String::from("interface"), kind: alloc::string::String::from("http"), document: alloc::string::String::from("schema"), authentication: alloc::string::String::from("oidc"), compatibility: alloc::string::String::from("exact"), protocol: alloc::string::String::from("https"), errors: alloc::vec::Vec::new(), acceptance: _x_51 }; { let _x_54 = alloc::vec![_x_52]; { let _x_57 = crate::Schema { id: alloc::string::String::from("schema"), kind: alloc::string::String::from("json-schema"), value: alloc::string::String::from("{}"), dependsOn: alloc::vec::Vec::new(), compatibility: alloc::string::String::from("exact") }; { let _x_59 = alloc::vec![_x_57]; { let _x_66 = alloc::vec![alloc::string::String::from("amd64")]; { let _x_69 = crate::PlatformRequirement { id: alloc::string::String::from("platform"), os: alloc::string::String::from("linux"), architectures: _x_66, runtime: alloc::string::String::from("oci"), runtimeVersion: alloc::string::String::from("1.1"), capabilities: _x_22.clone() }; { let _x_71 = alloc::vec![_x_69]; { let _x_77 = alloc::vec![alloc::string::String::from("missing")]; { let _x_79 = crate::SecretReference { id: alloc::string::String::from("secret"), providerKey: alloc::string::String::from("secret://provider/key"), consumers: _x_77, rotation: alloc::string::String::from("automatic") }; { let _x_81 = alloc::vec![_x_79]; { let _x_87 = crate::Migration { id: alloc::string::String::from("migration"), kind: alloc::string::String::from("initialize"), value: alloc::string::String::from("SELECT 1;"), dependsOn: alloc::vec::Vec::new() }; { let _x_89 = alloc::vec![_x_87]; { let _x_91 = crate::Observability { logs: alloc::vec::Vec::new(), metrics: alloc::vec::Vec::new(), traces: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), redactedFields: alloc::vec::Vec::new() }; { let _x_97 = crate::Capability { id: alloc::string::String::from("capability"), kind: alloc::string::String::from("runtime"), value: alloc::string::String::from("present"), dependsOn: alloc::vec::Vec::new() }; { let _x_99 = alloc::vec![_x_97]; { let _x_104 = Some(alloc::string::String::from("secret")); { let _x_105 = Some(alloc::string::String::from("artifact")); { let _x_106 = crate::TargetBinding { id: alloc::string::String::from("target"), kind: alloc::string::String::from("compose"), apiVersion: alloc::string::String::from("1"), adapterDigest: alloc::string::String::from("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"), minimumReleaseStatus: alloc::string::String::from("development"), capabilities: _x_22.clone(), credentials: _x_104, platformRequirements: _x_8.clone(), storageClass: None, storageProfile: None, ingressClassName: None, ingressControllerArtifact: _x_105 }; { let _x_108 = alloc::vec![_x_106]; { let _x_112 = alloc::vec![alloc::string::String::from("migration")]; { let _x_113 = crate::Rollout { id: alloc::string::String::from("rollout"), kind: alloc::string::String::from("rolling"), value: alloc::string::String::from("one-at-a-time"), dependsOn: _x_112 }; { let _x_115 = alloc::vec![_x_113]; { let _x_117 = alloc::vec![alloc::string::String::from("rollout")]; { let _x_118 = { let mut __list = alloc::vec![alloc::string::String::from("migration")]; __list.extend(_x_117); __list }; { let _x_119 = crate::Rollback { id: alloc::string::String::from("rollback"), kind: alloc::string::String::from("automatic"), value: alloc::string::String::from("rollback"), dependsOn: _x_118 }; { let _x_121 = alloc::vec![_x_119]; { let _x_126 = alloc::vec![alloc::string::String::from("probe")]; { let _x_128 = true; { let _x_129 = crate::Acceptance { id: alloc::string::String::from("acceptance"), kind: alloc::string::String::from("positive"), target: alloc::string::String::from("target"), component: alloc::string::String::from("component"), command: _x_126, evidence: alloc::string::String::from("evidence"), bounded: _x_128 }; { let _x_131 = alloc::vec![_x_129]; { let _x_132 = crate::Lifecycle { backup: alloc::string::String::from("value"), drift: alloc::string::String::from("value"), migration: alloc::string::String::from("value"), recovery: alloc::string::String::from("value"), retirement: alloc::string::String::from("value"), rollback: alloc::string::String::from("value"), rollout: alloc::string::String::from("value") }; { let _x_134 = crate::TransactionalCommandView { title: alloc::string::String::from("value"), heading: alloc::string::String::from("value"), identityHeading: alloc::string::String::from("value"), commandFormHeading: alloc::string::String::from("value"), historyHeading: alloc::string::String::from("value"), principalLabel: alloc::string::String::from("value"), principalDefault: alloc::string::String::from("value"), roleLabel: alloc::string::String::from("value"), submitterRoleLabel: alloc::string::String::from("value"), observerRoleLabel: alloc::string::String::from("value"), deniedRoleLabel: alloc::string::String::from("value"), authenticateLabel: alloc::string::String::from("value"), commandIdLabel: alloc::string::String::from("value"), inputALabel: alloc::string::String::from("value"), operationLabel: alloc::string::String::from("value"), inputBLabel: alloc::string::String::from("value"), annotationLabel: alloc::string::String::from("value"), submitLabel: alloc::string::String::from("value"), refreshLabel: alloc::string::String::from("value"), emptyText: alloc::string::String::from("value"), loadingText: alloc::string::String::from("value"), authenticatedText: alloc::string::String::from("value"), accessDeniedText: alloc::string::String::from("value"), retryLabel: alloc::string::String::from("value"), sequenceHeading: alloc::string::String::from("value"), commandColumnHeading: alloc::string::String::from("value"), inputSummaryHeading: alloc::string::String::from("value"), outcomeHeading: alloc::string::String::from("value"), annotationHeading: alloc::string::String::from("value"), releasePrefix: alloc::string::String::from("value"), historyLoadedText: alloc::string::String::from("value"), historyEmptyText: alloc::string::String::from("value"), authenticatingText: alloc::string::String::from("value"), authenticationFailedText: alloc::string::String::from("value"), submittingText: alloc::string::String::from("value"), outcomePrefix: alloc::string::String::from("value"), commandFailedText: alloc::string::String::from("value"), historyFailedText: alloc::string::String::from("value") }; { let _x_135 = crate::TransactionalCommandServiceProfile { contract: alloc::string::String::from("value"), applicationErrors: alloc::vec::Vec::new(), applicationModelDigest: alloc::string::String::from("value"), commandEncoding: alloc::string::String::from("value"), responseVersion: alloc::string::String::from("value"), optionalAnnotation: alloc::string::String::from("value"), coreArtifact: alloc::string::String::from("value"), identityAudience: alloc::string::String::from("value"), submitterRole: alloc::string::String::from("value"), observerRole: alloc::string::String::from("value"), eventSource: alloc::string::String::from("value"), acceptedEventType: alloc::string::String::from("value"), rejectedEventType: alloc::string::String::from("value"), databaseName: alloc::string::String::from("value"), databaseUser: alloc::string::String::from("value"), eventExchange: alloc::string::String::from("value"), auditQueue: alloc::string::String::from("value"), commandIdPattern: alloc::string::String::from("value"), commandPath: alloc::string::String::from("value"), tokenPath: alloc::string::String::from("value"), commandOperations: alloc::vec::Vec::new(), annotationMaxScalars: _x_137, historyDefaultLimit: _x_137, historyMaxLimit: _x_137, maxRequestBytes: _x_139, availabilityThresholdMillionths: _x_139, outboxLagThresholdMillis: _x_139, commandIdField: alloc::string::String::from("value"), operationField: alloc::string::String::from("value"), inputAField: alloc::string::String::from("value"), inputBField: alloc::string::String::from("value"), annotationField: alloc::string::String::from("value"), commandIdColumn: alloc::string::String::from("value"), operationColumn: alloc::string::String::from("value"), inputAColumn: alloc::string::String::from("value"), inputBColumn: alloc::string::String::from("value"), annotationColumn: alloc::string::String::from("value"), acceptanceErrorOperation: alloc::string::String::from("value"), acceptanceErrorInputA: alloc::string::String::from("value"), acceptanceErrorInputB: alloc::string::String::from("value"), acceptanceExpectedError: alloc::string::String::from("value"), acceptanceConflictInputB: alloc::string::String::from("value"), acceptanceSuccessOperation: alloc::string::String::from("value"), acceptanceSuccessInputA: alloc::string::String::from("value"), acceptanceSuccessInputB: alloc::string::String::from("value"), acceptanceExpectedResult: alloc::string::String::from("value"), view: _x_134, brokerUser: alloc::string::String::from("value"), historyTable: alloc::string::String::from("value"), outboxTable: alloc::string::String::from("value"), auditTable: alloc::string::String::from("value"), publicHostnameParameter: alloc::string::String::from("value") }; { let _x_136 = crate::SystemModel { product: _x_9, artifacts: _x_18, components: _x_44, interfaces: _x_54, schemas: _x_59, calls: alloc::vec::Vec::new(), events: alloc::vec::Vec::new(), flows: alloc::vec::Vec::new(), topology: alloc::vec::Vec::new(), platformRequirements: _x_71, scalingPolicies: alloc::vec::Vec::new(), storageClasses: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), secretReferences: _x_81, identityRequirements: alloc::vec::Vec::new(), persistence: alloc::vec::Vec::new(), migrations: _x_89, backups: alloc::vec::Vec::new(), observability: _x_91, slis: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), controls: alloc::vec::Vec::new(), capabilities: _x_99, architecture: alloc::vec::Vec::new(), targets: _x_108, rollouts: _x_115, rollbacks: _x_121, drifts: alloc::vec::Vec::new(), retirements: alloc::vec::Vec::new(), acceptance: _x_131, lifecycle: _x_132, applicationProfile: _x_135, standards: alloc::vec::Vec::new() }; _x_136 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn corpusInvalidUniquenessModel() -> crate::SystemModel {
    { let _x_8 = alloc::vec![alloc::string::String::from("platform")]; { let _x_9 = crate::Product { id: alloc::string::String::from("product"), owner: alloc::string::String::from("PrismPM"), version: alloc::string::String::from("1"), lifecycle: alloc::string::String::from("development"), sourcePolicy: alloc::string::String::from("closed"), supportedPlatforms: _x_8.clone() }; { let _x_16 = crate::Artifact { id: alloc::string::String::from("artifact"), mediaType: alloc::string::String::from("application/octet-stream"), digest: alloc::string::String::from("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), licenseExpression: alloc::string::String::from("MIT"), path: alloc::string::String::from("artifact.bin"), role: alloc::string::String::from("runtime"), platformRequirements: _x_8.clone() }; { let _x_18 = alloc::vec![_x_16]; { let _x_21 = alloc::vec![alloc::string::String::from("capability")]; { let _x_23 = alloc::vec![alloc::string::String::from("run")]; { let _x_25 = alloc::vec![alloc::string::String::from("interface")]; { let _x_136 = 1; { let _x_138 = 1; { let _x_32 = crate::ResourceRequirements { cpuMillis: _x_136, memoryBytes: _x_138, replicasMin: _x_136, replicasMax: _x_136 }; { let _x_34 = alloc::vec![alloc::string::String::from("secret")]; { let _x_41 = crate::Component { id: alloc::string::String::from("artifact"), kind: alloc::string::String::from("service"), version: alloc::string::String::from("1"), artifact: alloc::string::String::from("artifact"), capabilities: _x_21.clone(), command: _x_23, dependsOn: alloc::vec::Vec::new(), interfaces: _x_25, health: alloc::string::String::from("healthy"), liveness: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), ports: alloc::vec::Vec::new(), readiness: alloc::vec::Vec::new(), resources: _x_32, secrets: _x_34, startup: alloc::vec::Vec::new(), volumes: alloc::vec::Vec::new(), isolation: alloc::string::String::from("value"), placement: alloc::vec::Vec::new(), platformRequirements: _x_8.clone(), scalingPolicy: None, failurePolicy: alloc::string::String::from("fail-closed"), retryPolicy: alloc::string::String::from("bounded"), degradationPolicy: alloc::string::String::from("unavailable"), idempotency: alloc::string::String::from("keyed") }; { let _x_43 = alloc::vec![_x_41]; { let _x_50 = alloc::vec![alloc::string::String::from("acceptance")]; { let _x_51 = crate::Interface { id: alloc::string::String::from("interface"), kind: alloc::string::String::from("http"), document: alloc::string::String::from("schema"), authentication: alloc::string::String::from("oidc"), compatibility: alloc::string::String::from("exact"), protocol: alloc::string::String::from("https"), errors: alloc::vec::Vec::new(), acceptance: _x_50 }; { let _x_53 = alloc::vec![_x_51]; { let _x_56 = crate::Schema { id: alloc::string::String::from("schema"), kind: alloc::string::String::from("json-schema"), value: alloc::string::String::from("{}"), dependsOn: alloc::vec::Vec::new(), compatibility: alloc::string::String::from("exact") }; { let _x_58 = alloc::vec![_x_56]; { let _x_65 = alloc::vec![alloc::string::String::from("amd64")]; { let _x_68 = crate::PlatformRequirement { id: alloc::string::String::from("platform"), os: alloc::string::String::from("linux"), architectures: _x_65, runtime: alloc::string::String::from("oci"), runtimeVersion: alloc::string::String::from("1.1"), capabilities: _x_21.clone() }; { let _x_70 = alloc::vec![_x_68]; { let _x_76 = alloc::vec![alloc::string::String::from("component")]; { let _x_78 = crate::SecretReference { id: alloc::string::String::from("secret"), providerKey: alloc::string::String::from("secret://provider/key"), consumers: _x_76, rotation: alloc::string::String::from("automatic") }; { let _x_80 = alloc::vec![_x_78]; { let _x_86 = crate::Migration { id: alloc::string::String::from("migration"), kind: alloc::string::String::from("initialize"), value: alloc::string::String::from("SELECT 1;"), dependsOn: alloc::vec::Vec::new() }; { let _x_88 = alloc::vec![_x_86]; { let _x_90 = crate::Observability { logs: alloc::vec::Vec::new(), metrics: alloc::vec::Vec::new(), traces: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), redactedFields: alloc::vec::Vec::new() }; { let _x_96 = crate::Capability { id: alloc::string::String::from("capability"), kind: alloc::string::String::from("runtime"), value: alloc::string::String::from("present"), dependsOn: alloc::vec::Vec::new() }; { let _x_98 = alloc::vec![_x_96]; { let _x_103 = Some(alloc::string::String::from("secret")); { let _x_104 = Some(alloc::string::String::from("artifact")); { let _x_105 = crate::TargetBinding { id: alloc::string::String::from("target"), kind: alloc::string::String::from("compose"), apiVersion: alloc::string::String::from("1"), adapterDigest: alloc::string::String::from("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"), minimumReleaseStatus: alloc::string::String::from("development"), capabilities: _x_21.clone(), credentials: _x_103, platformRequirements: _x_8.clone(), storageClass: None, storageProfile: None, ingressClassName: None, ingressControllerArtifact: _x_104 }; { let _x_107 = alloc::vec![_x_105]; { let _x_111 = alloc::vec![alloc::string::String::from("migration")]; { let _x_112 = crate::Rollout { id: alloc::string::String::from("rollout"), kind: alloc::string::String::from("rolling"), value: alloc::string::String::from("one-at-a-time"), dependsOn: _x_111 }; { let _x_114 = alloc::vec![_x_112]; { let _x_116 = alloc::vec![alloc::string::String::from("rollout")]; { let _x_117 = { let mut __list = alloc::vec![alloc::string::String::from("migration")]; __list.extend(_x_116); __list }; { let _x_118 = crate::Rollback { id: alloc::string::String::from("rollback"), kind: alloc::string::String::from("automatic"), value: alloc::string::String::from("rollback"), dependsOn: _x_117 }; { let _x_120 = alloc::vec![_x_118]; { let _x_125 = alloc::vec![alloc::string::String::from("probe")]; { let _x_127 = true; { let _x_128 = crate::Acceptance { id: alloc::string::String::from("acceptance"), kind: alloc::string::String::from("positive"), target: alloc::string::String::from("target"), component: alloc::string::String::from("component"), command: _x_125, evidence: alloc::string::String::from("evidence"), bounded: _x_127 }; { let _x_130 = alloc::vec![_x_128]; { let _x_131 = crate::Lifecycle { backup: alloc::string::String::from("value"), drift: alloc::string::String::from("value"), migration: alloc::string::String::from("value"), recovery: alloc::string::String::from("value"), retirement: alloc::string::String::from("value"), rollback: alloc::string::String::from("value"), rollout: alloc::string::String::from("value") }; { let _x_133 = crate::TransactionalCommandView { title: alloc::string::String::from("value"), heading: alloc::string::String::from("value"), identityHeading: alloc::string::String::from("value"), commandFormHeading: alloc::string::String::from("value"), historyHeading: alloc::string::String::from("value"), principalLabel: alloc::string::String::from("value"), principalDefault: alloc::string::String::from("value"), roleLabel: alloc::string::String::from("value"), submitterRoleLabel: alloc::string::String::from("value"), observerRoleLabel: alloc::string::String::from("value"), deniedRoleLabel: alloc::string::String::from("value"), authenticateLabel: alloc::string::String::from("value"), commandIdLabel: alloc::string::String::from("value"), inputALabel: alloc::string::String::from("value"), operationLabel: alloc::string::String::from("value"), inputBLabel: alloc::string::String::from("value"), annotationLabel: alloc::string::String::from("value"), submitLabel: alloc::string::String::from("value"), refreshLabel: alloc::string::String::from("value"), emptyText: alloc::string::String::from("value"), loadingText: alloc::string::String::from("value"), authenticatedText: alloc::string::String::from("value"), accessDeniedText: alloc::string::String::from("value"), retryLabel: alloc::string::String::from("value"), sequenceHeading: alloc::string::String::from("value"), commandColumnHeading: alloc::string::String::from("value"), inputSummaryHeading: alloc::string::String::from("value"), outcomeHeading: alloc::string::String::from("value"), annotationHeading: alloc::string::String::from("value"), releasePrefix: alloc::string::String::from("value"), historyLoadedText: alloc::string::String::from("value"), historyEmptyText: alloc::string::String::from("value"), authenticatingText: alloc::string::String::from("value"), authenticationFailedText: alloc::string::String::from("value"), submittingText: alloc::string::String::from("value"), outcomePrefix: alloc::string::String::from("value"), commandFailedText: alloc::string::String::from("value"), historyFailedText: alloc::string::String::from("value") }; { let _x_134 = crate::TransactionalCommandServiceProfile { contract: alloc::string::String::from("value"), applicationErrors: alloc::vec::Vec::new(), applicationModelDigest: alloc::string::String::from("value"), commandEncoding: alloc::string::String::from("value"), responseVersion: alloc::string::String::from("value"), optionalAnnotation: alloc::string::String::from("value"), coreArtifact: alloc::string::String::from("value"), identityAudience: alloc::string::String::from("value"), submitterRole: alloc::string::String::from("value"), observerRole: alloc::string::String::from("value"), eventSource: alloc::string::String::from("value"), acceptedEventType: alloc::string::String::from("value"), rejectedEventType: alloc::string::String::from("value"), databaseName: alloc::string::String::from("value"), databaseUser: alloc::string::String::from("value"), eventExchange: alloc::string::String::from("value"), auditQueue: alloc::string::String::from("value"), commandIdPattern: alloc::string::String::from("value"), commandPath: alloc::string::String::from("value"), tokenPath: alloc::string::String::from("value"), commandOperations: alloc::vec::Vec::new(), annotationMaxScalars: _x_136, historyDefaultLimit: _x_136, historyMaxLimit: _x_136, maxRequestBytes: _x_138, availabilityThresholdMillionths: _x_138, outboxLagThresholdMillis: _x_138, commandIdField: alloc::string::String::from("value"), operationField: alloc::string::String::from("value"), inputAField: alloc::string::String::from("value"), inputBField: alloc::string::String::from("value"), annotationField: alloc::string::String::from("value"), commandIdColumn: alloc::string::String::from("value"), operationColumn: alloc::string::String::from("value"), inputAColumn: alloc::string::String::from("value"), inputBColumn: alloc::string::String::from("value"), annotationColumn: alloc::string::String::from("value"), acceptanceErrorOperation: alloc::string::String::from("value"), acceptanceErrorInputA: alloc::string::String::from("value"), acceptanceErrorInputB: alloc::string::String::from("value"), acceptanceExpectedError: alloc::string::String::from("value"), acceptanceConflictInputB: alloc::string::String::from("value"), acceptanceSuccessOperation: alloc::string::String::from("value"), acceptanceSuccessInputA: alloc::string::String::from("value"), acceptanceSuccessInputB: alloc::string::String::from("value"), acceptanceExpectedResult: alloc::string::String::from("value"), view: _x_133, brokerUser: alloc::string::String::from("value"), historyTable: alloc::string::String::from("value"), outboxTable: alloc::string::String::from("value"), auditTable: alloc::string::String::from("value"), publicHostnameParameter: alloc::string::String::from("value") }; { let _x_135 = crate::SystemModel { product: _x_9, artifacts: _x_18, components: _x_43, interfaces: _x_53, schemas: _x_58, calls: alloc::vec::Vec::new(), events: alloc::vec::Vec::new(), flows: alloc::vec::Vec::new(), topology: alloc::vec::Vec::new(), platformRequirements: _x_70, scalingPolicies: alloc::vec::Vec::new(), storageClasses: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), secretReferences: _x_80, identityRequirements: alloc::vec::Vec::new(), persistence: alloc::vec::Vec::new(), migrations: _x_88, backups: alloc::vec::Vec::new(), observability: _x_90, slis: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), controls: alloc::vec::Vec::new(), capabilities: _x_98, architecture: alloc::vec::Vec::new(), targets: _x_107, rollouts: _x_114, rollbacks: _x_120, drifts: alloc::vec::Vec::new(), retirements: alloc::vec::Vec::new(), acceptance: _x_130, lifecycle: _x_131, applicationProfile: _x_134, standards: alloc::vec::Vec::new() }; _x_135 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn corpusManifest() -> crate::SystemManifest {
    { let _x_1 = 13; { let _x_4 = 0; { let _x_8 = alloc::vec![_x_4]; { let _x_9 = crate::ValidationRelation { bound: _x_1, values: _x_8.clone() }; { let _x_10 = 1; { let _x_13 = 2; { let _x_16 = 3; { let _x_19 = 4; { let _x_22 = 5; { let _x_25 = 6; { let _x_28 = 7; { let _x_31 = 8; { let _x_34 = 9; { let _x_37 = 10; { let _x_40 = 11; { let _x_43 = 12; { let _x_46 = alloc::vec![_x_43]; { let _x_47 = { let mut __list = alloc::vec![_x_40]; __list.extend(_x_46); __list }; { let _x_48 = { let mut __list = alloc::vec![_x_37]; __list.extend(_x_47); __list }; { let _x_49 = { let mut __list = alloc::vec![_x_34]; __list.extend(_x_48); __list }; { let _x_50 = { let mut __list = alloc::vec![_x_31]; __list.extend(_x_49); __list }; { let _x_51 = { let mut __list = alloc::vec![_x_28]; __list.extend(_x_50); __list }; { let _x_52 = { let mut __list = alloc::vec![_x_25]; __list.extend(_x_51); __list }; { let _x_53 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_52); __list }; { let _x_54 = { let mut __list = alloc::vec![_x_19]; __list.extend(_x_53); __list }; { let _x_55 = { let mut __list = alloc::vec![_x_16]; __list.extend(_x_54); __list }; { let _x_56 = { let mut __list = alloc::vec![_x_13]; __list.extend(_x_55); __list }; { let _x_57 = { let mut __list = alloc::vec![_x_10]; __list.extend(_x_56); __list }; { let _x_58 = { let mut __list = alloc::vec![_x_4]; __list.extend(_x_57); __list }; { let _x_59 = crate::ValidationRelation { bound: _x_1, values: _x_58 }; { let _x_60 = crate::ValidationRelation { bound: _x_19, values: _x_8.clone() }; { let _x_61 = crate::ValidationRelation { bound: _x_10, values: _x_8.clone() }; { let _x_62 = alloc::vec![_x_31]; { let _x_63 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_62); __list }; { let _x_64 = crate::ValidationRelation { bound: _x_1, values: _x_63 }; { let _x_65 = alloc::vec![_x_40]; { let _x_66 = { let mut __list = alloc::vec![_x_16]; __list.extend(_x_65); __list }; { let _x_67 = crate::ValidationRelation { bound: _x_1, values: _x_66 }; { let _x_68 = 257; { let _x_71 = alloc::vec![_x_16]; { let _x_72 = crate::ValidationRelation { bound: _x_68, values: _x_71 }; { let _x_73 = crate::SystemManifest { closure: _x_9.clone(), uniqueness: _x_59, referentialIntegrity: _x_9.clone(), compatibility: _x_60, capabilitySatisfaction: _x_61.clone(), secretFlow: _x_61.clone(), deploymentOrder: _x_61.clone(), migrationOrder: _x_61.clone(), rollbackSafety: _x_64, evidenceClosure: _x_67, licenseClosure: _x_72, releaseCompleteness: _x_61.clone() }; _x_73 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn corpusProbeCapabilitySatisfaction() -> bool {
    { let _x_10 = corpusValidModel(); { let _x_11 = corpusManifest(); { let _x_12 = validateModelCapabilitySatisfaction(&(_x_10), &(_x_11)); match _x_12 {
        false => _x_12,
        true => { let _x_23 = corpusInvalidCapabilitySatisfactionModel(); { let _x_24 = corpusManifest(); { let _x_25 = validateModelCapabilitySatisfaction(&(_x_23), &(_x_24)); match _x_25 {
        false => _x_12,
        true => { let _x_27 = false; _x_27 },
    } } } },
    } } } }
}

pub fn corpusProbeClosure() -> Result<bool, crate::ComputeError> {
    Ok({ let _x_10 = corpusValidModel(); { let _x_11 = corpusManifest(); { let _x_12 = validateModelClosure(&(_x_10), &(_x_11))?; match _x_12 {
        false => _x_12,
        true => { let _x_23 = corpusInvalidClosureModel(); { let _x_24 = corpusManifest(); { let _x_25 = validateModelClosure(&(_x_23), &(_x_24))?; match _x_25 {
        false => _x_12,
        true => { let _x_27 = false; _x_27 },
    } } } },
    } } } })
}

pub fn corpusProbeCompatibility() -> bool {
    { let _x_10 = corpusValidModel(); { let _x_11 = corpusManifest(); { let _x_12 = validateModelCompatibility(&(_x_10), &(_x_11)); match _x_12 {
        false => _x_12,
        true => { let _x_23 = corpusInvalidCompatibilityModel(); { let _x_24 = corpusManifest(); { let _x_25 = validateModelCompatibility(&(_x_23), &(_x_24)); match _x_25 {
        false => _x_12,
        true => { let _x_27 = false; _x_27 },
    } } } },
    } } } }
}

pub fn corpusProbeDeploymentOrder() -> Result<bool, crate::ComputeError> {
    Ok({ let _x_10 = corpusValidModel(); { let _x_11 = corpusManifest(); { let _x_12 = validateModelDeploymentOrder(&(_x_10), &(_x_11))?; match _x_12 {
        false => _x_12,
        true => { let _x_23 = corpusInvalidDeploymentOrderModel(); { let _x_24 = corpusManifest(); { let _x_25 = validateModelDeploymentOrder(&(_x_23), &(_x_24))?; match _x_25 {
        false => _x_12,
        true => { let _x_27 = false; _x_27 },
    } } } },
    } } } })
}

pub fn corpusProbeEvidenceClosure() -> Result<bool, crate::ComputeError> {
    Ok({ let _x_10 = corpusValidModel(); { let _x_11 = corpusManifest(); { let _x_12 = validateModelEvidenceClosure(&(_x_10), &(_x_11))?; match _x_12 {
        false => _x_12,
        true => { let _x_23 = corpusInvalidEvidenceClosureModel(); { let _x_24 = corpusManifest(); { let _x_25 = validateModelEvidenceClosure(&(_x_23), &(_x_24))?; match _x_25 {
        false => _x_12,
        true => { let _x_27 = false; _x_27 },
    } } } },
    } } } })
}

pub fn corpusProbeLicenseClosure() -> bool {
    { let _x_10 = corpusValidModel(); { let _x_11 = corpusManifest(); { let _x_12 = validateModelLicenseClosure(&(_x_10), &(_x_11)); match _x_12 {
        false => _x_12,
        true => { let _x_23 = corpusInvalidLicenseClosureModel(); { let _x_24 = corpusManifest(); { let _x_25 = validateModelLicenseClosure(&(_x_23), &(_x_24)); match _x_25 {
        false => _x_12,
        true => { let _x_27 = false; _x_27 },
    } } } },
    } } } }
}

pub fn corpusProbeManifest() -> Result<bool, crate::ComputeError> {
    Ok({ let _x_1 = corpusValidModel(); { let _x_2 = corpusManifest(); { let _x_3 = validateManifest(&(_x_1), &(_x_2))?; _x_3 } } })
}

pub fn corpusProbeMigrationOrder() -> Result<bool, crate::ComputeError> {
    Ok({ let _x_10 = corpusValidModel(); { let _x_11 = corpusManifest(); { let _x_12 = validateModelMigrationOrder(&(_x_10), &(_x_11))?; match _x_12 {
        false => _x_12,
        true => { let _x_23 = corpusInvalidMigrationOrderModel(); { let _x_24 = corpusManifest(); { let _x_25 = validateModelMigrationOrder(&(_x_23), &(_x_24))?; match _x_25 {
        false => _x_12,
        true => { let _x_27 = false; _x_27 },
    } } } },
    } } } })
}

pub fn corpusProbeReferentialIntegrity() -> Result<bool, crate::ComputeError> {
    Ok({ let _x_10 = corpusValidModel(); { let _x_11 = corpusManifest(); { let _x_12 = validateModelReferentialIntegrity(&(_x_10), &(_x_11))?; match _x_12 {
        false => _x_12,
        true => { let _x_23 = corpusInvalidReferentialIntegrityModel(); { let _x_24 = corpusManifest(); { let _x_25 = validateModelReferentialIntegrity(&(_x_23), &(_x_24))?; match _x_25 {
        false => _x_12,
        true => { let _x_27 = false; _x_27 },
    } } } },
    } } } })
}

pub fn corpusProbeReleaseCompleteness() -> Result<bool, crate::ComputeError> {
    Ok({ let _x_10 = corpusValidModel(); { let _x_11 = corpusManifest(); { let _x_12 = validateModelReleaseCompleteness(&(_x_10), &(_x_11))?; match _x_12 {
        false => _x_12,
        true => { let _x_23 = corpusInvalidReleaseCompletenessModel(); { let _x_24 = corpusManifest(); { let _x_25 = validateModelReleaseCompleteness(&(_x_23), &(_x_24))?; match _x_25 {
        false => _x_12,
        true => { let _x_27 = false; _x_27 },
    } } } },
    } } } })
}

pub fn corpusProbeRollbackSafety() -> Result<bool, crate::ComputeError> {
    Ok({ let _x_10 = corpusValidModel(); { let _x_11 = corpusManifest(); { let _x_12 = validateModelRollbackSafety(&(_x_10), &(_x_11))?; match _x_12 {
        false => _x_12,
        true => { let _x_23 = corpusInvalidRollbackSafetyModel(); { let _x_24 = corpusManifest(); { let _x_25 = validateModelRollbackSafety(&(_x_23), &(_x_24))?; match _x_25 {
        false => _x_12,
        true => { let _x_27 = false; _x_27 },
    } } } },
    } } } })
}

pub fn corpusProbeSecretFlow() -> bool {
    { let _x_10 = corpusValidModel(); { let _x_11 = corpusManifest(); { let _x_12 = validateModelSecretFlow(&(_x_10), &(_x_11)); match _x_12 {
        false => _x_12,
        true => { let _x_23 = corpusInvalidSecretFlowModel(); { let _x_24 = corpusManifest(); { let _x_25 = validateModelSecretFlow(&(_x_23), &(_x_24)); match _x_25 {
        false => _x_12,
        true => { let _x_27 = false; _x_27 },
    } } } },
    } } } }
}

pub fn corpusProbeUniqueness() -> Result<bool, crate::ComputeError> {
    Ok({ let _x_10 = corpusValidModel(); { let _x_11 = corpusManifest(); { let _x_12 = validateModelUniqueness(&(_x_10), &(_x_11))?; match _x_12 {
        false => _x_12,
        true => { let _x_23 = corpusInvalidUniquenessModel(); { let _x_24 = corpusManifest(); { let _x_25 = validateModelUniqueness(&(_x_23), &(_x_24))?; match _x_25 {
        false => _x_12,
        true => { let _x_27 = false; _x_27 },
    } } } },
    } } } })
}

pub fn corpusValidModel() -> crate::SystemModel {
    { let _x_8 = alloc::vec![alloc::string::String::from("platform")]; { let _x_9 = crate::Product { id: alloc::string::String::from("product"), owner: alloc::string::String::from("PrismPM"), version: alloc::string::String::from("1"), lifecycle: alloc::string::String::from("development"), sourcePolicy: alloc::string::String::from("closed"), supportedPlatforms: _x_8.clone() }; { let _x_16 = crate::Artifact { id: alloc::string::String::from("artifact"), mediaType: alloc::string::String::from("application/octet-stream"), digest: alloc::string::String::from("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), licenseExpression: alloc::string::String::from("MIT"), path: alloc::string::String::from("artifact.bin"), role: alloc::string::String::from("runtime"), platformRequirements: _x_8.clone() }; { let _x_18 = alloc::vec![_x_16]; { let _x_22 = alloc::vec![alloc::string::String::from("capability")]; { let _x_24 = alloc::vec![alloc::string::String::from("run")]; { let _x_26 = alloc::vec![alloc::string::String::from("interface")]; { let _x_136 = 1; { let _x_138 = 1; { let _x_33 = crate::ResourceRequirements { cpuMillis: _x_136, memoryBytes: _x_138, replicasMin: _x_136, replicasMax: _x_136 }; { let _x_35 = alloc::vec![alloc::string::String::from("secret")]; { let _x_42 = crate::Component { id: alloc::string::String::from("component"), kind: alloc::string::String::from("service"), version: alloc::string::String::from("1"), artifact: alloc::string::String::from("artifact"), capabilities: _x_22.clone(), command: _x_24, dependsOn: alloc::vec::Vec::new(), interfaces: _x_26, health: alloc::string::String::from("healthy"), liveness: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), ports: alloc::vec::Vec::new(), readiness: alloc::vec::Vec::new(), resources: _x_33, secrets: _x_35, startup: alloc::vec::Vec::new(), volumes: alloc::vec::Vec::new(), isolation: alloc::string::String::from("value"), placement: alloc::vec::Vec::new(), platformRequirements: _x_8.clone(), scalingPolicy: None, failurePolicy: alloc::string::String::from("fail-closed"), retryPolicy: alloc::string::String::from("bounded"), degradationPolicy: alloc::string::String::from("unavailable"), idempotency: alloc::string::String::from("keyed") }; { let _x_44 = alloc::vec![_x_42]; { let _x_51 = alloc::vec![alloc::string::String::from("acceptance")]; { let _x_52 = crate::Interface { id: alloc::string::String::from("interface"), kind: alloc::string::String::from("http"), document: alloc::string::String::from("schema"), authentication: alloc::string::String::from("oidc"), compatibility: alloc::string::String::from("exact"), protocol: alloc::string::String::from("https"), errors: alloc::vec::Vec::new(), acceptance: _x_51 }; { let _x_54 = alloc::vec![_x_52]; { let _x_57 = crate::Schema { id: alloc::string::String::from("schema"), kind: alloc::string::String::from("json-schema"), value: alloc::string::String::from("{}"), dependsOn: alloc::vec::Vec::new(), compatibility: alloc::string::String::from("exact") }; { let _x_59 = alloc::vec![_x_57]; { let _x_66 = alloc::vec![alloc::string::String::from("amd64")]; { let _x_69 = crate::PlatformRequirement { id: alloc::string::String::from("platform"), os: alloc::string::String::from("linux"), architectures: _x_66, runtime: alloc::string::String::from("oci"), runtimeVersion: alloc::string::String::from("1.1"), capabilities: _x_22.clone() }; { let _x_71 = alloc::vec![_x_69]; { let _x_76 = alloc::vec![alloc::string::String::from("component")]; { let _x_78 = crate::SecretReference { id: alloc::string::String::from("secret"), providerKey: alloc::string::String::from("secret://provider/key"), consumers: _x_76, rotation: alloc::string::String::from("automatic") }; { let _x_80 = alloc::vec![_x_78]; { let _x_86 = crate::Migration { id: alloc::string::String::from("migration"), kind: alloc::string::String::from("initialize"), value: alloc::string::String::from("SELECT 1;"), dependsOn: alloc::vec::Vec::new() }; { let _x_88 = alloc::vec![_x_86]; { let _x_90 = crate::Observability { logs: alloc::vec::Vec::new(), metrics: alloc::vec::Vec::new(), traces: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), redactedFields: alloc::vec::Vec::new() }; { let _x_96 = crate::Capability { id: alloc::string::String::from("capability"), kind: alloc::string::String::from("runtime"), value: alloc::string::String::from("present"), dependsOn: alloc::vec::Vec::new() }; { let _x_98 = alloc::vec![_x_96]; { let _x_103 = Some(alloc::string::String::from("secret")); { let _x_104 = Some(alloc::string::String::from("artifact")); { let _x_105 = crate::TargetBinding { id: alloc::string::String::from("target"), kind: alloc::string::String::from("compose"), apiVersion: alloc::string::String::from("1"), adapterDigest: alloc::string::String::from("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"), minimumReleaseStatus: alloc::string::String::from("development"), capabilities: _x_22.clone(), credentials: _x_103, platformRequirements: _x_8.clone(), storageClass: None, storageProfile: None, ingressClassName: None, ingressControllerArtifact: _x_104 }; { let _x_107 = alloc::vec![_x_105]; { let _x_111 = alloc::vec![alloc::string::String::from("migration")]; { let _x_112 = crate::Rollout { id: alloc::string::String::from("rollout"), kind: alloc::string::String::from("rolling"), value: alloc::string::String::from("one-at-a-time"), dependsOn: _x_111 }; { let _x_114 = alloc::vec![_x_112]; { let _x_116 = alloc::vec![alloc::string::String::from("rollout")]; { let _x_117 = { let mut __list = alloc::vec![alloc::string::String::from("migration")]; __list.extend(_x_116); __list }; { let _x_118 = crate::Rollback { id: alloc::string::String::from("rollback"), kind: alloc::string::String::from("automatic"), value: alloc::string::String::from("rollback"), dependsOn: _x_117 }; { let _x_120 = alloc::vec![_x_118]; { let _x_125 = alloc::vec![alloc::string::String::from("probe")]; { let _x_127 = true; { let _x_128 = crate::Acceptance { id: alloc::string::String::from("acceptance"), kind: alloc::string::String::from("positive"), target: alloc::string::String::from("target"), component: alloc::string::String::from("component"), command: _x_125, evidence: alloc::string::String::from("evidence"), bounded: _x_127 }; { let _x_130 = alloc::vec![_x_128]; { let _x_131 = crate::Lifecycle { backup: alloc::string::String::from("value"), drift: alloc::string::String::from("value"), migration: alloc::string::String::from("value"), recovery: alloc::string::String::from("value"), retirement: alloc::string::String::from("value"), rollback: alloc::string::String::from("value"), rollout: alloc::string::String::from("value") }; { let _x_133 = crate::TransactionalCommandView { title: alloc::string::String::from("value"), heading: alloc::string::String::from("value"), identityHeading: alloc::string::String::from("value"), commandFormHeading: alloc::string::String::from("value"), historyHeading: alloc::string::String::from("value"), principalLabel: alloc::string::String::from("value"), principalDefault: alloc::string::String::from("value"), roleLabel: alloc::string::String::from("value"), submitterRoleLabel: alloc::string::String::from("value"), observerRoleLabel: alloc::string::String::from("value"), deniedRoleLabel: alloc::string::String::from("value"), authenticateLabel: alloc::string::String::from("value"), commandIdLabel: alloc::string::String::from("value"), inputALabel: alloc::string::String::from("value"), operationLabel: alloc::string::String::from("value"), inputBLabel: alloc::string::String::from("value"), annotationLabel: alloc::string::String::from("value"), submitLabel: alloc::string::String::from("value"), refreshLabel: alloc::string::String::from("value"), emptyText: alloc::string::String::from("value"), loadingText: alloc::string::String::from("value"), authenticatedText: alloc::string::String::from("value"), accessDeniedText: alloc::string::String::from("value"), retryLabel: alloc::string::String::from("value"), sequenceHeading: alloc::string::String::from("value"), commandColumnHeading: alloc::string::String::from("value"), inputSummaryHeading: alloc::string::String::from("value"), outcomeHeading: alloc::string::String::from("value"), annotationHeading: alloc::string::String::from("value"), releasePrefix: alloc::string::String::from("value"), historyLoadedText: alloc::string::String::from("value"), historyEmptyText: alloc::string::String::from("value"), authenticatingText: alloc::string::String::from("value"), authenticationFailedText: alloc::string::String::from("value"), submittingText: alloc::string::String::from("value"), outcomePrefix: alloc::string::String::from("value"), commandFailedText: alloc::string::String::from("value"), historyFailedText: alloc::string::String::from("value") }; { let _x_134 = crate::TransactionalCommandServiceProfile { contract: alloc::string::String::from("value"), applicationErrors: alloc::vec::Vec::new(), applicationModelDigest: alloc::string::String::from("value"), commandEncoding: alloc::string::String::from("value"), responseVersion: alloc::string::String::from("value"), optionalAnnotation: alloc::string::String::from("value"), coreArtifact: alloc::string::String::from("value"), identityAudience: alloc::string::String::from("value"), submitterRole: alloc::string::String::from("value"), observerRole: alloc::string::String::from("value"), eventSource: alloc::string::String::from("value"), acceptedEventType: alloc::string::String::from("value"), rejectedEventType: alloc::string::String::from("value"), databaseName: alloc::string::String::from("value"), databaseUser: alloc::string::String::from("value"), eventExchange: alloc::string::String::from("value"), auditQueue: alloc::string::String::from("value"), commandIdPattern: alloc::string::String::from("value"), commandPath: alloc::string::String::from("value"), tokenPath: alloc::string::String::from("value"), commandOperations: alloc::vec::Vec::new(), annotationMaxScalars: _x_136, historyDefaultLimit: _x_136, historyMaxLimit: _x_136, maxRequestBytes: _x_138, availabilityThresholdMillionths: _x_138, outboxLagThresholdMillis: _x_138, commandIdField: alloc::string::String::from("value"), operationField: alloc::string::String::from("value"), inputAField: alloc::string::String::from("value"), inputBField: alloc::string::String::from("value"), annotationField: alloc::string::String::from("value"), commandIdColumn: alloc::string::String::from("value"), operationColumn: alloc::string::String::from("value"), inputAColumn: alloc::string::String::from("value"), inputBColumn: alloc::string::String::from("value"), annotationColumn: alloc::string::String::from("value"), acceptanceErrorOperation: alloc::string::String::from("value"), acceptanceErrorInputA: alloc::string::String::from("value"), acceptanceErrorInputB: alloc::string::String::from("value"), acceptanceExpectedError: alloc::string::String::from("value"), acceptanceConflictInputB: alloc::string::String::from("value"), acceptanceSuccessOperation: alloc::string::String::from("value"), acceptanceSuccessInputA: alloc::string::String::from("value"), acceptanceSuccessInputB: alloc::string::String::from("value"), acceptanceExpectedResult: alloc::string::String::from("value"), view: _x_133, brokerUser: alloc::string::String::from("value"), historyTable: alloc::string::String::from("value"), outboxTable: alloc::string::String::from("value"), auditTable: alloc::string::String::from("value"), publicHostnameParameter: alloc::string::String::from("value") }; { let _x_135 = crate::SystemModel { product: _x_9, artifacts: _x_18, components: _x_44, interfaces: _x_54, schemas: _x_59, calls: alloc::vec::Vec::new(), events: alloc::vec::Vec::new(), flows: alloc::vec::Vec::new(), topology: alloc::vec::Vec::new(), platformRequirements: _x_71, scalingPolicies: alloc::vec::Vec::new(), storageClasses: alloc::vec::Vec::new(), parameters: alloc::vec::Vec::new(), secretReferences: _x_80, identityRequirements: alloc::vec::Vec::new(), persistence: alloc::vec::Vec::new(), migrations: _x_88, backups: alloc::vec::Vec::new(), observability: _x_90, slis: alloc::vec::Vec::new(), slos: alloc::vec::Vec::new(), alerts: alloc::vec::Vec::new(), controls: alloc::vec::Vec::new(), capabilities: _x_98, architecture: alloc::vec::Vec::new(), targets: _x_107, rollouts: _x_114, rollbacks: _x_120, drifts: alloc::vec::Vec::new(), retirements: alloc::vec::Vec::new(), acceptance: _x_130, lifecycle: _x_131, applicationProfile: _x_134, standards: alloc::vec::Vec::new() }; _x_135 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn nonEmptyNatList(x_1: &[u64]) -> bool {
    match x_1 {
        [] => { let _x_18 = false; _x_18 },
        [head_12, tail_13 @ ..] => { let head_12 = head_12.clone(); { let _x_19 = true; _x_19 } },
    }
}

pub fn relationAllBelow(x_1: u64, x_2: &[u64]) -> bool {
    match x_2 {
        [] => { let _x_30 = true; _x_30 },
        [head_21, tail_22 @ ..] => { let head_21 = head_21.clone(); { let _x_31 = (head_21 < x_1); match _x_31 {
        false => _x_31,
        true => { let _x_34 = relationAllBelow(x_1, &(tail_22)); _x_34 },
    } } },
    }
}

pub fn relationAllConsecutive(x_1: u64, x_2: &[u64]) -> Result<bool, crate::ComputeError> {
    Ok(match x_2 {
        [] => { let _x_45 = true; _x_45 },
        [head_28, tail_29 @ ..] => { let head_28 = head_28.clone(); { let _x_50 = (x_1 == head_28); match _x_50 {
        false => _x_50,
        true => { let _x_54 = 1; { let _x_55 = ((x_1) as u64).checked_add(_x_54).ok_or(crate::ComputeError::AddOverflow)?; { let _x_56 = relationAllConsecutive(_x_55, &(tail_29))?; _x_56 } } },
    } } },
    })
}

pub fn relationAllPositive(x_1: &[u64]) -> bool {
    match x_1 {
        [] => { let _x_35 = true; _x_35 },
        [head_24, tail_25 @ ..] => { let head_24 = head_24.clone(); { let _x_36 = 0; { let _x_37 = (_x_36 < head_24); match _x_37 {
        false => _x_37,
        true => { let _x_40 = relationAllPositive(&(tail_25)); _x_40 },
    } } } },
    }
}

pub fn validateCapabilitySatisfaction(bound: u64, values: &[u64]) -> bool {
    { let _x_19 = nonEmptyNatList(&(values)); match _x_19 {
        false => _x_19,
        true => { let _x_34 = 0; { let _x_35 = (_x_34 < bound); match _x_35 {
        false => _x_35,
        true => { let _x_38 = relationAllBelow(bound, &(values)); _x_38 },
    } } },
    } }
}

pub fn validateClosure(bound: u64, values: &[u64]) -> bool {
    { let _x_19 = nonEmptyNatList(&(values)); match _x_19 {
        false => _x_19,
        true => { let _x_34 = 0; { let _x_35 = (_x_34 < bound); match _x_35 {
        false => _x_35,
        true => { let _x_38 = relationAllBelow(bound, &(values)); _x_38 },
    } } },
    } }
}

pub fn validateCompatibility(bound: u64, values: &[u64]) -> bool {
    { let _x_19 = nonEmptyNatList(&(values)); match _x_19 {
        false => _x_19,
        true => { let _x_34 = 0; { let _x_35 = (_x_34 < bound); match _x_35 {
        false => _x_35,
        true => { let _x_38 = relationAllBelow(bound, &(values)); _x_38 },
    } } },
    } }
}

pub fn validateDeploymentOrder(values: &[u64]) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_10 = nonEmptyNatList(&(values)); match _x_10 {
        false => _x_10,
        true => { let _x_21 = 0; { let _x_22 = relationAllConsecutive(_x_21, &(values))?; _x_22 } },
    } })
}

pub fn validateEvidenceClosure(bound: u64, values: &[u64]) -> bool {
    { let _x_19 = nonEmptyNatList(&(values)); match _x_19 {
        false => _x_19,
        true => { let _x_34 = 0; { let _x_35 = (_x_34 < bound); match _x_35 {
        false => _x_35,
        true => { let _x_38 = relationAllBelow(bound, &(values)); _x_38 },
    } } },
    } }
}

pub fn validateLicenseClosure(values: &[u64]) -> bool {
    { let _x_7 = nonEmptyNatList(&(values)); match _x_7 {
        false => _x_7,
        true => { let _x_17 = relationAllPositive(&(values)); _x_17 },
    } }
}

pub fn validateMigrationOrder(values: &[u64]) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_10 = nonEmptyNatList(&(values)); match _x_10 {
        false => _x_10,
        true => { let _x_21 = 0; { let _x_22 = relationAllConsecutive(_x_21, &(values))?; _x_22 } },
    } })
}

pub fn validateReferentialIntegrity(bound: u64, values: &[u64]) -> bool {
    { let _x_19 = nonEmptyNatList(&(values)); match _x_19 {
        false => _x_19,
        true => { let _x_34 = 0; { let _x_35 = (_x_34 < bound); match _x_35 {
        false => _x_35,
        true => { let _x_38 = relationAllBelow(bound, &(values)); _x_38 },
    } } },
    } }
}

pub fn validateReleaseCompleteness(values: &[u64]) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_10 = nonEmptyNatList(&(values)); match _x_10 {
        false => _x_10,
        true => { let _x_21 = 0; { let _x_22 = relationAllConsecutive(_x_21, &(values))?; _x_22 } },
    } })
}

pub fn validateRollbackSafety(bound: u64, values: &[u64]) -> bool {
    { let _x_19 = nonEmptyNatList(&(values)); match _x_19 {
        false => _x_19,
        true => { let _x_34 = 0; { let _x_35 = (_x_34 < bound); match _x_35 {
        false => _x_35,
        true => { let _x_38 = relationAllBelow(bound, &(values)); _x_38 },
    } } },
    } }
}

pub fn validateSecretFlow(bound: u64, values: &[u64]) -> bool {
    { let _x_19 = nonEmptyNatList(&(values)); match _x_19 {
        false => _x_19,
        true => { let _x_34 = 0; { let _x_35 = (_x_34 < bound); match _x_35 {
        false => _x_35,
        true => { let _x_38 = relationAllBelow(bound, &(values)); _x_38 },
    } } },
    } }
}

pub fn validateUniqueness(values: &[u64]) -> Result<bool, crate::ComputeError> {
    Ok({ let _x_10 = nonEmptyNatList(&(values)); match _x_10 {
        false => _x_10,
        true => { let _x_21 = 0; { let _x_22 = relationAllConsecutive(_x_21, &(values))?; _x_22 } },
    } })
}
