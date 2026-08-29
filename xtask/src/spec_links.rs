//! Specification-link validation gate.

use crate::Fail;
use repo_model::Model;
use std::collections::BTreeSet;
use std::path::Path;

struct TableRow {
    id: String,
    suite: String,
    statement: String,
}

fn parse_table(spec: &str) -> Result<Vec<TableRow>, Fail> {
    let registry = spec
        .split("## 3. Conformance ID Registry")
        .nth(1)
        .ok_or("SPEC.md has no ## 3. Conformance ID Registry")?;
    let mut rows = Vec::new();
    for line in registry.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("| `") {
            continue;
        }
        let cells: Vec<&str> = trimmed
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        if cells.len() != 4 {
            continue;
        }
        let strip_ticks = |cell: &str| cell.trim_matches('`').to_owned();
        let id = strip_ticks(cells[0]);
        if id.len() != 5 {
            continue;
        }
        rows.push(TableRow {
            id,
            suite: strip_ticks(cells[1]),
            statement: cells[2].to_owned(),
        });
    }
    Ok(rows)
}

/// Run spec-links validation.
pub fn validate(root: &Path) -> Result<(), Fail> {
    let spec = std::fs::read_to_string(root.join("SPEC.md"))?;
    let rows = parse_table(&spec)?;
    let model = Model::load(&root.join("model"))?;

    if rows.len() != model.ids.id.len() {
        return Err(format!(
            "RP-07: table has {} rows, register has {}",
            rows.len(),
            model.ids.id.len()
        )
        .into());
    }

    let mut seen = BTreeSet::new();
    for row in &rows {
        if !seen.insert(&row.id) {
            return Err(format!("RP-07: `{}` appears twice in table", row.id).into());
        }
        let Some(model_row) = model.ids.get(&row.id) else {
            return Err(format!("RP-07: `{}` in table but not in model/ids.toml", row.id).into());
        };
        if model_row.suite != row.suite {
            return Err(format!(
                "RP-07: `{}` suite mismatch: table `{}` vs register `{}`",
                row.id, row.suite, model_row.suite
            )
            .into());
        }
        if model_row.statement != row.statement {
            return Err(format!(
                "RP-07: `{}` statement mismatch:\n  table:    {}\n  register: {}",
                row.id, row.statement, model_row.statement
            )
            .into());
        }
    }

    println!(
        "validate-spec-links: {} table rows bijective with register (RP-07)",
        rows.len()
    );
    Ok(())
}
