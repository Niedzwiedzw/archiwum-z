use super::*;
use crate::models::*;
use futures::stream::{
    StreamExt,
    TryStreamExt,
};
use serde::{
    Deserialize,
    Serialize,
};
use std::path::{
    Path,
    PathBuf,
};
use std::sync::Arc;
use tokio::fs::DirEntry;
use tokio::sync::RwLock;

use anyhow::Result;
use tracing::{
    info,
    instrument,
    warn,
};

pub trait FillForm: Sized {
    fn serialize(&self) -> String;
    fn deserialize(val: &str) -> Result<Self>;
}

impl FillForm for crate::AppTime {
    fn serialize(&self) -> String {
        self.to_string()
    }

    fn deserialize(val: &str) -> Result<Self> {
        val.parse().context("zła data")
    }
}

impl FillForm for String {
    fn serialize(&self) -> String {
        self.clone()
    }

    fn deserialize(val: &str) -> Result<Self> {
        Ok(val.to_owned())
    }
}

#[derive(Debug, Clone)]
struct Db {
    pub base_dir: PathBuf,
}
#[derive(Debug, Clone)]
pub struct Database {
    db: Arc<RwLock<Db>>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepairContractEntry {
    pub path: PathBuf,
    pub model: RepairContract,
}

impl RepairContractEntry {
    #[instrument(level = "debug")]
    pub async fn from_dir_entry(entry: DirEntry) -> Option<Result<Self>> {
        match entry.metadata().await {
            Ok(metadata)
                if metadata.is_file()
                    && entry
                        .file_name()
                        .as_os_str()
                        .to_str()
                        .map(|name| name.ends_with(".toml"))
                        .unwrap_or_default() =>
            {
                Some(Self::from_path(&entry.path()).await)
            }
            v => {
                warn!("ignoruję [{entry:#?}] :: {v:#?}");
                None
            }
        }
    }
    #[instrument(level = "debug")]
    pub async fn from_path(path: &Path) -> Result<Self> {
        let content = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("odczytywanie zawartości zlecenia z {path:?}"))?;
        let entry = tokio::task::block_in_place(|| {
            toml::from_str(&content)
                .with_context(|| format!("parsowanie zawartości pliku {path:?} :: {content}"))
                .map(|model| Self {
                    path: path.into(),
                    model,
                })
        })?;

        Ok(entry)
    }
}
impl Database {
    pub fn new(base_dir: PathBuf) -> Self {
        if !base_dir.exists() {
            tracing::warn!("utworzono folder [{base_dir:?}] bo nie istniał");
            std::fs::create_dir_all(&base_dir).expect("nie udalo sie utworzyć folderu");
        }
        tracing::info!("program używa archiwum z folderu [{base_dir:?}]");
        Self {
            db: Arc::new(RwLock::new(Db { base_dir })),
        }
    }

    #[instrument(skip(self))]
    pub async fn get_entries(&self) -> Result<Vec<RepairContractEntry>> {
        info!("getting entries");
        let db = self.db.write().await;
        let base_dir = db.base_dir.clone();
        let dir_entries: Vec<_> = tokio::fs::read_dir(&base_dir)
            .await
            .with_context(|| format!("odczytywanie plików z {:?}", base_dir))
            .map(tokio_stream::wrappers::ReadDirStream::new)?
            .try_collect()
            .await?;

        let entries: Vec<_> = futures::stream::iter(dir_entries)
            .map(RepairContractEntry::from_dir_entry)
            .buffer_unordered(FS_CONCURRENCY_LIMIT)
            .filter_map(|v| async { v })
            .try_collect()
            .await?;

        Ok(entries)
    }
    #[instrument(skip(self))]
    pub async fn create_entry(&self, model: RepairContract) -> Result<RepairContractEntry> {
        let db = self.db.write().await;
        let base_dir = db.base_dir.clone();
        let contract_time = model.date;
        let filename = format!("{contract_time}.repair-contract.toml");

        let path = base_dir.join(&filename);
        let entry = {
            let path = path.clone();
            RepairContractEntry { path, model }
        };
        let contents = tokio::task::block_in_place(|| toml::to_string_pretty(&entry))
            .context("serializacja modelu {model:#?}")?;
        tokio::fs::write(path, &contents)
            .await
            .with_context(|| format!("pisanie do pliku {filename:?}"))?;
        Ok(entry)
    }
}
