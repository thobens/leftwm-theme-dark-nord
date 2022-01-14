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

    use crate::{
        theme::{Rgba, Theme},
        PbStatusError, Result,
    };
    use serde_derive::{Deserialize, Serialize};
    use std::{
        fs::{File, OpenOptions},
        io::prelude::*,
        path::PathBuf,
    };
    use toml;

    #[derive(Default, Serialize, Deserialize)]
    pub struct Config {
        pub theme: Theme,
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
        pub separator: Option<String>,
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
        pub color: Rgba,
        pub threshold: f32,
    }

    impl Default for BatteryConfig {
        fn default() -> Self {
            Self {
                format: "${separator}{label} {value}%".into(),
                separator: None,
                critical: BatteryStateConfig {
                    color: "#bf616a".into(),
                    label: "".into(),
                    charging_label: "".into(),
                    threshold: 5_f32,
                },
                low: BatteryStateConfig {
                    color: "#bf616a".into(),
                    label: "".into(),
                    charging_label: "".into(),
                    threshold: 25_f32,
                },
                normal: BatteryStateConfig {
                    color: "#e5e9f0".into(),
                    label: "".into(),
                    charging_label: "".into(),
                    threshold: 75_f32,
                },
                high: BatteryStateConfig {
                    color: "#a3be8c".into(),
                    label: "".into(),
                    charging_label: "".into(),
                    threshold: 98_f32,
                },
                full: BatteryStateConfig {
                    color: "#a3be8c".into(),
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
        pub separator: Option<String>,
        pub low: CpuStateConfig,
        pub mid: CpuStateConfig,
        pub high: CpuStateConfig,
    }

    impl Default for CpuConfig {
        fn default() -> Self {
            Self {
                format: "{separator}{load_label} {load}% | {temp_label} {temp}{temp_unit}°C".into(),
                separator: None,
                low: CpuStateConfig {
                    load_label: "".into(),
                    temp_label: "".into(),
                    color: Some("#aeb3bb".into()),
                    background: Some("#2e3440".into()),
                    threshold: Some(33.3),
                },
                mid: CpuStateConfig {
                    load_label: "".into(),
                    temp_label: "".into(),
                    color: Some("#ebcb8b".into()),
                    background: Some("#2e3440".into()),
                    threshold: Some(66.6),
                },
                high: CpuStateConfig {
                    load_label: "".into(),
                    temp_label: "".into(),
                    color: Some("#bf616a".into()),
                    background: Some("#2e3440".into()),
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
        pub color: Option<Rgba>,
        pub background: Option<Rgba>,
        pub threshold: Option<f32>,
    }
}

pub mod theme {
    use serde_derive::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Clone, Default, Debug, PartialEq, Eq, Deserialize, Serialize)]
    pub struct Theme {
        pub colors: Colors,
        pub vars: HashMap<String, Value>,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
    pub enum Value {
        Rgba(Rgba),
        ColorName(String),
        Ref(String),
    }

    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Deserialize, Serialize)]
    pub struct Colors {
        normal: ColorScheme,
        bright: ColorScheme,
        dim: ColorScheme,
        fg: Rgba,
        bg: Rgba,
        fg_alt: Rgba,
        bg_alt: Rgba,
    }

    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Deserialize, Serialize)]
    pub struct ColorScheme {
        pub black: Rgba,
        pub red: Rgba,
        pub green: Rgba,
        pub yellow: Rgba,
        pub blue: Rgba,
        pub magenta: Rgba,
        pub cyan: Rgba,
        pub white: Rgba,
    }

    const RGBA_RED: u32 = 24;
    const RGBA_GREEN: u32 = 16;
    const RGBA_BLUE: u32 = 8;
    const RGBA_ALPHA: u32 = 0;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Rgba {
        raw: u32,
    }

    impl Rgba {
        pub fn new(r: u32, g: u32, b: u32, a: u32) -> Rgba {
            Rgba {
                raw: a | (b << RGBA_BLUE) | (g << RGBA_GREEN) | (r << RGBA_RED),
            }
        }

        pub fn component(&self, component: u32) -> u8 {
            let addr = 0x000000FF << component;
            let cv = (self.raw & addr) >> component;
            match cv.try_into() {
                Ok(c) => c,
                Err(_) => 0u8,
            }
        }

        pub fn red(&self) -> u8 {
            self.component(RGBA_RED)
        }

        pub fn green(&self) -> u8 {
            self.component(RGBA_GREEN)
        }

        pub fn blue(&self) -> u8 {
            self.component(RGBA_BLUE)
        }

        pub fn alpha(&self) -> u8 {
            self.component(RGBA_ALPHA)
        }

        pub fn int(&self) -> u32 {
            self.raw
        }
    }

    impl Default for Rgba {
        fn default() -> Self {
            Self::new(0, 0, 0, 255) // black
        }
    }

    impl From<u32> for Rgba {
        fn from(raw: u32) -> Self {
            Rgba { raw }
        }
    }

    impl From<Rgba> for u32 {
        fn from(rgba: Rgba) -> Self {
            rgba.int()
        }
    }

    impl From<&str> for Rgba {
        /// Converts a hex string to `Rgba`. The string should
        /// represent either a 3 or four byte hex value, starting with a hash.
        /// If the string s not valid, we return `Rgba::default()`;
        ///
        /// # Example
        ///
        /// ```rust
        /// let yellow = Rgba::from("#00FF00");
        /// let blue = Rgba::from("#0000ff");
        /// let transparent = Rgba::from("#00000000"); // black with 0 opacity
        /// let default = Rgba::from("unknown"); // black with 0 opacity
        /// assert_eq!(yellow, Rgba::new(0,255,0,255));
        /// assert_eq!(blue, Rgba::new(0,0,255,255));
        /// assert_eq!(transparent, Rgba::new(0,0,0,0));
        /// assert_eq!(default, Rgba::default());
        /// ```
        fn from(hex: &str) -> Self {
            if hex.len() < 1 || &hex[0..1] != "#" {
                return Self::default();
            }
            let hex = match hex.len() {
                7 => format!("{}FF", hex),
                _ => hex.to_string(),
            };
            return match u32::from_str_radix(&hex[1..], 16) {
                Ok(raw) => Rgba::from(raw),
                Err(_) => Rgba::default(),
            };
        }
    }

    impl From<Rgba> for String {
        /// Converts an `Rgba` to a hex string, prefixed with a "#"
        ///
        /// # Example
        ///
        /// ```rust
        /// let yellow = Rgba::from("#00ffff");
        /// let result: String = yellow.into();
        /// assert_eq!(result, "#00ff00ff".to_string());
        /// ```
        fn from(rgba: Rgba) -> Self {
            format!("#{:08x}", rgba.raw)
        }
    }

    #[cfg(feature = "manager")]
    impl From<Rgba> for tui::style::Color {
        fn from(rgba: Rgba) -> Self {
            tui::style::Color::Rgb(rgba.red(), rgba.green(), rgba.blue())
        }
    }

    use serde::{
        de::{self, Deserialize, Deserializer, Visitor},
        Serialize, Serializer,
    };
    use std::fmt;

    impl Serialize for Rgba {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let hex: String = (*self).into();
            serializer.serialize_str(&hex)
        }
    }

    impl<'de> Deserialize<'de> for Rgba {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct RgbaVisitor;

            impl<'de> Visitor<'de> for RgbaVisitor {
                type Value = Rgba;

                fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(Rgba::from(v.as_str()))
                }

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(Rgba::from(v))
                }

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("valid color value")
                }
            }

            deserializer.deserialize_string(RgbaVisitor)
        }
    }

    #[cfg(test)]
    mod test {
        use super::Rgba;

        #[test]
        fn test_rgba_from_hex_str() {
            let yellow = Rgba::from("#00FF00FF");
            let blue = Rgba::from("#0000ff");
            let transparent = Rgba::from("#00000000"); // black with 0 opacity
            let default = Rgba::from("unknown"); // black with 0 opacity
            assert_eq!(yellow, Rgba::new(0, 255, 0, 255));
            assert_eq!(blue, Rgba::new(0, 0, 255, 255));
            assert_eq!(transparent, Rgba::new(0, 0, 0, 0));
            assert_eq!(default, Rgba::default());
        }

        #[test]
        fn test_rgba_into_hex_string() {
            let yellow = Rgba::from("#00FF00");
            let yellow: String = yellow.into();
            let red_semi_transparent = Rgba::from("#FF000090");
            let red_semi_transparent: String = red_semi_transparent.into();
            assert_eq!(yellow, "#00ff00ff".to_string());
            assert_eq!(red_semi_transparent, "#ff000090".to_string());
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
            pub value: i32,
            pub state_name: String,
            pub separator: String,
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
                        let value = format!("{:.0}", percentage.clone());
                        let sep = cfg.battery.separator.unwrap_or("".into());
                        values.insert("label".into(), label.into());
                        values.insert("value".into(), value);
                        values.insert("separator".into(), sep.clone());
                        let out = strfmt(&cfg.battery.format, &values)?;

                        Ok(BatteryStatus {
                            full_text: out.clone(),
                            value: percentage.clone().round() as i32,
                            state_name: sn.to_string(),
                            separator: sep,
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
            pub load: i32,
            pub temp: i32,
            pub state_name: String,
            pub separator: String,
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
                let sep = cfg.cpu.separator.unwrap_or("".into());
                values.insert("load_label".into(), cs_config.load_label.clone());
                values.insert("load".into(), load);
                values.insert("temp_label".into(), cs_config.temp_label.clone());
                values.insert("temp".into(), temp.to_string());
                values.insert("separator".into(), sep.clone());
                let out = strfmt(&cfg.cpu.format, &values)?;

                Ok(CpuStatus {
                    full_text: out.clone(),
                    load: percentage as i32,
                    temp: temp.clone() as i32,
                    state_name: tn.to_string(),
                    separator: sep,
                    state: cs_config.clone(),
                })
            }
        }
    }
}
