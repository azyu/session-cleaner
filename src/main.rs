use std::{path::PathBuf, time::Duration};

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use session_cleaner::{
    explain_file, explain_report, json_report, scan_roots, text_report, trash_candidates, Provider,
    RootConfig, ScanConfig,
};

#[derive(Debug, Parser)]
#[command(name = "session-cleaner")]
#[command(version)]
#[command(about = concat!(
    "session-cleaner ",
    env!("CARGO_PKG_VERSION"),
    "\nFind low-value Codex, Claude Code, and OMP session files."
))]
#[command(after_help = "Quick start:
  session-cleaner scan --older-than 24h
  session-cleaner explain <path>
  session-cleaner trash --older-than 24h --yes

Safety:
  - scan never mutates files.
  - trash moves files to platform Trash, not permanent deletion.
  - ambiguous or low-confidence sessions are kept.")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(
        about = "List cleanup candidates. Dry-run by default.",
        after_help = "Examples:
  session-cleaner scan --older-than 24h
  session-cleaner scan -r codex=/tmp/sessions --older-than 7d
  session-cleaner scan --config config.toml --json

Notes:
  - --root uses provider=path, for example codex=/tmp/sessions.
  - --older-than accepts 24h, 7d, 30m, and 0s."
    )]
    Scan {
        #[arg(short = 'r', long = "root", value_name = "provider=path")]
        roots: Vec<String>,
        #[arg(short = 'c', long)]
        config: Option<PathBuf>,
        #[arg(long, default_value = "24h", value_name = "DURATION")]
        older_than: String,
        #[arg(long, help = "Accepted for explicitness; scan is always non-mutating")]
        dry_run: bool,
        #[arg(long, help = "Print machine-readable JSON")]
        json: bool,
    },
    #[command(
        about = "Move high-confidence candidates to platform Trash.",
        after_help = "Examples:
  session-cleaner trash --older-than 24h --yes
  session-cleaner trash -r codex=/tmp/sessions --older-than 7d -y
  session-cleaner trash --config config.toml --yes

Notes:
  - trash moves files to platform Trash, not permanent deletion.
  - --root uses provider=path, for example claude=/tmp/projects.
  - --older-than accepts 24h, 7d, 30m, and 0s."
    )]
    Trash {
        #[arg(short = 'r', long = "root", value_name = "provider=path")]
        roots: Vec<String>,
        #[arg(short = 'c', long)]
        config: Option<PathBuf>,
        #[arg(long, default_value = "24h", value_name = "DURATION")]
        older_than: String,
        #[arg(
            short = 'y',
            long,
            help = "Confirm moving candidates without prompting"
        )]
        yes: bool,
    },
    #[command(
        about = "Explain why one session file is kept or a cleanup candidate.",
        after_help = "Examples:
  session-cleaner explain /path/to/session.jsonl
  session-cleaner explain /path/to/session.jsonl --provider claude"
    )]
    Explain {
        path: PathBuf,
        #[arg(long, default_value = "codex")]
        provider: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Command::Scan {
        roots: default_roots(),
        config: None,
        older_than: "24h".to_owned(),
        dry_run: true,
        json: false,
    }) {
        Command::Scan {
            roots,
            config,
            older_than,
            dry_run: _,
            json,
        } => {
            let config_file = config.as_deref().map(load_file_config).transpose()?;
            let roots = effective_roots(roots, config_file.as_ref())?;
            let older_than = effective_older_than(&older_than, config_file.as_ref());
            let config = ScanConfig {
                roots: parse_roots(&roots)?,
                older_than: parse_duration(&older_than)?,
                max_inspect_lines: 30,
            };
            let report = scan_roots(&config)?;
            if json {
                println!("{}", json_report(&report)?);
            } else {
                print!("{}", text_report(&report, true));
            }
        }
        Command::Trash {
            roots,
            config,
            older_than,
            yes,
        } => {
            if !yes {
                return Err(anyhow!("trash requires --yes"));
            }
            let config_file = config.as_deref().map(load_file_config).transpose()?;
            let roots = effective_roots(roots, config_file.as_ref())?;
            let older_than = effective_older_than(&older_than, config_file.as_ref());
            let config = ScanConfig {
                roots: parse_roots(&roots)?,
                older_than: parse_duration(&older_than)?,
                max_inspect_lines: 30,
            };
            let report = scan_roots(&config)?;
            let trash_report = trash_candidates(&report)?;
            println!("moved: {}", trash_report.moved);
            println!("log: {}", trash_report.log_path.display());
            for path in trash_report.paths {
                println!("trashed: {}", path.display());
            }
        }
        Command::Explain { path, provider } => {
            let provider = parse_provider(&provider)?;
            let item = explain_file(provider, &path, 30)?;
            print!("{}", explain_report(&item));
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct FileConfig {
    older_than: Option<String>,
    roots: Option<FileRoots>,
}

#[derive(Debug, Deserialize)]
struct FileRoots {
    #[serde(default)]
    codex: Vec<String>,
    #[serde(default)]
    claude: Vec<String>,
    #[serde(default)]
    omp: Vec<String>,
}

fn load_file_config(path: &std::path::Path) -> Result<FileConfig> {
    let content = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&content)?)
}

fn effective_roots(
    cli_roots: Vec<String>,
    file_config: Option<&FileConfig>,
) -> Result<Vec<String>> {
    if !cli_roots.is_empty() {
        return Ok(cli_roots);
    }

    if let Some(roots) = file_config.and_then(|config| config.roots.as_ref()) {
        let mut values = Vec::new();
        values.extend(roots.codex.iter().map(|path| format!("codex={path}")));
        values.extend(roots.claude.iter().map(|path| format!("claude={path}")));
        values.extend(roots.omp.iter().map(|path| format!("omp={path}")));
        if !values.is_empty() {
            return Ok(values);
        }
    }

    Ok(default_roots())
}

fn effective_older_than(cli_value: &str, file_config: Option<&FileConfig>) -> String {
    if cli_value != "24h" {
        return cli_value.to_owned();
    }
    file_config
        .and_then(|config| config.older_than.as_ref())
        .cloned()
        .unwrap_or_else(|| cli_value.to_owned())
}

fn default_roots() -> Vec<String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_owned());
    vec![
        format!("codex={home}/.codex/sessions"),
        format!("codex={home}/.codex/archived_sessions"),
        format!("claude={home}/.claude/projects"),
        format!("omp={home}/.omp/agent/sessions"),
    ]
}

fn parse_roots(values: &[String]) -> Result<Vec<RootConfig>> {
    values
        .iter()
        .map(|value| {
            let (provider, path) = value
                .split_once('=')
                .ok_or_else(|| anyhow!("root must use provider=path: {value}"))?;
            Ok(RootConfig {
                provider: parse_provider(provider)?,
                path: PathBuf::from(path),
            })
        })
        .collect()
}

fn parse_provider(value: &str) -> Result<Provider> {
    match value {
        "codex" => Ok(Provider::Codex),
        "claude" => Ok(Provider::Claude),
        "omp" => Ok(Provider::Omp),
        _ => Err(anyhow!("unknown provider: {value}")),
    }
}

fn parse_duration(value: &str) -> Result<Duration> {
    let (number, unit) = value.split_at(value.len().saturating_sub(1));
    let amount: u64 = number.parse()?;
    match unit {
        "s" => Ok(Duration::from_secs(amount)),
        "m" => Ok(Duration::from_secs(amount * 60)),
        "h" => Ok(Duration::from_secs(amount * 60 * 60)),
        "d" => Ok(Duration::from_secs(amount * 60 * 60 * 24)),
        _ => Err(anyhow!("duration must end with s, m, h, or d: {value}")),
    }
}
