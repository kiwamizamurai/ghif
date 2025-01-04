## Local Development

1. Clone the repository:
```shell
git clone https://github.com/kiwamizamurai/ghif.git
cd ghif
```

2. Set up your GitHub Personal Access Token:
```shell
export GITHUB_TOKEN=your_github_token
```

To create a new token:
1. Go to GitHub Settings -> Developer settings -> Personal access tokens -> Tokens (classic)
2. Generate new token
3. Select at least these scopes:
   - `repo` (Full control of private repositories)
   - `read:org` (Read org and team membership)

3. Build and run locally:
```shell
# Build the project
cargo build

# Run with default settings
cargo run

# Run with custom options
cargo run -- --output custom/path
cargo run -- --issues 1,2,3
cargo run -- --state open
```

4. Run tests:
```shell
cargo test
```
