use clap::Parser;
use crates_io_api::AsyncClient;
use futures::stream::{FuturesUnordered, StreamExt};
use github::*;
use std::collections::HashMap;
use std::env;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::{Layer, Registry};

mod github;

#[derive(clap::ArgEnum, Clone, Debug, Eq, PartialEq)]
enum SortBy {
    Contributions,
    Sponsors,
}

#[derive(clap::ArgEnum, Clone, Debug, Eq, PartialEq)]
enum SortBehaviour {
    Ascending,
    Descending,
}

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
    #[clap(long, arg_enum)]
    /// Field to sort by when listing people you can sponsor
    sort_by: Option<SortBy>,
    /// Method to sort, if not
    #[clap(long, arg_enum)]
    ordering: Option<SortBehaviour>,
}

#[derive(Clone, Debug)]
pub struct CrateInfo {
    repository: Option<String>,
    depth: usize,
    funding_links: Vec<String>,
}

impl CrateInfo {
    fn is_github(&self) -> bool {
        match self.repository.as_ref() {
            Some(s) => s.contains("github.com"),
            None => false,
        }
    }

    fn owner(&self) -> Option<&str> {
        self.repository
            .as_ref()
            .and_then(|s| s.split('/').rev().nth(1))
    }

    fn name(&self) -> Option<&str> {
        self.repository
            .as_ref()
            .and_then(|s| s.split('/').rev().next())
    }
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
    let info = client.get_crate(&name).await.unwrap();
    let dependencies = client
        .crate_dependencies(&name, &info.versions[0].num)
        .await?;

    let crate_info = CrateInfo {
        repository: info.crate_data.repository.clone(),
        depth,
        funding_links: vec![],
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _log = setup_logging();
    let args = Args::parse();

    let email = Command::new("git")
        .args(&["config", "user.email"])
        .output()
        .map(|x| String::from_utf8_lossy(&x.stdout).trim().to_string())
        .unwrap_or_default();

    let agent = format!("fundamental ({})", email.trim());

    let client = AsyncClient::new(&agent, Duration::from_millis(1000)).unwrap();

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

    let client = get_github_client();

    let mut user_map: HashMap<String, UserFundingInfo> = HashMap::new();

    for (name, repo) in crate_list.iter_mut() {
        if !repo.is_github() {
            warn!("Can't provide sponsorship info for: {}: {:?}", name, repo);
            continue;
        }

        if let Some(owner) = repo.owner() {
            let repo_name = repo.name().unwrap(); // Impossible to get owner and not name

            let result = get_sponsor_info_for_repo(&client, owner, repo_name).await;

            match result {
                Ok(res) => {
                    for user in &res.fundable_users {
                        user_map
                            .entry(user.login.clone())
                            .and_modify(|x| {
                                x.contributions += user.contributions;
                                x.crates += 1;
                            })
                            .or_insert(user.clone());
                    }
                    repo.funding_links = res.funding_links;
                }
                Err(e) => {
                    error!("Failed to get info for {}/{}: {}", owner, repo_name, e);
                }
            }
        } else {
            error!("No owner for repo: {} [{:?}]", name, repo);
        }
    }

    let mut crates: Vec<CrateInfo> = crate_list.values().cloned().collect();
    crates.sort_by(|a, b| a.depth.cmp(&b.depth));

    println!("You can sponsor these projects directly!\n=========================================");
    for c in crates.iter().filter(|x| !x.funding_links.is_empty()) {
        println!(
            "{} links: {:?}",
            c.repository.as_ref().unwrap(),
            c.funding_links
        );
    }

    let sort_by = args.sort_by.unwrap_or(SortBy::Contributions);
    let behaviour = args.ordering.unwrap_or_else(|| match sort_by {
        SortBy::Contributions => SortBehaviour::Descending,
        SortBy::Sponsors => SortBehaviour::Ascending,
    });

    let mut user_vec: Vec<UserFundingInfo> = user_map.values().cloned().collect();

    match sort_by {
        SortBy::Contributions => user_vec.sort_by(|a, b| a.contributions.cmp(&b.contributions)),
        SortBy::Sponsors => {
            user_vec.sort_by(|a, b| a.number_of_sponsors.cmp(&b.number_of_sponsors))
        }
    }

    if behaviour == SortBehaviour::Descending {
        user_vec.reverse();
    }

    println!(
        "\nYou can sponsor these users for their work!\n============================================"
    );
    for user in &user_vec {
        println!(
            "http://github.com/{} ({} contributions) ({} crates) ({} sponsors)",
            user.login, user.contributions, user.crates, user.number_of_sponsors
        );
    }

    Ok(())
}
