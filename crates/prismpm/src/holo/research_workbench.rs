//! Closed, capability-free Text/4 workbench presentation and result contracts.

use super::model_document::{ApplicationAcceptanceVector, TextApplicationModel, TextView};
use serde::{Deserialize, Serialize};

/// Immutable workbench application profile.
pub const PROFILE: &str = "prismpm/text-application/4";

/// Closed FieldKind presentation choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldKind {
    /// Text choice.
    Text,
    /// Multiline choice.
    Multiline,
    /// Natural choice.
    Natural,
    /// Choice choice.
    Choice,
}

/// Closed Palette presentation choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Palette {
    /// Indigo choice.
    Indigo,
    /// Ocean choice.
    Ocean,
}

/// Closed Density presentation choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Density {
    /// Comfortable choice.
    Comfortable,
    /// Compact choice.
    Compact,
}

/// Closed Width presentation choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Width {
    /// Balanced choice.
    Balanced,
    /// Wide choice.
    Wide,
}

/// Closed ColorScheme presentation choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorScheme {
    /// System choice.
    System,
    /// Light choice.
    Light,
    /// Dark choice.
    Dark,
}

/// Source-owned Choice presentation record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    /// Modeled value value.
    pub value: String,
    /// Modeled label value.
    pub label: String,
}

/// Source-owned Field presentation record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    /// Modeled id value.
    pub id: String,
    /// Modeled label value.
    pub label: String,
    /// Modeled kind value.
    pub kind: FieldKind,
    /// Modeled help value.
    pub help: String,
    /// Modeled placeholder value.
    pub placeholder: String,
    /// Modeled initialValue value.
    pub initial_value: String,
    /// Modeled required value.
    pub required: bool,
    /// Modeled choices value.
    pub choices: Vec<Choice>,
}

/// Source-owned Example presentation record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Example {
    /// Modeled id value.
    pub id: String,
    /// Modeled label value.
    pub label: String,
    /// Modeled description value.
    pub description: String,
    /// Modeled values value.
    pub values: Vec<String>,
    /// Modeled expectedResponse value.
    pub expected_response: String,
    /// Modeled explanation value.
    pub explanation: String,
}

/// Source-owned Operation presentation record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Operation {
    /// Modeled id value.
    pub id: String,
    /// Modeled label value.
    pub label: String,
    /// Modeled summary value.
    pub summary: String,
    /// Modeled help value.
    pub help: String,
    /// Modeled requestPrefix value.
    pub request_prefix: String,
    /// Modeled fields value.
    pub fields: Vec<Field>,
    /// Modeled examples value.
    pub examples: Vec<Example>,
    /// Explicit selected-cell source action.
    pub trace: ActionSpec,
    /// Explicit source route comparison.
    pub comparison: ActionSpec,
    /// Explicit source inspection action.
    pub inspection: ActionSpec,
}

/// Exact generated-model identity policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeIdentityPolicy {
    /// Require the actual generated adapter model.
    CurrentModel,
}
/// Closed source-owned RuntimeBinding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeBinding {
    /// Modeled model_identity.
    pub model_identity: RuntimeIdentityPolicy,
}

/// Closed source-owned ActionSpec.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionSpec {
    /// Modeled id.
    pub id: String,
    /// Modeled label.
    pub label: String,
    /// Modeled help.
    pub help: String,
    /// Modeled request_prefix.
    pub request_prefix: String,
    /// Modeled timeout_milliseconds.
    pub timeout_milliseconds: u32,
    /// Modeled fields.
    pub fields: Vec<Field>,
}

/// Closed source-owned ToolSpec.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolSpec {
    /// Modeled id.
    pub id: String,
    /// Modeled label.
    pub label: String,
    /// Modeled help.
    pub help: String,
    /// Modeled action.
    pub action: ActionSpec,
}

/// Closed source-owned ResearchSpec.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchSpec {
    /// Modeled title.
    pub title: String,
    /// Modeled help.
    pub help: String,
    /// Modeled trace_heading.
    pub trace_heading: String,
    /// Modeled comparison_heading.
    pub comparison_heading: String,
    /// Modeled inspection_heading.
    pub inspection_heading: String,
    /// Modeled selected_cell_label.
    pub selected_cell_label: String,
    /// Modeled inspect_cell_label.
    pub inspect_cell_label: String,
    /// Modeled left_panel_label.
    pub left_panel_label: String,
    /// Modeled right_panel_label.
    pub right_panel_label: String,
    /// Modeled equivalent_label.
    pub equivalent_label: String,
    /// Modeled different_label.
    pub different_label: String,
    /// Modeled no_selection_label.
    pub no_selection_label: String,
    /// Modeled table_page_label.
    pub table_page_label: String,
    /// Modeled previous_page_label.
    pub previous_page_label: String,
    /// Modeled next_page_label.
    pub next_page_label: String,
    /// Modeled table_rows_label.
    pub table_rows_label: String,
    /// Modeled page_rows.
    pub page_rows: u32,
    /// Modeled maximum_tables.
    pub maximum_tables: u32,
    /// Modeled maximum_table_rows.
    pub maximum_table_rows: u32,
    /// Modeled maximum_table_columns.
    pub maximum_table_columns: u32,
    /// Modeled maximum_panels.
    pub maximum_panels: u32,
    /// Modeled tools.
    pub tools: Vec<ToolSpec>,
}

/// Closed source-owned CaseExchangeSpec.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseExchangeSpec {
    /// Modeled title.
    pub title: String,
    /// Modeled help.
    pub help: String,
    /// Modeled privacy.
    pub privacy: String,
    /// Modeled file_label.
    pub file_label: String,
    /// Modeled import_action.
    pub import_action: ActionSpec,
    /// Modeled export_action.
    pub export_action: ActionSpec,
    /// Modeled recompute_action.
    pub recompute_action: ActionSpec,
    /// Modeled file_name.
    pub file_name: String,
    /// Modeled media_type.
    pub media_type: String,
    /// Modeled maximum_bytes.
    pub maximum_bytes: u32,
    /// Modeled identity_label.
    pub identity_label: String,
    /// Modeled mismatch_label.
    pub mismatch_label: String,
    /// Modeled download_label.
    pub download_label: String,
    /// Modeled file_content_label.
    pub file_content_label: String,
    /// Modeled file_read_error_code.
    pub file_read_error_code: String,
    /// Modeled file_encoding_error_code.
    pub file_encoding_error_code: String,
    /// Modeled file_too_large_error_code.
    pub file_too_large_error_code: String,
}

/// Source-owned Section presentation record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Section {
    /// Modeled id value.
    pub id: String,
    /// Modeled title value.
    pub title: String,
    /// Modeled paragraphs value.
    pub paragraphs: Vec<String>,
}

/// Source-owned Link presentation record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    /// Modeled label value.
    pub label: String,
    /// Modeled httpsUrl value.
    pub https_url: String,
}

/// Source-owned ErrorHint presentation record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorHint {
    /// Modeled code value.
    pub code: String,
    /// Modeled title value.
    pub title: String,
    /// Modeled explanation value.
    pub explanation: String,
    /// Modeled recovery value.
    pub recovery: String,
    /// Modeled fields value.
    pub fields: Vec<String>,
}

/// Source-owned StatusCopy presentation record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatusCopy {
    /// Source-owned message when JavaScript is disabled.
    pub script_required: String,
    /// Modeled initializing value.
    pub initializing: String,
    /// Modeled ready value.
    pub ready: String,
    /// Modeled running value.
    pub running: String,
    /// Modeled cancelling value.
    pub cancelling: String,
    /// Modeled cancelled value.
    pub cancelled: String,
    /// Modeled timeout value.
    pub timeout: String,
    /// Modeled loadError value.
    pub load_error: String,
    /// Modeled computeError value.
    pub compute_error: String,
    /// Modeled inputError value.
    pub input_error: String,
    /// Modeled responseError value.
    pub response_error: String,
}

/// Source-owned Design presentation record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Design {
    /// Modeled palette value.
    pub palette: Palette,
    /// Modeled density value.
    pub density: Density,
    /// Modeled width value.
    pub width: Width,
    /// Modeled colorScheme value.
    pub color_scheme: ColorScheme,
}

/// Source-owned RawEditor presentation record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawEditor {
    /// Modeled requestPrefix value.
    pub request_prefix: String,
    /// Modeled label value.
    pub label: String,
    /// Modeled help value.
    pub help: String,
    /// Modeled placeholder value.
    pub placeholder: String,
    /// Modeled initialValue value.
    pub initial_value: String,
    /// Modeled submitLabel value.
    pub submit_label: String,
    /// Modeled responseLabel value.
    pub response_label: String,
    /// Modeled expanded value.
    pub expanded: bool,
}

/// Source-owned DisplaySpec presentation record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DisplaySpec {
    /// Modeled title value.
    pub title: String,
    /// Modeled emptyText value.
    pub empty_text: String,
    /// Modeled rawLabel value.
    pub raw_label: String,
    /// Modeled maximumCells value.
    pub maximum_cells: u32,
    /// Modeled maximumDetails value.
    pub maximum_details: u32,
}

/// Closed browser Wasm cache policy owned by the source execution model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WasmCachePolicy {
    /// Fetch the deployed Wasm without reusing a prior HTTP cache entry.
    #[serde(rename = "no-store")]
    NoStore,
}

/// Source-owned Execution presentation record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Execution {
    /// Modeled timeoutMilliseconds value.
    pub timeout_milliseconds: u32,
    /// Modeled cancelLabel value.
    pub cancel_label: String,
    /// Modeled retryLabel value.
    pub retry_label: String,
    /// Required source-owned Wasm cache policy, with no implicit default.
    pub wasm_cache_policy: WasmCachePolicy,
}

/// Source-owned WorkbenchView presentation record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchView {
    /// Modeled title value.
    pub title: String,
    /// Modeled heading value.
    pub heading: String,
    /// Modeled purpose value.
    pub purpose: String,
    /// Modeled audience value.
    pub audience: String,
    /// Modeled guidance value.
    pub guidance: Vec<String>,
    /// Modeled initialOperation value.
    pub initial_operation: String,
    /// Modeled initialExample value.
    pub initial_example: String,
    /// Modeled operations value.
    pub operations: Vec<Operation>,
    /// Modeled sections value.
    pub sections: Vec<Section>,
    /// Modeled links value.
    pub links: Vec<Link>,
    /// Modeled errors value.
    pub errors: Vec<ErrorHint>,
    /// Modeled statuses value.
    pub statuses: StatusCopy,
    /// Modeled design value.
    pub design: Design,
    /// Modeled raw value.
    pub raw: RawEditor,
    /// Modeled response value.
    pub response: DisplaySpec,
    /// Modeled execution value.
    pub execution: Execution,
    /// Modeled labels value.
    pub labels: WorkbenchLabels,
    /// Explicit research presentation authority.
    pub research: ResearchSpec,
    /// Explicit local file exchange authority.
    pub case_exchange: CaseExchangeSpec,
    /// Runtime generated-model identity binding.
    pub runtime_binding: RuntimeBinding,
}

/// Source-owned WorkbenchLabels presentation record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchLabels {
    /// Modeled navigation value.
    pub navigation: String,
    /// Modeled operation value.
    pub operation: String,
    /// Modeled examples value.
    pub examples: String,
    /// Modeled loadExample value.
    pub load_example: String,
    /// Modeled compute value.
    pub compute: String,
    /// Modeled guide value.
    pub guide: String,
    /// Modeled links value.
    pub links: String,
    /// Modeled fields value.
    pub fields: String,
    /// Modeled expectedResponse value.
    pub expected_response: String,
    /// Modeled details value.
    pub details: String,
    /// Modeled resultTable value.
    pub result_table: String,
    /// Modeled row value.
    pub row: String,
    /// Modeled column value.
    pub column: String,
    /// Modeled request value.
    pub request: String,
    /// Modeled reset value.
    pub reset: String,
    /// Modeled skipToWorkbench value.
    pub skip_to_workbench: String,
}

/// A capability-free byte application with a closed guided workbench.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchApplication {
    /// Exact immutable text profile identifier.
    pub profile: String,
    /// Human-facing application name.
    pub name: String,
    /// Generated registry package name.
    pub cargo_name: String,
    /// Generated registry package version.
    pub cargo_version: String,
    /// Generated package description.
    pub cargo_description: String,
    /// Canonical package source repository URL.
    pub cargo_repository: String,
    /// Canonical package homepage URL.
    pub cargo_homepage: String,
    /// Fully qualified generated library roots in bytewise order.
    pub library_roots: Vec<String>,
    /// Application-owned byte-level acceptance corpus.
    pub acceptance_vectors: Vec<ApplicationAcceptanceVector>,
    /// Fully qualified generated byte request dispatcher.
    pub entry_root: String,
    /// Hologram guest contract.
    pub core_contract: String,
    /// Maximum UTF-8 request byte length.
    pub request_maximum: u32,
    /// Maximum UTF-8 response byte length.
    pub response_maximum: u32,
    /// Guest input allocation cap.
    pub guest_allocation_maximum: u32,
    /// Explicit Core-Wasm guest memory pages, required by text/4.
    pub core_memory_pages: u32,
    /// Explicit Core-Wasm linear-memory stack bytes, required by text/4.
    pub core_stack_bytes: u32,
    /// Whether the capability request is canonically empty.
    pub capabilities_empty: bool,
    /// Whether the application requests a composed fat archive.
    pub fat_archive: bool,
    /// Zero-based primary layer.
    pub primary_layer: u8,
    /// Zero-based View layer.
    pub view_layer: u8,
    /// Text presentation metadata; byte bounds are owned by the application.
    pub view: WorkbenchView,
}

impl WorkbenchApplication {
    /// Existing Text/2 byte and resource contract, without changing its validation.
    pub(super) fn byte_contract(&self) -> TextApplicationModel {
        TextApplicationModel {
            profile: "prismpm/text-application/2".to_owned(),
            name: self.name.clone(),
            cargo_name: self.cargo_name.clone(),
            cargo_version: self.cargo_version.clone(),
            cargo_description: self.cargo_description.clone(),
            cargo_repository: self.cargo_repository.clone(),
            cargo_homepage: self.cargo_homepage.clone(),
            library_roots: self.library_roots.clone(),
            acceptance_vectors: self.acceptance_vectors.clone(),
            entry_root: self.entry_root.clone(),
            core_contract: self.core_contract.clone(),
            request_maximum: self.request_maximum,
            response_maximum: self.response_maximum,
            guest_allocation_maximum: self.guest_allocation_maximum,
            core_memory_pages: Some(self.core_memory_pages),
            core_stack_bytes: Some(self.core_stack_bytes),
            capabilities_empty: self.capabilities_empty,
            fat_archive: self.fat_archive,
            primary_layer: self.primary_layer,
            view_layer: self.view_layer,
            view: TextView {
                title: self.view.title.clone(),
                heading: self.view.heading.clone(),
                input_label: self.view.raw.label.clone(),
                submit_label: self.view.raw.submit_label.clone(),
                output_label: self.view.raw.response_label.clone(),
                input_error: self.view.statuses.input_error.clone(),
                response_error: self.view.statuses.response_error.clone(),
            },
        }
    }
}

/// Strict, source-owned display result. All keys are required and duplicates and
/// unknown fields are refused by Serde before any presentation is constructed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchResult {
    /// Exact schema version.
    pub version: u32,
    /// Closed success or error discriminator.
    pub status: ResultStatus,
    /// Source operation ID, or `raw` for a raw wrapper.
    pub operation: String,
    /// Source-owned result summary.
    pub message: String,
    /// Safe table row count.
    pub rows: u32,
    /// Safe table column count.
    pub columns: u32,
    /// Row-major plain-text cells.
    pub cells: Vec<String>,
    /// Plain-text source explanations.
    pub details: Vec<String>,
    /// Exact underlying protocol request.
    pub request: String,
    /// Exact underlying protocol response.
    pub response: String,
    /// Modeled error code, empty for success.
    pub error: String,
    /// Correlated source action id.
    pub action: String,
    /// Selected zero-based output cell.
    pub selection: Selection,
    /// Complete source tables; pagination never drops rows.
    pub tables: Vec<ReportTable>,
    /// Exact independently executed source routes.
    pub panels: Vec<ReportPanel>,
    /// Source-computed comparison disposition.
    pub comparison: Comparison,
    /// Explicit case exchange result.
    pub exchange: Exchange,
}

/// Closed source-owned Selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    /// Modeled row.
    pub row: u32,
    /// Modeled column.
    pub column: u32,
}

/// Closed source-owned ReportTable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportTable {
    /// Modeled id.
    pub id: String,
    /// Modeled title.
    pub title: String,
    /// Modeled columns.
    pub columns: Vec<String>,
    /// Modeled rows.
    pub rows: Vec<Vec<String>>,
}

/// Closed source-owned ReportPanel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportPanel {
    /// Modeled id.
    pub id: String,
    /// Modeled title.
    pub title: String,
    /// Modeled rows.
    pub rows: u32,
    /// Modeled columns.
    pub columns: u32,
    /// Modeled cells.
    pub cells: Vec<String>,
    /// Modeled request.
    pub request: String,
    /// Modeled response.
    pub response: String,
}

/// Closed ComparisonStatus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ComparisonStatus {
    /// None disposition.
    None,
    /// Equivalent disposition.
    Equivalent,
    /// Different disposition.
    Different,
    /// Error disposition.
    Error,
}
/// Closed ExchangeStatus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExchangeStatus {
    /// None disposition.
    None,
    /// Exported disposition.
    Exported,
    /// Imported disposition.
    Imported,
    /// IdentityMismatch disposition.
    IdentityMismatch,
}
/// Closed source-owned Comparison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Comparison {
    /// Modeled status.
    pub status: ComparisonStatus,
    /// Modeled message.
    pub message: String,
}

/// Closed source-owned Draft.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    /// Modeled operation.
    pub operation: String,
    /// Modeled values.
    pub values: Vec<String>,
    /// Modeled raw.
    pub raw: String,
    /// Modeled controls.
    pub controls: Vec<String>,
    /// Modeled row.
    pub row: u32,
    /// Modeled column.
    pub column: u32,
}

/// Closed source-owned Exchange.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exchange {
    /// Modeled status.
    pub status: ExchangeStatus,
    /// Modeled file_name.
    pub file_name: String,
    /// Modeled file_text.
    pub file_text: String,
    /// Modeled draft.
    pub draft: Option<Draft>,
}

/// Closed result disposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultStatus {
    /// A computed result.
    Ok,
    /// A source-rejected request.
    Error,
}

fn invalid(message: &str) -> crate::error::PrismError {
    crate::error::PrismError::new("PP2009", message)
}

fn copy(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 16_384 && !value.chars().any(char::is_control)
}

fn id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.as_bytes()[0].is_ascii_alphabetic()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
}

fn field_value(value: &str) -> bool {
    value.len() <= 1_048_576
}

/// Decode a closed bounded source report without interpreting domain values.
pub fn result(
    application: &WorkbenchApplication,
    operation: &str,
    action: &str,
    bytes: &[u8],
) -> Result<WorkbenchResult, crate::error::PrismError> {
    use std::collections::BTreeSet;
    if bytes.len() > application.response_maximum as usize {
        return Err(invalid("research result exceeds its response byte limit"));
    }
    let result: WorkbenchResult = serde_json::from_slice(bytes)
        .map_err(|_| invalid("research result does not match its closed JSON contract"))?;
    let view = &application.view;
    let initial = view
        .operations
        .iter()
        .find(|op| op.id == view.initial_operation)
        .ok_or_else(|| invalid("research initial operation is missing"))?;
    let exchange = &result.exchange;
    let restored = exchange.draft.as_ref();
    let import_action = action == view.case_exchange.import_action.id
        || action == view.case_exchange.recompute_action.id;
    let operation_matches = result.operation == operation
        || operation == "raw" && view.operations.iter().any(|op| op.id == result.operation)
        || import_action && restored.is_some_and(|draft| draft.operation == result.operation);
    let shape = |rows: u32, columns: u32, cells: &[String]| {
        rows <= view.response.maximum_cells
            && columns <= view.response.maximum_cells
            && u64::from(rows) * u64::from(columns) == cells.len() as u64
            && cells.len() <= view.response.maximum_cells as usize
            && cells.iter().all(|v| copy(v))
    };
    let mut table_ids = BTreeSet::new();
    let mut panel_ids = BTreeSet::new();
    let tables = result.tables.len() <= view.research.maximum_tables as usize
        && result.tables.iter().all(|table| {
            id(&table.id)
                && table_ids.insert(&table.id)
                && copy(&table.title)
                && !table.columns.is_empty()
                && table.columns.len() <= view.research.maximum_table_columns as usize
                && table.columns.iter().all(|v| copy(v))
                && table.rows.len() <= view.research.maximum_table_rows as usize
                && table
                    .rows
                    .iter()
                    .all(|row| row.len() == table.columns.len() && row.iter().all(|v| copy(v)))
        });
    let panels = result.panels.len() <= view.research.maximum_panels as usize
        && result.panels.iter().all(|panel| {
            id(&panel.id)
                && panel_ids.insert(&panel.id)
                && copy(&panel.title)
                && shape(panel.rows, panel.columns, &panel.cells)
        });
    let draft_valid = restored.is_none_or(|draft| {
        let declared = view.operations.iter().find(|op| op.id == draft.operation);
        let tool = view
            .research
            .tools
            .iter()
            .find(|tool| tool.id == draft.operation);
        (draft.operation == "raw" && draft.values.is_empty()
            || tool.is_some() && draft.values.is_empty()
            || declared.is_some_and(|op| {
                draft.values.len() == op.fields.len()
                    && draft.values.iter().zip(&op.fields).all(|(value, field)| {
                        field_value(value)
                            && (!field.required || !value.trim().is_empty())
                            && (field.kind != FieldKind::Choice
                                || field.choices.iter().any(|choice| choice.value == *value))
                    })
            }))
            && draft.controls.len() <= 32
            && draft.controls.iter().all(|v| field_value(v))
            && {
                let fields = tool
                    .map(|tool| tool.action.fields.as_slice())
                    .or_else(|| declared.map(|op| op.inspection.fields.as_slice()))
                    .unwrap_or(initial.inspection.fields.as_slice());
                draft.controls.len() == fields.len()
                    && fields.iter().zip(&draft.controls).all(|(field, value)| {
                        (!field.required || !value.trim().is_empty())
                            && (field.kind != FieldKind::Choice
                                || field.choices.iter().any(|choice| choice.value == *value))
                    })
            }
            && draft.raw.len() <= application.request_maximum as usize
    });
    let exchange_valid = match exchange.status {
        ExchangeStatus::None => {
            exchange.file_name.is_empty() && exchange.file_text.is_empty() && restored.is_none()
        }
        ExchangeStatus::Exported => {
            action == view.case_exchange.export_action.id
                && exchange.file_name == view.case_exchange.file_name
                && !exchange.file_text.is_empty()
                && exchange.file_text.len() <= view.case_exchange.maximum_bytes as usize
                && restored.is_none()
        }
        ExchangeStatus::Imported => {
            import_action
                && restored.is_some()
                && exchange.file_name.is_empty()
                && exchange.file_text.is_empty()
        }
        ExchangeStatus::IdentityMismatch => {
            action == view.case_exchange.import_action.id
                && restored.is_some()
                && result.cells.is_empty()
                && result.rows == 0
                && result.columns == 0
                && result.request.is_empty()
                && result.response.is_empty()
                && exchange.file_name.is_empty()
                && exchange.file_text.is_empty()
        }
    };
    let compared = result.comparison.status != ComparisonStatus::None;
    if result.version != 2
        || result.action != action
        || !operation_matches
        || !copy(&result.message)
        || !shape(result.rows, result.columns, &result.cells)
        || !tables
        || !panels
        || result.status == ResultStatus::Ok
            && view.operations.iter().any(|op| op.comparison.id == action)
            && result.panels.len() < 2
        || !draft_valid
        || !exchange_valid
        || result.details.len() > view.response.maximum_details as usize
        || !result.details.iter().all(|v| copy(v))
        || if compared {
            !copy(&result.comparison.message)
        } else {
            !result.comparison.message.is_empty()
        }
        || match result.status {
            ResultStatus::Ok => !result.error.is_empty(),
            ResultStatus::Error => {
                !view.errors.iter().any(|hint| hint.code == result.error)
                    || result.rows != 0
                    || result.columns != 0
                    || !result.cells.is_empty()
            }
        }
    {
        return Err(invalid(
            "research result violates its action, operation, shape, exchange or display contract",
        ));
    }
    Ok(result)
}

/// Validate the closed workbench before generation and after strict decoding.
pub fn validate(application: &WorkbenchApplication) -> Result<(), crate::error::PrismError> {
    use std::collections::BTreeSet;
    if application.profile != PROFILE {
        return Err(invalid("unsupported workbench application profile"));
    }
    super::validate::validate_text_application(&application.byte_contract())?;
    if application.request_maximum > 2_097_152 || application.response_maximum > 4_194_304 {
        return Err(invalid(
            "Text/4 transport limits exceed official portable intent version2",
        ));
    }
    let view = &application.view;
    let encoded =
        serde_json::to_value(view).map_err(|_| invalid("workbench view cannot be encoded"))?;
    // Every visible label is source-owned; this exhaustive closed-record pass
    // keeps labels and prose independent from input values and expected JSON.
    fn record_copy(value: &serde_json::Value, key: &str) -> bool {
        match value {
            serde_json::Value::Object(fields) => {
                fields.iter().all(|(key, value)| record_copy(value, key))
            }
            serde_json::Value::Array(values) => values.iter().all(|value| record_copy(value, key)),
            serde_json::Value::String(value) => match key {
                "initial_value" | "placeholder" | "values" | "value" => field_value(value),
                "expected_response" => true,
                _ => copy(value),
            },
            _ => true,
        }
    }
    if !record_copy(&encoded, "view")
        || view.operations.is_empty()
        || view.operations.len() > 32
        || view.guidance.is_empty()
        || view.guidance.len() > 8
        || view.sections.len() > 32
        || view.links.len() > 32
        || view.errors.is_empty()
        || view.errors.len() > 128
        || view.response.maximum_cells == 0
        || view.response.maximum_cells > 4096
        || view.response.maximum_details == 0
        || view.response.maximum_details > 128
        || !(100..=120_000).contains(&view.execution.timeout_milliseconds)
        || !id(&view.raw.request_prefix)
    {
        return Err(invalid(
            "workbench view exceeds its closed copy, count, or execution bounds",
        ));
    }
    let mut operations = BTreeSet::new();
    let mut prefixes = BTreeSet::from([view.raw.request_prefix.as_str()]);
    let mut all_fields = BTreeSet::new();
    for operation in &view.operations {
        if !id(&operation.id)
            || operation.id == "raw"
            || !operations.insert(operation.id.as_str())
            || !id(&operation.request_prefix)
            || !prefixes.insert(&operation.request_prefix)
            || operation.fields.is_empty()
            || operation.fields.len() > 32
            || operation.examples.is_empty()
            || operation.examples.len() > 64
        {
            return Err(invalid(
                "workbench operations require unique IDs, framing prefixes, fields, and examples",
            ));
        }
        let mut fields = BTreeSet::new();
        for field in &operation.fields {
            if !id(&field.id)
                || !fields.insert(field.id.as_str())
                || field.choices.len() > 64
                || (field.kind == FieldKind::Choice) == field.choices.is_empty()
            {
                return Err(invalid("workbench field descriptor or choices are invalid"));
            }
            all_fields.insert(field.id.as_str());
            let mut choices = BTreeSet::new();
            for choice in &field.choices {
                if choice.value.is_empty() || !choices.insert(choice.value.as_str()) {
                    return Err(invalid(
                        "workbench choice values must be unique and nonempty",
                    ));
                }
            }
            let valid_value = |value: &str| {
                field_value(value)
                    && (!field.required || !value.trim().is_empty())
                    && (field.kind != FieldKind::Choice || choices.contains(value))
            };
            if !field_value(&field.initial_value)
                || (field.kind == FieldKind::Choice
                    && !choices.contains(field.initial_value.as_str()))
            {
                return Err(invalid("workbench initial field value is invalid"));
            }
            for example in &operation.examples {
                if example.values.len() != operation.fields.len() {
                    return Err(invalid(
                        "workbench example values do not match the operation fields",
                    ));
                }
                let index = operation
                    .fields
                    .iter()
                    .position(|candidate| candidate.id == field.id)
                    .expect("current field exists");
                if !valid_value(&example.values[index]) {
                    return Err(invalid("workbench example field value is invalid"));
                }
            }
        }
        let mut examples = BTreeSet::new();
        for example in &operation.examples {
            let request = example_request(operation, example)?;
            if !id(&example.id)
                || !examples.insert(example.id.as_str())
                || request.len() > application.request_maximum as usize
                || !application.acceptance_vectors.iter().any(|vector| {
                    vector.request == request.as_bytes()
                        && vector.response == example.expected_response.as_bytes()
                })
            {
                return Err(invalid(
                    "workbench examples require unique IDs and exact framed acceptance-vector bindings",
                ));
            }
            result(
                application,
                &operation.id,
                "compute",
                example.expected_response.as_bytes(),
            )?;
        }
    }
    if !view.operations.iter().any(|operation| {
        operation.id == view.initial_operation
            && operation
                .examples
                .iter()
                .any(|example| example.id == view.initial_example)
    }) {
        return Err(invalid(
            "workbench initial operation and example are not source-owned references",
        ));
    }
    let mut sections = BTreeSet::new();
    for section in &view.sections {
        if !id(&section.id)
            || !sections.insert(section.id.as_str())
            || section.paragraphs.is_empty()
            || section.paragraphs.len() > 32
        {
            return Err(invalid("workbench guide section is invalid"));
        }
    }
    for link in &view.links {
        let valid_uri = jsonschema::options().should_validate_formats(true).build(&serde_json::json!({"type":"string","format":"uri","pattern":"^https://[^/?#@]+(?:[/?#]|$)"})).expect("static workbench HTTPS URI schema");
        if !valid_uri.is_valid(&serde_json::Value::String(link.https_url.clone()))
            || !link.https_url.is_ascii()
            || link
                .https_url
                .bytes()
                .any(|b| b.is_ascii_whitespace() || b.is_ascii_control())
        {
            return Err(invalid(
                "workbench links require an absolute credential-free HTTPS URL",
            ));
        }
    }
    for action in view
        .operations
        .iter()
        .flat_map(|op| [&op.trace, &op.comparison, &op.inspection])
        .chain(view.research.tools.iter().map(|tool| &tool.action))
        .chain([
            &view.case_exchange.import_action,
            &view.case_exchange.export_action,
            &view.case_exchange.recompute_action,
        ])
    {
        for field in &action.fields {
            all_fields.insert(field.id.as_str());
        }
    }
    let mut codes = BTreeSet::new();
    for hint in &view.errors {
        let refs = hint
            .fields
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        if !id(&hint.code)
            || !codes.insert(&hint.code)
            || refs.len() != hint.fields.len()
            || refs.iter().any(|field| !all_fields.contains(field))
        {
            return Err(invalid(
                "workbench error hints require unique codes and declared field references",
            ));
        }
    }
    let mut guided = false;
    let mut raw = false;
    let mut rejected = false;
    for index in oracle_vectors(application).ui {
        let vector = &application.acceptance_vectors[index];
        let text = std::str::from_utf8(&vector.request).expect("selected UTF-8");
        let operation = if text.starts_with(&format!("{}\u{1f}", view.raw.request_prefix)) {
            "raw"
        } else {
            view.operations
                .iter()
                .find(|op| text.starts_with(&format!("{}\u{1f}", op.request_prefix)))
                .expect("selected prefix")
                .id
                .as_str()
        };
        let reply = result(application, operation, "compute", &vector.response)?;
        if reply.status == ResultStatus::Error {
            rejected = true;
        } else if operation == "raw" {
            raw = true;
        } else {
            guided = true;
        }
    }
    if !guided || !raw || !rejected {
        return Err(invalid(
            "research acceptance requires successful guided/raw computes and a typed source error",
        ));
    }
    validate_research(application)?;
    Ok(())
}

/// Stable source-only example envelope, independent of its eventual model hash.
pub fn example_request(
    operation: &Operation,
    example: &Example,
) -> Result<String, crate::error::PrismError> {
    #[derive(Serialize)]
    struct Envelope<'a> {
        version: u32,
        operation: &'a str,
        request: &'a str,
        values: &'a [String],
        row: u32,
        column: u32,
        controls: Vec<String>,
        model_id: &'a str,
        case_text: &'a str,
    }
    let value = Envelope {
        version: 1,
        operation: &operation.id,
        request: "",
        values: &example.values,
        row: 0,
        column: 0,
        controls: vec![],
        model_id: "",
        case_text: "",
    };
    Ok(format!(
        "{}\u{1f}{}",
        operation.request_prefix,
        serde_json::to_string(&value).map_err(|_| invalid("research example cannot encode"))?
    ))
}

fn validate_research(application: &WorkbenchApplication) -> Result<(), crate::error::PrismError> {
    use std::collections::{BTreeMap, BTreeSet};
    let view = &application.view;
    let r = &view.research;
    let c = &view.case_exchange;
    if r.page_rows == 0
        || r.page_rows > 256
        || r.maximum_tables == 0
        || r.maximum_tables > 64
        || r.maximum_table_rows == 0
        || r.maximum_table_rows > 65536
        || r.maximum_table_columns == 0
        || r.maximum_table_columns > 64
        || r.maximum_panels == 0
        || r.maximum_panels > 16
        || r.tools.is_empty()
        || r.tools.len() > 16
        || c.maximum_bytes == 0
        || c.maximum_bytes > application.request_maximum
        || c.media_type != "application/json"
        || !c.file_name.ends_with(".json")
        || c.file_name.len() > 128
        || c.file_name
            .bytes()
            .any(|b| !b.is_ascii_alphanumeric() && !matches!(b, b'.' | b'_' | b'-'))
    {
        return Err(invalid(
            "research table, tool or explicit file limits are invalid",
        ));
    }
    for code in [
        &c.file_read_error_code,
        &c.file_encoding_error_code,
        &c.file_too_large_error_code,
    ] {
        if !view.errors.iter().any(|hint| &hint.code == code) {
            return Err(invalid("file IO errors require source-owned error hints"));
        }
    }
    if !c.import_action.fields.is_empty()
        || !c.export_action.fields.is_empty()
        || !c.recompute_action.fields.is_empty()
        || BTreeSet::from([
            &c.import_action.id,
            &c.export_action.id,
            &c.recompute_action.id,
        ])
        .len()
            != 3
    {
        return Err(invalid(
            "case actions require distinct identities and explicit file/context inputs",
        ));
    }
    let mut prefixes = BTreeMap::new();
    let mut tools = BTreeSet::new();
    let mut actions = vec![&c.import_action, &c.export_action, &c.recompute_action];
    for op in &view.operations {
        if BTreeSet::from([&op.trace.id, &op.comparison.id, &op.inspection.id]).len() != 3
            || [
                &c.import_action.id,
                &c.export_action.id,
                &c.recompute_action.id,
            ]
            .iter()
            .any(|id| [&op.trace.id, &op.comparison.id, &op.inspection.id].contains(id))
        {
            return Err(invalid("research and case actions require distinct roles"));
        }
        actions.extend([&op.trace, &op.comparison, &op.inspection]);
    }
    for tool in &r.tools {
        if !id(&tool.id) || !tools.insert(&tool.id) {
            return Err(invalid("research tools require unique source ids"));
        }
        actions.push(&tool.action);
    }
    for action in actions {
        if !id(&action.id)
            || action.id == "compute"
            || !id(&action.request_prefix)
            || !(100..=180_000).contains(&action.timeout_milliseconds)
            || action.fields.len() > 32
            || prefixes
                .insert(&action.request_prefix, &action.id)
                .is_some_and(|id| id != &action.id)
            || view
                .operations
                .iter()
                .any(|op| op.request_prefix == action.request_prefix)
            || view.raw.request_prefix == action.request_prefix
        {
            return Err(invalid(
                "research action identity, prefix or deadline is invalid",
            ));
        }
        let mut ids = BTreeSet::new();
        for field in &action.fields {
            if !id(&field.id)
                || !ids.insert(&field.id)
                || !field_value(&field.initial_value)
                || field.choices.len() > 64
                || (field.kind == FieldKind::Choice) == field.choices.is_empty()
                || field.kind == FieldKind::Choice
                    && !field
                        .choices
                        .iter()
                        .any(|choice| choice.value == field.initial_value)
            {
                return Err(invalid("research action field contract is invalid"));
            }
        }
    }
    Ok(())
}
/// Complete direct-byte and exactly representable V4 compute partition.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct OracleVectors {
    pub(crate) direct: Vec<usize>,
    pub(crate) ui: Vec<usize>,
    pub(crate) transport_rejected: Vec<usize>,
    pub(crate) normalized: Vec<usize>,
    pub(crate) unrepresentable: Vec<usize>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct OracleEnvelope {
    version: u32,
    operation: String,
    request: String,
    values: Vec<String>,
    row: u32,
    column: u32,
    controls: Vec<String>,
    model_id: String,
    case_text: String,
}

pub(crate) fn oracle_vectors(application: &WorkbenchApplication) -> OracleVectors {
    let mut selected = OracleVectors::default();
    for (index, vector) in application.acceptance_vectors.iter().enumerate() {
        let Ok(text) = std::str::from_utf8(&vector.request) else {
            continue;
        };
        if vector.request.len() > application.request_maximum as usize {
            continue;
        }
        selected.direct.push(index);
        let Some((prefix, envelope)) = text.split_once('\u{1f}') else {
            continue;
        };
        let raw = prefix == application.view.raw.request_prefix;
        let op = application
            .view
            .operations
            .iter()
            .find(|op| op.request_prefix == prefix);
        if !raw && op.is_none() {
            continue;
        }
        let original = envelope;
        let Ok(envelope) = serde_json::from_str::<OracleEnvelope>(envelope) else {
            selected.unrepresentable.push(index);
            continue;
        };
        let fields = op.map(|op| op.fields.as_slice()).unwrap_or(&[]);
        if serde_json::to_string(&envelope).ok().as_deref() != Some(original)
            || envelope.version != 1
            || envelope.operation != if raw { "raw" } else { op.unwrap().id.as_str() }
            || (!raw && !envelope.request.is_empty())
            || !envelope.controls.is_empty()
            || !envelope.case_text.is_empty()
            || (!envelope.model_id.is_empty()
                && (envelope.model_id.len() != 64
                    || !envelope
                        .model_id
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))))
            || envelope.values.len() != fields.len()
        {
            selected.unrepresentable.push(index);
            continue;
        }
        let values = &envelope.values;
        if fields
            .iter()
            .zip(values)
            .any(|(field, value)| field.required && value.trim().is_empty())
        {
            selected.transport_rejected.push(index);
            continue;
        }
        if fields.iter().zip(values).any(|(field, value)| {
            field.kind == FieldKind::Choice
                && !field.choices.iter().any(|choice| choice.value == *value)
        }) {
            selected.unrepresentable.push(index);
            continue;
        }
        if fields.iter().zip(values).any(|(field, value)| {
            value.contains('\r') || field.kind != FieldKind::Multiline && value.contains('\n')
        }) || raw && envelope.request.contains('\r')
        {
            selected.normalized.push(index);
            continue;
        }
        selected.ui.push(index);
    }
    selected
}
