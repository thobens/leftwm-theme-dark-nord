use std::path::PathBuf;

use leftwm_theme_dark_nord::{
    config::Config,
    modules::{battery as battery_mod, Module},
    Result,
};

use clap::{App, IntoApp, Parser, ValueHint};
use clap_generate::{
    generate,
    generators::{Bash, Elvish, Fish, PowerShell, Zsh},
    Generator,
};

#[derive(Parser, Debug)]
#[clap(name = "pb-status", author, version, about)]
struct StatusBarApp {
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
}

impl ModuleArgs {
    fn run(&self, cfg: Config) -> Result<()> {
        match self.module.as_str() {
            "battery" => {
                let bm = battery_mod::Mod {};
                bm.run(cfg)
            }
            _ => {
                println!("No module named {}", &self.module);
                Ok(())
            }
        }
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
fn main() -> Result<()> {
    let pb =
        PathBuf::from("/home/andreasd/.config/leftwm/themes/current/.config/pb-status/config.toml");
    let cfg = if !Config::exists(pb.clone()) {
        let cfg = Config::default();
        cfg.write_config(pb.clone())?;
        cfg
    } else {
        Config::try_from(pb.clone())?
    };

    let main = StatusBarApp::parse();
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
