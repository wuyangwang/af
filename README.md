# AF (Agent Files)

闈㈠悜 AI Coding Agent 鐨勫畨鍏ㄦ枃浠舵搷浣滀笌鎼滅储 CLI銆?
- 鏂板缓鏂囦欢榛樿锛歎TF-8 鏃?BOM + LF
- 淇敼宸叉湁鏂囦欢榛樿锛歅reserve锛堜繚鐣欏師缂栫爜銆丅OM銆佹崲琛岄鏍硷級
- 缂栫爜涓嶇‘瀹氭椂鎷掔粷鍐掗櫓鍐欏叆锛屾敮鎸?`--dry-run`銆佸搱甯屾牎楠屼笌鍘熷瓙鍐欏叆
- 鍐呯疆鎼滅储锛坮ipgrep crates锛夛紝浜虹被鍙涓?Agent JSON 鍙岃緭鍑烘ā寮?
璇︾粏璁捐瑙?[`docs/design.md`](docs/design.md) 涓庢牴鐩綍 [`af-system-design.md`](af-system-design.md)銆?
## 鍛戒护姒傝

```text
af read    # 璇诲彇鏂囦欢銆佽鑼冨洿銆佸師濮嬫枃鏈笌鍏冧俊鎭?af write   # 鍒涘缓鎴栬鐩栨枃浠?af append  # 鍦ㄦ枃浠舵湯灏捐拷鍔犲唴瀹?af search  # 鍦ㄦ枃浠舵垨鐩綍涓悳绱㈡枃鏈?af patch   # 鎸夌簿纭尮閰嶆垨 Unified Diff 淇敼鏂囦欢
```

## 寮€鍙?
```powershell
cargo build
cargo test
cargo run -p af-cli -- --help
```

## CI / Release

- 宸ヤ綔娴侊細`.github/workflows/ci.yml`锛圕I & Release 鍚堝苟锛?  - `push` 鍒伴潪 `master` 鍒嗘敮 / `pull_request`锛氬湪 Windows/macOS 涓婃瀯寤恒€佹祴璇曞苟鎵ц CLI smoke test
  - `push` 鍒?`master`锛氳嚜鍔ㄨ鍙?`Cargo.toml` 鐗堟湰锛屽垱寤哄苟鎺ㄩ€?`vX.Y.Z` tag锛堣嫢涓嶅瓨鍦級锛岀敱 tag 瑙﹀彂 Release 娴佺▼
  - `tag v*`锛氭瀯寤?Windows x64 涓?macOS x64/arm64 鍙戝竷浜х墿锛屾墦鍖?`.zip`/`.tar.gz` + `.sha256`锛岃嚜鍔ㄥ垱寤?GitHub Release
  - `workflow_dispatch`锛氬彲閫夎緭鍏ョ増鏈彿锛堢暀绌哄垯浣跨敤 `Cargo.toml` 鐗堟湰锛夎嚜鍔ㄥ垱寤哄苟鎺ㄩ€?tag锛岄殢鍚庤Е鍙?Release 娴佺▼
- 鏈湴鎵撳寘鑴氭湰锛歚scripts/release.ps1`锛圵indows锛夈€乣scripts/release.sh`锛坢acOS/Linux锛?- 鏈湴涓€閿彂鐗堣剼鏈細`scripts/bump-release.ps1` / `scripts/bump-release.sh`锛堟洿鏂扮増鏈彿銆佹彁浜?`chore(release): vX.Y.Z`銆佸垱寤?tag 骞?push commit+tag锛?
## Scoop锛圵indows锛?
- Manifest 妯℃澘锛歚scoop/af.json`
- 鍙戝竷璇存槑涓庡懡浠わ細`scoop/PUBLISHING.md`
- 鑷姩鏇存柊 bucket 鑴氭湰锛歚scripts/scoop-publish.ps1`

## 椤圭洰缁撴瀯

- `crates/af-cli`锛欳LI 鍏ュ彛涓庡懡浠よ矾鐢?- `crates/af-core`锛氱紪鐮併€佹崲琛屻€佽鍐欍€佸師瀛愬啓鍏ャ€佸搱甯屼笌閿欒绫诲瀷
- `crates/af-search`锛氭悳绱㈠紩鎿庝笌杩囨护
- `crates/af-render`锛氶珮浜€丏iff 涓?JSON 杈撳嚭
- `docs/`锛氳璁′笌鍗忚鏂囨。
- `tests/`锛氶泦鎴愭祴璇曚笌 fixtures
- `examples/`锛氱ず渚嬭緭鍏ワ紙濡?patch spec锛?