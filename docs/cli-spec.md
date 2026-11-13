# AF CLI 规范

> 适用版本：当前实现（v0.1.0，配置功能不在范围内）

## 全局参数
- `--output <human|json>`：输出模式，默认 `human`
- `--json`：等同于 `--output json`

## 退出码
- `0` 成功
- `1` 通用错误（`GENERAL_ERROR` 等）
- `2` 参数/输入错误（`INVALID_INPUT`）
- `3` 文件不存在或不可访问（`FILE_NOT_FOUND`/`PERMISSION_DENIED`/`IO_ERROR`）
- `4` 编码或二进制问题（`AMBIGUOUS_ENCODING`/`BINARY_FILE`/`INVALID_ENCODING`/`DECODE_ERROR`/`ENCODE_ERROR`）
- `5` 并发/哈希冲突（`HASH_CONFLICT`）
- `6` 匹配数量不符（`MATCH_COUNT_MISMATCH`）
- `7` 安全策略拒绝（`TARGET_EXISTS`/`MIXED_NEWLINE`/`STRICT_FORMAT_VIOLATION`/`PATH_TRAVERSAL`/`CONTEXT_MISMATCH` 等）

## `af read <path>`
读取文件内容、行范围或元信息。
- `--start <N>` / `--end <N>`：1-based 包含边界
- `--plain`：纯文本输出（不加行号/高亮）
- `--number`：显示行号
- `--meta`：输出元信息
- `--encoding <label>`：显式解码编码
- Human 默认：终端输出时启用语法高亮与行号（`--plain` 可关闭）
- JSON：`schemaVersion`、`ok`、`command`、`path`、`encoding`、`bom`、`eol`、`sha256`，内容模式含 `startLine/endLine/content`，元信息模式含 `size/mixedNewlines`

## `af write <path>`
创建或覆盖文件。
- `--stdin` / `--from <file>`：输入来源（互斥；未指定 `--from` 时可从 stdin 读取）
- `--encoding <label>`、`--bom <true|false>`、`--eol <lf|crlf|cr>`
- `--normalize`：规范化为 UTF-8 无 BOM + LF
- `--strict`：严格目标格式校验
- `--dry-run`：计算并返回/显示计划结果，不落盘
- `--force`：允许覆盖已存在文件
- `--expected-hash <sha256>`：覆盖前校验
- 新文件默认 UTF-8 无 BOM + LF；已存在文件默认 Preserve

## `af append <path>`
追加内容；目标不存在时创建。
- `--stdin` / `--from <file>`、`--ensure-newline`、`--dry-run`、`--expected-hash`
- 已存在文件按原格式追加并原子写回；混合换行文件拒绝追加

## `af search <pattern> [path]`
搜索文本（默认 `path=.`）。
- `--regex`、`--type <ext>`、`--glob <glob>`、`--before <N>`、`--after <N>`、`--hidden`、`--no-ignore`、`-i`
- 遵循 ignore 规则（除非 `--no-ignore`），不跟随符号链接
- JSON 返回 `matches[]`（`path,line_number,column,line,before?,after?`）

## `af patch <path>`
应用 JSON 补丁或 Unified Diff。
- `--spec <file>` 或 `--stdin`
- `--dry-run`、`--normalize`、`--expected-hash`
- JSON spec：`{version, operations[]}`，操作类型：
  - `replace`: `{type:"replace", old, new, expectedMatches?}`
  - `insert`: `{type:"insert", line, content, context?}`
  - `delete`: `{type:"delete", line, content?, context?}`
  - `setLine`: `{type:"setLine", line, new, old?, context?}`
- 行操作可提供 `context:{before?,after?}` 进行上下文校验
- Preserve 模式下混合换行文件拒绝（除非 `--normalize`）

## `af agent`
输出面向 AI Agent 的使用说明与能力信息（适用于单 exe 分发）。
- 全局 `--json`：输出结构化能力/协议/工作流/便捷方法
- 默认：输出可读文本指南
- 内置 `--help` 已增强各命令示例与 Agent 使用提示
