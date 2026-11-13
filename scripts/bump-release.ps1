param(
  [Parameter(Mandatory = $true)][string]$Version
)

$ErrorActionPreference = "Stop"

if ($Version -notmatch '^v?\d+\.\d+\.\d+.*$') {
  throw "Version must look like 0.1.0 or v0.1.0"
}
$plain = $Version.TrimStart('v')
$tag = "v$plain"

# Update workspace version in Cargo.toml
$cargo = Get-Content "Cargo.toml" -Raw
$cargo = [regex]::Replace($cargo, '(?m)(^\[workspace\.package\][\s\S]*?^version\s*=\s*")[^"]+(")', "`${1}$plain`${2}")
Set-Content -Path "Cargo.toml" -Value $cargo -Encoding UTF8

# Refresh lockfile metadata
cargo update -w 2>$null | Out-Null

# Update scoop manifest template version if present
if (Test-Path "scoop/af.json") {
  $json = Get-Content "scoop/af.json" -Raw | ConvertFrom-Json
  $json.version = $plain
  if ($json.architecture.'64bit'.url) {
    $json.architecture.'64bit'.url = "https://github.com/<owner>/<repo>/releases/download/$tag/af-$tag-x86_64-pc-windows-msvc.zip"
  }
  $json | ConvertTo-Json -Depth 10 | Set-Content -Path "scoop/af.json" -Encoding UTF8
}

git add Cargo.toml Cargo.lock scoop/af.json 2>$null
git commit -m "chore(release): $tag"
git tag $tag
git push origin HEAD
git push origin $tag

Write-Host "Released $tag (commit and tag pushed)."
