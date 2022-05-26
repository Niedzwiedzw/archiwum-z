use anyhow::Context;
use rust_decimal::Decimal;
use serde::{
    Deserialize,
    Serialize,
};
use uuid::Uuid;

use derive_more::From;
use std::default::Default;
use std::{
    path::Path,
    str::FromStr,
};
// pub mod db {
//     use std::{
//         fs::DirEntry,
//         path::PathBuf,
//     };

//     use super::*;
//     use anyhow::{
//         Context,
//         Result,
//     };
//     use itertools::Itertools;

//     fn db_location() -> Result<PathBuf> {
//         let executable = std::env::current_exe().context("finding current directory path")?;
//         let db_dir = executable
//             .parent()
//             .context(".exe is not in a directory")?
//             .join("archiwum");
//         if !db_dir.exists() {
//             std::fs::create_dir_all(&db_dir).context("creating ./archiwum folder")?;
//         }
//         Ok(db_dir)
//     }

//     fn entries() -> Result<Vec<RepairContract>> {
//         let location = db_location()?;
//         let entries = std::fs::read_dir(&location)
//             .with_context(|| format!("reading contents of {location:?}"))?
//             .into_iter()
//             .map(|e| e.with_context(|| format!("reading contents of {location:?}")))
//             .collect::<Result<Vec<_>>>()?;
//         let file_entries = entries.into_iter().filter_map(|e| match e.metadata() {
//             Ok(m) if m.is_file() => Some(e),
//             _ => None,
//         });

//         file_entries
//             .map(|entry| {
//                 RepairContract::from_file(&entry.path())
//                     .with_context(|| format!("reading contents of {entry:?}"))
//             })
//             .collect()
//     }
// }

pub struct Validated<T>(T);
use anyhow::Result;

use crate::AppTime;

pub trait Validate: Sized {
    fn check(&self) -> Result<()>;
    fn validated(self) -> Result<Validated<Self>> {
        self.check()?;
        Ok(Validated(self))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Company {
    pub name: String,
    pub tax_number: String,
    pub phone: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PrivateCustomer {
    pub name: String,
    pub phone: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, From)]
#[serde(untagged)]
pub enum Customer {
    PrivateCustomer(PrivateCustomer),
    Company(Company),
}

impl Default for Customer {
    fn default() -> Self {
        Self::PrivateCustomer(Default::default())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ReplacementPart {
    pub id: String,
    pub name: String,
    pub price: Decimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PerformedRepair {
    pub id: String,
    pub name: String,
    pub price: Decimal,
}

pub mod protocols {
    use rust_decimal::Decimal;

    use crate::AppTime;

    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct FinalProtocol {
        pub date: AppTime,
        pub final_price: Decimal,
        pub performed_repairs: Vec<PerformedRepair>,
        pub parts_replaced: Vec<ReplacementPart>,
    }

    impl Default for FinalProtocol {
        fn default() -> Self {
            Self {
                date: crate::now(),
                final_price: Default::default(),
                performed_repairs: Default::default(),
                parts_replaced: Default::default(),
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplacementDevice {
    pub device: Device,
    pub id: Uuid,
}

impl Default for ReplacementDevice {
    fn default() -> Self {
        Self {
            device: Default::default(),
            id: uuid::Uuid::new_v4(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, From, Default)]
pub struct Device {
    pub model_name: String,
    pub serial_number: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientContactEvent {
    pub date: AppTime,
    pub note: String,
}

impl Default for ClientContactEvent {
    fn default() -> Self {
        Self {
            date: crate::now(),
            note: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RepairContractInfo {
    pub customer: Customer,
    pub expected_repair_time_work_days: i64,
    pub prognosis_price: Decimal,
    pub description: Vec<String>,
    pub notes: String,
    pub visible_damages: Vec<String>,
}

/// this is the main app model
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepairContract {
    pub id: Uuid,
    pub date: AppTime,
    pub info: RepairContractInfo,
    pub client_contact_events: Vec<ClientContactEvent>,
    pub replacement_device: Option<ReplacementDevice>,
    pub final_protocol: Option<protocols::FinalProtocol>,
}

impl Default for RepairContract {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            date: crate::now(),
            info: Default::default(),
            client_contact_events: Default::default(),
            replacement_device: Default::default(),
            final_protocol: Default::default(),
        }
    }
}

impl FromStr for RepairContract {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s).with_context(|| "parsing contents")
    }
}

impl RepairContract {
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        std::fs::read_to_string(path)
            .with_context(|| format!("reading {path:?}"))
            .and_then(|contents| Self::from_str(&contents))
    }
}
