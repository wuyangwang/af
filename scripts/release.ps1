param(
  [string]$Version = "",
  [string]$Target = ""
)

$ErrorActionPreference = "Stop"
$bin = "af"

if (-not $Version) {
  $cargo = Get-Content "Cargo.toml" -Raw
  if ($cargo -match 'version\s*=\s*"([^"]+)"') { $Version = "v$($Matches[1])" } else { throw "Version not found" }
}

if (-not $Target) {
  $rustc = & rustc -vV
  $host = ($rustc | Select-String "^host:").ToString().Split(":")[1].Trim()
  $Target = $host
}

Write-Host "Building $bin $Version for $Target"
cargo build -p af-cli --release --target $Target

$name = "$bin-$Version-$Target"
$dist = Join-Path "dist" $name
New-Item -ItemType Directory -Force -Path $dist | Out-Null
Copy-Item "target/$Target/release/$bin.exe" $dist/
Copy-Item "README.md","LICENSE" $dist/

$zip = "dist/$name.zip"
Compress-Archive -Path "$dist/*" -DestinationPath $zip -Force
$hash = (Get-FileHash $zip -Algorithm SHA256).Hash.ToLower()
"$hash  $([IO.Path]::GetFileName($zip))" | Out-File -Encoding ascii "$zip.sha256"

Write-Host "Created $zip"
Write-Host "SHA256: $hash"
