param(
  [string]$Version = ""
)

$ErrorActionPreference = "Stop"

function Write-Utf8NoBom([string]$Path, [string]$Content) {
  $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
  [System.IO.File]::WriteAllText((Resolve-Path $Path), $Content, $utf8NoBom)
}

if (-not $Version) {
  $cargoText = [System.IO.File]::ReadAllText("Cargo.toml")
  if ($cargoText -match '(?ms)^\[workspace\.package\].*?^version\s*=\s*"([^"]+)"') {
    $Version = $Matches[1]
  } else {
    throw "Version not provided and could not be read from Cargo.toml"
  }
}

if ($Version -notmatch '^v?\d+\.\d+\.\d+.*$') {
  throw "Version must look like 0.1.0 or v0.1.0"
}
$plain = $Version.TrimStart('v')
$tag = "v$plain"

# Update workspace version in Cargo.toml
$cargo = [System.IO.File]::ReadAllText("Cargo.toml")
$cargo = [regex]::Replace($cargo, '(?m)(^\[workspace\.package\][\s\S]*?^version\s*=\s*")[^"]+(")', "`${1}$plain`${2}")
Write-Utf8NoBom -Path "Cargo.toml" -Content $cargo

# Refresh lockfile metadata (best effort)
try { & cargo update -w | Out-Null } catch { }

# Update scoop manifest template version if present
if (Test-Path "scoop/af.json") {
  $jsonText = [System.IO.File]::ReadAllText("scoop/af.json")
  $json = $jsonText | ConvertFrom-Json
  $json.version = $plain
  if ($json.architecture.'64bit'.url) {
    $json.architecture.'64bit'.url = "https://github.com/<owner>/<repo>/releases/download/$tag/af-$tag-x86_64-pc-windows-msvc.zip"
  }
  $out = $json | ConvertTo-Json -Depth 10
  Write-Utf8NoBom -Path "scoop/af.json" -Content ($out + "`n")
}

git add Cargo.toml Cargo.lock scoop/af.json 2>$null
git commit -m "chore(release): $tag"
git tag $tag
git push origin HEAD
git push origin $tag

Write-Host "Released $tag (commit and tag pushed)."
