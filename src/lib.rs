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
        SerializeJson(#[from] serde_json::Error), // source and Display delegate to anyhow::Error
        #[error(transparent)]
        Battery(#[from] battery::Error), // source and Display delegate to anyhow::Error
        #[error(transparent)]
        DynamicFormat(#[from] strfmt::FmtError), // source and Display delegate to anyhow::Error
        #[error(transparent)]
        StaticFormat(#[from] std::fmt::Error), // source and Display delegate to anyhow::Error
        #[error(transparent)]
        MsgRecv(#[from] std::sync::mpsc::RecvError), // source and Display delegate to anyhow::Error
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

    use crate::{theme::Color, PbStatusError, Result};
    use serde_derive::{Deserialize, Serialize};
    use std::{
        fs::{File, OpenOptions},
        io::prelude::*,
        path::PathBuf,
    };
    use toml;

    #[derive(Default, Serialize, Deserialize)]
    pub struct Config {
        pub battery: BatteryConfig,
        pub cpu: CpuConfig,
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
        pub format: String,
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
                format: "{label} {value}%".into(),
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

    #[derive(Serialize, Deserialize, Clone)]
    pub struct CpuConfig {
        pub format: String,
        pub low: CpuStateConfig,
        pub mid: CpuStateConfig,
        pub high: CpuStateConfig,
    }

    impl Default for CpuConfig {
        fn default() -> Self {
            Self {
                format: "{load_label} {load}% | {temp_label} {temp}{temp_unit}°C".into(),
                low: CpuStateConfig {
                    load_label: "".into(),
                    temp_label: "".into(),
                    color: Some(Color("#aeb3bb".into())),
                    background: Some(Color("#2e3440".into())),
                    threshold: Some(33.3),
                },
                mid: CpuStateConfig {
                    load_label: "".into(),
                    temp_label: "".into(),
                    color: Some(Color("#ebcb8b".into())),
                    background: Some(Color("#2e3440".into())),
                    threshold: Some(66.6),
                },
                high: CpuStateConfig {
                    load_label: "".into(),
                    temp_label: "".into(),
                    color: Some(Color("#bf616a".into())),
                    background: Some(Color("#2e3440".into())),
                    threshold: None,
                },
            }
        }
    }

    impl CpuConfig {}

    #[derive(Serialize, Deserialize, Clone)]
    pub struct CpuStateConfig {
        pub load_label: String,
        pub temp_label: String,
        pub color: Option<Color>,
        pub background: Option<Color>,
        pub threshold: Option<f32>,
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

        use std::{collections::HashMap, fmt::Display};

        use super::Module;
        use crate::{config::*, PbStatusError};
        use serde_derive::{Deserialize, Serialize};
        use strfmt::strfmt;

        #[derive(Deserialize, Serialize)]
        pub struct BatteryStatus {
            pub full_text: String,
            pub state_name: String,
            pub state: BatteryStateConfig,
        }

        impl Display for BatteryStatus {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_fmt(format_args!("{}", self.full_text))
            }
        }

        pub struct Mod {}

        impl Module<'_> for Mod {
            type Status = BatteryStatus;
            type Error = crate::PbStatusError;

            fn run(&self, cfg: Config) -> Result<Self::Status, Self::Error> {
                let manager = battery::Manager::new()?;
                let battery = manager.batteries()?.next();
                match battery {
                    None => Err(PbStatusError::ModuleNotFound("battery".into())),
                    Some(r) => {
                        let b = r?;
                        let soc = b.state_of_charge().value;

                        let percentage = soc * 100f32;
                        let (sn, bs_config) = match percentage {
                            p if p < cfg.battery.critical.threshold => {
                                ("critical", cfg.battery.critical)
                            }
                            p if p < cfg.battery.low.threshold => ("low", cfg.battery.low),
                            p if p < cfg.battery.normal.threshold => ("normal", cfg.battery.normal),
                            p if p < cfg.battery.high.threshold => ("high", cfg.battery.high),
                            p if p <= cfg.battery.full.threshold => ("full", cfg.battery.full),
                            _ => ("NaN", cfg.battery.full),
                        };
                        let label = match b.state() {
                            battery::State::Charging => &bs_config.charging_label,
                            _ => &bs_config.label,
                        };
                        let mut values = HashMap::new();
                        let value = format!("{}", percentage.clone());
                        values.insert("label".into(), label.into());
                        values.insert("value".into(), value);
                        let out = strfmt(&cfg.battery.format, &values)?;

                        Ok(BatteryStatus {
                            full_text: out.clone(),
                            state_name: sn.to_string(),
                            state: bs_config.clone(),
                        })
                    }
                }
            }
        }
    }

    pub mod cpu {
        use super::Module;
        use crate::config::*;
        use serde_derive::{Deserialize, Serialize};
        use std::{collections::HashMap, fmt::Display, thread};
        use strfmt::strfmt;
        use systemstat::{Duration, Platform, System};

        #[derive(Deserialize, Serialize)]
        pub struct CpuStatus {
            pub full_text: String,
            pub load: f32,
            pub temp: f32,
            pub state_name: String,
            pub state: CpuStateConfig,
        }

        impl Display for CpuStatus {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_fmt(format_args!("{}", self.full_text))
            }
        }

        pub struct Mod {}

        impl Module<'_> for Mod {
            type Status = CpuStatus;
            type Error = crate::PbStatusError;

            fn run(&self, cfg: Config) -> Result<Self::Status, Self::Error> {
                let sys = System::new();
                thread::sleep(Duration::from_secs(1));
                let cpu = sys.cpu_load_aggregate()?.done()?;
                let percentage =
                    (cpu.user + cpu.nice + cpu.system + cpu.interrupt + cpu.idle) * 100.0;

                let (tn, cs_config) = match percentage {
                    p if cfg.cpu.low.threshold.is_some() && p < cfg.cpu.low.threshold.unwrap() => {
                        ("low", cfg.cpu.low)
                    }
                    p if cfg.cpu.mid.threshold.is_some() && p < cfg.cpu.mid.threshold.unwrap() => {
                        ("mid", cfg.cpu.mid)
                    }
                    _ => ("high", cfg.cpu.high),
                };

                let mut values = HashMap::new();
                let load = format!("{}", percentage.clone());
                let temp = sys.cpu_temp()?;
                values.insert("load_label".into(), cs_config.load_label.clone());
                values.insert("load".into(), load);
                values.insert("temp_label".into(), cs_config.temp_label.clone());
                values.insert("temp".into(), temp.to_string());
                let out = strfmt(&cfg.cpu.format, &values)?;

                Ok(CpuStatus {
                    full_text: out.clone(),
                    load: percentage,
                    temp: temp.clone(),
                    state_name: tn.to_string(),
                    state: cs_config.clone(),
                })
            }
        }
    }
}
