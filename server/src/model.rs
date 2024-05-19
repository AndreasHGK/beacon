use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::file::FileId;

#[derive(Debug, Deserialize, Serialize)]
pub struct File {
    pub file_id: FileId,
    pub file_name: String,
    pub file_size: u64,
    pub upload_date: DateTime<Utc>,
}
