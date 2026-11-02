mod codec;
mod crypto;
mod json_tool;
mod qr;
mod telemetry;
mod text_tool;
mod time_tool;

use clap::{Args, Parser, Subcommand, ValueEnum};
use colored::*;
use std::fs;
use std::io::{self, Read};

#[derive(Parser)]
#[command(
    name = "omnitool",
    author = "pranov888",
    version = "0.6.0",
    about = "All-in-one developer Swiss-Army knife CLI written in Rust",
    long_about = "Omnitool provides everyday developer utilities (crypto, QR codes, JSON tools, encoders, system info, and network diagnostics) in a single high-performance binary."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Cryptographic helpers (hashes, base64, uuid, password generation)
    Crypto(CryptoArgs),
    /// URL and Hex data encoding and decoding
    Encode(EncodeArgs),
    /// UNIX epoch timestamps and ISO 8601 UTC date-time conversions
    Time(TimeArgs),
    /// Text analysis, counting, and casing transformations
    Text(TextArgs),
    /// Generate scannable QR codes directly in your terminal
    Qr(QrArgs),
    /// JSON data processing (format, minify, validate)
    Json(JsonArgs),
    /// System hardware and operating system telemetry
    Sys,
    /// Network diagnostics and TCP connectivity latency
    Net(NetArgs),
}

#[derive(Args)]
struct CryptoArgs {
    #[command(subcommand)]
    action: CryptoSubcommand,
}

#[derive(Subcommand)]
enum CryptoSubcommand {
    /// Compute cryptographic hash (SHA-256, MD5)
    Hash {
        #[arg(short, long, default_value = "sha256")]
        algorithm: HashAlgorithm,
        /// Input text to hash
        text: String,
    },
    /// Base64 encode an ASCII or UTF-8 string
    B64Encode {
        /// Text to encode
        text: String,
    },
    /// Base64 decode a base64 encoded string
    B64Decode {
        /// Encoded text to decode
        encoded: String,
    },
    /// Generate a random UUIDv4
    Uuid,
    /// Generate a secure random password
    Pass {
        /// Length of password
        #[arg(short, long, default_value_t = 16)]
        length: usize,
        /// Include special symbols
        #[arg(short, long, default_value_t = true)]
        symbols: bool,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum HashAlgorithm {
    Sha256,
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
        /// Text to URL-encode
        text: String,
    },
    /// Decode a percent-encoded URL string
    UrlDecode {
        /// URL encoded text to decode
        encoded: String,
    },
    /// Hex-encode an input string into lowercase hexadecimal
    HexEncode {
        /// Text to hex-encode
        text: String,
    },
    /// Hex-decode a hexadecimal string back to UTF-8
    HexDecode {
        /// Hexadecimal string to decode
        hex: String,
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
        /// Epoch timestamp number
        epoch: i64,
    },
    /// Parse an ISO 8601 date-time string (e.g. 2026-11-02T12:00:00Z) to UNIX epoch
    ToEpoch {
        /// ISO 8601 date string
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
    /// Convert text case style (snake, camel, kebab, upper, lower)
    Case {
        /// Target case formatting style
        #[arg(short, long, value_enum)]
        style: TextCaseStyle,
        /// Input text to transform
        text: String,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum TextCaseStyle {
    Snake,
    Camel,
    Kebab,
    Upper,
    Lower,
}

#[derive(Args)]
struct QrArgs {
    /// URL or text content to encode into a terminal QR code
    content: String,
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
        /// JSON string or file path
        target: String,
    },
    /// Minify JSON string or file
    Minify {
        /// JSON string or file path
        target: String,
    },
    /// Validate JSON syntax and inspect structure
    Validate {
        /// JSON string or file path
        target: String,
    },
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
    },
}

fn resolve_input_or_file(target: &str) -> String {
    if target == "-" {
        let mut buffer = String::new();
        if io::stdin().read_to_string(&mut buffer).is_ok() {
            return buffer;
        }
    }
    if let Ok(content) = fs::read_to_string(target) {
        content
    } else {
        target.to_string()
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Crypto(crypto_args) => match crypto_args.action {
            CryptoSubcommand::Hash { algorithm, text } => {
                let digest = match algorithm {
                    HashAlgorithm::Sha256 => crypto::hash_sha256(&text),
                    HashAlgorithm::Md5 => crypto::hash_md5(&text),
                };
                println!("{} {}", "Hash:".bold().cyan(), digest.green().bold());
            }
            CryptoSubcommand::B64Encode { text } => {
                let encoded = crypto::base64_encode(&text);
                println!("{} {}", "Base64:".bold().cyan(), encoded.green());
            }
            CryptoSubcommand::B64Decode { encoded } => match crypto::base64_decode(&encoded) {
                Ok(decoded) => {
                    println!("{} {}", "Decoded:".bold().cyan(), decoded.green());
                }
                Err(err) => {
                    eprintln!("{} {}", "Error:".bold().red(), err);
                    std::process::exit(1);
                }
            },
            CryptoSubcommand::Uuid => {
                let id = crypto::generate_uuid();
                println!("{} {}", "UUIDv4:".bold().cyan(), id.bright_yellow().bold());
            }
            CryptoSubcommand::Pass { length, symbols } => {
                let password = crypto::generate_password(length, symbols);
                println!(
                    "{} {}",
                    "Generated Password:".bold().cyan(),
                    password.bright_green().bold()
                );
            }
        },
        Commands::Encode(encode_args) => match encode_args.action {
            EncodeSubcommand::UrlEncode { text } => {
                let encoded = codec::url_encode(&text);
                println!("{} {}", "URL Encoded:".bold().cyan(), encoded.green());
            }
            EncodeSubcommand::UrlDecode { encoded } => match codec::url_decode(&encoded) {
                Ok(decoded) => {
                    println!("{} {}", "URL Decoded:".bold().cyan(), decoded.green());
                }
                Err(err) => {
                    eprintln!("{} {}", "Error:".bold().red(), err);
                    std::process::exit(1);
                }
            },
            EncodeSubcommand::HexEncode { text } => {
                let hex = codec::hex_encode(text.as_bytes());
                println!("{} {}", "Hex Encoded:".bold().cyan(), hex.green());
            }
            EncodeSubcommand::HexDecode { hex } => match codec::hex_decode_to_string(&hex) {
                Ok(decoded) => {
                    println!("{} {}", "Hex Decoded:".bold().cyan(), decoded.green());
                }
                Err(err) => {
                    eprintln!("{} {}", "Error:".bold().red(), err);
                    std::process::exit(1);
                }
            },
        },
        Commands::Time(time_args) => match time_args.action {
            TimeSubcommand::Now => {
                let secs = time_tool::current_epoch_secs();
                let millis = time_tool::current_epoch_millis();
                let iso = time_tool::epoch_to_iso8601(secs as i64);
                println!("{}", "Current UTC Time Telemetry".bold().cyan());
                println!("  {:<15} {}", "Epoch (s):".bold(), secs.to_string().green());
                println!(
                    "  {:<15} {}",
                    "Epoch (ms):".bold(),
                    millis.to_string().green()
                );
                println!("  {:<15} {}", "ISO 8601:".bold(), iso.bright_yellow().bold());
            }
            TimeSubcommand::ToIso { epoch } => {
                let iso = time_tool::epoch_to_iso8601(epoch);
                println!("{} {}", "ISO 8601 UTC:".bold().cyan(), iso.green().bold());
            }
            TimeSubcommand::ToEpoch { iso } => match time_tool::iso8601_to_epoch(&iso) {
                Ok(epoch) => {
                    println!("{} {}", "Unix Epoch (s):".bold().cyan(), epoch.to_string().green().bold());
                }
                Err(err) => {
                    eprintln!("{} {}", "Error:".bold().red(), err);
                    std::process::exit(1);
                }
            },
        },
        Commands::Text(text_args) => match text_args.action {
            TextSubcommand::Count { target } => {
                let payload = resolve_input_or_file(&target);
                let stats = text_tool::analyze_text(&payload);
                println!("{}", "Text Metrics & Content Statistics".bold().cyan());
                println!("  {:<15} {}", "Lines:".bold(), stats.lines.to_string().green());
                println!("  {:<15} {}", "Words:".bold(), stats.words.to_string().green());
                println!(
                    "  {:<15} {}",
                    "Characters:".bold(),
                    stats.characters.to_string().green()
                );
                println!(
                    "  {:<15} {} bytes",
                    "Byte Size:".bold(),
                    stats.bytes.to_string().green()
                );
            }
            TextSubcommand::Case { style, text } => {
                let style_enum = match style {
                    TextCaseStyle::Snake => text_tool::CaseStyle::Snake,
                    TextCaseStyle::Camel => text_tool::CaseStyle::Camel,
                    TextCaseStyle::Kebab => text_tool::CaseStyle::Kebab,
                    TextCaseStyle::Upper => text_tool::CaseStyle::Upper,
                    TextCaseStyle::Lower => text_tool::CaseStyle::Lower,
                };
                let transformed = text_tool::convert_case(&text, style_enum);
                println!("{} {}", "Formatted Text:".bold().cyan(), transformed.green().bold());
            }
        },
        Commands::Qr(qr_args) => match qr::generate_terminal_qr(&qr_args.content) {
            Ok(qr_art) => {
                println!(
                    "{}",
                    format!("QR Code for: {}", qr_args.content).bold().cyan()
                );
                print!("{qr_art}");
            }
            Err(e) => {
                eprintln!("{} Failed to render QR code: {}", "Error:".bold().red(), e);
                std::process::exit(1);
            }
        },
        Commands::Json(json_args) => match json_args.action {
            JsonSubcommand::Format { target } => {
                let payload = resolve_input_or_file(&target);
                match json_tool::format_json(&payload) {
                    Ok(formatted) => println!("{formatted}"),
                    Err(e) => {
                        eprintln!("{} {e}", "Error:".bold().red());
                        std::process::exit(1);
                    }
                }
            }
            JsonSubcommand::Minify { target } => {
                let payload = resolve_input_or_file(&target);
                match json_tool::minify_json(&payload) {
                    Ok(minified) => println!("{minified}"),
                    Err(e) => {
                        eprintln!("{} {e}", "Error:".bold().red());
                        std::process::exit(1);
                    }
                }
            }
            JsonSubcommand::Validate { target } => {
                let payload = resolve_input_or_file(&target);
                match json_tool::validate_json(&payload) {
                    Ok(status) => println!("{} {status}", "Success:".bold().green()),
                    Err(e) => {
                        eprintln!("{} {e}", "Error:".bold().red());
                        std::process::exit(1);
                    }
                }
            }
        },
        Commands::Sys => {
            let sys = telemetry::collect_system_overview();
            println!("{}", "System Hardware & Environment Overview".bold().cyan());
            println!("  {:<15} {}", "Host:".bold(), sys.host_name);
            println!(
                "  {:<15} {} (version {})",
                "OS:".bold(),
                sys.os_name,
                sys.os_version
            );
            println!(
                "  {:<15} {} ({} cores)",
                "CPU:".bold(),
                sys.cpu_brand,
                sys.cpu_count
            );
            println!(
                "  {:<15} Used: {} MB / Total: {} MB (Free: {} MB)",
                "Memory:".bold(),
                sys.used_memory_mb,
                sys.total_memory_mb,
                sys.free_memory_mb
            );
            println!("{}", "Mounted Storage:".bold().cyan());
            for d in sys.disks {
                println!(
                    "  - {:<18} Available: {:.1} GB / Total: {:.1} GB ({})",
                    d.mount_point, d.available_space_gb, d.total_space_gb, d.file_system
                );
            }
        }
        Commands::Net(net_args) => match net_args.action {
            NetSubcommand::Interfaces => {
                let interfaces = telemetry::collect_network_interfaces();
                println!("{}", "Active Network Interfaces".bold().cyan());
                for iface in interfaces {
                    println!(
                        "  {:<15} RX: {:.2} MB | TX: {:.2} MB",
                        iface.name.bold(),
                        (iface.received_bytes as f64) / (1024.0 * 1024.0),
                        (iface.transmitted_bytes as f64) / (1024.0 * 1024.0)
                    );
                }
            }
            NetSubcommand::Ping { host, port } => {
                println!(
                    "{} {}:{}...",
                    "Probing TCP connectivity to".cyan(),
                    host.bold(),
                    port
                );
                match telemetry::probe_tcp_latency(&host, port) {
                    Ok(latency) => {
                        println!(
                            "{} Connected in {:.2} ms",
                            "Success:".bold().green(),
                            latency.as_secs_f64() * 1000.0
                        );
                    }
                    Err(e) => {
                        eprintln!("{} {e}", "Error:".bold().red());
                        std::process::exit(1);
                    }
                }
            }
        },
    }
}
