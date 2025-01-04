mod error;
mod github;

use anyhow::{Context, Result};
use clap::Parser;
use console::style;
use error::GhError;
use ghif::{get_file_extension, get_writer, CommentData, GitHubClient, IssueData, OutputFormat};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};

/// CLI tool to fetch GitHub issues and save them as Markdown files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Output directory for issue files (default: "./issues")
    #[arg(short, long, default_value = "./issues")]
    output: PathBuf,

    /// Comma-separated list of issue numbers to fetch
    #[arg(short, long)]
    issues: Option<String>,

    /// Filter issues by state (open/closed)
    #[arg(short, long, default_value = "open")]
    state: Option<String>,

    /// Number of issues to fetch in each batch
    #[arg(long, default_value = "10")]
    batch_size: usize,

    /// Skip existing files
    #[arg(long, default_value_t = true)]
    skip_existing: bool,

    /// Output format (markdown/xml)
    #[arg(short, long, default_value = "markdown")]
    format: String,

    /// Repository URL or owner/repo format (e.g., "owner/repo")
    #[arg(short = 'r', long)]
    repository: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    run().await.with_context(|| "Application error occurred")?;
    Ok(())
}

async fn run() -> Result<()> {
    let args = Args::parse();
    println!("{} Starting ghif...", style("Info:").cyan().bold());
    println!(
        "{} Output directory: {}",
        style("Info:").cyan().bold(),
        args.output.display()
    );

    std::fs::create_dir_all(&args.output).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            args.output.display()
        )
    })?;

    let repo = if let Some(repo) = args.repository {
        println!(
            "{} Using provided repository: {}",
            style("Info:").cyan().bold(),
            repo
        );
        repo
    } else {
        println!(
            "{} Attempting to detect GitHub repository...",
            style("Info:").cyan().bold()
        );
        detect_github_repo().with_context(|| "Failed to detect GitHub repository")?
    };

    let (owner, repo_name) = repo.split_once('/').ok_or_else(|| {
        println!(
            "{} Invalid repository format: {}",
            style("Error:").red().bold(),
            repo
        );
        GhError::InvalidRepoUrl("Repository should be in format 'owner/repo'".to_string())
    })?;

    println!(
        "{} Repository: {}/{}",
        style("Info:").cyan().bold(),
        owner,
        repo_name
    );

    let issue_numbers = args.issues.map(|s| {
        s.split(',')
            .filter_map(|n| {
                n.trim()
                    .parse::<u32>()
                    .map_err(|e| {
                        eprintln!(
                            "{} Invalid issue number '{}': {}",
                            style("Warning:").yellow().bold(),
                            style(n).red(),
                            e
                        );
                    })
                    .ok()
            })
            .collect::<Vec<_>>()
    });

    let client = match GitHubClient::new().await {
        Ok(client) => {
            println!(
                "{} GitHub client initialized successfully",
                style("Info:").cyan().bold()
            );
            client
        }
        Err(e) => {
            println!(
                "{} Failed to initialize GitHub client: {}",
                style("Error:").red().bold(),
                e
            );
            return Err(e.into());
        }
    };

    println!("{}", client.get_rate_limit_info().await?);

    fetch_issues(
        &client,
        owner,
        repo_name,
        &args.output,
        issue_numbers.as_deref(),
        args.state.as_deref(),
    )
    .await?;

    Ok(())
}

fn detect_github_repo() -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["config", "--get", "remote.origin.url"])
        .output()
        .with_context(|| "Failed to execute git command")?;

    if !output.status.success() {
        anyhow::bail!("Failed to detect GitHub repository. Are you in a git repository?");
    }

    let url = String::from_utf8(output.stdout).with_context(|| "Invalid UTF-8 in git output")?;
    parse_github_repo_url(&url)
}

fn parse_github_repo_url(url: &str) -> Result<String> {
    let url = url.trim();

    if url.starts_with("https://github.com/") {
        Ok(url
            .trim_start_matches("https://github.com/")
            .trim_end_matches(".git")
            .to_string())
    } else if url.starts_with("git@github.com:") {
        Ok(url
            .trim_start_matches("git@github.com:")
            .trim_end_matches(".git")
            .to_string())
    } else {
        anyhow::bail!("Unsupported repository URL format: {}", url)
    }
}

async fn fetch_issues(
    client: &GitHubClient,
    owner: &str,
    repo: &str,
    output_dir: &Path,
    issue_numbers: Option<&[u32]>,
    state: Option<&str>,
) -> Result<()> {
    println!(
        "\n{} issues from {}/{}...",
        style("Fetching").cyan().bold(),
        style(owner).green(),
        style(repo).green()
    );

    let args = Args::parse();
    let format = args
        .format
        .parse::<OutputFormat>()
        .map_err(GhError::InvalidFormat)?;

    let issues = client
        .fetch_issues(owner, repo, state, issue_numbers, args.batch_size)
        .await?;

    println!("Found {} issues", style(issues.len()).cyan());
    let pb = ProgressBar::new(issues.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} issues")?
            .progress_chars("=>-"),
    );

    for issue in issues {
        let file_path = get_issue_file_path(output_dir, &issue, format);

        if args.skip_existing && file_path.exists() {
            println!(
                "{} existing issue #{}",
                style("Skipping").yellow(),
                style(issue.number()).cyan()
            );
            pb.inc(1);
            continue;
        }

        let comments = client.fetch_comments(owner, repo, issue.number()).await?;
        save_issue_to_file(output_dir, &issue, &comments, format)?;
        pb.inc(1);
    }

    pb.finish_with_message(format!(
        "{}",
        style("All issues downloaded successfully!").green().bold()
    ));
    Ok(())
}

fn get_issue_file_path(output_dir: &Path, issue: &IssueData, format: OutputFormat) -> PathBuf {
    let filename = format!(
        "issue-{}-{}.{}",
        issue.number(),
        sanitize_filename(issue.title()),
        get_file_extension(format)
    );
    output_dir.join(filename)
}

fn save_issue_to_file(
    output_dir: &Path,
    issue: &IssueData,
    comments: &[CommentData],
    format: OutputFormat,
) -> Result<()> {
    let path = get_issue_file_path(output_dir, issue, format);
    let writer = get_writer(format);
    let content = writer.write_issue(issue, comments);

    let path_display = path.display().to_string();
    std::fs::write(&path, content)?;
    println!("Saved issue #{} to {}", issue.number(), path_display);
    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| match c {
            'a'..='z' | '0'..='9' => c,
            _ => '-',
        })
        .collect::<String>()
        .replace("--", "-")
        .trim_matches('-')
        .to_string()
}
