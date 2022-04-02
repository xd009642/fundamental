use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{info, instrument};

#[derive(Clone, Debug)]
pub struct RepoFundingInfo {
    pub funding_links: Vec<String>,
    pub fundable_users: Vec<UserFundingInfo>,
}

#[derive(Clone, Debug)]
pub struct UserFundingInfo {
    pub login: String,
    pub number_of_sponsors: usize,
    pub contributions: usize,
    pub crates: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributorResponse {
    login: String,
    url: String,
    #[serde(rename = "type")]
    ty: String,
    contributions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryResponse {
    repository: RepoFundingResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoFundingResponse {
    funding_links: Vec<FundingUrl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundingUrl {
    url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    user: UserSponsorsResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSponsorsResponse {
    has_sponsors_listing: bool,
    sponsors: SponsorsCount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQlResponse<Data> {
    data: Data,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SponsorsCount {
    total_count: usize,
}

pub fn get_github_client() -> Octocrab {
    let api_token = match env::var("GITHUB_API_TOKEN") {
        Ok(s) => s,
        Err(_) => panic!("No github API token listed"),
    };

    Octocrab::builder()
        .personal_token(api_token)
        .build()
        .unwrap()
}

pub async fn get_sponsor_info_for_repo(
    client: &Octocrab,
    owner: &str,
    repo_name: &str,
) -> Result<RepoFundingInfo, Box<dyn std::error::Error>> {
    let repo_urls = client.repos(owner, repo_name).get().await?;

    let mut funding_query = r#"query {repository(owner:""#.to_string();
    funding_query.push_str(owner);
    funding_query.push_str(r#"", name:""#);
    funding_query.push_str(repo_name);
    funding_query.push_str(r#"") {fundingLinks {url}}}"#);

    let repo_response: GraphQlResponse<RepositoryResponse> = client.graphql(&funding_query).await?;

    let funding_links = repo_response
        .data
        .repository
        .funding_links
        .iter()
        .map(|x| x.url.clone())
        .collect();

    let mut fundable_users = vec![];

    if let Some(contributors) = repo_urls.contributors_url {
        let res: Vec<ContributorResponse> = client.get(contributors, Option::<&str>::None).await?;

        for user in res.iter().filter(|x| x.ty == "User") {
            let mut query = r#"query { user(login:""#.to_string();

            query.push_str(user.login.as_str());
            query.push_str(r#"") {hasSponsorsListing, sponsors {totalCount}}}"#);

            let user_response: GraphQlResponse<UserResponse> = client.graphql(&query).await?;

            if user_response.data.user.has_sponsors_listing {
                fundable_users.push(UserFundingInfo {
                    login: user.login.clone(),
                    number_of_sponsors: user_response.data.user.sponsors.total_count,
                    contributions: user.contributions,
                    crates: 1,
                });
            }
        }
    }

    Ok(RepoFundingInfo {
        funding_links,
        fundable_users,
    })
}

/*
    Working out the query bullshit

query {
   repository(owner:"xd009642", name:"tarpaulin") {
      fundingLinks {
      url
    },
   }
}

{
  "data": {
    "repository": {
      "fundingLinks": [
        {
          "url": "https://github.com/xd009642"
        },
        {
          "url": "https://patreon.com/xd009642"
        }
      ]
    }
  }
}

query {
  user(login:"xd009642") {
      hasSponsorsListing,
    sponsors {
      totalCount
    }
  }
}

{
  "data": {
    "user": {
      "hasSponsorsListing": true,
      "sponsors": {
        "totalCount": 14
      }
    }
  }
}
*/
