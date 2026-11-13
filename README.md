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

## 项目结构

- `crates/af-cli`：CLI 入口与命令路由
- `crates/af-core`：编码、换行、读写、原子写入、哈希与错误类型
- `crates/af-search`：搜索引擎与过滤
- `crates/af-render`：高亮、Diff 与 JSON 输出
- `docs/`：设计与协议文档
- `tests/`：集成测试与 fixtures
- `examples/`：示例输入（如 patch spec）
