# AF (Agent Files)

面向 AI Coding Agent 的文件操作与搜索 CLI（Windows 优先，Rust 实现）。

## 功能

- 命令：`read` / `write` / `append` / `search` / `patch` / `agent`
- 新建文件默认 UTF-8 无 BOM + LF；已有文件默认 Preserve（可 `--normalize`/`--strict`）
- 编码识别与安全写入：BOM/UTF-8/显式编码、二进制检测、路径穿越防护、原子写入、SHA-256 校验
- 搜索：`ignore` + `grep-regex`/`grep-searcher`，支持正则/字面量、glob、类型、上下文、ignore 控制
- Patch：JSON 操作（replace/insert/delete/setLine，含上下文校验）与 Unified Diff
- 输出：Human（高亮/行号/Diff）与 Agent JSON（`schemaVersion: "1"`、稳定命名错误码）

## 使用

```powershell
af --help
af read <path> [--start N --end N] [--meta] [--plain] [--json]
af write <path> --from <file> --force [--normalize|--strict] [--dry-run] [--json]
af append <path> --from <file> [--ensure-newline] [--dry-run] [--json]
af search <pattern> [path] [--regex] [--glob "*.rs"] [--before 2 --after 2] [--json]
af patch <path> --spec change.json [--dry-run] [--normalize] [--json]
af agent [--json]
```

单 exe 分发时，可通过 `af agent` / `af agent --json` 获取面向 AI Agent 的使用说明、协议与推荐工作流。

文档：[`docs/design.md`](docs/design.md)、[`docs/cli-spec.md`](docs/cli-spec.md)、[`docs/agent-json-schema.md`](docs/agent-json-schema.md)

## 开发

```powershell
cargo build
cargo test
```

## 结构

- `crates/af-cli`：CLI 与命令实现
- `crates/af-core`：编码、换行、读写、原子写入、哈希、安全与错误
- `crates/af-search`：搜索与过滤
- `crates/af-render`：高亮、Diff、JSON 协议辅助
- `tests/`、`crates/*/tests/`：自动化测试
- `examples/change.json`：补丁示例

