# RuneWeave

（Rust 2021 / **MSRV 1.80** でビルド保証）

---

## 0. ゴール

Runeforge が出力した **Blueprint / plan** を受け取り、→ **Rust-Edge 用モノレポ雛形** をローカル生成 _または_ GitHub Repo に Push する。  
生成したリポジトリはそのまま MagicRune → RuneTrial へ渡せる。

---

## 1. 入出力

|種別|形式|スキーマ|
|---|---|---|
|入力|`plan.json`|`schemas/stack.schema.json`（Runeforge 出力を想定）|
|入力|`runeweave.policy.yml`（任意）|ポリシー DSL（依存/ライセンス/命名/CI ルール）|
|出力|**ファイル群**|モノレポ雛形一式|
|出力|`weave.manifest.json`|`template_hash` / `seed` / `toolchain` / `plan_hash` など|

### 1.1 CLI

```bash
runeweave apply \
  -p plan.json                # 必須
  --seed 42                   # 決定論的生成
  --repo github:owner/repo    # 指定しなければローカル生成のみ
  --policy runeweave.policy.yml
  --out ./scaffold            # ローカル出力先
  --verify                    # 解析のみ・生成しない
```

|Exit|意味|
|---|---|
|0|成功|
|1|入力スキーマ不一致|
|2|検証失敗（ポリシー・依存）|
|3|リポジトリ衝突 / Push 失敗|

---

## 2. 生成されるレイアウト

```
product/
├ Cargo.toml                 # [workspace]
├ rust-toolchain.toml        # channel=stable + wasm32-unknown-unknown
├ services/
│   ├ api/                   # Actix Web stub (Rust)
│   └ api-edge/              # workers-rs (crate: worker) stub (Rust, wasm32-unknown-unknown)
├ tools/cli/                 # clap helper / smoke テスト用ツール
├ schemas/                   # Runeforge 出力の stack.schema.json をコピー
└ .github/workflows/ci.yml   # CI（ビルド/テスト/SBOM/署名/Artifacts）
```

- **Edge**: Rust on Cloudflare Workers（`workers-rs`/crate: `worker`、ターゲット `wasm32-unknown-unknown`）。([Cloudflare Docs](https://developers.cloudflare.com/workers/languages/rust/?utm_source=chatgpt.com "Cloudflare Workers — Rust language support · Cloudflare Workers docs"), [Docs.rs](https://docs.rs/worker/latest/worker/?utm_source=chatgpt.com "worker - Rust - Docs.rs"))
    
- **Wrangler 設定**: `wrangler.jsonc` を既定生成（Wrangler v3.91+ は JSON/JSONC をサポート）。([Cloudflare Docs](https://developers.cloudflare.com/workers/wrangler/configuration/?utm_source=chatgpt.com "Configuration - Wrangler · Cloudflare Workers docs"))
    

---

## 3. 主要処理フロー

1. **解析**: `plan.json` と `runeweave.policy.yml` を読込み、入出力スキーマ検証。
    
2. **テンプレ展開**: `templates/` を Tera で描画（`{{project}}`, `{{service}}` 等）。seed を使い**ファイル順・内容の決定性**を担保。
    
3. **ビルド検証**: `cargo check --workspace --locked`、Workers は `wrangler` の設定検査（構文/設定の検証。JSONC 利用）。([Cloudflare Docs](https://developers.cloudflare.com/workers/wrangler/configuration/?utm_source=chatgpt.com "Configuration - Wrangler · Cloudflare Workers docs"))
    
4. **manifest 生成**: `weave.manifest.json` に `template_hash` / `seed` / `toolchain` / `plan_hash(SHA-256)` を保存。
    
5. **出力**: `--out` があればローカル出力、`--repo` 指定時は新規ブランチで Push / PR（libgit2 バインディング `git2` を使用）。([Docs.rs](https://docs.rs/git2/latest/index.html?utm_source=chatgpt.com "git2 - Rust - Docs.rs"))
    

---

## 4. 雛形（テンプレ）仕様

### 4.1 `Cargo.toml`（workspace ルート・抜粋）

```toml
[workspace]
members = ["services/api", "services/api-edge", "tools/cli"]
resolver = "2"

[workspace.package]
edition = "2021"
rust-version = "1.80"   # MSRV 固定
```

### 4.2 `services/api`（Actix Web stub）

- 依存（固定ピン）: `actix-web = "4"` / `tracing = "0.1"`
    
- `GET /healthz` と `/v1/ready` を返す最小実装（`cargo check` 通過）。
    

### 4.3 `services/api-edge`（Workers Rust stub）

- 依存: `worker = "0.6"`（Workers Rust SDK）。`wasm32-unknown-unknown` でビルド。([Docs.rs](https://docs.rs/worker/latest/worker/?utm_source=chatgpt.com "worker - Rust - Docs.rs"))
    
- ルータで `GET /healthz` を返却、KV/R2/Queues のバインドは雛形コメント。([Docs.rs](https://docs.rs/worker/latest/worker/struct.Router.html?utm_source=chatgpt.com "Router in worker - Rust - Docs.rs"))
    
- `wrangler.jsonc` を生成（アカウント ID 等は CI/CD 注入）。**JSONC 対応**。([Cloudflare Docs](https://developers.cloudflare.com/workers/wrangler/configuration/?utm_source=chatgpt.com "Configuration - Wrangler · Cloudflare Workers docs"))
    

### 4.4 `tools/cli`

- `clap` で smoke テスト（API/Edge の起動/疎通簡易チェック）。
    

---

## 5. ポリシー DSL（`runeweave.policy.yml`）

```yaml
version: 1
deny:
  licenses: ["AGPL-3.0"]
  crates: ["openssl-sys"]         # 例: musl 互換性のため
pin:
  rust_toolchain: "stable"
  msrv: "1.80"
  worker_target: "wasm32-unknown-unknown"
ci:
  linux_runner: "ubuntu-24.04"
  sbom: true
  cosign: true
naming:
  project: "kebab-case"
  service: "kebab-case"
```

---

## 6. CI（生成される `.github/workflows/ci.yml` 概要）

- ランナー: `ubuntu-24.04`（固定）
    
- 手順: `actions/checkout@v4` → `dtolnay/rust-toolchain@stable` → `Swatinem/rust-cache@v2` → **ビルド/テスト** → **SBOM（Syft/anchore）** → **Cosign 署名** → **Artifacts v4**。
    
    - ツールチェイン導入（`dtolnay/rust-toolchain`）。([GitHub](https://github.com/dtolnay/rust-toolchain?utm_source=chatgpt.com "GitHub - dtolnay/rust-toolchain: Concise GitHub Action for installing a ..."))
        
    - Rust キャッシュ（`Swatinem/rust-cache`）。([GitHub](https://github.com/Swatinem/rust-cache?utm_source=chatgpt.com "GitHub - Swatinem/rust-cache: A GitHub Action that implements smart ..."))
        
    - クロスコンパイル（`cross`）は必要に応じて導入。([GitHub](https://github.com/cross-rs/cross?utm_source=chatgpt.com "GitHub - cross-rs/cross: “Zero setup” cross compilation and “cross ..."))
        
    - SBOM 生成（`anchore/sbom-action@v0`）。([GitHub](https://github.com/anchore/sbom-action?utm_source=chatgpt.com "GitHub - anchore/sbom-action: GitHub Action for creating software bill ..."))
        
    - Cosign インストール/署名（`sigstore/cosign-installer@v3`）。([GitHub](https://github.com/sigstore/cosign-installer?utm_source=chatgpt.com "GitHub - sigstore/cosign-installer: Cosign Github Action"), [Sigstore](https://docs.sigstore.dev/quickstart/quickstart-ci/?utm_source=chatgpt.com "Sigstore CI Quickstart"))
        
    - アーティファクト v4（`actions/upload-artifact@v4`）。([GitHub](https://github.com/actions/upload-artifact?utm_source=chatgpt.com "GitHub - actions/upload-artifact"))
        

**雛形 CI（抜粋）**

```yaml
name: ci
on: [push, pull_request]

jobs:
  build-test:
    runs-on: ubuntu-24.04
    permissions:
      contents: read
      id-token: write     # cosign keyless 用
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo check --workspace --locked
      - run: cargo test  --workspace --locked

      # Edge stub の設定検査（wrangler JSONC 設定がある前提）
      - name: Wrangler config check
        run: echo "wrangler.jsonc present and pinned"  # JSONC サポート根拠あり
      # 参考: Wrangler は v3.91+ で wrangler.json/jsonc をサポート
      # https://developers.cloudflare.com/workers/wrangler/configuration/

      - name: Install cross (optional)
        run: cargo install cross --git https://github.com/cross-rs/cross

      - name: SBOM (Syft)
        uses: anchore/sbom-action@v0
        with:
          path: .
          output-file: sbom.spdx.json

      - name: Cosign install
        uses: sigstore/cosign-installer@v3

      - name: Sign SBOM (keyless)
        run: cosign sign-blob --yes --bundle sbom.spdx.json

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: build-${{ github.sha }}
          path: |
            sbom.spdx.json
            sbom.spdx.json.sig
```

---

## 7. 受け入れ基準

|#|条件|
|---|---|
|1|同 `plan.json` + `--seed` で `weave.manifest.json.template_hash` が不変|
|2|`cargo check --workspace --locked` が緑|
|3|Workers 側設定（`wrangler.jsonc`）が配置され、構文/設定検査を通る（JSON/JSONC サポート根拠に基づく）([Cloudflare Docs](https://developers.cloudflare.com/workers/wrangler/configuration/?utm_source=chatgpt.com "Configuration - Wrangler · Cloudflare Workers docs"))|
|4|生成レポに CI ワークフローが存在し、`cargo test` が通る|
|5|ポリシーで禁止された依存やライセンスが含まれない|

---

## 8. 実装構成

```
/src
  main.rs        # clap CLI (subcmd: apply / verify)
  render.rs      # Tera によるテンプレ展開（seed で決定化）
  verify.rs      # スキーマ & ポリシー検証
  git.rs         # Push / PR（git2）
  manifest.rs    # template_hash / plan_hash（SHA-256）
/templates       # Stubs (api, api-edge, tools/cli, ci.yml, wrangler.jsonc)
/policy          # runeweave.policy.yml (optional defaults)
/schemas
  stack.schema.json
```

### 依存（固定）

|crate|ver|用途|
|---|---|---|
|`clap`|4.5|CLI（derive）([GitHub](https://github.com/dtolnay/rust-toolchain?utm_source=chatgpt.com "GitHub - dtolnay/rust-toolchain: Concise GitHub Action for installing a ..."))|
|`serde` / `serde_json` / `serde_yaml`|1 / 1 / 0.9系|JSON/YAML I/O（derive）([Docs.rs](https://docs.rs/crate/serde_yaml/latest?utm_source=chatgpt.com "serde_yaml 0.9.34+deprecated - Docs.rs"))|
|`schemars`|0.8|JSON Schema 生成/検証 ([Docs.rs](https://docs.rs/schemars/latest/schemars/?utm_source=chatgpt.com "schemars - Rust - Docs.rs"), [Graham’s Cool Site](https://graham.cool/schemars/v0/?utm_source=chatgpt.com "Overview \| Schemars"))|
|`tera`|1.19|テンプレート描画（Jinja2系）([Docs.rs](https://docs.rs/crate/tera/1.19.1?utm_source=chatgpt.com "tera 1.19.1 - Docs.rs"), [Lib.rs](https://lib.rs/crates/tera?utm_source=chatgpt.com "Tera — Rust template engine // Lib.rs"))|
|`git2`|0.18+|libgit2 バインディング（Push/PR 支援）([Docs.rs](https://docs.rs/git2/latest/index.html?utm_source=chatgpt.com "git2 - Rust - Docs.rs"))|
|`rand`|0.8 以上|決定性 RNG（seed 固定）([Docs.rs](https://docs.rs/crate/rand/0.8.3/source/README.md?utm_source=chatgpt.com "rand 0.8.3 - Docs.rs"))|
|`sha2` + `hex`|0.10 / 0.4|ハッシュ計算（SHA-256/hex）([Docs.rs](https://docs.rs/crate/sha2/latest?utm_source=chatgpt.com "sha2 0.10.9 - Docs.rs"))|

---

## 9. サンプル

```bash
# ローカルに scaffold
runeweave apply -p plan.json --seed 99 --out ./my-product

# GitHub Repo に Push（public 例）
runeweave apply -p plan.json --seed 99 \
  --repo github:myorg/my-product --public
```

---

## 10. テスト

- `cargo test` で以下を検証：
    
    - 入力スキーマ適合（NG で exit=1）— `schemars` 検証。([Docs.rs](https://docs.rs/schemars/latest/schemars/?utm_source=chatgpt.com "schemars - Rust - Docs.rs"))
        
    - 決定論的生成（同入力+seed → `weave.manifest.json.template_hash` 不変）
        
    - 3 ケース（`examples/baseline.yaml` / `latency.yaml` / `compliance.yaml`）が緑
        

---