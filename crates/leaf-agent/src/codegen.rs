//! TypeScript code generation for LEAF cards
//!
//! This module provides templates and utilities for generating TypeScript
//! programs that run in Deno.

use crate::error::{AgentError, AgentResult};

/// The template for generated TypeScript programs
const PROGRAM_TEMPLATE: &str = r#"// LEAF Generated Card Program
// This code runs in a Deno sandbox when the card is triggered.

// Environment from LEAF
const projectRoot = Deno.env.get("LEAF_PROJECT_ROOT")!;
const eventPayload = JSON.parse(Deno.env.get("LEAF_EVENT_PAYLOAD") || "{}");

// Event type definitions
interface FileEvent {
  path: string;
  size?: number;
  mime_type?: string;
}

interface ManualEvent {
  input?: string;
}

// Main function
async function main() {
  try {
    // === USER CODE START ===
{{USER_CODE}}
    // === USER CODE END ===
  } catch (error) {
    console.error("Card execution failed:", error);
    Deno.exit(1);
  }
}

// Run the main function
main();
"#;

/// Generate a complete card program from user-provided code
///
/// The user code is inserted into a template that provides:
/// - Environment variable access (projectRoot, eventPayload)
/// - Type definitions for events
/// - Error handling wrapper
pub fn generate_card_program(user_code: &str) -> AgentResult<String> {
    // Validate the user code isn't empty
    let user_code = user_code.trim();
    if user_code.is_empty() {
        return Err(AgentError::CodeGenerationError(
            "User code cannot be empty".to_string(),
        ));
    }

    // Indent the user code properly (4 spaces for template indentation)
    let indented_code = indent_code(user_code, 4);

    // Insert into template
    let program = PROGRAM_TEMPLATE.replace("{{USER_CODE}}", &indented_code);

    Ok(program)
}

/// Indent code by a specified number of spaces
fn indent_code(code: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    code.lines()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("{}{}", indent, line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Generate a minimal example card program
pub fn example_csv_parser() -> String {
    r#"const event = eventPayload as FileEvent;
const inputPath = event.path;

// Read the CSV file
console.log(`Processing CSV: ${inputPath}`);
const content = await Deno.readTextFile(inputPath);

// Parse CSV (simple implementation)
const lines = content.trim().split("\n");
const headers = lines[0].split(",").map(h => h.trim());
const rows = lines.slice(1).map(line => {
  const values = line.split(",").map(v => v.trim());
  const row: Record<string, string> = {};
  headers.forEach((h, i) => row[h] = values[i] || "");
  return row;
});

// Output results
const outputPath = `${projectRoot}/processed/${inputPath.split("/").pop()}.json`;
await Deno.mkdir(`${projectRoot}/processed`, { recursive: true });
await Deno.writeTextFile(outputPath, JSON.stringify(rows, null, 2));

console.log(`Processed ${rows.length} rows`);
console.log(`Output written to: ${outputPath}`);"#
        .to_string()
}

/// Generate a minimal example for file copy
pub fn example_file_copy() -> String {
    r#"const event = eventPayload as FileEvent;
const inputPath = event.path;
const fileName = inputPath.split("/").pop() || "file";

// Create backup directory
const backupDir = `${projectRoot}/backups`;
await Deno.mkdir(backupDir, { recursive: true });

// Copy the file with timestamp
const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
const backupPath = `${backupDir}/${timestamp}_${fileName}`;

await Deno.copyFile(inputPath, backupPath);
console.log(`Backed up: ${inputPath} -> ${backupPath}`);"#
        .to_string()
}

/// Generate a minimal example for file analysis
pub fn example_file_analysis() -> String {
    r#"const event = eventPayload as FileEvent;
const inputPath = event.path;
const fileName = inputPath.split("/").pop() || "file";

// Get file info
const fileInfo = await Deno.stat(inputPath);
const content = await Deno.readTextFile(inputPath);

// Analyze the file
const analysis = {
  name: fileName,
  size: fileInfo.size,
  lines: content.split("\n").length,
  words: content.split(/\s+/).filter(w => w.length > 0).length,
  characters: content.length,
  mimeType: event.mime_type || "unknown",
  analyzedAt: new Date().toISOString()
};

// Write analysis
const outputPath = `${projectRoot}/analysis/${fileName}.analysis.json`;
await Deno.mkdir(`${projectRoot}/analysis`, { recursive: true });
await Deno.writeTextFile(outputPath, JSON.stringify(analysis, null, 2));

console.log(`Analysis complete for: ${fileName}`);
console.log(JSON.stringify(analysis, null, 2));"#
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_card_program() {
        let user_code = r#"console.log("Hello");
const x = 1;"#;

        let program = generate_card_program(user_code).unwrap();

        assert!(program.contains("LEAF Generated Card Program"));
        assert!(program.contains("LEAF_PROJECT_ROOT"));
        assert!(program.contains("LEAF_EVENT_PAYLOAD"));
        assert!(program.contains("console.log(\"Hello\")"));
    }

    #[test]
    fn test_generate_card_program_empty_code() {
        let result = generate_card_program("");
        assert!(result.is_err());
    }

    #[test]
    fn test_indent_code() {
        let code = "line1\nline2\n\nline3";
        let indented = indent_code(code, 2);

        assert_eq!(indented, "  line1\n  line2\n\n  line3");
    }

    #[test]
    fn test_example_programs_compile() {
        // Just verify the examples are valid TypeScript-looking code
        let csv = example_csv_parser();
        assert!(csv.contains("FileEvent"));
        assert!(csv.contains("eventPayload"));

        let copy = example_file_copy();
        assert!(copy.contains("Deno.copyFile"));

        let analysis = example_file_analysis();
        assert!(analysis.contains("Deno.stat"));
    }
}
