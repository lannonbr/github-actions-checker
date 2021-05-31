#[macro_use]
extern crate prettytable;

use color_eyre::eyre::Result;
use hubcaps::{Credentials, Github};
use prettytable::Table;
use std::collections::HashSet;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Flags {
    #[structopt(parse(from_os_str), short = "f", long = "file")]
    file: std::path::PathBuf,
    #[structopt(long)]
    fix: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let flags = Flags::from_args();

    let github_client = Github::new(
        "Github-Actions-Checker/0.1.0",
        Credentials::Token(std::env::var("GITHUB_TOKEN").unwrap()),
    )?;

    // Read workflow file
    let file = tokio::fs::read_to_string(&flags.file).await?;

    let lines: HashSet<(&str, &str)> = file
        .lines()
        .filter(|line| line.contains("uses:")) // uses: actions/checkout@v2
        .map(|line| line.split_once(": ").unwrap().1) // actions/checkout@v2
        .map(|repo| repo.split_once("@").unwrap()) // (actions/checkout, v2)
        .collect();

    let mut version_table = Table::new();
    version_table.set_format(*prettytable::format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    version_table.set_titles(row!["Action", "Current Version", "Latest Version"]);

    for (_repo, version) in lines {
        let (owner, repo) = _repo.split_once("/").unwrap();

        let repo = github_client.repo(owner, repo);

        let newest_release = repo.releases().latest().await?;

        version_table.add_row(row![_repo, version, newest_release.tag_name]);
    }

    print!("{}", version_table);

    // TODO: Use GitHub Releases API to check if there is a new release

    // TODO: Print out outdated actions (And maybe suggest using --fix to update them automatically)
    if flags.fix {
        todo!("Modify the files with new version of the actions")
    }

    Ok(())
}
