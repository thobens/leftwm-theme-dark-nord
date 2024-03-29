use std::{fmt::Display, path::PathBuf};

use clap::{App, IntoApp, Parser, ValueHint};
use clap_generate::{
    generate,
    generators::{Bash, Elvish, Fish, PowerShell, Zsh},
    Generator,
};
use leftwm_theme_dark_nord::{
    config::Config,
    modules::{
        battery::{self as battery_mod},
        cpu, Module,
    },
    PbStatusError, Result,
};
use serde::Serialize;

use tokio;

#[derive(Debug)]
enum OutputFormat {
    Plain,
    Json,
}

impl From<String> for OutputFormat {
    fn from(s: String) -> Self {
        match s.as_str() {
            "plain" => Self::Plain,
            "json" => Self::Json,
            _ => panic!("OutputFormat {} unknown", s),
        }
    }
}

#[derive(Parser, Debug)]
#[clap(name = "lefty-status", author, version, about)]
struct StatusBarApp {
    #[clap(short = 'c', long = "config", value_hint = ValueHint::FilePath, default_value = "~/.config/leftwm/themes/current/.config/lefty-status/config.toml")]
    cfg_path: PathBuf,
    #[clap(subcommand)]
    cmd: StatusBarCmd,
}

#[derive(Parser, Debug)]
enum StatusBarCmd {
    Module(ModuleArgs),
    Completion(Completion),
}

#[derive(clap::Args, Debug)]
struct ModuleArgs {
    /// The name of the module to execute
    #[clap(value_name = "MODULE")]
    module: String,
    #[clap(short = 'o', long = "format", possible_values = ["json", "plain"], default_value = "plain")]
    format: String,
}

impl ModuleArgs {
    fn run(&self, cfg: Config) -> Result<()> {
        match self.module.as_str() {
            "battery" => {
                let bm = battery_mod::Mod {};
                serialize_module_result(self.format.clone().into(), bm.run(cfg)?).map(|s| {
                    println!("{}", s);
                    ()
                })
            }
            "cpu" => {
                let cm = cpu::Mod {};
                serialize_module_result(self.format.clone().into(), cm.run(cfg)?).map(|s| {
                    println!("{}", s);
                    ()
                })
            }
            _ => {
                println!("No module named {}", &self.module);
                Err(PbStatusError::ModuleNotFound("battery".into()))
            }
        }
    }
}

fn serialize_module_result<T: Serialize + Display>(
    format: OutputFormat,
    result: T,
) -> Result<String> {
    match format {
        OutputFormat::Plain => Ok(format!("{}", result)),
        OutputFormat::Json => serde::__private::Ok(serde_json::to_string(&result)?),
    }
}

#[derive(clap::Args, Debug)]
struct Completion {
    /// Specifies for which type of shell the completion script is generated.
    /// Must be one of `"bash"`, `"elvish"`, `"fish"`, `"powershell"` or `"zsh"`.
    #[clap(long = "for", name = "for", possible_values = &["bash", "elvish", "fish", "powershell", "zsh"])]
    generate_for: String,
}

impl Completion {
    fn print_completions<G: Generator>(gen: G, app: &mut App) -> Result<()> {
        generate::<G, _>(gen, app, app.get_name().to_string(), &mut std::io::stdout());
        Ok(())
    }

    pub fn run(&self, app: &mut App) -> Result<()> {
        match self.generate_for.as_str() {
            "bash" => Self::print_completions(Bash, app),
            "elvish" => Self::print_completions(Elvish, app),
            "fish" => Self::print_completions(Fish, app),
            "powershell" => Self::print_completions(PowerShell, app),
            "zsh" => Self::print_completions(Zsh, app),
            _ => panic!("Unknown generator"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let main = StatusBarApp::parse();
    let pb = PathBuf::from(main.cfg_path);
    let cfg = if !Config::exists(pb.clone()) {
        let cfg = Config::default();
        cfg.write_config(pb.clone())?;
        cfg
    } else {
        Config::try_from(pb.clone())?
    };
    let mut app = StatusBarCmd::into_app();
    let result = match main.cmd {
        StatusBarCmd::Completion(c) => c.run(&mut app),
        StatusBarCmd::Module(m) => m.run(cfg),
    };

    if let Err(err) = result {
        return Err(err);
    }

    Ok(())
}
