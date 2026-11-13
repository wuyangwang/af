# AF Agent JSON 协议

> `schemaVersion`: `"1"`（所有 JSON 响应均包含）

## 成功响应（通用）
```json
{
  "ok": true,
  "schemaVersion": "1",
  "command": "read|write|append|search|patch",
  ...
}
```

## 错误响应（通用）
```json
{
  "ok": false,
  "schemaVersion": "1",
  "error": {
    "code": "STABLE_ERROR_CODE",
    "message": "...",
    "path": "..." 
  }
}
```
- `path` 可选
- 常见错误码：`INVALID_INPUT`、`FILE_NOT_FOUND`、`PERMISSION_DENIED`、`IO_ERROR`、`AMBIGUOUS_ENCODING`、`BINARY_FILE`、`INVALID_ENCODING`、`DECODE_ERROR`、`ENCODE_ERROR`、`HASH_CONFLICT`、`MATCH_COUNT_MISMATCH`、`TARGET_EXISTS`、`MIXED_NEWLINE`、`STRICT_FORMAT_VIOLATION`、`PATH_TRAVERSAL`、`CONTEXT_MISMATCH`、`GENERAL_ERROR`

## `read`
- 内容：`path, encoding, bom, eol, startLine, endLine, content, sha256`
- 元信息（`--meta`）：`path, encoding, bom, eol, size, sha256, mixedNewlines`

## `write`
- dry-run：`path, dryRun, bytes, encoding, bom, eol, sha256`
- 成功：`path, bytesWritten, encoding, bom, eol, sha256`

## `append`
- dry-run（新建）：`path, dryRun, created, bytes, encoding, bom, eol, sha256, contentToAppend`
- dry-run（追加）：`path, dryRun, bytesAppended, encoding, bom, eol, sha256, contentToAppend`
- 成功（新建）：`path, created, bytesWritten, sha256, encoding, bom, eol`
- 成功（追加）：`path, bytesAppended, encoding, bom, eol, sha256`

## `search`
- `pattern, path, matches[]`
- `matches[]`：`{path, line_number, column, line, before?:[{line_number,text}], after?:[{line_number,text}]}`

## `patch`
- dry-run：`path, dryRun, operations, beforeHash, afterHash, encoding, bom, eol`
- 成功：`path, operations, beforeHash, afterHash, encoding, bom, eol`

## `agent`
- `--json`：`name, description, schemaVersionProtocol, globalFlags, commands, protocol, recommendedWorkflow, convenienceMethods, examples`（并包含通用 `ok/schemaVersion/command` 字段）
