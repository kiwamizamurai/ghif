# ghif

A CLI tool that bridges GitHub issues and LLM-powered development by converting issues into structured files, enabling seamless integration with AI development workflows.

![Demo](demo.gif)

## Why ghif?

- **LLM-Ready Format**: Issues are converted to structured formats (Markdown/XML) that are optimized for LLM processing
- **Context-Rich Development**: Feed your LLM with real project context from issues and discussions
- **Efficient Workflows**: Integrate with your favorite AI tools to analyze issues, generate code, or get development insights
- **Local Knowledge Base**: Build a searchable archive of project issues for offline access and AI processing

## Features

- Automatically detects GitHub repository from current directory
- Fetches all issues from the repository
- Includes issue comments with author and timestamp
- Shows progress with a nice progress bar
- Supports multiple output formats (Markdown, XML) for different use cases
- Saves issues as structured files in `/issues` directory
- Skips existing files by default to prevent overwriting

## Installation

### macOS

Using Homebrew:
```shell
brew tap kiwamizamurai/tap
brew install ghif
```

> [!NOTE]
> The released binaries are built with Link Time Optimization (LTO) and maximum optimizations enabled for best performance.

Or download the binary directly from [GitHub Releases](https://github.com/kiwamizamurai/ghif/releases) and add it to your PATH.

## Integration with LLM Workflows

Here are some ways to leverage ghif with LLMs:

1. **Issue Analysis**
   - Feed issues to your LLM to identify patterns and common problems
   - Get AI-powered issue summaries and categorization
   - Generate task breakdowns and implementation plans

2. **Code Generation**
   - Use issue context to generate relevant code snippets
   - Let LLMs propose solutions based on similar issues
   - Generate tests based on issue descriptions

3. **Knowledge Management**
   - Build a local knowledge base of project issues
   - Enable semantic search across issue history
   - Train custom models on your project's issue data

## Usage

> [!WARNING]
> Running without `--skip-existing=false` will preserve existing issue files. Use the flag to overwrite them.

```shell
# In your GitHub repository directory
ghif

# Specify repository (required when not in a git repository)
ghif --repository owner/repo
ghif -r owner/repo

# Specify custom output directory
ghif --output custom/path

# Fetch specific issue numbers
ghif --issues 1,2,3

# Fetch only open/closed issues
ghif --state open
ghif --state closed

# Choose output format (markdown/xml)
ghif --format markdown  # default
ghif --format xml

# Force overwrite existing issue files
ghif --skip-existing=false

# Specify batch size for API requests
ghif --batch-size 20

# Combine options
ghif --format xml --skip-existing=false --batch-size 20 --state open
```

## Command Options

| Option | Description |
|--------|-------------|
| `-o, --output` | Output directory for issue files (default: "./issues") |
| `-r, --repository` | Repository URL or owner/repo format (e.g., "owner/repo"). Required when not in a git repository |
| `-i, --issues` | Comma-separated list of issue numbers to fetch |
| `-s, --state` | Filter issues by state (open/closed) |
| `-f, --format` | Output format (markdown/xml) |
| `--batch-size` | Number of issues to fetch in each batch |
| `--skip-existing` | Skip existing files |

## Output Format

> [!NOTE]
> The tool supports both Markdown and XML formats. Choose based on your needs:
> - **Markdown**: Better for human readability and documentation
> - **XML**: Better for automated processing and parsing

The tool supports multiple output formats to accommodate different use cases:

### Markdown Format (Default)
```markdown
# Issue #123: Issue Title

**State:** open
**Created:** 2024-01-04T12:34:56Z
**Updated:** 2024-01-04T12:34:56Z
**Labels:** bug, enhancement
**Assignees:** username1, username2
**User:** reporter

## Description

Issue description here...

## Comments

### @commenter (2024-01-04T13:45:67Z)

Comment content here...
```

### XML Format
```xml
<?xml version="1.0" encoding="UTF-8"?>
<issue>
    <number>123</number>
    <title><![CDATA[Issue Title]]></title>
    <state>open</state>
    <created_at>2024-01-04T12:34:56Z</created_at>
    <updated_at>2024-01-04T12:34:56Z</updated_at>
    <labels>
        <label>bug</label>
        <label>enhancement</label>
    </labels>
    <assignees>
        <assignee>username1</assignee>
        <assignee>username2</assignee>
    </assignees>
    <user>reporter</user>
    <description><![CDATA[
        Issue description here...
    ]]></description>
    <comments>
        <comment>
            <user>commenter</user>
            <created_at>2024-01-04T13:45:67Z</created_at>
            <body><![CDATA[Comment content here...]]></body>
        </comment>
    </comments>
</issue>
```

Choose the format that best suits your needs:
- Use **Markdown** for human-readable format and easy integration with documentation tools
- Use **XML** for structured data processing, parsing, and integration with XML-based tools

## Authentication

> [!NOTE]
> A GitHub Personal Access Token is optional but recommended for higher API rate limits.
> If provided, set it in your environment:
> ```shell
> export GITHUB_TOKEN=$(gh auth token)
> ```

The tool can run without authentication, but will have restricted API rate limits. To increase these limits, you can create and use a GitHub Personal Access Token:

1. Go to GitHub Settings -> Developer settings -> Personal access tokens -> Tokens (classic)
2. Generate new token
3. Select at least these scopes:
   - `repo` (Full control of private repositories)
   - `read:org` (Read org and team membership)

### Docker

You can also run ghif using Docker:

1. Download the Dockerfile and build the image:
```shell
curl -O https://raw.githubusercontent.com/kiwamizamurai/ghif/refs/heads/main/Dockerfile
docker build -t ghif .
```

2. Run the container:
```shell
# Using GitHub CLI for authentication
docker run --rm -v $(pwd)/issues:/issues -e GITHUB_TOKEN=$(gh auth token) ghif -o /issues -i 1 -r "owner/repo"

# Or using a personal access token
docker run --rm -v $(pwd)/issues:/issues -e GITHUB_TOKEN=$GITHUB_TOKEN ghif -o /issues -i 1 -r "owner/repo"
```

> [!NOTE]
> - Publication to Docker Hub is planned as an upcoming feature
> - The `-r` flag is required when running in Docker to specify the target repository
> - Mount a local directory to `/issues` to save the downloaded issues

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License#
