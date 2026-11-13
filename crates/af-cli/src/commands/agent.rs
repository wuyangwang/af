use af_render::json as rjson;
use anyhow::Result;
use clap::Args;
use serde_json::json;

#[derive(Debug, Args)]
pub struct AgentArgs {}

const TEXT_GUIDE: &str = r#"AF Agent Guide (single-exe friendly)

OVERVIEW
- af is a safe file CLI for AI agents: read, write, append, search, patch.
- Prefer machine mode for automation: `--json` (or `--output json`).
- JSON responses include `schemaVersion: "1"` and stable `error.code` values.

CORE COMMANDS
- af read <path> [--start N --end N] [--meta] [--encoding ENC] [--plain|--number] [--json]
  Inspect content or metadata (encoding, bom, eol, size, sha256, mixedNewlines).
- af write <path> (--stdin|--from FILE) [--force] [--normalize|--strict] [--encoding ENC --bom BOOL --eol lf|crlf|cr] [--dry-run] [--expected-hash SHA] [--json]
  Create/overwrite files. Existing targets require --force. New files default to UTF-8 no BOM + LF.
- af append <path> (--stdin|--from FILE) [--ensure-newline] [--dry-run] [--expected-hash SHA] [--json]
  Append while preserving existing format; creates file if missing.
- af search <pattern> [path] [--regex] [--type EXT] [--glob GLOB] [--before N --after N] [--hidden] [--no-ignore] [-i] [--json]
  Locate code/text with ignore-aware search and context.
- af patch <path> (--spec FILE | --stdin) [--dry-run] [--normalize] [--expected-hash SHA] [--json]
  Apply JSON operations (replace/insert/delete/setLine with context checks) or unified diff.

RECOMMENDED AGENT WORKFLOW
1. Discover: `af search "<term>" <root> --json` to find candidate files/lines.
2. Inspect: `af read <file> --meta --json` and/or `af read <file> --start A --end B --json`.
3. Plan safely: prepare minimal change; prefer `af patch --dry-run --json` or `af write/append --dry-run --json`.
4. Prevent conflicts: pass `--expected-hash <sha256>` from a prior read/metadata step.
5. Apply: run patch/write/append; verify result hash/fields from JSON output.
6. Handle errors by `error.code` and exit code (do not branch on localized message text).

SAFETY & CORRECTNESS PRACTICES
- Use `--dry-run` before modifications.
- Use `--expected-hash` for concurrent-edit protection.
- Respect encoding/newline behavior: existing files are preserved by default; use `--normalize` only when intended.
- Ambiguous encoding, binary content, mixed newlines (in preserve flows), path traversal, and match/context mismatches are rejected with stable error codes.
- Overwriting existing files with `af write` requires `--force`.

JSON PROTOCOL (summary)
- Success: `{ "ok": true, "schemaVersion": "1", "command": "...", ... }`
- Error:   `{ "ok": false, "schemaVersion": "1", "error": { "code": "...", "message": "...", "path"?: "..." } }`
- Common error.code: INVALID_INPUT, FILE_NOT_FOUND, PERMISSION_DENIED, IO_ERROR, AMBIGUOUS_ENCODING, BINARY_FILE, INVALID_ENCODING, DECODE_ERROR, ENCODE_ERROR, HASH_CONFLICT, MATCH_COUNT_MISMATCH, TARGET_EXISTS, MIXED_NEWLINE, STRICT_FORMAT_VIOLATION, PATH_TRAVERSAL, CONTEXT_MISMATCH, GENERAL_ERROR
- Exit codes: 0 success, 1 general, 2 invalid input, 3 io/not found, 4 encoding/binary, 5 conflict, 6 match count, 7 safety rejection

OTHER CONVENIENT METHODS FOR AI AGENTS (single exe)
- Start with `af --help` and `af <command> --help` for embedded usage examples.
- Use `af agent --json` to programmatically load capabilities and protocol details.
- In PowerShell, pass file content via `--from` or pipe stdin; prefer explicit args and `--json` for reliable parsing.
- Compose operations: search -> read ranges -> patch (small, verifiable edits) instead of whole-file rewrites when possible.
- Cache/read `sha256` from results to chain safe follow-up edits.
- Treat non-zero exit codes as actionable states; retry logic should be code-aware (e.g. HASH_CONFLICT => re-read and rebase patch).

EXAMPLES
- af search "createUser" src --before 2 --after 2 --json
- af read src/api.ts --meta --json
- af patch src/api.ts --spec change.json --expected-hash <sha256> --dry-run --json
- af write README.generated.md --from tmp.md --force --normalize --json
"#;

pub fn run(_args: AgentArgs, json_mode: bool) -> Result<()> {
    if json_mode {
        let out = json!({
            "ok": true,
            "command": "agent",
            "name": "af",
            "description": "Agent Files: safe, encoding-aware file operations and search for AI agents",
            "schemaVersionProtocol": "1",
            "globalFlags": [
                {"flag":"--output <human|json>", "description":"Select output mode"},
                {"flag":"--json", "description":"Shorthand for JSON output"}
            ],
            "commands": [
                {"name":"read", "purpose":"Read content, line ranges, or metadata", "keyFlags":["--start","--end","--meta","--encoding","--plain","--number","--json"]},
                {"name":"write", "purpose":"Create or overwrite file", "keyFlags":["--stdin","--from","--force","--normalize","--strict","--encoding","--bom","--eol","--dry-run","--expected-hash","--json"]},
                {"name":"append", "purpose":"Append to file or create if missing", "keyFlags":["--stdin","--from","--ensure-newline","--dry-run","--expected-hash","--json"]},
                {"name":"search", "purpose":"Search files/directories", "keyFlags":["--regex","--type","--glob","--before","--after","--hidden","--no-ignore","-i","--json"]},
                {"name":"patch", "purpose":"Apply JSON operations or unified diff", "keyFlags":["--spec","--stdin","--dry-run","--normalize","--expected-hash","--json"]},
                {"name":"agent", "purpose":"Show agent usage guidance and capabilities", "keyFlags":["--json"]}
            ],
            "protocol": {
                "schemaVersion": "1",
                "successShape": {"ok": true, "schemaVersion": "1", "command": "string"},
                "errorShape": {"ok": false, "schemaVersion": "1", "error": {"code": "string", "message": "string", "path": "string?"}},
                "exitCodes": {"0":"success","1":"general","2":"invalid input","3":"io/not found","4":"encoding/binary","5":"conflict","6":"match count","7":"safety rejection"},
                "errorCodes": ["INVALID_INPUT","FILE_NOT_FOUND","PERMISSION_DENIED","IO_ERROR","AMBIGUOUS_ENCODING","BINARY_FILE","INVALID_ENCODING","DECODE_ERROR","ENCODE_ERROR","HASH_CONFLICT","MATCH_COUNT_MISMATCH","TARGET_EXISTS","MIXED_NEWLINE","STRICT_FORMAT_VIOLATION","PATH_TRAVERSAL","CONTEXT_MISMATCH","GENERAL_ERROR"]
            },
            "recommendedWorkflow": [
                "search to locate targets",
                "read --meta and/or read ranges to inspect",
                "dry-run write/append/patch with expected hash",
                "apply change and verify returned hash",
                "branch on error.code and exit code"
            ],
            "convenienceMethods": [
                "Use embedded help: af --help and af <command> --help",
                "Use af agent --json for programmatic capability discovery in single-exe deployments",
                "Prefer --json and parse schemaVersion/error.code instead of messages",
                "Use --dry-run and --expected-hash for safe automated edits",
                "Use search -> read -> patch for minimal verifiable changes",
                "In PowerShell prefer --from/explicit args and capture exit codes"
            ],
            "examples": [
                "af agent --json",
                "af search \"TODO\" src --type rs --json",
                "af read src/lib.rs --meta --json",
                "af patch src/lib.rs --spec change.json --dry-run --expected-hash <sha256> --json"
            ]
        });
        println!("{}", rjson::success(out));
    } else {
        print!("{}", TEXT_GUIDE);
    }
    Ok(())
}
