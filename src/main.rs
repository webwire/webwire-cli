use std::collections::HashSet;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::path::{Path, PathBuf};

use clap::{App, Arg, ArgMatches, SubCommand};

use webwire_cli::codegen;
use webwire_cli::idl;
use webwire_cli::schema;

type GenFn = fn(&schema::Document) -> String;

const LANGUAGES: &[(&str, &str, GenFn)] = &[
    ("rust", "Rust", codegen::rust::gen),
    ("ts", "TypeScript", codegen::ts::gen),
];

fn get_gen_fn(lang: &str) -> Option<GenFn> {
    LANGUAGES
        .iter()
        .find(|(name, _, _)| &lang == name)
        .map(|(_, _, fn_)| *fn_)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(&*format!(
            "{}\n\nFor more information please visit {}",
            env!("CARGO_PKG_DESCRIPTION"),
            env!("CARGO_PKG_HOMEPAGE")
        ))
        .subcommand(
            SubCommand::with_name("gen")
                .about("Generate source")
                .arg(
                    Arg::with_name("language")
                        .required(true)
                        .validator(|lang| match get_gen_fn(&lang) {
                            Some(_) => Ok(()),
                            None => Err(format!("Unsupported language: {}", lang)),
                        })
                        .long_help(&format!(
                            "Available choices are:\n{}",
                            LANGUAGES
                                .iter()
                                .map(|(name, description, _)| format!(
                                    "        {}: {}",
                                    name, description
                                ))
                                .collect::<Vec<_>>()
                                .join("\n")
                        )),
                )
                .arg(Arg::with_name("source"))
                .arg(Arg::with_name("target")),
        )
        .get_matches();

    if let Some(args) = matches.subcommand_matches("gen") {
        cmd_gen(args)
    } else {
        println!("{}", matches.usage());
        Ok(())
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

fn cmd_gen(args: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let gen_fn = get_gen_fn(args.value_of("language").unwrap()).unwrap();

    let path = match args.value_of("source") {
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
            .map(|p| p.parent())
            .flatten()
            .ok_or_else(|| "base_dir could not be determined from source")?;
        let mut included_files: HashSet<String> = HashSet::new();
        let mut includes = idocs[0].includes.clone();
        while !includes.is_empty() {
            let include = includes.remove(0);
            if included_files.contains(&include.filename) {
                continue;
            }
            included_files.insert(include.filename.clone());
            let source = Source::File(base_dir.join(include.filename));
            let idoc = idl::parse_document(&source.read()?).map_err(|e| format!("{}", e))?;
            includes.extend(idoc.includes.iter().cloned());
            idocs.push(idoc);
        }
    }

    // Convert IDL to Schema
    let doc = schema::Document::from_idl(idocs.iter())?;

    // Call code generator function
    let target_code = gen_fn(&doc);

    // Write target file
    let mut target: Box<dyn Write> = match args.value_of("target") {
        None | Some("--") => Box::new(stdout()),
        Some(filename) => Box::new(File::create(filename)?),
    };
    target.write_all(&target_code.into_bytes())?;

    Ok(())
}
