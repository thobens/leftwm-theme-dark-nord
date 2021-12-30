mod errors {
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum PbStatusError {
        #[error(transparent)]
        Io(#[from] std::io::Error), // source and Display delegate to anyhow::Error
        #[error(transparent)]
        ParseToml(#[from] toml::de::Error), // source and Display delegate to anyhow::Error
        #[error(transparent)]
        SerializeToml(#[from] toml::ser::Error), // source and Display delegate to anyhow::Error
        #[error(transparent)]
        Battery(#[from] battery::Error), // source and Display delegate to anyhow::Error
        #[error("Module not found: {0}")]
        ModuleNotFound(String),
    }
}

mod result {
    use crate::PbStatusError;

    pub type Result<T> = std::result::Result<T, PbStatusError>;
}

pub use errors::PbStatusError;
pub use result::Result;

pub mod config {

    use crate::PbStatusError;
    use crate::{theme::Color, Result};
    use serde_derive::{Deserialize, Serialize};
    use std::fs::OpenOptions;
    use std::io::prelude::*;
    use std::{fs::File, path::PathBuf};
    use toml;

    #[derive(Default, Serialize, Deserialize)]
    pub struct Config {
        pub battery: BatteryConfig,
    }

    impl Config {
        pub fn exists(pb: PathBuf) -> bool {
            pb.as_path().exists()
        }

        pub fn write_config(&self, pb: PathBuf) -> Result<()> {
            let s = toml::to_string_pretty(&self)?;
            let mut file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .append(false)
                .open(pb.as_path())?;
            file.write_all(s.as_bytes())?;

            Ok(())
        }
    }

    impl TryFrom<PathBuf> for Config {
        type Error = PbStatusError;

        fn try_from(pb: PathBuf) -> Result<Config> {
            let mut contents = String::new();
            let mut file = File::open(pb)?;
            file.read_to_string(&mut contents)?;
            let cfg = toml::from_str(&contents)?;
            Ok(cfg)
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct BatteryConfig {
        pub critical: BatteryStateConfig,
        pub low: BatteryStateConfig,
        pub normal: BatteryStateConfig,
        pub high: BatteryStateConfig,
        pub full: BatteryStateConfig,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct BatteryStateConfig {
        pub label: String,
        pub charging_label: String,
        pub color: Color,
        pub threshold: f32,
    }

    impl Default for BatteryConfig {
        fn default() -> Self {
            Self {
                critical: BatteryStateConfig {
                    color: Color("#bf616a".into()),
                    label: "".into(),
                    charging_label: "".into(),
                    threshold: 5_f32,
                },
                low: BatteryStateConfig {
                    color: Color("#bf616a".into()),
                    label: "".into(),
                    charging_label: "".into(),
                    threshold: 25_f32,
                },
                normal: BatteryStateConfig {
                    color: Color("#e5e9f0".into()),
                    label: "".into(),
                    charging_label: "".into(),
                    threshold: 75_f32,
                },
                high: BatteryStateConfig {
                    color: Color("#a3be8c".into()),
                    label: "".into(),
                    charging_label: "".into(),
                    threshold: 98_f32,
                },
                full: BatteryStateConfig {
                    color: Color("#a3be8c".into()),
                    label: "".into(),
                    charging_label: "".into(),
                    threshold: 100_f32,
                },
            }
        }
    }
}

pub mod theme {
    use serde_derive::{Deserialize, Serialize};
    use std::fmt::Display;

    #[derive(Deserialize, Serialize, Clone)]
    pub struct Color(pub String);
    // pub struct Color(pub &'a str);

    impl Display for Color {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&self.0)
        }
    }

    pub struct PbPaint {}

    impl PbPaint {
        pub fn color(s: &str, color: Color) -> String {
            format!("%{{F{}}}{}%{{F-}}", color, s)
        }
    }
}

pub mod modules {
    use serde::{Deserialize, Serialize};

    use crate::config::Config;

    pub trait Module<'de> {
        type Status: Deserialize<'de> + Serialize;
        type Error;
        fn run(&self, cfg: Config) -> Result<Self::Status, Self::Error>;
    }

    pub mod battery {

        use serde_derive::{Deserialize, Serialize};

        use super::Module;
        use crate::{config::*, PbStatusError};

        #[derive(Deserialize, Serialize)]
        pub struct BatteryStatus {
            pub full_text: String,
            pub theme: BatteryStateConfig,
        }

        pub struct Mod {}

        impl Module<'_> for Mod {
            type Status = BatteryStatus;
            type Error = crate::PbStatusError;

            fn run(&self, cfg: Config) -> Result<Self::Status, Self::Error> {
                let manager = battery::Manager::new()?;
                let battery = manager.batteries()?.next();
                match battery {
                    None => {
                        println!("NaN");
                        Err(PbStatusError::ModuleNotFound("battery".into()))
                    }
                    Some(r) => {
                        let b = r?;
                        let soc = b.state_of_charge();

                        let percentage = soc * 100f32;
                        let bs_config = match percentage {
                            p if p.value < cfg.battery.critical.threshold => cfg.battery.critical,
                            p if p.value < cfg.battery.low.threshold => cfg.battery.low,
                            p if p.value < cfg.battery.normal.threshold => cfg.battery.normal,
                            p if p.value < cfg.battery.high.threshold => cfg.battery.high,
                            p if p.value <= cfg.battery.full.threshold => cfg.battery.full,
                            _ => cfg.battery.full,
                        };
                        let label = match b.state() {
                            battery::State::Charging => &bs_config.charging_label,
                            _ => &bs_config.label,
                        };
                        let out = format!("{} {}%", label, percentage.clone().value);
                        let status = BatteryStatus {
                            full_text: out.clone(),
                            theme: bs_config.clone(),
                        };
                        println!("{}", out);
                        Ok(status)
                    }
                }
            }

            // pub mod wifi {}
        }
    }
}
