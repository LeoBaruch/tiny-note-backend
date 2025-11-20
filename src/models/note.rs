use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct Note {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub content: String,
    pub category: Option<String>,
    pub tags: Option<String>, // comma-separated tags, simple approach
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateNoteRequest {
    pub title: String,
    pub content: String,
    pub category: String,
    pub tags: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateNoteRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub tags: Option<String>,
    pub category: Option<String>,
}

// Manual FromRow for Note
impl<'r> sqlx::FromRow<'r, MySqlRow> for Note {
    fn from_row(row: &'r MySqlRow) -> Result<Self, sqlx::Error> {
        let offset = FixedOffset::east_opt(8 * 3600).unwrap();
        let created_naive: NaiveDateTime = row.try_get("created_at")?;
        let updated_naive: NaiveDateTime = row.try_get("updated_at")?;
        let created_at = offset
            .from_local_datetime(&created_naive)
            .single()
            .unwrap_or_else(|| {
                let utc_dt = chrono::DateTime::<Utc>::from_naive_utc_and_offset(created_naive, Utc);
                utc_dt.with_timezone(&offset)
            });
        let updated_at = offset
            .from_local_datetime(&updated_naive)
            .single()
            .unwrap_or_else(|| {
                let utc_dt = chrono::DateTime::<Utc>::from_naive_utc_and_offset(updated_naive, Utc);
                utc_dt.with_timezone(&offset)
            });
        Ok(Note {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            title: row.try_get("title")?,
            content: row.try_get("content")?,
            category: row.try_get("category")?,
            tags: row.try_get("tags")?,
            created_at,
            updated_at,
        })
    }
}
