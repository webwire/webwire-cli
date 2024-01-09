use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand, ValueEnum};

use webwire_cli::codegen;
use webwire_cli::idl;
use webwire_cli::schema;

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum Language {
    #[value(name = "rs", help = "Rust")]
    Rust,
    #[value(name = "ts", help = "TypeScript")]
    TypeScript,
}

#[derive(Debug, Parser)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = format!("{}\n\nFor more information please visit {}",
        env!("CARGO_PKG_DESCRIPTION"),
        env!("CARGO_PKG_HOMEPAGE")
    )
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(about = "Generate source")]
    Gen(Gen),
}

#[derive(Debug, Parser)]
struct Gen {
    language: Language,
    source: Option<String>,
    target: Option<String>,
    #[arg(
        short,
        long,
        help = "Type name that should be treated as a built-in type"
    )]
    r#type: Option<Vec<String>>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    match args.command {
        Command::Gen(gen_args) => cmd_gen(&gen_args),
    }
}

enum Source {
    Stdin,
    File(PathBuf),
}

impl Source {
    fn read(&self) -> Result<String, String> {
        let mut read: Box<dyn Read> = match self {
            Self::Stdin => Box::new(stdin()),
            Self::File(filename) => Box::new(
                File::open(filename)
                    .map_err(|e| format!("Could not open file {:?}: {}", filename, e))?,
            ),
        };
        let mut content = String::new();
        read.read_to_string(&mut content).map_err(|e| {
            format!(
                "An error occured while reading source file {:?}: {}",
                self.filename(),
                e
            )
        })?;
        Ok(content)
    }
    fn filename(&self) -> String {
        match self {
            Self::Stdin => String::from("--"),
            Self::File(path) => format!("{:?}", path),
        }
    }
}

#[derive(Debug)]
struct GenError {
    message: String,
}

impl std::fmt::Display for GenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for GenError {}

fn cmd_gen(args: &Gen) -> Result<(), Box<dyn std::error::Error>> {
    let path = match args.source.as_deref() {
        None | Some("--") => None,
        Some(path) => Some(Path::new(path)),
    };
    let source = path.map_or(Source::Stdin, |p| Source::File(p.to_owned()));

    // Parse IDL file
    let mut idocs: Vec<idl::Document> = Vec::new();
    let idoc = idl::parse_document(&source.read()?).map_err(|e| format!("{}", e))?;
    idocs.push(idoc);

    // Parse all included files (recursively)
    if !idocs[0].includes.is_empty() {
        if matches!(source, Source::Stdin) {
            return Err(Box::new(GenError {
                message: "Source must not contain any includes if reading from stdin".to_owned(),
            }));
        }
        let base_dir = path
            .and_then(|p| p.parent())
            .ok_or("base_dir could not be determined from source")?;
        let mut included_files: HashSet<PathBuf> = HashSet::new();
        let mut includes = idocs[0]
            .includes
            .iter()
            .map(|p| (base_dir.join(&p.filename)))
            .collect::<Vec<_>>();
        while !includes.is_empty() {
            let include = includes.remove(0);
            if included_files.contains(&include) {
                continue;
            }
            included_files.insert(include.clone());
            let source = Source::File(include.clone());
            let idoc = idl::parse_document(&source.read()?).map_err(|e| format!("{}", e))?;
            let dir = include.parent().unwrap();
            includes.extend(idoc.includes.iter().map(|inc| dir.join(&inc.filename)));
            idocs.push(idoc);
        }
    }

    let types = args
        .r#type
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|v| {
            let parts = v.splitn(2, '=').collect::<Vec<_>>();
            if parts.len() == 1 {
                (parts[0].to_owned(), parts[0].to_owned())
            } else if parts.len() == 2 {
                (parts[0].to_owned(), parts[1].to_owned())
            } else {
                unreachable!();
            }
        })
        .collect::<HashMap<_, _>>();

    // Convert IDL to Schema
    let doc = schema::Document::from_idl(idocs.iter(), &types)?;

    // Call code generator function
    let target_code = match args.language {
        Language::Rust => codegen::rust::gen(&doc),
        Language::TypeScript => codegen::ts::gen(&doc),
    };

    // Write target file
    let mut target: Box<dyn Write> = match args.target.as_deref() {
        None | Some("--") => Box::new(stdout()),
        Some(filename) => Box::new(File::create(filename)?),
    };
    target.write_all(&target_code.into_bytes())?;

    Ok(())
}
