use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};
use sqlx::{mysql::MySqlRow, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Note {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub content: String,
    pub tags: Option<String>, // comma-separated tags, simple approach
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Clone)]
pub struct CreateNoteRequest {
    pub title: String,
    pub content: String,
    pub tags: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateNoteRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub tags: Option<String>,
}

// Manual Serialize for Note
impl serde::Serialize for Note {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut st = serializer.serialize_struct("Note", 8)?;
        st.serialize_field("id", &self.id)?;
        st.serialize_field("user_id", &self.user_id)?;
        st.serialize_field("title", &self.title)?;
        st.serialize_field("content", &self.content)?;
        st.serialize_field("tags", &self.tags)?;
        st.serialize_field("created_at", &self.created_at)?;
        st.serialize_field("updated_at", &self.updated_at)?;
        st.end()
    }
}

// Manual Deserialize for CreateNoteRequest
impl<'de> serde::Deserialize<'de> for CreateNoteRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        struct CreateVisitor;
        impl<'de> Visitor<'de> for CreateVisitor {
            type Value = CreateNoteRequest;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a map with title, content, optional tags")
            }
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut title: Option<String> = None;
                let mut content: Option<String> = None;
                let mut tags: Option<String> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "title" => title = Some(map.next_value()?),
                        "content" => content = Some(map.next_value()?),
                        "tags" => tags = Some(map.next_value()?),
                        _ => { let _ = map.next_value::<serde::de::IgnoredAny>()?; }
                    }
                }
                let title = title.ok_or_else(|| de::Error::missing_field("title"))?;
                let content = content.ok_or_else(|| de::Error::missing_field("content"))?;
                Ok(CreateNoteRequest { title, content, tags })
            }
        }
        deserializer.deserialize_map(CreateVisitor)
    }
}

// Manual Deserialize for UpdateNoteRequest
impl<'de> serde::Deserialize<'de> for UpdateNoteRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};
        struct UpdateVisitor;
        impl<'de> Visitor<'de> for UpdateVisitor {
            type Value = UpdateNoteRequest;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a map with optional title, content, tags")
            }
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut title: Option<String> = None;
                let mut content: Option<String> = None;
                let mut tags: Option<String> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "title" => title = Some(map.next_value()?),
                        "content" => content = Some(map.next_value()?),
                        "tags" => tags = Some(map.next_value()?),
                        _ => { let _ = map.next_value::<serde::de::IgnoredAny>()?; }
                    }
                }
                Ok(UpdateNoteRequest { title, content, tags })
            }
        }
        deserializer.deserialize_map(UpdateVisitor)
    }
}

// Manual FromRow for Note
impl<'r> sqlx::FromRow<'r, MySqlRow> for Note {
    fn from_row(row: &'r MySqlRow) -> Result<Self, sqlx::Error> {
        let offset = FixedOffset::east_opt(8 * 3600).unwrap();
        let created_naive: NaiveDateTime = row.try_get("created_at")?;
        let updated_naive: NaiveDateTime = row.try_get("updated_at")?;
        let created_at = offset.from_local_datetime(&created_naive).single().unwrap_or_else(|| {
            let utc_dt = chrono::DateTime::<Utc>::from_naive_utc_and_offset(created_naive, Utc);
            utc_dt.with_timezone(&offset)
        });
        let updated_at = offset.from_local_datetime(&updated_naive).single().unwrap_or_else(|| {
            let utc_dt = chrono::DateTime::<Utc>::from_naive_utc_and_offset(updated_naive, Utc);
            utc_dt.with_timezone(&offset)
        });
        Ok(Note {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            title: row.try_get("title")?,
            content: row.try_get("content")?,
            tags: row.try_get("tags")?,
            created_at,
            updated_at,
        })
    }
}