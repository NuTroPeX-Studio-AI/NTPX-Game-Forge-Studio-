use gamespec::{from_json, json_schema, migrate_json, to_pretty_json, DependencyGraph};
use gamespec_compiler::compile_manifest;
use serde_json::json;
use std::{env, error::Error, fs, process};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [command, path] if command == "validate" => {
            let input = fs::read_to_string(path)?;
            let spec = from_json(&input)?;
            println!("VALID {}", spec.project.id);
        }
        [command, path] if command == "inspect" => {
            let input = fs::read_to_string(path)?;
            let spec = from_json(&input)?;
            let summary = json!({
                "project": spec.project.id,
                "title": spec.project.title,
                "modules": spec.architecture.modules.len(),
                "features": spec.features.len(),
                "entities": spec.entities.len(),
                "assets": spec.assets.len(),
                "tests": spec.tests.len()
            });
            println!("{}", serde_json::to_string_pretty(&summary)?);
        }
        [command, path] if command == "graph" => {
            let input = fs::read_to_string(path)?;
            let spec = from_json(&input)?;
            println!("{}", serde_json::to_string_pretty(&DependencyGraph::from_spec(&spec))?);
        }
        [command, path] if command == "compile" => {
            let input = fs::read_to_string(path)?;
            let spec = from_json(&input)?;
            println!("{}", serde_json::to_string_pretty(&compile_manifest(&spec)?)?);
        }
        [command, path] if command == "migrate" => {
            let input = fs::read_to_string(path)?;
            println!("{}", migrate_json(&input)?);
        }
        [command] if command == "schema" => {
            println!("{}", serde_json::to_string_pretty(&json_schema()?)?);
        }
        [command, path] if command == "normalize" => {
            let input = fs::read_to_string(path)?;
            let spec = from_json(&input)?;
            println!("{}", to_pretty_json(&spec)?);
        }
        _ => {
            return Err("usage: gamespec-compiler <validate|inspect|graph|compile|migrate|normalize> <file> | schema".into());
        }
    }
    Ok(())
}
