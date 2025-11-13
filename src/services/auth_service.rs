use crate::{
    models::user::{LoginRequest, LoginResponse, RegisterRequest, User, UserInfo},
    utils::{
        jwt::generate_token,
        password::{hash_password, verify_password},
    },
    AppState,
};
use sqlx::{self};
use uuid::Uuid;

#[derive(Debug)]
pub enum AuthError {
    Conflict,
    InvalidCredentials,
    Db(sqlx::Error),
    Internal(anyhow::Error),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::Conflict => write!(f, "username or email already exists"),
            AuthError::InvalidCredentials => write!(f, "invalid username or password"),
            AuthError::Db(e) => write!(f, "db error: {}", e),
            AuthError::Internal(e) => write!(f, "internal error: {}", e),
        }
    }
}

impl std::error::Error for AuthError {}

impl From<sqlx::Error> for AuthError {
    fn from(e: sqlx::Error) -> Self {
        AuthError::Db(e)
    }
}

impl From<anyhow::Error> for AuthError {
    fn from(e: anyhow::Error) -> Self {
        AuthError::Internal(e)
    }
}

pub async fn register(state: &AppState, req: RegisterRequest) -> Result<User, AuthError> {
    let exists: Option<(i64,)> =
        sqlx::query_as("SELECT 1 as count FROM users WHERE username = ? OR email = ? LIMIT 1")
            .bind(&req.username)
            .bind(&req.email)
            .fetch_optional(&state.db)
            .await?;
    if exists.is_some() {
        return Err(AuthError::Conflict);
    }

    let user_id = Uuid::new_v4();
    let password_hash = hash_password(&req.password)?;

    sqlx::query("INSERT INTO users (id, username, email, password_hash, created_at) VALUES (?, ?, ?, ?, CONVERT_TZ(UTC_TIMESTAMP(), '+00:00', '+08:00'))")
        .bind(user_id)
        .bind(&req.username)
        .bind(&req.email)
        .bind(&password_hash)
        .execute(&state.db)
        .await?;

    let user = sqlx::query_as::<_, User>("SELECT id, username, email, password_hash, CAST(created_at AS DATETIME) AS created_at FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&state.db)
        .await?;
    Ok(user)
}

pub async fn login(state: &AppState, req: LoginRequest) -> Result<LoginResponse, AuthError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, CAST(created_at AS DATETIME) AS created_at FROM users WHERE email = ? LIMIT 1")
        .bind(&req.email)
        .fetch_optional(&state.db)
        .await?;
    let user = match user {
        Some(u) => u,
        None => return Err(AuthError::InvalidCredentials),
    };

    let valid = verify_password(&req.password, &user.password_hash)?;
    if !valid {
        return Err(AuthError::InvalidCredentials);
    }

    let (token, _) = generate_token(user.id, &state.jwt_secret, 60)?; // 60 minutes

    let response = LoginResponse {
        token: token.clone(),
        user_info: UserInfo {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            // TODO: 这里暂时先写死，以后再做优化
            avatar: "/api/tiny-note/static/images/default_avatar.jpeg".to_string(),
        },
    };

    tracing::info!(
        user_id = %user.id,
        username = %user.username,
        email = %user.email,
        avatar = %response.user_info.avatar,
        "User logged in successfully"
    );

    // No need to store token in redis by default; blacklist on logout feature could be added.
    // For demonstration, we simply return the token.
    Ok(response)
}
