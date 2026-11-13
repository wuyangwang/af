# AF (Agent Files)

面向 AI Coding Agent 的安全文件操作与搜索 CLI。

- 新建文件默认：UTF-8 无 BOM + LF
- 修改已有文件默认：Preserve（保留原编码、BOM、换行风格）
- 编码不确定时拒绝冒险写入，支持 `--dry-run`、哈希校验与原子写入
- 内置搜索（ripgrep crates），人类可读与 Agent JSON 双输出模式

详细设计见 [`docs/design.md`](docs/design.md) 与根目录 [`af-system-design.md`](af-system-design.md)。

## 命令概览

```text
af read    # 读取文件、行范围、原始文本与元信息
af write   # 创建或覆盖文件
af append  # 在文件末尾追加内容
af search  # 在文件或目录中搜索文本
af patch   # 按精确匹配或 Unified Diff 修改文件
```

## 开发

```powershell
cargo build
cargo test
cargo run -p af-cli -- --help
```

## CI / Release

- 工作流：`.github/workflows/ci.yml`（CI & Release 合并）
  - `push` 到非 `master` 分支 / `pull_request`：在 Windows/macOS 上构建、测试并执行 CLI smoke test
  - `push` 到 `master`：自动读取 `Cargo.toml` 版本，创建并推送 `vX.Y.Z` tag（若不存在），由 tag 触发 Release 流程
  - `tag v*`：构建 Windows x64 与 macOS x64/arm64 发布产物，打包 `.zip`/`.tar.gz` + `.sha256`，自动创建 GitHub Release
  - `workflow_dispatch`：可选输入版本号（留空则使用 `Cargo.toml` 版本）自动创建并推送 tag，随后触发 Release 流程
- 本地打包脚本：`scripts/release.ps1`（Windows）、`scripts/release.sh`（macOS/Linux）

## Scoop（Windows）

- Manifest 模板：`scoop/af.json`
- 发布说明与命令：`scoop/PUBLISHING.md`
- 自动更新 bucket 脚本：`scripts/scoop-publish.ps1`

## 项目结构

- `crates/af-cli`：CLI 入口与命令路由
- `crates/af-core`：编码、换行、读写、原子写入、哈希与错误类型
- `crates/af-search`：搜索引擎与过滤
- `crates/af-render`：高亮、Diff 与 JSON 输出
- `docs/`：设计与协议文档
- `tests/`：集成测试与 fixtures
- `examples/`：示例输入（如 patch spec）
