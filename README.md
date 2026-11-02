# omnitool

[![CI](https://github.com/pranov888/omnitool/actions/workflows/ci.yml/badge.svg)](https://github.com/pranov888/omnitool/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

> The all-in-one developer Swiss-Army knife CLI written in Rust. Fast, zero-dependency runtime, single static binary.

Instead of installing a dozen disconnected utilities, `omnitool` bundles everyday developer workflows into a fast terminal tool.

---

## Features

- **Crypto and Security**:
  - Hash computation (`SHA-256`, `MD5`)
  - Base64 encoding and decoding (standard, URL-safe, unpadded)
  - UUIDv4 generator
  - High-entropy secure password generation
- **Data Encoding and Codecs**:
  - URL percent-encoding and decoding (RFC 3986 with space/plus handling)
  - Hexadecimal encoding and decoding with ASCII/UTF-8 validation
  - Standard input (`-`) pipeline streaming support
- **Time and Timestamp Telemetry**:
  - High-precision UNIX epoch timestamps (seconds and milliseconds)
  - ISO 8601 UTC date-time formatting and conversion
  - Strict calendar and leap-year validation
- **Terminal QR Code**:
  - Instant ASCII/UTF-8 scannable QR generation for links, text, and keys
- **Data and JSON Tools**:
  - Pretty formatting with syntax verification
  - JSON minification
  - Structural validation for arrays and objects
- **System and Environment Telemetry**:
  - Host OS, kernel version, and hardware stats
  - Multi-core CPU detection and real-time RAM/Swap metrics
  - Mounted disk partitions and storage space inspection
- **Network Diagnostics**:
  - Network interface activity telemetry (RX/TX counters)
  - Ultra-fast TCP port connectivity and latency probing (`ping`)

---

## Installation

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

## Usage Examples

### 1. Cryptographic Hashes
```bash
# Compute SHA-256
omnitool crypto hash "my-secret-payload"

# Compute MD5
omnitool crypto hash --algorithm md5 "my-secret-payload"
```

### 2. Base64 Encoding and Decoding
```bash
# Encode
omnitool crypto b64-encode "developer-token-123"

# Decode (handles standard, URL-safe, and unpadded strings)
omnitool crypto b64-decode "ZGV2ZWxvcGVyLXRva2VuLTEyMw=="
```

### 3. Generate UUID and Passwords
```bash
# Generate UUIDv4
omnitool crypto uuid

# Generate a 24-character high-entropy password
omnitool crypto pass --length 24 --symbols
```

### 4. URL and Hex Encoders
```bash
# URL percent-encode
omnitool encode url-encode "hello world & rust=fast"

# URL percent-decode
omnitool encode url-decode "hello%20world%20%26%20rust%3Dfast"

# Hex-encode text
omnitool encode hex-encode "developer toolkit"

# Hex-decode to text
omnitool encode hex-decode "646576656c6f70657220746f6f6c6b6974"
```

### 5. Time and Timestamp Operations
```bash
# Current epoch seconds, milliseconds, and ISO 8601 UTC
omnitool time now

# Convert epoch to ISO 8601 UTC
omnitool time to-iso 1700000000

# Parse ISO 8601 UTC to Unix epoch seconds
omnitool time to-epoch 2023-11-14T22:13:20Z
```

### 6. Terminal QR Codes
```bash
# Generate scannable QR in terminal
omnitool qr "https://github.com/pranov888/omnitool"
```

### 6. JSON Formatting and Validation
```bash
# Format and prettify JSON string or file
omnitool json format '{"service":"api","status":"ok","latency_ms":14}'

# Minify JSON from standard input pipeline
cat data.json | omnitool json minify -

# Validate JSON structure
omnitool json validate '{"items":[1,2,3]}'
```

### 7. System and Hardware Telemetry
```bash
# Print host info, CPU model, RAM usage, and disk space
omnitool sys
```

### 8. Network Probes and Latency
```bash
# Inspect network interfaces
omnitool net interfaces

# Test TCP latency to host (default port 443)
omnitool net ping github.com --port 443
```

---

## Roadmap and Changelog

### v0.5.0 - Time and Timestamp Telemetry
- Added `time` module with high-precision UNIX epoch timestamps
- Implemented bidirectional ISO 8601 UTC formatting and parsing
- Expanded unit test coverage to 24 tests

### v0.4.0 - Data Codecs, Dual-Stack Probing, and Stdin Pipeline Support
- Added `encode` module for URL percent-encoding/decoding and hexadecimal encoding/decoding
- Added standard input (`-`) streaming support for input-processing commands
- Enhanced network TCP ping with automatic dual-stack (IPv4/IPv6) socket fallback
- Expanded test suite to 19 unit tests with full cross-platform coverage

### v0.3.0 - Hardware and Network Diagnostics
- Added `sys` telemetry for hardware, memory, and mounted disk metrics
- Added `net` module with interface telemetry and low-latency TCP ping probing
- Full cross-platform support and unit test coverage

### v0.2.0 - JSON Tooling and Formatters
- Added `json` subcommand with `format`, `minify`, and `validate` actions
- Support for inline JSON payloads or direct file path inputs
- Unit test coverage for serialization and structural validation

### v0.1.0 - Initial Release
- Core CLI framework powered by `clap`
- High-speed cryptographic operations (`crypto` module)
- UTF-8 terminal QR renderer (`qr` module)
- Automated GitHub Actions CI workflow

---

## Contributing

Contributions, issues, and feature requests are welcome. Feel free to check the [issues page](https://github.com/pranov888/omnitool/issues).

---

## License

Distributed under the [MIT](LICENSE) License.
