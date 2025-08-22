# RuneWeave

## 0. ゴール

RuneWeave は **Runeforge が生成した plan.json** を入力に、

- **Polyglot 言語構成のモノレポ雛形**をローカルに生成、または GitHub Repo に Push
    
- 生成リポジトリはそのまま **MagicRune → RuneTrial** に渡せる
    
- すべての出力は **決定論的**（同じ plan.json + seed → 同一雛形）
    

---

## 1. CLI 仕様

```bash
runeweave apply \
  -p plan.json                # 必須: Runeforge出力
  --seed 42                   # 決定性のための乱数シード
  --repo github:owner/repo    # 指定時はリモートPush/PR
  --policy runeweave.policy.yml  # 任意: ライセンス/依存制約
  --out ./scaffold            # ローカル出力先（省略時はカレント）
  --verify                    # 解析・検証のみ、生成なし
```

### Exit Codes

|Code|意味|
|---|---|
|0|成功|
|1|入力 plan.json が stack.schema.json に不一致|
|2|ポリシー／依存検証に失敗|
|3|GitHub Push / PR 失敗・衝突|

---

## 2. 入出力

### 入力

- `plan.json` : Runeforge の出力 (stack.schema.json に準拠)
    
- `runeweave.policy.yml` : 任意。ライセンス禁止・依存制御・命名規約・CI設定ルール
    

### 出力

- **モノレポ雛形一式**
    
- `weave.manifest.json` : `template_hash`, `seed`, `toolchain`, `plan_hash` など
    

---

## 3. 生成されるレイアウト

```
product/
├ services/
│   ├ api-rs/           # Rust/Actix
│   ├ api-ts/           # Node/Fastify
│   ├ worker-cf/        # Cloudflare Workers (TS/Rust)
│   ├ job-py/           # Python (Poetry/uv)
│   └ job-go/           # Go
├ toolchain/            # 各言語のバージョンpin
│   ├ rust-toolchain.toml
│   ├ .node-version
│   ├ .python-version
│   ├ go.mod
│   └ .java-version
├ schemas/
│   └ stack.schema.json (コピー)
└ .github/workflows/ci.yml   # 言語横断 CI
```

---

## 4. 処理フロー

1. **解析**
    
    - plan.json を読み込み stack.schema.json に従って検証
        
    - policy.yml があれば制約チェック（ライセンス/依存禁止/命名規則）
        
2. **テンプレ展開**
    
    - 各サービスの `language` / `framework` / `runtime` をもとに  
        Language Pack（Rust, Node, Python, Go, Java, .NET, Deno…）を呼び出し
        
    - seed でファイル順序・UUID・生成内容を決定
        
3. **ビルド検証**
    
    - 各言語ごとに `cargo check` / `pnpm install --frozen-lockfile` / `pytest --collect-only` / `go build -n` 等を走らせ雛形整合性を確認
        
4. **manifest 生成**
    
    - `weave.manifest.json` に `template_hash` / `seed` / `toolchain` / `plan_hash` を記録
        
5. **出力**
    
    - `--out` があればローカルに書き出し
        
    - `--repo` があれば新規ブランチを作成して push / PR
        

---

## 5. 雛形仕様

### 5.1 各サービス

- **Rust (api, worker-rs)**
    
    - `Cargo.toml` with pinned MSRV
        
    - `/healthz` エンドポイント
        
- **Node/TS (Fastify, Workers)**
    
    - `package.json` / `tsconfig.json` / `eslint`
        
    - smoke test スクリプト
        
- **Python (job)**
    
    - `pyproject.toml` (Poetry/uv)
        
    - `pytest` 雛形
        
- **Go**
    
    - `go.mod` pinned
        
    - `go test` 雛形
        
- **Java/.NET**
    
    - Maven/Gradle Wrapper or dotnet SDK pin
        
    - 最小REST stub
        

### 5.2 CI (`.github/workflows/ci.yml`)

- `ubuntu-24.04` runner 固定
    
- 言語マトリクス展開
    
    - Rust: `cargo check/test --locked`
        
    - Node: `pnpm install --frozen-lockfile && pnpm test`
        
    - Python: `pytest`
        
    - Go: `go test ./...`
        
    - Java: `mvn test`
        
- 共通: SBOM (syft) 生成, Cosign 署名, Artifacts 保存
    

---

## 6. ポリシー DSL（例）

```yaml
version: 1
deny:
  licenses: ["AGPL-3.0"]
  crates: ["openssl-sys"]
  npm: ["left-pad@*"]
  pypi: ["cryptography<42.0"]
pin:
  rust.msrv: "1.82"
  node.version: "22.6.0"
  python.version: "3.12.6"
  go.version: "1.22.5"
  java.version: "21"
ci:
  linux_runner: "ubuntu-24.04"
  sbom: true
  cosign: true
naming:
  project: "kebab-case"
  service: "kebab-case"
```

---

## 7. 受け入れ基準

|#|条件|
|---|---|
|1|同一 plan.json + seed で weave.manifest.json が不変|
|2|各サービスがビルド検証に通る|
|3|CI ワークフローが生成され、最低限のテストが緑|
|4|ポリシー違反の依存やライセンスを含まない|

---

## 8. 実装構成

```
/src
  main.rs        # CLI
  render.rs      # テンプレート展開
  verify.rs      # スキーマ・ポリシー検証
  git.rs         # GitHub Push/PR
  manifest.rs    # ハッシュ生成
/plugins         # Language Pack (WASM/Process)
  rust-pack/
  node-pack/
  python-pack/
  go-pack/
  ...
/templates       # stub集
/policy          # デフォルトポリシー
/schemas         # stack.schema.json (Runeforgeと同じ)
```

---

## 9. 出力サンプル

```bash
runeweave apply -p plan.json --seed 99 --out ./my-product
```

結果:

```
my-product/
├ services/api-rs/    # Rust Actix stub
├ services/worker-cf/ # TS Workers stub
├ toolchain/
├ schemas/stack.schema.json
└ .github/workflows/ci.yml
```

---

## まとめ

- **Runeforge が決めた Polyglot スタック**を、
    
- **RuneWeave が雛形モノレポに落とし込み、CI付きで再現性保証**
    
- 以降 MagicRune / RuneTrial に安全に渡せる