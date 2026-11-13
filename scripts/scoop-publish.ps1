param(
  [Parameter(Mandatory = $true)][string]$Version,   # e.g. 0.1.0
  [Parameter(Mandatory = $true)][string]$Owner,     # GitHub owner/org
  [Parameter(Mandatory = $true)][string]$Repo,      # GitHub repo name
  [Parameter(Mandatory = $true)][string]$BucketPath # Local path to scoop bucket repo
)

$ErrorActionPreference = "Stop"
$tag = "v$Version"
$asset = "af-$tag-x86_64-pc-windows-msvc.zip"
$url = "https://github.com/$Owner/$Repo/releases/download/$tag/$asset"
$tmp = Join-Path $env:TEMP $asset

Write-Host "Downloading $url"
Invoke-WebRequest -Uri $url -OutFile $tmp
$hash = (Get-FileHash $tmp -Algorithm SHA256).Hash.ToLower()
Write-Host "SHA256: $hash"

$manifestPath = Join-Path $BucketPath "bucket/af.json"
New-Item -ItemType Directory -Force -Path (Split-Path $manifestPath) | Out-Null

$manifest = @{
  version = $Version
  description = "AF (Agent Files) - safe file ops and search CLI for AI coding agents"
  homepage = "https://github.com/$Owner/$Repo"
  license = "MIT"
  architecture = @{
    "64bit" = @{
      url = $url
      hash = $hash
    }
  }
  bin = "af.exe"
  checkver = @{ github = "https://github.com/$Owner/$Repo" }
  autoupdate = @{
    architecture = @{
      "64bit" = @{
        url = "https://github.com/$Owner/$Repo/releases/download/v`$version/af-v`$version-x86_64-pc-windows-msvc.zip"
      }
    }
  }
} | ConvertTo-Json -Depth 10

$manifest | Set-Content -Path $manifestPath -Encoding UTF8
Write-Host "Wrote manifest: $manifestPath"

Push-Location $BucketPath
git add "bucket/af.json"
git commit -m "af: update to $Version"
git push
Pop-Location

Write-Host "Scoop bucket updated and pushed."
