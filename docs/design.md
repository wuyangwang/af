# AF（Agent Files）设计与实现说明（无配置功能）

> 版本：0.1.0  
> 目标平台：Windows 优先，架构预留跨平台  
> 实现语言：Rust  
> 文档性质：当前实现说明

## 1. 项目定位

AF 是面向 AI Coding Agent 的安全文件操作与搜索 CLI，提供可预测的文本读写、追加、搜索与补丁修改能力，减少对 PowerShell 文本 cmdlet 的依赖。

## 2. 已实现能力

### 2.1 命令
- `af read`：读取文件内容、行范围或元信息
- `af write`：创建或覆盖文件
- `af append`：追加内容（目标不存在时创建）
- `af search`：在文件或目录中搜索文本
- `af patch`：应用 JSON 规格补丁或 Unified Diff
- `af agent`：输出面向 AI Agent 的使用说明与能力信息（文本或 `--json` 结构化）

### 2.2 编码与换行
- 新建文件默认：UTF-8 无 BOM + LF
- 已有文件默认：Preserve（保留原编码/BOM/换行风格），除非显式 `--normalize`/格式参数
- BOM 识别：UTF-8、UTF-16 LE、UTF-16 BE
- 无 BOM 解码：优先严格 UTF-8；无法可靠识别时返回 `AMBIGUOUS_ENCODING`（需显式 `--encoding`）
- 显式编码支持：通过 `encoding_rs` 解码/编码；不可无损转换时拒绝
- 换行检测：LF / CRLF / CR / Mixed / None
- 混合换行：Preserve 写入、追加与（非 normalize）补丁默认拒绝，避免静默猜测

### 2.3 安全与原子性
- 二进制检测：无 BOM 且包含 NUL 字节的文件在未显式编码时按二进制拒绝
- 路径安全：拒绝包含 `..` 父目录穿越组件的路径
- 写入：同目录临时文件 + `flush/sync` + `persist` 原子替换
- 哈希：返回/校验 SHA-256；`write/append/patch` 支持 `--expected-hash`，`append/patch` 在提交前进行状态复核
- 覆盖策略：`af write` 对已存在目标要求显式 `--force`

### 2.4 搜索
- 基于 `ignore`（目录遍历与 ignore 规则）+ `grep-regex`/`grep-searcher`（匹配）实现
- 支持：字面量/正则、`--glob`、`--type`（扩展名）、`--before`/`--after`、`--hidden`、`--no-ignore`、`-i`
- 默认遵循 ignore 规则且不跟随符号链接
- 跳过含 NUL 的二进制内容与非 UTF-8 文本文件的匹配扫描

### 2.5 Patch
- 输入：JSON spec（`--spec`）或 Unified Diff（`--stdin` 或输入内容）
- JSON 操作：`replace`（`old/new/expectedMatches`）、`insert`、`delete`、`setLine`（行号编辑支持 `context`/`old` 校验）
- 替换匹配数校验；操作在副本上验证并应用后统一提交
- 支持 `--dry-run`、`--normalize`、`--expected-hash`
- Preserve 模式下混合换行文件拒绝补丁（除非 `--normalize`）

### 2.6 输出与协议
- 全局：`--output human|json`、`--json`
- JSON 成功响应包含 `ok: true`、`command`、`schemaVersion: "1"` 及命令字段
- JSON 错误响应包含 `ok: false`、`schemaVersion: "1"`、`error{code,message,path?}`（稳定命名错误码）
- Human 模式：`read` 支持语法高亮（终端输出）与行号；`patch --dry-run` 输出 unified diff
- 诊断日志：非 JSON 模式下可通过 `RUST_LOG` 启用 tracing 输出到 stderr

## 3. 命令行为摘要

### `af read <path>`
- 选项：`--start`、`--end`、`--plain`、`--number`、`--meta`、`--encoding`
- `--meta` 返回编码/BOM/EOL/大小/SHA-256/混合换行
- 内容读取返回指定行范围（UTF-8 无 BOM 场景采用按行流式范围读取）

### `af write <path>`
- 选项：`--stdin`、`--from`、`--encoding`、`--bom`、`--eol`、`--normalize`、`--strict`、`--dry-run`、`--force`、`--expected-hash`
- 目标存在时需 `--force`；`--dry-run` 返回/显示计划写入的元信息

### `af append <path>`
- 选项：`--stdin`、`--from`、`--ensure-newline`、`--dry-run`、`--expected-hash`
- 目标存在时按原编码/BOM/EOL 追加并原子写回；目标不存在时按新文件规则创建

### `af search <pattern> [path]`
- 选项：`--regex`、`--type`、`--glob`、`--before`、`--after`、`--hidden`、`--no-ignore`、`-i`
- JSON 返回匹配列表（路径、行号、列号、行文本、上下文）

### `af patch <path>`
- 选项：`--spec`、`--stdin`、`--dry-run`、`--normalize`、`--expected-hash`
- 支持 JSON 操作与 Unified Diff；提交前进行哈希复核

### `af agent`
- 无专用参数；支持全局 `--json`
- 输出面向 AI Agent 的使用说明、协议摘要、推荐工作流与便捷使用方法（单 exe 场景可直接获取）

## 4. 退出码

- `0` 成功
- `1` 通用错误
- `2` 参数/输入错误
- `3` 文件不存在或不可访问
- `4` 编码或二进制问题
- `5` 并发/哈希冲突
- `6` 匹配数量不符
- `7` 安全策略拒绝

## 5. 测试与验证

- `crates/af-core` 单元与 roundtrip 测试覆盖编码、换行、哈希与基础读写保留行为
- `crates/af-cli/tests/cli_basic.rs` 覆盖 read/write/append/patch/search、JSON 协议字段、覆盖保护与哈希冲突等核心流程
- 构建与测试命令：`cargo build`、`cargo test`
