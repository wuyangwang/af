# Scoop 发布指南（AF）

## 前置条件
- 已创建 GitHub Release（tag 格式 `vX.Y.Z`），并包含 Windows 资产：`af-vX.Y.Z-x86_64-pc-windows-msvc.zip`
- 已有 Scoop bucket 仓库（本地 clone）并配置好 push 权限

## 方式一：使用脚本自动更新 bucket

```powershell
# 在本仓库根目录执行
pwsh ./scripts/scoop-publish.ps1 `
  -Version 0.1.0 `
  -Owner <github-owner> `
  -Repo <github-repo> `
  -BucketPath <path-to-local-scoop-bucket>
```

脚本会：
1. 下载对应 release 资产并计算 SHA256
2. 生成/更新 `<BucketPath>/bucket/af.json`
3. 在 bucket 仓库中 `git add/commit/push`

## 方式二：手动命令

```powershell
$Version = "0.1.0"
$Owner = "<github-owner>"
$Repo = "<github-repo>"
$BucketPath = "<path-to-local-scoop-bucket>"
$tag = "v$Version"
$asset = "af-$tag-x86_64-pc-windows-msvc.zip"
$url = "https://github.com/$Owner/$Repo/releases/download/$tag/$asset"

# 下载并计算哈希
Invoke-WebRequest -Uri $url -OutFile "$env:TEMP\$asset"
$hash = (Get-FileHash "$env:TEMP\$asset" -Algorithm SHA256).Hash.ToLower()

# 复制模板并替换（或直接编辑 scoop/af.json 后拷贝到 bucket）
Copy-Item .\scoop\af.json "$BucketPath\bucket\af.json"
# 编辑 $BucketPath\bucket\af.json：替换 version/url/hash/homepage/checkver

# 提交 bucket
cd $BucketPath
git add bucket/af.json
git commit -m "af: update to $Version"
git push
```

## 用户安装/更新（bucket 发布后）

```powershell
scoop bucket add <bucket-name> https://github.com/<owner>/<bucket-repo>
scoop install af
# 更新
scoop update af
```

> 提示：若使用 `extras` 等公共 bucket，请按该 bucket 的 PR 流程提交 manifest 更新。
