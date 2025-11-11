use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone};
use sqlx::{mysql::MySqlRow, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<FixedOffset>,
}

#[derive(Debug, Clone)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Clone)]
pub struct LoginResponse {
    pub token: String,
    pub user_info: UserInfo,
}

// Manual Serialize implementations to avoid proc-macro usage
impl serde::Serialize for User {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut st = serializer.serialize_struct("User", 5)?;
        st.serialize_field("id", &self.id)?;
        st.serialize_field("username", &self.username)?;
        st.serialize_field("email", &self.email)?;
        st.serialize_field("password_hash", &self.password_hash)?;
        st.serialize_field("created_at", &self.created_at)?;
        st.end()
    }
}

impl serde::Serialize for UserInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut st = serializer.serialize_struct("UserInfo", 3)?;
        st.serialize_field("id", &self.id)?;
        st.serialize_field("username", &self.username)?;
        st.serialize_field("email", &self.email)?;
        st.end()
    }
}

impl serde::Serialize for LoginResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut st = serializer.serialize_struct("LoginResponse", 2)?;
        st.serialize_field("token", &self.token)?;
        st.serialize_field("user_info", &self.user_info)?;
        st.end()
    }
}

// Manual Deserialize implementations for request bodies
impl<'de> serde::Deserialize<'de> for RegisterRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        struct RegVisitor;
        impl<'de> Visitor<'de> for RegVisitor {
            type Value = RegisterRequest;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a map with username, email, password")
            }
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut username: Option<String> = None;
                let mut email: Option<String> = None;
                let mut password: Option<String> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "username" => username = Some(map.next_value()?),
                        "email" => email = Some(map.next_value()?),
                        "password" => password = Some(map.next_value()?),
                        _ => {
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                let username = username.ok_or_else(|| de::Error::missing_field("username"))?;
                let email = email.ok_or_else(|| de::Error::missing_field("email"))?;
                let password = password.ok_or_else(|| de::Error::missing_field("password"))?;
                Ok(RegisterRequest {
                    username,
                    email,
                    password,
                })
            }
        }
        deserializer.deserialize_map(RegVisitor)
    }
}

impl<'de> serde::Deserialize<'de> for LoginRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        struct LoginVisitor;
        impl<'de> Visitor<'de> for LoginVisitor {
            type Value = LoginRequest;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a map with email and password")
            }
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut email: Option<String> = None;
                let mut password: Option<String> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "email" => email = Some(map.next_value()?),
                        "password" => password = Some(map.next_value()?),
                        _ => {
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                let email = email.ok_or_else(|| de::Error::missing_field("email"))?;
                let password = password.ok_or_else(|| de::Error::missing_field("password"))?;
                Ok(LoginRequest { email, password })
            }
        }
        deserializer.deserialize_map(LoginVisitor)
    }
}

// Manual FromRow to avoid sqlx macros
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
