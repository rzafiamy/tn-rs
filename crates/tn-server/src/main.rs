use anyhow::Result;
use clap::{Parser, Subcommand};
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use tn::{Lang, Lexicon, Mode, normalize_text};
use tn_server::{LexiconSource, router};

/// Text normalizer for speech synthesis (French, English).
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Serve the HTTP API (GET /health, POST /v1/normalize)
    Serve {
        #[arg(long, env = "TN_HOST", default_value = "127.0.0.1")]
        host: String,
        #[arg(short, long, env = "TN_PORT", default_value_t = 8090)]
        port: u16,
        /// Pronunciation lexicon (TSV, reloaded when it changes)
        #[arg(short, long, env = "TN_LEXICON")]
        lexicon: Option<PathBuf>,
    },
    /// Normalize TEXT (or standard input) and print it
    Normalize {
        /// Language code or name: fr, en, french, en-US, ...
        #[arg(long, default_value = "en")]
        lang: String,
        /// Leave ambiguous numbers as digits
        #[arg(long)]
        safe: bool,
        #[arg(short, long, env = "TN_LEXICON")]
        lexicon: Option<PathBuf>,
        text: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Serve {
            host,
            port,
            lexicon,
        } => {
            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "info".into()),
                )
                .init();
            let source = LexiconSource::open(lexicon).map_err(anyhow::Error::msg)?;
            let listener = tokio::net::TcpListener::bind((host.as_str(), port)).await?;
            tracing::info!(
                "tn-server {} listening on {host}:{port}",
                env!("CARGO_PKG_VERSION")
            );
            axum::serve(listener, router(Arc::new(source))).await?;
        }
        Command::Normalize {
            lang,
            safe,
            lexicon,
            text,
        } => {
            let text = if text.is_empty() {
                let mut s = String::new();
                std::io::stdin().read_to_string(&mut s)?;
                s
            } else {
                text.join(" ")
            };
            let lex = lexicon
                .map(|p| Lexicon::load(&p))
                .transpose()
                .map_err(anyhow::Error::msg)?;
            let mode = if safe { Mode::Safe } else { Mode::Strict };
            println!(
                "{}",
                normalize_text(&text, Lang::from_code(&lang), mode, lex.as_ref())
            );
        }
    }
    Ok(())
}
