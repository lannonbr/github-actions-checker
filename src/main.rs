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

        let repo_ref = github_client.repo(owner, repo);

        let current_tag_ref = match repo_ref
            .git()
            .reference(format!("tags/{}", version))
            .await?
        {
            hubcaps::git::GetReferenceResponse::Exact(reference) => reference,
            _ => panic!("no exact match found"),
        };

        // let current_release = repo_ref.releases().by_tag(version).await?;
        let newest_release = repo_ref.releases().latest().await?;

        let newest_tag = match repo_ref
            .git()
            .reference(format!("tags/{}", newest_release.tag_name))
            .await?
        {
            hubcaps::git::GetReferenceResponse::Exact(reference) => reference,
            _ => panic!("no exact match found"),
        };
        dbg!(&current_tag_ref);

        // If the reference for the current tag is actually a tag, go resolve the tag's commit sha
        let current_sha = if current_tag_ref.object.object_type == "tag" {
            // grab current_tag.object.url and fetch the data found there
            todo!("Resolve tag commit sha");
            "".to_string()
        } else if current_tag_ref.object.object_type == "commit" {
            current_tag_ref.object.sha
        } else {
            panic!("Resolved to neither a tag or commit");
        };

        let new_sha = newest_tag.object.sha;

        println!("{}\n{} - {}", _repo, current_sha, new_sha);

        if newest_release.tag_name.starts_with(&version) && current_sha == new_sha {
            println!("{} is up to date", _repo);
        } else {
            println!("NEW UPDATE");
            println!("{}, {}", _repo, newest_release.name);
        }

        version_table.add_row(row![_repo, version, newest_release.tag_name]);
        println!("");
    }

    print!("{}", version_table);

    // TODO: Use GitHub Releases API to check if there is a new release

    // TODO: Print out outdated actions (And maybe suggest using --fix to update them automatically)
    if flags.fix {
        todo!("Modify the files with new version of the actions")
    }

    Ok(())
}
