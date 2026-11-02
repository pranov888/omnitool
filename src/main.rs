mod crypto;
mod qr;

use clap::{Args, Parser, Subcommand, ValueEnum};
use colored::*;

#[derive(Parser)]
#[command(
    name = "omnitool",
    author = "pranov888",
    version = "0.1.0",
    about = "⚡ All-in-one developer Swiss-Army knife CLI written in Rust",
    long_about = "Omnitool provides everyday developer utilities (crypto, QR codes, JSON tools, system info, and network diagnostics) in a single high-performance binary."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 🔐 Cryptographic helpers (hashes, base64, uuid, password generation)
    Crypto(CryptoArgs),
    /// 📱 Generate scannable QR codes directly in your terminal
    Qr(QrArgs),
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
struct QrArgs {
    /// URL or text content to encode into a terminal QR code
    content: String,
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
    }
}
