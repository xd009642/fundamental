use async_recursion::async_recursion;
use clap::Parser;
use crates_io_api::AsyncClient;
use futures::stream::{FuturesUnordered, StreamExt};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::{Layer, Registry};

#[derive(Debug, Parser)]
#[clap(version, about, long_about=None)]
pub struct Args {
    /// Name of the crate to inspect
    #[clap(long, short)]
    input: String,
    /// process dev dependencies as well
    #[clap(long)]
    dev: bool,
    /// Max depth to crawl
    #[clap(long, default_value = "1000")]
    max_depth: usize,
}

#[derive(Clone, Debug)]
pub struct CrateInfo {
    repository: Option<String>,
    depth: usize,
}

fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    let filter = match env::var("RUST_LOG") {
        Ok(_) => EnvFilter::from_default_env(),
        _ => EnvFilter::new("fundamental=info"),
    };

    let fmt = tracing_subscriber::fmt::Layer::default().with_ansi(env::var("REMOVE_ANSI").is_err());

    let subscriber = filter.and_then(fmt).with_subscriber(Registry::default());

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

#[instrument(skip(client))]
async fn fetch_crate_info(
    name: String,
    depth: usize,
    dev: bool,
    client: AsyncClient,
) -> Result<(CrateInfo, Vec<String>), Box<dyn std::error::Error>> {
    info!("Processing");
    let info = client.get_crate(&name).await.unwrap();
    let dependencies = client
        .crate_dependencies(&name, &info.versions[0].num)
        .await?;

    let crate_info = CrateInfo {
        repository: info.crate_data.repository.clone(),
        depth,
    };
    let children = if dev {
        dependencies.iter().map(|x| x.crate_id.clone()).collect()
    } else {
        dependencies
            .iter()
            .filter(|x| x.kind != "dev")
            .map(|x| x.crate_id.clone())
            .collect()
    };
    Ok((crate_info, children))
}

#[tokio::main]
async fn main() {
    let _log = setup_logging();
    let args = Args::parse();

    let client = AsyncClient::new("fundamental", Duration::from_millis(1000)).unwrap();

    let mut crate_list = HashMap::<String, CrateInfo>::new();

    let mut pending = vec![];
    pending.push((args.input.clone(), 0));

    while let Some((res, depth)) = pending.pop() {
        let response = fetch_crate_info(res.clone(), depth, args.dev, client.clone()).await;
        match response {
            Ok((info, children)) => {
                crate_list.insert(res.clone(), info);
                if args.max_depth > depth {
                    for child in children.iter().filter(|x| !crate_list.contains_key(*x)) {
                        if !pending.iter().any(|(name, _)| name == child) {
                            pending.push((child.clone(), depth + 1));
                        }
                    }
                }
                debug!("Pending queue: {:?}", pending);
            }
            Err(e) => error!("Error on {}: {}", res, e),
        }
    }

    info!("{:#?}", crate_list);
}
