use std::{
    hash::Hash,
    path::PathBuf,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

use anyhow::{Context, Result};
use clap::Parser;
use enable_ansi_support::enable_ansi_support;
use futures::TryStreamExt as _;
use identity_hash::IntSet;
use reqwest::{Client, Url};
use tokio::{
    fs::{create_dir_all, File},
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::mpsc::unbounded_channel,
    task::JoinHandle,
};
use xxhash_rust::xxh3::Xxh3;

#[tokio::main]
async fn main() -> Result<()> {
    let ansi_codes_enabled = enable_ansi_support().is_ok();
    let cli = Cli::parse();

    create_dir_all(&cli.out_dir)
        .await
        .with_context(|| "Failed to create output directory")?;
    let in_file = File::open(&cli.in_file)
        .await
        .with_context(|| "Failed to open input file")?;

    let mut in_lines = BufReader::new(in_file).lines();
    let load_ctx = Arc::new(LoadCtx {
        counter: AtomicU32::new(1),
        out_dir: cli.out_dir,
        client: Client::new(),
    });
    let mut url_hashes: IntSet<u64> = Default::default();
    let mut hasher = Xxh3::new();
    let (out_sender, mut out_receiver) = unbounded_channel();

    let spawner = tokio::spawn(async move {
        while let Some(line) = in_lines.next_line().await? {
            line.hash(&mut hasher);
            let hash = hasher.digest();
            if !url_hashes.insert(hash) {
                continue;
            }
            hasher.reset();

            match Url::parse(&line) {
                Ok(url) => {
                    let join_handle = tokio::spawn(load(url, hash, load_ctx.clone()));
                    out_sender.send(Output::JoinHandle(join_handle)).unwrap();
                }
                Err(err) => {
                    out_sender
                        .send(Output::UrlParseError { line, err })
                        .unwrap();
                }
            }
        }
        anyhow::Ok(())
    });

    let mut counter = 0;
    println!("{}", counter);
    while let Some(out) = out_receiver.recv().await {
        let message = match out {
            Output::JoinHandle(handle) => handle
                .await?
                .err()
                .map(|err| format!("Error loading: {}", err)),
            Output::UrlParseError { line, err } => {
                Some(format!("Couldn't parse url {}: {}", line, err))
            }
        };
        if ansi_codes_enabled {
            print!("{}", ansi_escapes::EraseLines(2));
        }
        if let Some(message) = message {
            println!("{}", message);
        }
        counter += 1;
        println!("{}", counter);
    }

    spawner.await??;

    Ok(())
}

async fn load(url: Url, hash: u64, ctx: Arc<LoadCtx>) -> Result<()> {
    let ext = url
        .path_segments()
        .and_then(|segs| segs.last())
        .and_then(|last| last.rsplit_once('.'))
        .map(|split| split.1.to_owned());

    let res = ctx.client.get(url).send().await?.error_for_status()?;

    let i = ctx.counter.fetch_add(1, Ordering::Relaxed);
    let filename = if let Some(ext) = ext {
        format!("{}-{:x}.{}", i, hash, ext)
    } else {
        format!("{}-{:x}", i, hash)
    };
    let mut file = File::create(ctx.out_dir.join(filename)).await?;

    let mut chunks = res.bytes_stream();
    while let Some(chunk) = chunks.try_next().await? {
        file.write_all(&chunk).await?;
    }

    Ok(())
}

struct LoadCtx {
    counter: AtomicU32,
    out_dir: PathBuf,
    client: Client,
}

enum Output {
    JoinHandle(JoinHandle<Result<()>>),
    UrlParseError { line: String, err: url::ParseError },
}

#[derive(Parser)]
struct Cli {
    #[arg(long, default_value = "./out", help = "Output directory")]
    out_dir: PathBuf,
    #[arg(help = "Input file")]
    in_file: PathBuf,
}
