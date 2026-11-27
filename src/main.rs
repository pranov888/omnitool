mod codec;
mod crypto;
mod json_tool;
mod qr;
mod telemetry;
mod text_tool;
mod time_tool;
mod ui;

use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{Args, Parser, Subcommand, ValueEnum};
use colored::*;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::Path;
use std::time::Duration;

fn cli_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::BrightCyan.on_default() | Effects::BOLD)
        .usage(AnsiColor::BrightCyan.on_default() | Effects::BOLD)
        .literal(AnsiColor::BrightGreen.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::BrightYellow.on_default())
        .valid(AnsiColor::BrightGreen.on_default())
        .invalid(AnsiColor::BrightRed.on_default() | Effects::BOLD)
        .error(AnsiColor::BrightRed.on_default() | Effects::BOLD)
}

#[derive(Parser)]
#[command(
    name = "omnitool",
    author = "pranov888",
    version = "0.8.0",
    styles = cli_styles(),
    about = "All-in-one developer toolkit for crypto, encoding, JSON, text, time, QR codes, and diagnostics",
    long_about = "Omnitool bundles everyday developer utilities (crypto, codecs, QR codes, JSON tools, time conversions, text transforms, system overview, and network diagnostics) into a fast terminal utility. Run without arguments to launch interactive mode."
)]
struct Cli {
    /// Print bare raw output without panels or labels (ideal for scripts and pipes)
    #[arg(short, long, global = true)]
    raw: bool,

    /// Disable colored terminal output
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Cryptographic helpers (hashes, file checksums, base64, uuid, passwords)
    Crypto(CryptoArgs),
    /// URL, Hex, and JWT data encoding, decoding, and inspection
    Encode(EncodeArgs),
    /// UNIX epoch timestamps and ISO 8601 UTC date-time conversions
    Time(TimeArgs),
    /// Text analysis, counting, casing, slug, and reversal utilities
    Text(TextArgs),
    /// Generate scannable QR codes in the terminal or as SVG
    Qr(QrArgs),
    /// JSON data processing (format, minify, validate)
    Json(JsonArgs),
    /// System hardware, memory, and storage overview
    Sys(SysArgs),
    /// Network diagnostics, interface counters, DNS lookup, and TCP latency
    Net(NetArgs),
    /// Launch the interactive guided terminal menu
    Interactive,
}

#[derive(Args)]
struct CryptoArgs {
    #[command(subcommand)]
    action: CryptoSubcommand,
}

#[derive(Subcommand)]
enum CryptoSubcommand {
    /// Compute cryptographic hash (SHA-256, SHA-512, MD5)
    Hash {
        #[arg(short, long, default_value = "sha256")]
        algorithm: HashAlgorithm,
        /// Input text to hash (or '-' for stdin)
        text: String,
    },
    /// Base64 encode an ASCII or UTF-8 string
    B64Encode {
        /// Text to encode (or '-' for stdin)
        text: String,
    },
    /// Base64 decode a base64 encoded string (standard, URL-safe, or unpadded)
    B64Decode {
        /// Encoded text to decode (or '-' for stdin)
        encoded: String,
    },
    /// Generate a random UUIDv4
    Uuid,
    /// Generate a secure random password
    Pass {
        /// Length of password
        #[arg(short, long, default_value_t = 16)]
        length: usize,
        /// Include special symbols (pass --no-symbols or --symbols=false to disable)
        #[arg(
            short,
            long,
            default_value_t = true,
            num_args = 0..=1,
            default_missing_value = "true",
            action = clap::ArgAction::Set
        )]
        symbols: bool,
        /// Exclude special symbols (alphanumeric characters only)
        #[arg(long)]
        no_symbols: bool,
    },
    /// Calculate file checksum hash (SHA-256, SHA-512, MD5)
    Checksum {
        #[arg(short, long, default_value = "sha256")]
        algorithm: HashAlgorithm,
        /// Path to the file
        path: String,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum HashAlgorithm {
    Sha256,
    Sha512,
    Md5,
}

#[derive(Args)]
struct EncodeArgs {
    #[command(subcommand)]
    action: EncodeSubcommand,
}

#[derive(Subcommand)]
enum EncodeSubcommand {
    /// URL percent-encode an ASCII or UTF-8 string
    UrlEncode {
        /// Text to URL-encode (or '-' for stdin)
        text: String,
    },
    /// Decode a percent-encoded URL string
    UrlDecode {
        /// URL encoded text to decode (or '-' for stdin)
        encoded: String,
    },
    /// Hex-encode an input string into lowercase hexadecimal
    HexEncode {
        /// Text to hex-encode (or '-' for stdin)
        text: String,
    },
    /// Hex-decode a hexadecimal string back to UTF-8
    HexDecode {
        /// Hexadecimal string to decode (or '-' for stdin)
        hex: String,
    },
    /// Inspect and decode JSON Web Token (JWT) header and payload claims
    Jwt {
        /// JWT token string (or '-' for stdin)
        token: String,
    },
}

#[derive(Args)]
struct TimeArgs {
    #[command(subcommand)]
    action: TimeSubcommand,
}

#[derive(Subcommand)]
enum TimeSubcommand {
    /// Show current UNIX epoch in seconds and milliseconds with UTC timestamp
    Now,
    /// Convert a UNIX epoch timestamp (seconds or milliseconds) to ISO 8601 UTC
    ToIso {
        /// Epoch timestamp number (supports negative timestamps)
        #[arg(allow_hyphen_values = true)]
        epoch: i64,
    },
    /// Parse an ISO 8601 date-time string (e.g. 2026-11-02T12:00:00Z) to UNIX epoch
    ToEpoch {
        /// ISO 8601 date string (or '-' for stdin)
        iso: String,
    },
}

#[derive(Args)]
struct TextArgs {
    #[command(subcommand)]
    action: TextSubcommand,
}

#[derive(Subcommand)]
enum TextSubcommand {
    /// Count lines, words, characters, and byte size of string or file
    Count {
        /// Text string, file path, or '-' for stdin
        target: String,
    },
    /// Convert text case style (snake, camel, kebab, upper, lower, title)
    Case {
        /// Target case formatting style
        #[arg(short, long, value_enum)]
        style: TextCaseStyle,
        /// Input text to transform (or '-' for stdin)
        text: String,
    },
    /// Generate a URL-friendly slug from text or file
    Slug {
        /// Text string, file path, or '-' for stdin
        target: String,
    },
    /// Reverse characters of a text string or file
    Reverse {
        /// Text string, file path, or '-' for stdin
        target: String,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum TextCaseStyle {
    Snake,
    Camel,
    Kebab,
    Upper,
    Lower,
    Title,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum QrEcArg {
    L,
    M,
    Q,
    H,
}

impl From<QrEcArg> for qr::QrEcLevel {
    fn from(val: QrEcArg) -> Self {
        match val {
            QrEcArg::L => qr::QrEcLevel::L,
            QrEcArg::M => qr::QrEcLevel::M,
            QrEcArg::Q => qr::QrEcLevel::Q,
            QrEcArg::H => qr::QrEcLevel::H,
        }
    }
}

#[derive(Args)]
struct QrArgs {
    /// URL or text content to encode into a QR code (or '-' for stdin)
    content: String,

    /// Error correction level (l=7%, m=15%, q=25%, h=30%)
    #[arg(long, value_enum, default_value_t = QrEcArg::M)]
    ec: QrEcArg,

    /// Render classic double-width full blocks instead of compact half-blocks
    #[arg(long)]
    full: bool,

    /// Invert QR colors (light modules on dark background)
    #[arg(long)]
    invert: bool,

    /// Disable ANSI color escapes in the QR matrix
    #[arg(long)]
    plain: bool,

    /// Export QR code to an SVG file
    #[arg(long, value_name = "PATH")]
    svg: Option<String>,

    /// Append #no_universal_links to http(s) URLs without a fragment (helps bypass iOS app handoff)
    #[arg(long)]
    no_universal_links: bool,
}

#[derive(Args)]
struct JsonArgs {
    #[command(subcommand)]
    action: JsonSubcommand,
}

#[derive(Subcommand)]
enum JsonSubcommand {
    /// Format and prettify JSON string or file
    Format {
        /// JSON string, file path, or '-' for stdin
        target: String,
    },
    /// Minify JSON string or file
    Minify {
        /// JSON string, file path, or '-' for stdin
        target: String,
    },
    /// Validate JSON syntax and inspect structure
    Validate {
        /// JSON string, file path, or '-' for stdin
        target: String,
    },
}

#[derive(Args)]
struct SysArgs {
    /// Include virtual, loop, and temporary filesystems
    #[arg(short, long)]
    all: bool,
}

#[derive(Args)]
struct NetArgs {
    #[command(subcommand)]
    action: NetSubcommand,
}

#[derive(Subcommand)]
enum NetSubcommand {
    /// List network interfaces and activity counters
    Interfaces,
    /// Probe TCP connection latency to a host and port
    Ping {
        /// Host to probe (e.g. github.com, 1.1.1.1)
        host: String,
        /// Port number to connect to
        #[arg(short, long, default_value_t = 443)]
        port: u16,
        /// Connection timeout per address in milliseconds
        #[arg(short, long, default_value_t = 3000)]
        timeout: u64,
    },
    /// Perform DNS lookup resolution for a hostname
    Dns {
        /// Hostname to resolve (e.g. github.com, rust-lang.org)
        host: String,
    },
}

fn read_stdin_string() -> Result<String, String> {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .map_err(|e| format!("Failed to read from standard input: {e}"))?;
    Ok(buffer)
}

fn resolve_arg_or_stdin(arg: &str) -> Result<String, String> {
    if arg == "-" {
        read_stdin_string()
    } else {
        Ok(arg.to_string())
    }
}

enum InputKind {
    Stdin,
    File,
    Literal,
}

struct ResolvedInput {
    content: String,
    kind: InputKind,
    source_label: String,
}

fn looks_like_explicit_path(target: &str) -> bool {
    target.starts_with("./")
        || target.starts_with("../")
        || target.starts_with("~/")
        || target.starts_with(".\\")
        || target.starts_with("..\\")
}

fn resolve_input_or_file(target: &str, expect_json: bool) -> Result<ResolvedInput, String> {
    if target == "-" {
        let content = read_stdin_string()?;
        return Ok(ResolvedInput {
            content,
            kind: InputKind::Stdin,
            source_label: "stdin".to_string(),
        });
    }

    let path = Path::new(target);
    if path.exists() {
        if path.is_dir() {
            return Err(format!(
                "'{target}' is a directory, expected a file or text input"
            ));
        }
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read file '{target}': {e}"))?;
        return Ok(ResolvedInput {
            content,
            kind: InputKind::File,
            source_label: target.to_string(),
        });
    }

    let looks_json_literal = {
        let trimmed = target.trim_start();
        trimmed.starts_with(['{', '[', '"'])
            || matches!(trimmed, "true" | "false" | "null")
            || trimmed.parse::<f64>().is_ok()
    };

    let missing_file_intended = looks_like_explicit_path(target)
        || (expect_json
            && !looks_json_literal
            && (target.to_ascii_lowercase().ends_with(".json")
                || target.contains('/')
                || target.contains('\\')));

    if missing_file_intended {
        return Err(format!("File not found: '{target}'"));
    }

    Ok(ResolvedInput {
        content: target.to_string(),
        kind: InputKind::Literal,
        source_label: "inline text".to_string(),
    })
}

fn execute_command(command: Commands, raw: bool, use_color: bool) -> Result<(), String> {
    match command {
        Commands::Crypto(crypto_args) => match crypto_args.action {
            CryptoSubcommand::Hash { algorithm, text } => {
                let input = resolve_arg_or_stdin(&text)?;
                let (algo_label, digest) = match algorithm {
                    HashAlgorithm::Sha256 => ("SHA-256", crypto::hash_sha256(&input)),
                    HashAlgorithm::Sha512 => ("SHA-512", crypto::hash_sha512(&input)),
                    HashAlgorithm::Md5 => ("MD5", crypto::hash_md5(&input)),
                };
                if raw {
                    println!("{digest}");
                } else {
                    ui::print_open_panel(
                        &format!("{algo_label} Hash"),
                        &[
                            (
                                "Input",
                                format!("{} bytes", input.len()).dimmed().to_string(),
                            ),
                            ("Digest", digest.bright_green().bold().to_string()),
                        ],
                    );
                }
            }
            CryptoSubcommand::B64Encode { text } => {
                let input = resolve_arg_or_stdin(&text)?;
                let encoded = crypto::base64_encode(&input);
                if raw {
                    println!("{encoded}");
                } else {
                    ui::print_open_panel(
                        "Base64 Encode",
                        &[
                            (
                                "Input",
                                format!("{} bytes", input.len()).dimmed().to_string(),
                            ),
                            ("Output", encoded.bright_green().bold().to_string()),
                        ],
                    );
                }
            }
            CryptoSubcommand::B64Decode { encoded } => {
                let input = resolve_arg_or_stdin(&encoded)?;
                let decoded = crypto::base64_decode(&input)?;
                if raw {
                    println!("{decoded}");
                } else {
                    ui::print_open_panel(
                        "Base64 Decode",
                        &[("Decoded", decoded.bright_green().bold().to_string())],
                    );
                }
            }
            CryptoSubcommand::Uuid => {
                let id = crypto::generate_uuid();
                if raw {
                    println!("{id}");
                } else {
                    ui::print_open_panel(
                        "UUIDv4 Generator",
                        &[("UUID", id.bright_yellow().bold().to_string())],
                    );
                }
            }
            CryptoSubcommand::Pass {
                length,
                symbols,
                no_symbols,
            } => {
                let include_symbols = symbols && !no_symbols;
                let password = crypto::try_generate_password(length, include_symbols)?;
                if raw {
                    println!("{password}");
                } else {
                    let strength = if length >= 16 && include_symbols {
                        ui::format_badge("STRONG", ui::BadgeKind::Success)
                    } else if length >= 10 {
                        ui::format_badge("MODERATE", ui::BadgeKind::Info)
                    } else {
                        ui::format_badge("SHORT", ui::BadgeKind::Warn)
                    };
                    let charset = if include_symbols {
                        "A-Z, a-z, 0-9, symbols"
                    } else {
                        "A-Z, a-z, 0-9 (alphanumeric)"
                    };
                    ui::print_open_panel(
                        "Password Generator",
                        &[
                            ("Length", format!("{length} chars   {strength}")),
                            ("Charset", charset.dimmed().to_string()),
                            ("Password", password.bright_green().bold().to_string()),
                        ],
                    );
                }
            }
            CryptoSubcommand::Checksum { algorithm, path } => {
                let algo = match algorithm {
                    HashAlgorithm::Sha256 => crypto::ChecksumAlgorithm::Sha256,
                    HashAlgorithm::Sha512 => crypto::ChecksumAlgorithm::Sha512,
                    HashAlgorithm::Md5 => crypto::ChecksumAlgorithm::Md5,
                };
                let res = crypto::compute_file_checksum(&path, algo)
                    .map_err(|e| format!("Failed to compute checksum for '{path}': {e}"))?;
                if raw {
                    println!("{}", res.digest);
                } else {
                    ui::print_open_panel(
                        "File Checksum",
                        &[
                            ("File", path),
                            (
                                "Algorithm",
                                res.algorithm.bright_yellow().bold().to_string(),
                            ),
                            (
                                "Size",
                                format!(
                                    "{} ({} bytes)",
                                    ui::format_bytes(res.bytes_read),
                                    ui::format_number_commas(res.bytes_read)
                                )
                                .dimmed()
                                .to_string(),
                            ),
                            ("Checksum", res.digest.bright_green().bold().to_string()),
                        ],
                    );
                }
            }
        },
        Commands::Encode(encode_args) => match encode_args.action {
            EncodeSubcommand::UrlEncode { text } => {
                let input = resolve_arg_or_stdin(&text)?;
                let encoded = codec::url_encode(&input);
                if raw {
                    println!("{encoded}");
                } else {
                    ui::print_open_panel(
                        "URL Encode",
                        &[("Encoded", encoded.bright_green().bold().to_string())],
                    );
                }
            }
            EncodeSubcommand::UrlDecode { encoded } => {
                let input = resolve_arg_or_stdin(&encoded)?;
                let decoded = codec::url_decode(&input)?;
                if raw {
                    println!("{decoded}");
                } else {
                    ui::print_open_panel(
                        "URL Decode",
                        &[("Decoded", decoded.bright_green().bold().to_string())],
                    );
                }
            }
            EncodeSubcommand::HexEncode { text } => {
                let input = resolve_arg_or_stdin(&text)?;
                let hex = codec::hex_encode(input.as_bytes());
                if raw {
                    println!("{hex}");
                } else {
                    ui::print_open_panel(
                        "Hex Encode",
                        &[("Hex", hex.bright_green().bold().to_string())],
                    );
                }
            }
            EncodeSubcommand::HexDecode { hex } => {
                let input = resolve_arg_or_stdin(&hex)?;
                let decoded = codec::hex_decode_to_string(&input)?;
                if raw {
                    println!("{decoded}");
                } else {
                    ui::print_open_panel(
                        "Hex Decode",
                        &[("Decoded", decoded.bright_green().bold().to_string())],
                    );
                }
            }
            EncodeSubcommand::Jwt { token } => {
                let input = resolve_arg_or_stdin(&token)?;
                let inspection = codec::inspect_jwt(&input)?;
                let header_str = serde_json::to_string_pretty(&inspection.header)
                    .unwrap_or_else(|_| "{}".to_string());
                let payload_str = serde_json::to_string_pretty(&inspection.payload)
                    .unwrap_or_else(|_| "{}".to_string());

                if raw {
                    println!("{payload_str}");
                    return Ok(());
                }

                let now = time_tool::current_epoch_secs() as i64;
                let mut rows: Vec<(&str, String)> = Vec::new();
                if let Some(alg) = inspection.header.get("alg").and_then(|v| v.as_str()) {
                    rows.push(("Algorithm", alg.bright_yellow().bold().to_string()));
                }
                if let Some(iat) = inspection.issued_at {
                    rows.push(("Issued At", time_tool::epoch_to_iso8601(iat)));
                }
                if let Some(nbf) = inspection.not_before {
                    rows.push(("Not Before", time_tool::epoch_to_iso8601(nbf)));
                }
                if let Some(exp) = inspection.expires_at {
                    let iso = time_tool::epoch_to_iso8601(exp);
                    let status = if exp <= now {
                        let ago = ui::format_duration_human((now - exp) as u64);
                        format!(
                            "{iso}  {}",
                            ui::format_badge(&format!("EXPIRED {ago} AGO"), ui::BadgeKind::Error)
                        )
                    } else {
                        let left = ui::format_duration_human((exp - now) as u64);
                        format!(
                            "{iso}  {}",
                            ui::format_badge(
                                &format!("VALID ({left} left)"),
                                ui::BadgeKind::Success
                            )
                        )
                    };
                    rows.push(("Expires At", status));
                }
                if !inspection.signature.is_empty() {
                    let sig_display = if inspection.signature.len() > 36 {
                        format!("{}...", &inspection.signature[..36])
                    } else {
                        inspection.signature
                    };
                    rows.push((
                        "Signature",
                        format!(
                            "{} {}",
                            sig_display.magenta(),
                            "(not verified offline)".dimmed()
                        ),
                    ));
                } else {
                    rows.push(("Signature", "None (unsigned token)".dimmed().to_string()));
                }

                ui::print_open_panel("JWT Token Inspector", &rows);
                ui::print_section_divider("Header");
                println!("{header_str}");
                ui::print_section_divider("Payload Claims");
                println!("{payload_str}");
            }
        },
        Commands::Time(time_args) => match time_args.action {
            TimeSubcommand::Now => {
                let secs = time_tool::current_epoch_secs();
                let millis = time_tool::current_epoch_millis();
                let iso = time_tool::epoch_to_iso8601(secs as i64);
                if raw {
                    println!("{iso}");
                } else {
                    ui::print_kv_box(
                        "Current UTC Time",
                        &[
                            ("Epoch (s)", secs.to_string().bright_green().to_string()),
                            ("Epoch (ms)", millis.to_string().green().to_string()),
                            ("ISO 8601", iso.bright_yellow().bold().to_string()),
                        ],
                    );
                }
            }
            TimeSubcommand::ToIso { epoch } => {
                let (iso, detected_millis) = time_tool::epoch_to_iso8601_with_meta(epoch);
                if raw {
                    println!("{iso}");
                } else {
                    let input_desc = if detected_millis {
                        format!("{epoch} {}", "(auto-detected milliseconds)".dimmed())
                    } else {
                        format!("{epoch} {}", "(seconds)".dimmed())
                    };
                    ui::print_kv_box(
                        "Epoch to ISO 8601 UTC",
                        &[
                            ("Input", input_desc),
                            ("UTC", iso.bright_yellow().bold().to_string()),
                        ],
                    );
                }
            }
            TimeSubcommand::ToEpoch { iso } => {
                let input = resolve_arg_or_stdin(&iso)?;
                let epoch = time_tool::iso8601_to_epoch(&input)?;
                if raw {
                    println!("{epoch}");
                } else {
                    ui::print_kv_box(
                        "ISO 8601 to Unix Epoch",
                        &[
                            ("Input", input.trim().to_string()),
                            (
                                "Epoch (s)",
                                epoch.to_string().bright_green().bold().to_string(),
                            ),
                            (
                                "Epoch (ms)",
                                (epoch as i128 * 1000).to_string().green().to_string(),
                            ),
                        ],
                    );
                }
            }
        },
        Commands::Text(text_args) => match text_args.action {
            TextSubcommand::Count { target } => {
                let resolved = resolve_input_or_file(&target, false)?;
                let stats = text_tool::analyze_text(&resolved.content);
                if raw {
                    println!(
                        "{} {} {} {}",
                        stats.lines, stats.words, stats.characters, stats.bytes
                    );
                } else {
                    let source_kind = match resolved.kind {
                        InputKind::Stdin => "stdin".to_string(),
                        InputKind::File => format!("file ({})", resolved.source_label),
                        InputKind::Literal => "inline string".to_string(),
                    };
                    ui::print_kv_box(
                        "Text Summary",
                        &[
                            ("Source", source_kind.dimmed().to_string()),
                            (
                                "Lines",
                                ui::format_number_commas(stats.lines as u64)
                                    .bright_green()
                                    .to_string(),
                            ),
                            (
                                "Words",
                                ui::format_number_commas(stats.words as u64)
                                    .bright_green()
                                    .to_string(),
                            ),
                            (
                                "Characters",
                                ui::format_number_commas(stats.characters as u64)
                                    .bright_green()
                                    .to_string(),
                            ),
                            (
                                "Byte Size",
                                format!(
                                    "{} ({} bytes)",
                                    ui::format_bytes(stats.bytes as u64),
                                    ui::format_number_commas(stats.bytes as u64)
                                )
                                .green()
                                .to_string(),
                            ),
                        ],
                    );
                }
            }
            TextSubcommand::Case { style, text } => {
                let input = resolve_arg_or_stdin(&text)?;
                let (style_name, style_enum) = match style {
                    TextCaseStyle::Snake => ("snake_case", text_tool::CaseStyle::Snake),
                    TextCaseStyle::Camel => ("camelCase", text_tool::CaseStyle::Camel),
                    TextCaseStyle::Kebab => ("kebab-case", text_tool::CaseStyle::Kebab),
                    TextCaseStyle::Upper => ("UPPER_CASE", text_tool::CaseStyle::Upper),
                    TextCaseStyle::Lower => ("lower case", text_tool::CaseStyle::Lower),
                    TextCaseStyle::Title => ("Title Case", text_tool::CaseStyle::Title),
                };
                let transformed = text_tool::convert_case(&input, style_enum);
                if raw {
                    println!("{transformed}");
                } else {
                    ui::print_open_panel(
                        "Case Transform",
                        &[
                            ("Style", style_name.bright_yellow().to_string()),
                            ("Result", transformed.bright_green().bold().to_string()),
                        ],
                    );
                }
            }
            TextSubcommand::Slug { target } => {
                let resolved = resolve_input_or_file(&target, false)?;
                let slug = text_tool::slugify(&resolved.content);
                if raw {
                    println!("{slug}");
                } else {
                    ui::print_open_panel(
                        "URL Slugify",
                        &[("Slug", slug.bright_green().bold().to_string())],
                    );
                }
            }
            TextSubcommand::Reverse { target } => {
                let resolved = resolve_input_or_file(&target, false)?;
                let reversed = text_tool::reverse_text(&resolved.content);
                if raw {
                    println!("{reversed}");
                } else {
                    ui::print_open_panel(
                        "Reverse Text",
                        &[("Reversed", reversed.bright_green().bold().to_string())],
                    );
                }
            }
        },
        Commands::Qr(qr_args) => {
            let input = resolve_arg_or_stdin(&qr_args.content)?;
            let ec_level: qr::QrEcLevel = qr_args.ec.into();
            let opts = qr::QrRenderOptions {
                ec_level,
                compact: !qr_args.full,
                invert: qr_args.invert,
                use_color: use_color && !qr_args.plain && !raw,
                no_universal_links: qr_args.no_universal_links,
            };
            let result = qr::render_qr_code(&input, &opts)?;

            if let Some(svg_path) = &qr_args.svg {
                let svg = qr::generate_qr_svg(&result.effective_content, ec_level, qr_args.invert)?;
                fs::write(svg_path, svg)
                    .map_err(|e| format!("Failed to write SVG file '{svg_path}': {e}"))?;
            }

            if raw {
                print!("{}", result.terminal_art);
                return Ok(());
            }

            let mut rows = vec![
                (
                    "Payload",
                    result.effective_content.bright_white().to_string(),
                ),
                (
                    "Matrix",
                    format!(
                        "Version {} ({}x{} modules)  •  EC {}",
                        result.version_number,
                        result.module_width,
                        result.module_width,
                        ec_level.label()
                    )
                    .dimmed()
                    .to_string(),
                ),
            ];
            if let Some(svg_path) = &qr_args.svg {
                rows.push(("SVG Saved", svg_path.bright_green().to_string()));
            }
            ui::print_open_panel("QR Code", &rows);
            println!();
            print!("{}", result.terminal_art);

            if let Some(advisory) = result.advisory {
                println!();
                ui::print_tip(&advisory);
            }
        }
        Commands::Json(json_args) => match json_args.action {
            JsonSubcommand::Format { target } => {
                let resolved = resolve_input_or_file(&target, true)?;
                let formatted = json_tool::format_json(&resolved.content)?;
                println!("{formatted}");
            }
            JsonSubcommand::Minify { target } => {
                let resolved = resolve_input_or_file(&target, true)?;
                let minified = json_tool::minify_json(&resolved.content)?;
                println!("{minified}");
            }
            JsonSubcommand::Validate { target } => {
                let resolved = resolve_input_or_file(&target, true)?;
                let status = json_tool::validate_json(&resolved.content)?;
                if raw {
                    println!("{status}");
                } else {
                    ui::print_open_panel(
                        "JSON Validation",
                        &[
                            ("Source", resolved.source_label.dimmed().to_string()),
                            (
                                "Status",
                                format!(
                                    "{}  {status}",
                                    ui::format_badge("VALID", ui::BadgeKind::Success)
                                ),
                            ),
                        ],
                    );
                }
            }
        },
        Commands::Sys(sys_args) => {
            let sys = telemetry::collect_system_overview_filtered(sys_args.all);
            if raw {
                println!(
                    "host={} os=\"{} {}\" cpu=\"{}\" cores={} mem_used_mb={} mem_total_mb={}",
                    sys.host_name,
                    sys.os_name,
                    sys.os_version,
                    sys.cpu_brand,
                    sys.cpu_count,
                    sys.used_memory_mb,
                    sys.total_memory_mb
                );
                for d in &sys.disks {
                    println!(
                        "disk mount={} fs={} avail_gb={:.1} total_gb={:.1}",
                        d.mount_point, d.file_system, d.available_space_gb, d.total_space_gb
                    );
                }
                return Ok(());
            }

            let mem_ratio = if sys.total_memory_mb > 0 {
                sys.used_memory_mb as f64 / sys.total_memory_mb as f64
            } else {
                0.0
            };
            let mut sys_rows = vec![
                ("Host", sys.host_name.bright_white().bold().to_string()),
                ("OS", format!("{} {}", sys.os_name, sys.os_version)),
                (
                    "CPU",
                    format!("{} ({} cores)", sys.cpu_brand, sys.cpu_count),
                ),
                (
                    "Memory",
                    format!(
                        "{}  {} / {} MB ({} MB free)",
                        ui::progress_bar(mem_ratio, 18),
                        ui::format_number_commas(sys.used_memory_mb).bright_white(),
                        ui::format_number_commas(sys.total_memory_mb),
                        ui::format_number_commas(sys.free_memory_mb).dimmed()
                    ),
                ),
            ];
            if sys.total_swap_mb > 0 {
                let swap_ratio = sys.used_swap_mb as f64 / sys.total_swap_mb as f64;
                sys_rows.push((
                    "Swap",
                    format!(
                        "{}  {} / {} MB",
                        ui::progress_bar(swap_ratio, 18),
                        ui::format_number_commas(sys.used_swap_mb),
                        ui::format_number_commas(sys.total_swap_mb)
                    ),
                ));
            }
            ui::print_kv_box("System Overview", &sys_rows);

            if !sys.disks.is_empty() {
                println!();
                let mount_w = sys
                    .disks
                    .iter()
                    .map(|d| d.mount_point.len())
                    .max()
                    .unwrap_or(5)
                    .clamp(5, 24);
                let fs_w = sys
                    .disks
                    .iter()
                    .map(|d| d.file_system.len())
                    .max()
                    .unwrap_or(4)
                    .max(4);

                let mut disk_lines = vec![
                    format!(
                        "{:<mount_w$}  {:<fs_w$}  {:<26}  {}",
                        "Mount".dimmed().bold(),
                        "FS".dimmed().bold(),
                        "Usage".dimmed().bold(),
                        "Free / Total".dimmed().bold()
                    ),
                    format!(
                        "{}  {}  {}  {}",
                        "─".repeat(mount_w).dimmed(),
                        "─".repeat(fs_w).dimmed(),
                        "─".repeat(26).dimmed(),
                        "─".repeat(17).dimmed()
                    ),
                ];

                for d in &sys.disks {
                    let used_gb = (d.total_space_gb - d.available_space_gb).max(0.0);
                    let ratio = if d.total_space_gb > 0.0 {
                        used_gb / d.total_space_gb
                    } else {
                        0.0
                    };
                    let mount_display = if d.mount_point.len() > mount_w {
                        format!("{}...", &d.mount_point[..mount_w.saturating_sub(3)])
                    } else {
                        d.mount_point.clone()
                    };
                    let virtual_tag = if d.is_virtual {
                        " [virtual]".dimmed().to_string()
                    } else {
                        String::new()
                    };
                    disk_lines.push(format!(
                        "{:<mount_w$}  {:<fs_w$}  {}  {:>5.1} / {:>5.1} GB{}",
                        mount_display.bright_white(),
                        d.file_system.dimmed(),
                        ui::progress_bar(ratio, 18),
                        d.available_space_gb,
                        d.total_space_gb,
                        virtual_tag
                    ));
                }
                ui::print_boxed_lines("Mounted Storage", &disk_lines);
            }
        }
        Commands::Net(net_args) => match net_args.action {
            NetSubcommand::Interfaces => {
                let interfaces = telemetry::collect_network_interfaces();
                if raw {
                    for iface in interfaces {
                        println!(
                            "{} rx={} tx={}",
                            iface.name, iface.received_bytes, iface.transmitted_bytes
                        );
                    }
                    return Ok(());
                }

                let name_w = interfaces
                    .iter()
                    .map(|i| i.name.len())
                    .max()
                    .unwrap_or(9)
                    .max(9);
                let mut lines = vec![
                    format!(
                        "{:<name_w$}  {:>12}  {:>12}",
                        "Interface".dimmed().bold(),
                        "Received".dimmed().bold(),
                        "Transmitted".dimmed().bold()
                    ),
                    format!(
                        "{}  {}  {}",
                        "─".repeat(name_w).dimmed(),
                        "─".repeat(12).dimmed(),
                        "─".repeat(12).dimmed()
                    ),
                ];
                for iface in &interfaces {
                    let active = iface.received_bytes > 0 || iface.transmitted_bytes > 0;
                    let name_styled = if active {
                        format!("{:<name_w$}", iface.name).bright_white().bold()
                    } else {
                        format!("{:<name_w$}", iface.name).dimmed()
                    };
                    lines.push(format!(
                        "{}  {:>12}  {:>12}",
                        name_styled,
                        ui::format_bytes(iface.received_bytes).green(),
                        ui::format_bytes(iface.transmitted_bytes).cyan()
                    ));
                }
                ui::print_boxed_lines("Network Interfaces", &lines);
            }
            NetSubcommand::Ping {
                host,
                port,
                timeout,
            } => {
                let res =
                    telemetry::probe_tcp_detailed(&host, port, Duration::from_millis(timeout))?;
                let ms = res.tcp_latency.as_secs_f64() * 1000.0;
                if raw {
                    println!("{ms:.2}");
                } else {
                    let quality = if ms < 50.0 {
                        ui::format_badge("FAST", ui::BadgeKind::Success)
                    } else if ms < 150.0 {
                        ui::format_badge("MODERATE", ui::BadgeKind::Warn)
                    } else {
                        ui::format_badge("SLOW", ui::BadgeKind::Error)
                    };
                    ui::print_kv_box(
                        "TCP Latency Probe",
                        &[
                            ("Host", res.host.bright_white().bold().to_string()),
                            ("Endpoint", res.endpoint.to_string().cyan().to_string()),
                            (
                                "DNS Lookup",
                                format!("{:.2} ms", res.dns_duration.as_secs_f64() * 1000.0)
                                    .dimmed()
                                    .to_string(),
                            ),
                            (
                                "TCP Handshake",
                                format!("{ms:.2} ms   {quality}")
                                    .bright_green()
                                    .bold()
                                    .to_string(),
                            ),
                        ],
                    );
                }
            }
            NetSubcommand::Dns { host } => {
                let res = telemetry::resolve_dns(&host)?;
                if raw {
                    for ip in res.addresses {
                        println!("{ip}");
                    }
                } else {
                    let mut rows = vec![
                        ("Host", res.host.bright_white().bold().to_string()),
                        (
                            "Lookup Time",
                            format!("{:.2} ms", res.lookup_duration.as_secs_f64() * 1000.0)
                                .green()
                                .to_string(),
                        ),
                    ];
                    for (idx, ip) in res.addresses.iter().enumerate() {
                        let label = if idx == 0 { "Addresses" } else { "" };
                        rows.push((label, ip.bright_cyan().to_string()));
                    }
                    ui::print_kv_box("DNS Resolution", &rows);
                }
            }
        },
        Commands::Interactive => run_interactive(raw, use_color),
    }

    Ok(())
}

fn prompt_line(prompt: &str) -> Option<String> {
    print!("{prompt}");
    let _ = io::stdout().flush();
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(0) => None,
        Ok(_) => Some(input.trim().to_string()),
        Err(_) => None,
    }
}

fn split_command_args(line: &str) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut escape = false;
    let mut in_token = false;

    for ch in line.chars() {
        if escape {
            current.push(ch);
            escape = false;
            in_token = true;
            continue;
        }
        match quote {
            Some(q) => {
                if ch == q {
                    quote = None;
                } else if ch == '\\' && q == '"' {
                    escape = true;
                } else {
                    current.push(ch);
                }
                in_token = true;
            }
            None => {
                if ch.is_whitespace() {
                    if in_token {
                        args.push(std::mem::take(&mut current));
                        in_token = false;
                    }
                } else if ch == '\'' || ch == '"' {
                    quote = Some(ch);
                    in_token = true;
                } else if ch == '\\' {
                    escape = true;
                    in_token = true;
                } else {
                    current.push(ch);
                    in_token = true;
                }
            }
        }
    }

    if quote.is_some() {
        return Err("Unclosed quote in command".to_string());
    }
    if in_token {
        args.push(current);
    }
    Ok(args)
}

fn print_interactive_menu() {
    let items = [
        format!(
            "  {}  {}  {}",
            "1.".bright_cyan().bold(),
            "Crypto ".bright_white().bold(),
            "Hashes, checksums, Base64, UUIDv4, passwords".dimmed()
        ),
        format!(
            "  {}  {}  {}",
            "2.".bright_cyan().bold(),
            "Encode ".bright_white().bold(),
            "URL codec, Hex codec, JWT token inspector".dimmed()
        ),
        format!(
            "  {}  {}  {}",
            "3.".bright_cyan().bold(),
            "Time   ".bright_white().bold(),
            "Current UTC time, Epoch <-> ISO 8601".dimmed()
        ),
        format!(
            "  {}  {}  {}",
            "4.".bright_cyan().bold(),
            "Text   ".bright_white().bold(),
            "Metrics, case styles, slugs, text reversal".dimmed()
        ),
        format!(
            "  {}  {}  {}",
            "5.".bright_cyan().bold(),
            "QR Code".bright_white().bold(),
            "Scannable terminal QR codes and SVG export".dimmed()
        ),
        format!(
            "  {}  {}  {}",
            "6.".bright_cyan().bold(),
            "JSON   ".bright_white().bold(),
            "Format, minify, and validate JSON payloads".dimmed()
        ),
        format!(
            "  {}  {}  {}",
            "7.".bright_cyan().bold(),
            "System ".bright_white().bold(),
            "CPU, memory, and mounted storage usage".dimmed()
        ),
        format!(
            "  {}  {}  {}",
            "8.".bright_cyan().bold(),
            "Network".bright_white().bold(),
            "Interfaces, TCP latency ping, DNS lookup".dimmed()
        ),
    ];
    ui::print_boxed_lines("Tools Menu", &items);
}

fn run_guided_choice(choice: &str, raw: bool, use_color: bool) -> Result<(), String> {
    match choice {
        "1" => {
            println!(
                "  {} 1=SHA-256  2=Base64 Encode  3=Base64 Decode  4=UUIDv4  5=Password",
                "Action:".dimmed()
            );
            let act = prompt_line("  Select [1-5]: ").unwrap_or_default();
            match act.as_str() {
                "1" => {
                    let text = prompt_line("  Text to hash: ").unwrap_or_default();
                    execute_command(
                        Commands::Crypto(CryptoArgs {
                            action: CryptoSubcommand::Hash {
                                algorithm: HashAlgorithm::Sha256,
                                text,
                            },
                        }),
                        raw,
                        use_color,
                    )
                }
                "2" => {
                    let text = prompt_line("  Text to encode: ").unwrap_or_default();
                    execute_command(
                        Commands::Crypto(CryptoArgs {
                            action: CryptoSubcommand::B64Encode { text },
                        }),
                        raw,
                        use_color,
                    )
                }
                "3" => {
                    let encoded = prompt_line("  Base64 string: ").unwrap_or_default();
                    execute_command(
                        Commands::Crypto(CryptoArgs {
                            action: CryptoSubcommand::B64Decode { encoded },
                        }),
                        raw,
                        use_color,
                    )
                }
                "4" => execute_command(
                    Commands::Crypto(CryptoArgs {
                        action: CryptoSubcommand::Uuid,
                    }),
                    raw,
                    use_color,
                ),
                "5" => {
                    let len_str = prompt_line("  Length [16]: ").unwrap_or_default();
                    let length = if len_str.is_empty() {
                        16
                    } else {
                        len_str
                            .parse()
                            .map_err(|_| "Invalid number for length".to_string())?
                    };
                    let sym_str = prompt_line("  Include symbols? [Y/n]: ").unwrap_or_default();
                    let symbols = !matches!(sym_str.to_ascii_lowercase().as_str(), "n" | "no");
                    execute_command(
                        Commands::Crypto(CryptoArgs {
                            action: CryptoSubcommand::Pass {
                                length,
                                symbols,
                                no_symbols: !symbols,
                            },
                        }),
                        raw,
                        use_color,
                    )
                }
                _ => Err("Unknown crypto selection".to_string()),
            }
        }
        "2" => {
            println!(
                "  {} 1=URL Encode  2=URL Decode  3=Hex Encode  4=Hex Decode  5=Inspect JWT",
                "Action:".dimmed()
            );
            let act = prompt_line("  Select [1-5]: ").unwrap_or_default();
            match act.as_str() {
                "1" => {
                    let text = prompt_line("  Text: ").unwrap_or_default();
                    execute_command(
                        Commands::Encode(EncodeArgs {
                            action: EncodeSubcommand::UrlEncode { text },
                        }),
                        raw,
                        use_color,
                    )
                }
                "2" => {
                    let encoded = prompt_line("  URL-encoded text: ").unwrap_or_default();
                    execute_command(
                        Commands::Encode(EncodeArgs {
                            action: EncodeSubcommand::UrlDecode { encoded },
                        }),
                        raw,
                        use_color,
                    )
                }
                "3" => {
                    let text = prompt_line("  Text: ").unwrap_or_default();
                    execute_command(
                        Commands::Encode(EncodeArgs {
                            action: EncodeSubcommand::HexEncode { text },
                        }),
                        raw,
                        use_color,
                    )
                }
                "4" => {
                    let hex = prompt_line("  Hex string: ").unwrap_or_default();
                    execute_command(
                        Commands::Encode(EncodeArgs {
                            action: EncodeSubcommand::HexDecode { hex },
                        }),
                        raw,
                        use_color,
                    )
                }
                "5" => {
                    let token = prompt_line("  JWT token: ").unwrap_or_default();
                    execute_command(
                        Commands::Encode(EncodeArgs {
                            action: EncodeSubcommand::Jwt { token },
                        }),
                        raw,
                        use_color,
                    )
                }
                _ => Err("Unknown encode selection".to_string()),
            }
        }
        "3" => {
            println!(
                "  {} 1=Current Time  2=Epoch -> ISO  3=ISO -> Epoch",
                "Action:".dimmed()
            );
            let act = prompt_line("  Select [1-3]: ").unwrap_or_default();
            match act.as_str() {
                "1" => execute_command(
                    Commands::Time(TimeArgs {
                        action: TimeSubcommand::Now,
                    }),
                    raw,
                    use_color,
                ),
                "2" => {
                    let epoch_str = prompt_line("  Epoch (seconds or ms): ").unwrap_or_default();
                    let epoch = epoch_str
                        .parse()
                        .map_err(|_| "Invalid epoch integer".to_string())?;
                    execute_command(
                        Commands::Time(TimeArgs {
                            action: TimeSubcommand::ToIso { epoch },
                        }),
                        raw,
                        use_color,
                    )
                }
                "3" => {
                    let iso = prompt_line("  ISO 8601 UTC string: ").unwrap_or_default();
                    execute_command(
                        Commands::Time(TimeArgs {
                            action: TimeSubcommand::ToEpoch { iso },
                        }),
                        raw,
                        use_color,
                    )
                }
                _ => Err("Unknown time selection".to_string()),
            }
        }
        "4" => {
            println!(
                "  {} 1=Count Stats  2=Case Convert  3=Slugify  4=Reverse",
                "Action:".dimmed()
            );
            let act = prompt_line("  Select [1-4]: ").unwrap_or_default();
            let target = prompt_line("  Text or file path: ").unwrap_or_default();
            match act.as_str() {
                "1" => execute_command(
                    Commands::Text(TextArgs {
                        action: TextSubcommand::Count { target },
                    }),
                    raw,
                    use_color,
                ),
                "2" => {
                    println!(
                        "  {} snake | camel | kebab | upper | lower | title",
                        "Style:".dimmed()
                    );
                    let style_str = prompt_line("  Style [snake]: ").unwrap_or_default();
                    let style = match style_str.to_ascii_lowercase().as_str() {
                        "" | "snake" => TextCaseStyle::Snake,
                        "camel" => TextCaseStyle::Camel,
                        "kebab" => TextCaseStyle::Kebab,
                        "upper" => TextCaseStyle::Upper,
                        "lower" => TextCaseStyle::Lower,
                        "title" => TextCaseStyle::Title,
                        _ => return Err(format!("Unknown style '{style_str}'")),
                    };
                    execute_command(
                        Commands::Text(TextArgs {
                            action: TextSubcommand::Case {
                                style,
                                text: target,
                            },
                        }),
                        raw,
                        use_color,
                    )
                }
                "3" => execute_command(
                    Commands::Text(TextArgs {
                        action: TextSubcommand::Slug { target },
                    }),
                    raw,
                    use_color,
                ),
                "4" => execute_command(
                    Commands::Text(TextArgs {
                        action: TextSubcommand::Reverse { target },
                    }),
                    raw,
                    use_color,
                ),
                _ => Err("Unknown text selection".to_string()),
            }
        }
        "5" => {
            let content = prompt_line("  URL or text to encode: ").unwrap_or_default();
            let bypass = if content.starts_with("http://") || content.starts_with("https://") {
                let ans =
                    prompt_line("  Bypass iOS Universal Links (#no_universal_links)? [y/N]: ")
                        .unwrap_or_default();
                matches!(ans.to_ascii_lowercase().as_str(), "y" | "yes")
            } else {
                false
            };
            execute_command(
                Commands::Qr(QrArgs {
                    content,
                    ec: QrEcArg::M,
                    full: false,
                    invert: false,
                    plain: false,
                    svg: None,
                    no_universal_links: bypass,
                }),
                raw,
                use_color,
            )
        }
        "6" => {
            println!("  {} 1=Format  2=Minify  3=Validate", "Action:".dimmed());
            let act = prompt_line("  Select [1-3]: ").unwrap_or_default();
            let target = prompt_line("  JSON string or file path: ").unwrap_or_default();
            match act.as_str() {
                "1" => execute_command(
                    Commands::Json(JsonArgs {
                        action: JsonSubcommand::Format { target },
                    }),
                    raw,
                    use_color,
                ),
                "2" => execute_command(
                    Commands::Json(JsonArgs {
                        action: JsonSubcommand::Minify { target },
                    }),
                    raw,
                    use_color,
                ),
                "3" => execute_command(
                    Commands::Json(JsonArgs {
                        action: JsonSubcommand::Validate { target },
                    }),
                    raw,
                    use_color,
                ),
                _ => Err("Unknown JSON selection".to_string()),
            }
        }
        "7" => execute_command(Commands::Sys(SysArgs { all: false }), raw, use_color),
        "8" => {
            println!(
                "  {} 1=Interfaces  2=TCP Ping  3=DNS Lookup",
                "Action:".dimmed()
            );
            let act = prompt_line("  Select [1-3]: ").unwrap_or_default();
            match act.as_str() {
                "1" => execute_command(
                    Commands::Net(NetArgs {
                        action: NetSubcommand::Interfaces,
                    }),
                    raw,
                    use_color,
                ),
                "2" => {
                    let host = prompt_line("  Host (e.g. github.com): ").unwrap_or_default();
                    let port_str = prompt_line("  Port [443]: ").unwrap_or_default();
                    let port = if port_str.is_empty() {
                        443
                    } else {
                        port_str
                            .parse()
                            .map_err(|_| "Invalid port number".to_string())?
                    };
                    execute_command(
                        Commands::Net(NetArgs {
                            action: NetSubcommand::Ping {
                                host,
                                port,
                                timeout: 3000,
                            },
                        }),
                        raw,
                        use_color,
                    )
                }
                "3" => {
                    let host = prompt_line("  Hostname: ").unwrap_or_default();
                    execute_command(
                        Commands::Net(NetArgs {
                            action: NetSubcommand::Dns { host },
                        }),
                        raw,
                        use_color,
                    )
                }
                _ => Err("Unknown network selection".to_string()),
            }
        }
        _ => Err(format!("Unknown menu option '{choice}'")),
    }
}

fn run_interactive(raw: bool, use_color: bool) {
    ui::print_banner(env!("CARGO_PKG_VERSION"));
    println!();
    print_interactive_menu();
    println!(
        "\n{}",
        "Choose [1-8], type any command (e.g. \"qr https://github.com\"), \"menu\", \"help\", or \"q\" to quit."
            .dimmed()
    );

    while let Some(line) = prompt_line(&format!("\n{} ", "omnitool>".bright_cyan().bold())) {
        if line.is_empty() {
            continue;
        }
        match line.to_ascii_lowercase().as_str() {
            "q" | "quit" | "exit" => break,
            "menu" | "m" => {
                print_interactive_menu();
                continue;
            }
            "help" | "?" => {
                if let Err(e) = Cli::try_parse_from(["omnitool", "--help"]) {
                    let _ = e.print();
                }
                continue;
            }
            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" => {
                if let Err(err) = run_guided_choice(&line, raw, use_color) {
                    ui::print_error(&err);
                }
                continue;
            }
            _ => {}
        }

        match split_command_args(&line) {
            Ok(parts) => {
                let mut argv = vec!["omnitool".to_string()];
                argv.extend(parts);
                match Cli::try_parse_from(&argv) {
                    Ok(parsed) => {
                        if let Some(cmd) = parsed.command {
                            let cmd_color = use_color && !parsed.no_color;
                            colored::control::set_override(cmd_color);
                            if let Err(err) = execute_command(cmd, parsed.raw || raw, cmd_color) {
                                ui::print_error(&err);
                            }
                            colored::control::set_override(use_color);
                        }
                    }
                    Err(e) => {
                        let _ = e.print();
                    }
                }
            }
            Err(err) => ui::print_error(&err),
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let use_color = ui::should_use_color(cli.no_color);
    colored::control::set_override(use_color);

    match cli.command {
        Some(cmd) => {
            if let Err(err) = execute_command(cmd, cli.raw, use_color) {
                ui::print_error(&err);
                std::process::exit(1);
            }
        }
        None => {
            if io::stdin().is_terminal() && io::stdout().is_terminal() {
                run_interactive(cli.raw, use_color);
            } else {
                ui::print_banner(env!("CARGO_PKG_VERSION"));
                println!();
                print_interactive_menu();
                println!(
                    "\n{}",
                    "Run 'omnitool --help' for all subcommands and flags.".dimmed()
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parses_pass_flags() {
        let cli1 = Cli::try_parse_from(["omnitool", "crypto", "pass", "--symbols", "false"])
            .expect("--symbols false");
        match cli1.command.unwrap() {
            Commands::Crypto(c) => match c.action {
                CryptoSubcommand::Pass {
                    symbols,
                    no_symbols,
                    ..
                } => {
                    assert!(!symbols);
                    assert!(!no_symbols);
                }
                _ => panic!("expected pass"),
            },
            _ => panic!("expected crypto"),
        }

        let cli2 = Cli::try_parse_from(["omnitool", "crypto", "pass", "--no-symbols"])
            .expect("--no-symbols");
        match cli2.command.unwrap() {
            Commands::Crypto(c) => match c.action {
                CryptoSubcommand::Pass { no_symbols, .. } => assert!(no_symbols),
                _ => panic!("expected pass"),
            },
            _ => panic!("expected crypto"),
        }

        let cli3 =
            Cli::try_parse_from(["omnitool", "crypto", "pass", "--symbols"]).expect("--symbols");
        match cli3.command.unwrap() {
            Commands::Crypto(c) => match c.action {
                CryptoSubcommand::Pass { symbols, .. } => assert!(symbols),
                _ => panic!("expected pass"),
            },
            _ => panic!("expected crypto"),
        }
    }

    #[test]
    fn test_cli_parses_negative_epoch() {
        let cli =
            Cli::try_parse_from(["omnitool", "time", "to-iso", "-86400"]).expect("negative epoch");
        match cli.command.unwrap() {
            Commands::Time(t) => match t.action {
                TimeSubcommand::ToIso { epoch } => assert_eq!(epoch, -86400),
                _ => panic!("expected to-iso"),
            },
            _ => panic!("expected time"),
        }
    }

    #[test]
    fn test_split_command_args() {
        let args = split_command_args(r#"qr "hello world" --ec h"#).expect("split");
        assert_eq!(args, vec!["qr", "hello world", "--ec", "h"]);
        assert!(split_command_args(r#"qr "unclosed"#).is_err());
    }

    #[test]
    fn test_resolve_input_or_file_missing_paths() {
        assert!(resolve_input_or_file("./nonexistent_file_12345.txt", false).is_err());
        assert!(resolve_input_or_file("missing_payload.json", true).is_err());
        assert!(resolve_input_or_file("./src", false).is_err());
        assert!(resolve_input_or_file("https://example.com/a/b", false).is_ok());
    }
}
