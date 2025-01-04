use crate::error::GhError;
use console::{style, Term};
use octocrab::models::issues::Issue;
use octocrab::params;
use octocrab::params::issues::Sort;
use octocrab::params::Direction;
use serde::Serialize;

#[derive(Serialize)]
#[allow(dead_code)]
pub struct IssueData {
    number: u64,
    title: String,
    state: String,
    body: Option<String>,
    labels: Vec<String>,
    created_at: String,
    updated_at: String,
    assignees: Vec<String>,
    user: String,
    comments_url: Option<String>,
}

impl From<Issue> for IssueData {
    fn from(issue: Issue) -> Self {
        IssueData {
            number: issue.number,
            title: issue.title,
            state: match issue.state {
                octocrab::models::IssueState::Open => "open",
                octocrab::models::IssueState::Closed => "closed",
                _ => "unknown",
            }
            .to_string(),
            body: issue.body,
            labels: issue.labels.into_iter().map(|l| l.name).collect(),
            created_at: issue.created_at.to_string(),
            updated_at: issue.updated_at.to_string(),
            assignees: issue.assignees.into_iter().map(|a| a.login).collect(),
            user: issue.user.login,
            comments_url: Some(issue.comments_url.to_string()),
        }
    }
}

#[allow(dead_code)]
impl IssueData {
    pub fn number(&self) -> u64 {
        self.number
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn state(&self) -> &str {
        &self.state
    }

    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }

    pub fn created_at(&self) -> &str {
        &self.created_at
    }

    pub fn updated_at(&self) -> &str {
        &self.updated_at
    }

    pub fn labels(&self) -> &[String] {
        &self.labels
    }

    pub fn assignees(&self) -> &[String] {
        &self.assignees
    }

    pub fn user(&self) -> &str {
        &self.user
    }
}

#[derive(Serialize)]
pub struct CommentData {
    pub user: String,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
}

#[allow(dead_code)]
pub struct GitHubClient {
    octocrab: octocrab::Octocrab,
}

#[allow(dead_code)]
impl GitHubClient {
    pub async fn new() -> std::result::Result<Self, GhError> {
        let token = std::env::var("GITHUB_TOKEN").ok();
        let octocrab = if let Some(token) = token {
            octocrab::OctocrabBuilder::new()
                .personal_token(token)
                .build()
                .map_err(|e| GhError::AuthError(e.to_string()))?
        } else {
            let term = Term::stderr();
            term.write_line(&format!(
                "{} Running without GITHUB_TOKEN. API rate limits will be restricted.",
                style("Note:").blue().bold()
            ))?;
            term.write_line(&format!(
                "{} To increase rate limits, you can set the GITHUB_TOKEN environment variable.",
                style("Tip:").cyan().bold()
            ))?;
            octocrab::OctocrabBuilder::new()
                .build()
                .map_err(|e| GhError::AuthError(e.to_string()))?
        };

        Ok(Self { octocrab })
    }

    pub async fn fetch_issues(
        &self,
        owner: &str,
        repo: &str,
        state: Option<&str>,
        numbers: Option<&[u32]>,
        batch_size: usize,
    ) -> std::result::Result<Vec<IssueData>, GhError> {
        let mut issues = Vec::new();

        if let Some(nums) = numbers {
            for chunk in nums.chunks(batch_size) {
                for &number in chunk {
                    match self.octocrab.issues(owner, repo).get(number as u64).await {
                        Ok(issue) => {
                            println!("Successfully fetched issue #{}", number);
                            issues.push(IssueData::from(issue));
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to fetch issue #{}: {} (This issue might be private or deleted)", number, e);
                            continue;
                        }
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        } else {
            let issues_handler = self.octocrab.issues(owner, repo);
            let state_param = match state {
                Some("open") => params::State::Open,
                Some("closed") => params::State::Closed,
                _ => params::State::All,
            };

            let mut page = match issues_handler
                .list()
                .per_page(100)
                .state(state_param)
                .direction(Direction::Descending)
                .sort(Sort::Created)
                .send()
                .await
            {
                Ok(page) => page,
                Err(octocrab::Error::GitHub { source, .. }) if source.message == "Not Found" => {
                    // When there are no issues, GitHub returns 404
                    return Ok(Vec::new());
                }
                Err(e) => return Err(GhError::ApiError(e.to_string())),
            };

            loop {
                for issue in page.items {
                    if issue.pull_request.is_none() {
                        issues.push(IssueData::from(issue));
                    }
                }

                page = match self.octocrab.get_page(&page.next).await {
                    Ok(Some(next_page)) => next_page,
                    Ok(None) => break,
                    Err(e) => return Err(GhError::ApiError(e.to_string())),
                };
            }
        }

        Ok(issues)
    }

    pub async fn fetch_comments(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> std::result::Result<Vec<CommentData>, GhError> {
        let mut comments = Vec::new();
        let mut page = self
            .octocrab
            .issues(owner, repo)
            .list_comments(issue_number)
            .send()
            .await
            .map_err(|e| GhError::ApiError(format!("Failed to fetch comments: {}", e)))?;

        loop {
            for comment in page.items {
                comments.push(CommentData {
                    user: comment.user.login,
                    body: comment.body.unwrap_or_default(),
                    created_at: comment.created_at.to_string(),
                    updated_at: comment
                        .updated_at
                        .map_or_else(|| "N/A".to_string(), |dt| dt.to_string()),
                });
            }

            page = match self.octocrab.get_page(&page.next).await {
                Ok(Some(next_page)) => next_page,
                Ok(None) => break,
                Err(e) => return Err(GhError::ApiError(e.to_string())),
            };
        }

        Ok(comments)
    }

    pub async fn get_rate_limit_info(&self) -> std::result::Result<String, GhError> {
        let rate_limit = self
            .octocrab
            .ratelimit()
            .get()
            .await
            .map_err(|e| GhError::RateLimitError(e.to_string()))?;

        Ok(format!(
            "{} {}/{} remaining. Reset at: {}",
            style("API Rate Limit:").cyan().bold(),
            style(rate_limit.rate.remaining).green(),
            style(rate_limit.rate.limit).green(),
            style(
                chrono::DateTime::<chrono::Utc>::from(
                    std::time::UNIX_EPOCH
                        + std::time::Duration::from_secs(rate_limit.rate.reset as u64)
                )
                .to_rfc3339()
            )
            .blue()
        ))
    }
}
