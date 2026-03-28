use anyhow::Result;
use comfy_table::{ContentArrangement, Table};

/// Output format for CLI results.
#[derive(Debug, Clone, Copy, clap::ValueEnum, Default)]
pub enum OutputFormat {
    #[default]
    Json,
    Pretty,
    Table,
}

/// Print a serde_json::Value according to the chosen format.
pub fn print_value(value: &serde_json::Value, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(value)?);
        }
        OutputFormat::Pretty => {
            println!("{}", serde_json::to_string_pretty(value)?);
        }
        OutputFormat::Table => {
            print_as_table(value);
        }
    }
    Ok(())
}

/// Best-effort table rendering for JSON values.
/// Works well for arrays of objects (search results, download history, etc.).
fn print_as_table(value: &serde_json::Value) {
    match value {
        serde_json::Value::Array(arr) if !arr.is_empty() => {
            if let Some(serde_json::Value::Object(first)) = arr.first() {
                let headers: Vec<String> = first.keys().cloned().collect();
                let mut table = Table::new();
                table.set_content_arrangement(ContentArrangement::Dynamic);
                table.set_header(&headers);

                for item in arr {
                    if let serde_json::Value::Object(obj) = item {
                        let row: Vec<String> = headers
                            .iter()
                            .map(|h| match obj.get(h) {
                                Some(serde_json::Value::String(s)) => s.clone(),
                                Some(v) => v.to_string(),
                                None => String::new(),
                            })
                            .collect();
                        table.add_row(row);
                    }
                }
                println!("{table}");
            } else {
                // Array of non-objects — fall back to pretty JSON
                println!("{}", serde_json::to_string_pretty(value).unwrap_or_default());
            }
        }
        _ => {
            // Single value or empty array — fall back to pretty JSON
            println!("{}", serde_json::to_string_pretty(value).unwrap_or_default());
        }
    }
}
