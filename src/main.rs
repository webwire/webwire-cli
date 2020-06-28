use std::fs::File;
use std::io::{stdin, stdout, Read, Write};

use clap::{App, Arg, ArgMatches, SubCommand};

use webwire::codegen;
use webwire::idl;
use webwire::schema;

type GenFn = fn(&schema::Document) -> String;

const LANGUAGES: &[(&str, &str, GenFn)] = &[("rust", "Rust", codegen::rust::gen)];

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
        matches.usage();
        Ok(())
    }
}

fn cmd_gen(args: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let gen_fn = get_gen_fn(args.value_of("language").unwrap()).unwrap();

    // Read source file
    let mut source: Box<dyn Read> = match args.value_of("source") {
        None | Some("--") => Box::new(stdin()),
        Some(filename) => Box::new(File::open(filename)?),
    };
    let mut source_code = String::new();
    {
        source.read_to_string(&mut source_code)?;
    }

    // Parse IDL
    let idl = idl::parse_document(&source_code).map_err(|e| format!("{}", e))?;

    // Convert IDL to Schema
    let doc = schema::Document::from_idl(&idl)?;

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
