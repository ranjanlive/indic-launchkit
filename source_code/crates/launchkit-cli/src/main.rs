use clap::{Parser, Subcommand};
use launchkit_core as core;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "launchkit",
    version,
    about = "Offline release artwork and APK inspection for people and agents"
)]
struct Args {
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Init {
        directory: PathBuf,
        #[arg(long)]
        title: String,
        #[arg(long, default_value = "")]
        package: String,
        #[arg(long, default_value = "aurora")]
        template: String,
    },
    Add {
        #[arg(long)]
        project: PathBuf,
        #[arg(long)]
        icon: Option<PathBuf>,
        #[arg(long)]
        screenshot: Option<PathBuf>,
        #[arg(long)]
        apk: Option<PathBuf>,
        #[arg(long, default_value = "")]
        caption: String,
    },
    Inspect {
        apk: PathBuf,
    },
    Validate {
        #[arg(long)]
        project: PathBuf,
    },
    Preview {
        #[arg(long)]
        project: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long, default_value = "screenshot")]
        kind: String,
        #[arg(long, default_value_t = 0)]
        index: usize,
    },
    Export {
        #[arg(long)]
        project: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        include_apk: bool,
    },
    Templates,
    Schema,
}
fn run(command: Command) -> core::Result<(&'static str, serde_json::Value)> {
    use serde_json::to_value as value;
    match command {
        Command::Init {
            directory,
            title,
            package,
            template,
        } => Ok((
            "init",
            value(core::init(&directory, &title, &package, &template)?).unwrap(),
        )),
        Command::Add {
            project,
            icon,
            screenshot,
            apk,
            caption,
        } => {
            let inputs = [("icon", icon), ("screenshot", screenshot), ("apk", apk)];
            if inputs.iter().filter(|(_, p)| p.is_some()).count() != 1 {
                return Err("Supply exactly one of --icon, --screenshot or --apk".into());
            }
            let (kind, path) = inputs.into_iter().find(|(_, p)| p.is_some()).unwrap();
            Ok((
                "add",
                value(core::import(&project, &path.unwrap(), kind, &caption)?).unwrap(),
            ))
        }
        Command::Inspect { apk } => Ok(("inspect", value(core::apk::inspect(&apk)?).unwrap())),
        Command::Validate { project } => {
            let p = core::load(&project)?;
            core::validate(&project, &p, true)?;
            Ok(("validate", serde_json::json!({"valid":true,"project":p})))
        }
        Command::Preview {
            project,
            output,
            kind,
            index,
        } => {
            let p = core::load(&project)?;
            core::validate(&project, &p, true)?;
            let bytes = core::render::png(&project, &p, &kind, index, true)?;
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&output)
                .map_err(|e| e.to_string())?;
            file.write_all(&bytes).map_err(|e| e.to_string())?;
            Ok((
                "preview",
                serde_json::json!({"output":output,"bytes":bytes.len(),"sha256":core::hash_file(&output)?}),
            ))
        }
        Command::Export {
            project,
            output,
            dry_run,
            include_apk,
        } => {
            let p = core::load(&project)?;
            Ok((
                "export",
                value(core::export(&project, &p, &output, dry_run, include_apk)?).unwrap(),
            ))
        }
        Command::Templates => Ok((
            "templates",
            serde_json::json!([{"id":"aurora","title":"Aurora","colors":["#EEEAFB","#6451D2"]},{"id":"paper","title":"Paper","colors":["#FFF5E3","#AC612C"]},{"id":"midnight","title":"Midnight","colors":["#111B30","#8CD6DB"]}]),
        )),
        Command::Schema => Ok(("schema", core::schema())),
    }
}
fn main() {
    let parsed = match Args::try_parse() {
        Ok(v) => v,
        Err(e) => {
            if e.use_stderr() && std::env::args().any(|a| a == "--json") {
                println!(
                    "{}",
                    serde_json::json!({"schema_version":1,"ok":false,"command":"arguments","error":{"code":"invalid_arguments","message":e.to_string()}})
                );
                std::process::exit(2)
            }
            e.exit()
        }
    };
    match run(parsed.command) {
        Ok((command, data)) => {
            if parsed.json {
                println!(
                    "{}",
                    serde_json::json!({"schema_version":1,"ok":true,"command":command,"data":data})
                )
            } else {
                println!("{}", serde_json::to_string_pretty(&data).unwrap())
            }
        }
        Err(message) => {
            if parsed.json {
                println!(
                    "{}",
                    serde_json::json!({"schema_version":1,"ok":false,"command":"operation","error":{"code":"operation_failed","message":message}})
                )
            } else {
                eprintln!("Error: {message}")
            }
            std::process::exit(1)
        }
    }
}
