# ⚡ omnitool

[![CI](https://github.com/pranov888/omnitool/actions/workflows/ci.yml/badge.svg)](https://github.com/pranov888/omnitool/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

> The all-in-one developer Swiss-Army knife CLI written in Rust. Fast, zero-dependency runtime, single static binary.

Instead of installing a dozen disconnected utilities, `omnitool` bundles everyday developer workflows into a lightning-fast terminal tool.

---

## 🚀 Features

- 🔐 **Crypto & Security**:
  - Hash computation (`SHA-256`, `MD5`)
  - Base64 encoding and decoding
  - UUIDv4 generator
  - High-entropy secure password generation
- 📱 **Terminal QR Code**:
  - Instant ASCII/UTF-8 scannable QR generation for links, text, and keys
- 📄 **Data & JSON Tools**:
  - Pretty formatting with syntax verification
  - JSON minification
  - Structural validation for arrays and objects
- 💻 **System & Network Telemetry** *(Coming in v0.3)*

---

## 📦 Installation

### From Source
```bash
git clone https://github.com/pranov888/omnitool.git
cd omnitool
cargo install --path .
```

### Direct Run
```bash
cargo run -- --help
```

---

## 🛠️ Usage Examples

### 1. Cryptographic Hashes
```bash
# Compute SHA-256
omnitool crypto hash "my-secret-payload"

# Compute MD5
omnitool crypto hash --algorithm md5 "my-secret-payload"
```

### 2. Base64 Encoding & Decoding
```bash
# Encode
omnitool crypto b64-encode "developer-token-123"

# Decode
omnitool crypto b64-decode "ZGV2ZWxvcGVyLXRva2VuLTEyMw=="
```

### 3. Generate UUID & Passwords
```bash
# Generate UUIDv4
omnitool crypto uuid

# Generate a 24-character high-entropy password
omnitool crypto pass --length 24 --symbols
```

### 4. Terminal QR Codes
```bash
# Generate scannable QR in terminal
omnitool qr "https://github.com/pranov888/omnitool"
```

### 5. JSON Formatting & Validation
```bash
# Format and prettify JSON string or file
omnitool json format '{"service":"api","status":"ok","latency_ms":14}'

# Minify JSON
omnitool json minify '{\n  "status": "ok"\n}'

# Validate JSON structure
omnitool json validate '{"items":[1,2,3]}'
```

---

## 🗺️ Roadmap & Changelog

### v0.2.0 - JSON Tooling & Formatters
- Added `json` subcommand with `format`, `minify`, and `validate` actions
- Support for inline JSON payloads or direct file path inputs
- Unit test coverage for serialization and structural validation

### v0.1.0 - Initial Release
- Core CLI framework powered by `clap`
- High-speed cryptographic operations (`crypto` module)
- UTF-8 terminal QR renderer (`qr` module)
- Automated GitHub Actions CI workflow

---

## 🤝 Contributing

Contributions, issues, and feature requests are welcome! Feel free to check the [issues page](https://github.com/pranov888/omnitool/issues).

---

## 📄 License

Distributed under the [MIT](LICENSE) License.
