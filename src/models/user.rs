use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<FixedOffset>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub avatar: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_info: UserInfo,
}

// Manual FromRow implementation for custom DateTime handling
impl<'r> sqlx::FromRow<'r, MySqlRow> for User {
    fn from_row(row: &'r MySqlRow) -> Result<Self, sqlx::Error> {
        let naive: NaiveDateTime = row.try_get("created_at")?;
        let offset = FixedOffset::east_opt(8 * 3600).unwrap();
        let created_at = offset
            .from_local_datetime(&naive)
            .single()
            .unwrap_or_else(|| {
                // Fallback: interpret as UTC and convert to +08:00
                let utc_dt =
                    chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(naive, chrono::Utc);
                utc_dt.with_timezone(&offset)
            });
        Ok(User {
            id: row.try_get("id")?,
            username: row.try_get("username")?,
            email: row.try_get("email")?,
            password_hash: row.try_get("password_hash")?,
            created_at,
        })
    }
}
