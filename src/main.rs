use std::{ffi::OsStr, fs, io::Write, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use dirs::home_dir;
use markdown_parser::{read_file, Markdown};
use serde::{Deserialize, Serialize};

/// Publish obsidian vault with quartz
///
#[derive(Parser)]
struct Cli {
    /// The path to the obsidian vault
    // #[arg(short, long)]
    vault_path: PathBuf,

    /// The path to the quartz content folder
    // #[arg(short, long)]
    quartz_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Frontmatter {
    tags: Vec<String>,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let vault_path = args.vault_path;
    let quartz_path = args.quartz_path;
    let e = visit_dirs(&vault_path, &quartz_path);
    println!("{:?}", e);

    Ok(())
}

fn visit_dirs(dir: &PathBuf, quartz_path: &PathBuf) -> Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if !path.ends_with(".obsidian") && !path.ends_with(".trash") {
                    visit_dirs(&path, quartz_path)?;
                }
            } else {
                if path.extension().and_then(OsStr::to_str) == Some("md") {
                    let md = read_file(path.clone());
                    match md {
                        Ok(markdown) => {
                            let frontmatter = markdown.front_matter();
                            let frontmatter_yaml: Frontmatter =
                                match serde_yaml::from_str(frontmatter) {
                                    Ok(yaml) => yaml,
                                    Err(_) => Frontmatter::default(), // or some other fallback value
                                };
                            if frontmatter_yaml.tags.contains(&String::from("publish")) {
                                println!("copying {:?} to {:?}", path, quartz_path);
                                let file_name = path.file_name().and_then(OsStr::to_str);
                                let file_path = if let Some(name) = file_name {
                                    quartz_path.join(name)
                                } else {
                                    quartz_path.join("error.md")
                                };
                                let mut file = fs::File::create(file_path)?;
                                let metadata = file.metadata()?;
                                let mut permissions = metadata.permissions();
                                permissions.set_readonly(false);
                                file.write_all(markdown.content().as_bytes())?;
                                println!("copied {:?} to {:?}", path, quartz_path);
                            }
                        }
                        Err(_) => {}
                    }
                }
            }
        }
    }
    Ok(())
}
