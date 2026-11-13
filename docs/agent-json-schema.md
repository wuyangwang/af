# AF Agent JSON Schema（草案）

目标：为 Agent 模式提供稳定、可解析、无 ANSI 的 JSON 协议。

## 通用约定

- stdout 仅输出结果 JSON；stderr 用于诊断（Agent 模式默认静默非必要日志）
- 顶层字段：`ok`（bool）、`command`（string），失败时包含 `error`
- 错误码稳定，不依赖本地化消息
- Schema 版本化（建议顶层或 `meta.schemaVersion`，待定）

## 成功示例（read）

```json
{
  "ok": true,
  "command": "read",
  "path": "src/main.ts",
  "encoding": "utf-8",
  "bom": false,
  "eol": "lf",
  "startLine": 1,
  "endLine": 12,
  "content": "export const answer = 42;\n",
  "sha256": "..."
}
```

## 失败示例

```json
{
  "ok": false,
  "error": {
    "code": "AMBIGUOUS_ENCODING",
    "message": "无法可靠判断文件编码，请显式指定 --encoding。",
    "path": "legacy.txt"
  }
}
```

> TODO: 为 read/write/append/search/patch 定义严格字段与类型。
