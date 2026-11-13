# AF（Agent Files）系统设计文档

> 版本：0.1.0\
> 状态：初始设计基线\
> 目标平台：Windows 优先，架构预留跨平台\
> 实现语言：Rust\
> 项目定位：面向 AI Coding Agent 的安全文件操作与搜索 CLI

## 1. 项目概述

### 1.1 背景

Windows 上的 AI Coding Agent 经常通过 PowerShell 的
`Get-Content`、`Set-Content`、`Add-Content` 处理文件。这些命令在不同
PowerShell
版本、编码和参数组合下可能产生编码、BOM、换行符或文本格式变化。直接读取并重写整个文件，还可能造成不必要的大范围
Diff，甚至覆盖其他进程刚刚写入的内容。

AF 提供统一、可预测、可供 Agent 调用的文件操作接口，减少对 PowerShell
文本 cmdlet 的依赖。

### 1.2 设计目标

1.  覆盖日常 Agent 工作流中 `Get-Content`、`Set-Content`、`Add-Content`
    的核心用途。
2.  新建文件默认使用 UTF-8 无 BOM、LF。
3.  修改已有文件默认保留原编码、BOM 和换行格式。
4.  提供快速文本搜索，集成 ripgrep 的 Rust crates。
5.  人类模式支持语法高亮、行号和易读 Diff。
6.  Agent 模式提供稳定、无 ANSI 装饰的 JSON 输出。
7.  对编码不确定、二进制文件、并发修改和不安全写入采取保守策略。
8.  Windows 上尽量以单个可分发可执行文件运行，不要求用户另外安装
    `rg.exe` 或 PowerShell 模块。

### 1.3 非目标

-   不替代 VS Code、Visual Studio 等完整代码编辑器。
-   初期不实现完整 LSP、语义重构或 AST 级代码编辑。
-   不承诺对任意未知字符编码进行百分之百准确的自动识别。
-   不将所有现有文件强制转换成 UTF-8 + LF。
-   不执行 Shell 命令，也不承担通用文件管理器职责。

## 2. 核心设计原则

### 2.1 新建与修改采用不同默认策略

  场景           默认策略
  -------------- --------------------------------------
  新建文件       UTF-8 无 BOM + LF
  读取已有文件   检测并解码，尽可能保留原始字节信息
  修改已有文件   Preserve：保留原编码、BOM、换行风格
  主动规范化     用户显式传入 `--normalize`
  严格校验       `--strict`，不符合指定格式则拒绝修改

### 2.2 Preserve 优先

修改文件时只改变目标内容，不顺带改变文件格式。若原文件是 UTF-8 BOM +
CRLF，修改后仍应为 UTF-8 BOM + CRLF；若原文件是 UTF-16
LE，则在能够无损解码和编码时保留其格式。

对于混合换行文件，默认拒绝自动猜测或静默重写。可以在显式规范化模式下统一转换，并在预览中明确提示。

### 2.3 安全优先

-   写入前检查文件类型，默认拒绝二进制文件。
-   对有歧义的编码拒绝修改，除非用户显式指定编码。
-   修改操作支持预览和 Diff。
-   写入采用临时文件 + 替换的原子写入策略。
-   对已读取文件记录 SHA-256，写入前检查是否被外部修改。
-   修改失败时不应留下半写入文件。
-   不默认跟随符号链接；链接处理策略必须明确。
-   路径操作限制在用户指定的工作目录或显式路径范围内，避免路径穿越。

## 3. 默认五个命令

首版只把以下五个命令作为稳定、默认支持的公共命令。后续功能优先扩展这些命令的参数，不轻易增加顶层命令。

  ----------------------------------------------------------------------------
  命令                    主要职责                     PowerShell 对应用途
  ----------------------- ---------------------------- -----------------------
  `af read`               读取文件、行范围、原始文本   `Get-Content`

  `af write`              创建或覆盖文件               `Set-Content`

  `af append`             在文件末尾追加内容           `Add-Content`

  `af search`             在文件或目录中搜索文本       `Select-String` / `rg`

  `af patch`              按精确匹配或 Unified Diff    Agent
                          修改文件                     常见的代码编辑操作
  ----------------------------------------------------------------------------

`af info`
不作为首版独立顶层命令；文件编码、BOM、换行、大小和哈希信息通过
`af read --meta` 获取。

### 3.1 `af read`：读取文件

用途：覆盖 `Get-Content` 的常见读取场景。

``` powershell
# 读取完整文件
af read src/main.ts

# 读取指定行范围
af read src/main.ts --start 20 --end 80

# 原样读取文本，不添加行号或高亮
af read src/main.ts --plain

# 显示行号和语法高亮（人类模式默认）
af read src/main.ts --number

# 输出文件元信息
af read src/main.ts --meta

# 结构化输出，供 Agent 使用
af read src/main.ts --json
```

要求：

-   默认按文本读取；不得把二进制内容静默解码成乱码。
-   自动识别 UTF-8 BOM、UTF-16 LE/BE BOM。
-   无 BOM 文件优先验证
    UTF-8；其他编码只有在有可靠依据或用户显式指定时才使用。
-   `--encoding` 可显式指定编码，例如 `utf-8`、`gbk`、`gb18030`。
-   行号从 1 开始，`--start` 和 `--end` 均为包含边界。
-   人类模式可自动按扩展名选择语法高亮；未知类型退化为纯文本。
-   Agent 模式不得输出 ANSI 颜色码、分页器控制字符或装饰性信息。
-   `--meta`
    至少返回编码、BOM、换行类型、文件大小、SHA-256、检测是否存在混合换行。
-   对超大文件支持行范围读取，避免无必要地将整个文件载入内存。

### 3.2 `af write`：创建或覆盖文件

用途：覆盖 `Set-Content` 的常见场景。

``` powershell
# 从标准输入写入（PowerShell 示例）
"hello`nworld" | af write notes.txt

# 从文件写入
af write notes.txt --from temp.txt

# 显式指定编码和换行
af write notes.txt --encoding utf-8 --eol lf

# 覆盖前预览
af write notes.txt --from temp.txt --dry-run

# 创建新文件；默认 UTF-8 无 BOM + LF
af write new-file.ts --stdin
```

要求：

-   新文件默认 UTF-8 无 BOM + LF。
-   覆盖已有文件默认 `preserve`：保留目标文件的编码、BOM 和换行格式。
-   `--normalize` 将目标内容规范化为 UTF-8 无 BOM + LF。
-   `--encoding`、`--bom`、`--eol` 可显式覆盖格式策略。
-   `--stdin` 与 `--from` 互斥；若输入来自管道，可自动识别 stdin。
-   默认覆盖前检查目标文件是否存在；覆盖行为必须明确，可用 `--force`
    表示允许覆盖。
-   写入采用同目录临时文件、flush/sync
    和原子替换；平台不支持原子替换时必须说明降级行为。
-   写入成功后返回字节数、最终编码、BOM、换行类型和 SHA-256。
-   `--dry-run` 只计算和展示结果，不落盘。

### 3.3 `af append`：追加内容

用途：覆盖 `Add-Content` 的常见场景。

``` powershell
# 追加标准输入
"new line" | af append notes.txt

# 追加文件内容
af append notes.txt --from fragment.txt

# 追加时在末尾确保有换行
af append notes.txt --ensure-newline
```

要求：

-   已存在文件默认保留原编码、BOM 和换行风格。
-   新文件追加时采用 UTF-8 无 BOM + LF。
-   追加文本先按目标文件编码转换；无法无损转换时拒绝追加。
-   默认不自动在追加内容前后增加换行，`--ensure-newline`
    才执行明确的换行策略。
-   追加必须考虑并发写入；至少使用文件锁或写入前后状态校验，并在文档中明确平台保证。
-   提供 `--dry-run`，展示将追加的内容和最终换行行为。

### 3.4 `af search`：搜索文件内容

用途：集成 ripgrep 搜索能力，覆盖常见文本搜索和 `Select-String` 场景。

建议使用 ripgrep 的 Rust crates 组合，而不是依赖用户另外安装 `rg.exe`：

-   `grep`：搜索核心。
-   `ignore`：目录遍历、`.gitignore` 和文件过滤。
-   `regex`：正则表达式。
-   `grep-printer`：可借鉴其结果格式化逻辑；Agent JSON 输出由 AF
    自己定义。

``` powershell
# 搜索文本
af search "useState" ./src

# 正则表达式
af search "use[A-Z]\w+" ./src --regex

# 指定文件类型
af search "TODO" ./src --type ts

# 显示上下文
af search "createUser" ./src --before 3 --after 5

# 包含隐藏文件
af search "config" . --hidden

# 搜索被 ignore 的文件
af search "config" . --no-ignore

# JSON 输出
af search "useState" ./src --json
```

要求：

-   默认遵循 `.gitignore`、隐藏文件和常见二进制文件过滤规则。
-   支持固定字符串和正则表达式两种模式。
-   支持 glob、文件类型、上下文行、大小写敏感选项。
-   返回路径、行号、列号、匹配文本和上下文。
-   JSON 输出使用稳定 schema，不把终端格式化文本当作机器接口。
-   搜索结果可直接用于后续 `af read` 和 `af patch`。
-   不应把搜索匹配结果直接视为安全的修改定位；修改前仍须做唯一匹配或哈希校验。

### 3.5 `af patch`：精确修改文件

用途：给 Agent 提供可验证、可预览的文件修改能力。

``` powershell
# 根据 JSON 规格执行精确替换
af patch src/main.ts --spec change.json

# 从标准输入读取 unified diff
Get-Content patch.diff -Raw | af patch src/main.ts --stdin

# 只预览，不写入
af patch src/main.ts --spec change.json --dry-run

# 显式规范化文件格式
af patch src/main.ts --spec change.json --normalize
```

首版 `change.json` 建议格式：

``` json
{
  "version": 1,
  "operations": [
    {
      "type": "replace",
      "old": "const oldName = 1;",
      "new": "const newName = 1;",
      "expectedMatches": 1
    }
  ]
}
```

要求：

-   支持精确字符串替换和 Unified Diff。
-   默认要求每个替换项匹配次数符合
    `expectedMatches`；不符合时整个操作失败。
-   多操作默认先全部验证，再统一提交，避免部分成功。
-   支持插入、删除、替换；行号编辑必须附带上下文校验，不能只凭旧行号盲目修改。
-   支持 `--dry-run` 和人类可读 Diff。
-   支持 `--expected-hash`；如果目标文件哈希与读取时不同，拒绝写入。
-   修改已有文件默认 Preserve；`--normalize` 才转换成 UTF-8 无 BOM +
    LF。
-   成功结果返回变更摘要、前后哈希、最终格式和操作数量。
-   初版不做模糊匹配；后续如增加，必须明确标注置信度并要求预览或确认。

## 4. 编码与换行策略

### 4.1 三种编辑模式

  模式          行为                                   适用场景
  ------------- -------------------------------------- --------------------
  `preserve`    保留已有文件编码、BOM、换行风格        默认修改现有文件
  `normalize`   转为 UTF-8 无 BOM + LF                 主动规范化项目文件
  `strict`      不符合目标格式就拒绝修改，不自动转换   强制格式规范的仓库

### 4.2 编码识别原则

1.  先识别 BOM：UTF-8、UTF-16 LE、UTF-16 BE。
2.  无 BOM 时先严格验证 UTF-8。
3.  若 UTF-8 验证失败，不得仅凭猜测自动判定为 GBK。
4.  可提供显式 `--encoding gbk` 或项目级编码配置。
5.  编码存在歧义时，读取可返回原始字节或错误状态；修改默认拒绝。
6.  转码必须检查是否有字符无法表示，禁止静默替换成问号或替代字符。

### 4.3 换行检测原则

-   识别 LF、CRLF、CR。
-   文件只有一种换行符时，Preserve 模式保持该风格。
-   混合换行时标记 `mixed`，默认修改拒绝或要求用户显式选择策略。
-   新文件默认 LF。
-   Normalize 模式将所有换行统一为 LF。
-   不得把文件末尾是否存在 newline 当作无关信息；应在读写与 patch
    中保留或明确修改。

## 5. 输出模式与 Agent 协议

### 5.1 两种输出模式

  模式      面向对象   默认行为
  --------- ---------- -------------------------------------
  `human`   开发者     语法高亮、行号、彩色 Diff、友好错误
  `agent`   AI Agent   JSON、无 ANSI、无分页、稳定字段

可通过全局 `--output human|json` 设置；`--json` 作为便捷别名。

### 5.2 JSON 输出约定

成功示例：

``` json
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

失败示例：

``` json
{
  "ok": false,
  "error": {
    "code": "AMBIGUOUS_ENCODING",
    "message": "无法可靠判断文件编码，请显式指定 --encoding。",
    "path": "legacy.txt"
  }
}
```

约定：

-   stdout 只输出结果 JSON。
-   stderr 输出诊断日志；Agent 模式默认不输出非必要日志。
-   JSON schema 版本化，顶层包含 `ok`、`command`。
-   错误码稳定，不依赖本地化错误消息进行判断。
-   文本内容必须正确转义，不能混入 ANSI 序列。

### 5.3 退出码

    退出码 含义
  -------- ----------------------
         0 成功
         1 通用执行失败
         2 参数或输入格式错误
         3 文件不存在或不可访问
         4 编码或二进制文件问题
         5 并发修改冲突
         6 匹配数量不符合预期
         7 安全策略拒绝操作

## 6. 技术架构

### 6.1 技术选型

  模块       技术                     用途
  ---------- ------------------------ --------------------------
  语言       Rust                     主实现
  CLI        `clap`                   参数解析、帮助信息
  错误处理   `anyhow` + `thiserror`   错误上下文与稳定错误类型
  编码       `encoding_rs`            常见编码解码与编码
  高亮       `syntect`                语法高亮、主题
  Diff       `similar`                文本 Diff
  临时文件   `tempfile`               原子写入准备
  哈希       `sha2`                   SHA-256 并发校验
  搜索       ripgrep crates           搜索核心
  序列化     `serde` + `serde_json`   Agent JSON 协议
  日志       `tracing`                可控诊断日志

依赖版本在实现时检查 crates.io 和官方文档，选择兼容版本并在 `Cargo.lock`
固定。

### 6.2 分层结构

``` text
┌────────────────────────────────────────────┐
│                 AF CLI                     │
│       clap / 命令路由 / 配置加载             │
├────────────────────────────────────────────┤
│             Application Layer              │
│    read / write / append / search / patch   │
├──────────────────┬─────────────────────────┤
│   File Engine    │      Search Engine       │
│ 编码 / 换行 / I/O │ grep / ignore / regex   │
│ 哈希 / 原子写入   │ glob / context          │
├──────────────────┴─────────────────────────┤
│              Presentation Layer            │
│ Human: syntect / similar / terminal        │
│ Agent: serde_json / stable error schema    │
└────────────────────────────────────────────┘
```

### 6.3 建议目录结构

``` text
af/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── docs/
│   ├── design.md
│   ├── cli-spec.md
│   └── agent-json-schema.md
├── crates/
│   ├── af-cli/
│   │   └── src/
│   │       ├── main.rs
│   │       ├── cli.rs
│   │       └── commands/
│   ├── af-core/
│   │   └── src/
│   │       ├── encoding.rs
│   │       ├── newline.rs
│   │       ├── reader.rs
│   │       ├── writer.rs
│   │       ├── atomic.rs
│   │       ├── hash.rs
│   │       └── errors.rs
│   ├── af-search/
│   │   └── src/
│   │       ├── search.rs
│   │       ├── filters.rs
│   │       └── output.rs
│   └── af-render/
│       └── src/
│           ├── highlight.rs
│           ├── diff.rs
│           └── json.rs
├── tests/
│   ├── encoding/
│   ├── newline/
│   ├── commands/
│   └── fixtures/
└── examples/
    └── change.json
```

## 7. 配置文件

项目级配置文件：`af.config.json`。

``` json
{
  "version": 1,
  "defaults": {
    "newFileEncoding": "utf-8",
    "newFileBom": false,
    "newFileEol": "lf",
    "editMode": "preserve",
    "syntaxHighlight": true,
    "theme": "base16-ocean.dark"
  },
  "agent": {
    "output": "json",
    "disableAnsi": true,
    "disablePager": true
  },
  "safety": {
    "checkFileHash": true,
    "rejectAmbiguousEncoding": true,
    "rejectBinaryFiles": true,
    "atomicWrite": true,
    "backup": false
  },
  "search": {
    "respectGitIgnore": true,
    "followSymlinks": false
  }
}
```

配置优先级：

1.  命令行参数。
2.  当前项目 `af.config.json`。
3.  用户级配置。
4.  内置默认值。

未知配置字段应给出可理解的诊断；配置 schema 需要版本化。

## 8. Windows 兼容性要求

-   支持 Windows 10/11 x64，后续考虑 ARM64。
-   路径正确支持空格、中文和 Unicode。
-   不依赖 PowerShell 特定 API；PowerShell 仅作为调用环境。
-   标准输入和标准输出按字节处理，避免终端代码页改变文件内容。
-   正确处理 Windows 文件锁、文件替换和权限错误。
-   默认不跟随目录联接点或符号链接进行递归搜索，除非用户显式开启。
-   提供 PowerShell、CMD 示例。
-   终端颜色应检测能力；重定向输出时默认关闭颜色。
-   不应把控制台显示编码与文件编码混为一谈。

## 9. 安全、可靠性与边界条件

### 9.1 必须覆盖的边界测试

-   UTF-8 无 BOM、UTF-8 BOM。
-   UTF-16 LE/BE。
-   GBK/GB18030 显式指定。
-   LF、CRLF、CR、混合换行。
-   空文件、无末尾换行文件。
-   超长行、大文件。
-   中文、emoji、组合字符。
-   二进制文件和含 NUL 字节的文件。
-   只读文件、权限不足、磁盘空间不足。
-   文件被其他进程并发修改。
-   符号链接、目录联接点、路径穿越。
-   文件名含空格、中文、特殊字符。
-   stdin 被重定向、管道为空、输入中包含 NUL。

### 9.2 原子写入说明

推荐流程：

1.  读取原文件并记录元信息和哈希。
2.  在内存或临时文件中生成完整新内容。
3.  验证编码可表示、换行策略和目标内容。
4.  再次检查目标文件状态及哈希。
5.  在目标文件同一目录创建临时文件。
6.  写入并 flush/sync。
7.  使用平台适配的替换操作提交。
8.  返回最终文件元信息和新哈希。

注意：文件哈希检查与最终替换之间仍可能存在竞争窗口。实现时应尽量使用
Windows
文件句柄、锁和平台原语缩小窗口，并在文档中明确实际保证，不宣称跨平台绝对无竞态。

## 10. 开发阶段

### Phase 1：基础文件操作

-   Rust workspace 与 CLI 框架。
-   `af read`、`af write`、`af append`。
-   UTF-8、UTF-8 BOM、UTF-16 BOM 识别。
-   Preserve / Normalize / Strict 策略。
-   LF、CRLF、CR 检测。
-   原子写入、哈希校验。
-   基础测试矩阵。

### Phase 2：搜索能力

-   集成 ripgrep Rust crates。
-   固定字符串、正则、glob、gitignore。
-   上下文行、文件类型过滤。
-   JSON 搜索结果。
-   大仓库性能测试。

### Phase 3：安全修改

-   `af patch`。
-   精确替换和 Unified Diff。
-   唯一匹配检查。
-   多操作预验证与统一提交。
-   Dry-run、Diff 和并发修改检测。

### Phase 4：Agent 体验

-   Human / Agent 双输出模式。
-   稳定 JSON schema 和错误码。
-   syntect 语法高亮。
-   主题和终端能力检测。
-   PowerShell / CMD 使用体验测试。

### Phase 5：发布

-   Windows x64 单文件发布。
-   安装与 PATH 配置说明。
-   CI 测试和可复现构建。
-   性能基准与安全审查。

## 11. 验收标准

1.  新建文本文件默认是 UTF-8 无 BOM + LF。
2.  修改 UTF-8 BOM、UTF-16 或已知旧编码文件时，能够按 Preserve
    策略无损处理；不能可靠处理时拒绝写入。
3.  修改 CRLF 文件不会无意间把整份文件转换为 LF。
4.  `read` 支持完整读取和行范围读取，且人类模式具有语法高亮。
5.  `write` 覆盖、`append` 追加的行为明确且可测试。
6.  `search` 能正确遵循 `.gitignore` 并提供 JSON 结果。
7.  `patch` 在匹配数不符或哈希冲突时拒绝写入。
8.  原子写入失败时原文件保持完整。
9.  Agent JSON 输出不包含 ANSI 颜色码或非协议文本。
10. Windows 路径、中文文件名、PowerShell 管道均通过自动化测试。

## 12. 首版范围总结

首版只提供五个默认命令：

``` text
af read
af write
af append
af search
af patch
```

核心产品原则：

-   新建文件统一 UTF-8 无 BOM + LF。
-   修改已有文件默认 Preserve。
-   规范化必须显式请求。
-   编码不确定时不冒险写入。
-   搜索使用 ripgrep Rust crates。
-   人类模式可读，Agent 模式稳定可解析。
-   所有修改可验证、可预览、可检测并发冲突。

AF 的价值不是重新实现一个文本编辑器，而是给 AI Agent 提供一套跨 Windows
工具链、行为可预测、不会轻易破坏文件格式的文件操作接口。
