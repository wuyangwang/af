# AF CLI 规范（草案）

本文件用于细化 `af read/write/append/search/patch` 的参数、行为与示例。
权威设计基线见 [`design.md`](design.md)。

## 全局参数

- `--output human|json`：输出模式（默认 `human`）
- `--json`：`--output json` 的便捷别名

## 命令

### `af read`
- 输入：文件路径、可选行范围、元信息开关
- 输出：文本内容或元信息（编码/BOM/EOL/大小/SHA-256/混合换行）

### `af write`
- 新文件默认 UTF-8 无 BOM + LF；覆盖默认 Preserve
- `--stdin` / `--from` 输入，`--dry-run` 预览，`--normalize` 显式规范化

### `af append`
- 保留目标文件格式；支持 `--ensure-newline` 与 `--dry-run`

### `af search`
- 基于 ripgrep crates；支持正则、glob/类型、上下文、gitignore 控制
- 结果可用于后续 `af read` / `af patch`（修改前仍需唯一匹配或哈希校验）

### `af patch`
- 支持精确替换 JSON spec 与 Unified Diff
- 默认全量预验证后统一提交；支持 `--dry-run`、`--expected-hash`

> TODO: 补充完整参数表、示例与错误码映射。
