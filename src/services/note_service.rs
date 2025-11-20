use crate::{
    models::note::{CreateNoteRequest, Note, UpdateNoteRequest},
    AppState,
};
use uuid::Uuid;

#[derive(Debug)]
pub enum NoteError {
    NotFound,
    Db(sqlx::Error),
}

impl std::fmt::Display for NoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoteError::NotFound => write!(f, "note not found"),
            NoteError::Db(e) => write!(f, "db error: {}", e),
        }
    }
}

impl std::error::Error for NoteError {}

impl From<sqlx::Error> for NoteError {
    fn from(e: sqlx::Error) -> Self {
        NoteError::Db(e)
    }
}

pub async fn create_note(
    state: &AppState,
    user_id: Uuid,
    req: CreateNoteRequest,
) -> Result<Note, NoteError> {
    let note_id = Uuid::new_v4();
    sqlx::query("INSERT INTO notes (id, user_id, title, content, category, tags, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, CONVERT_TZ(UTC_TIMESTAMP(), '+00:00', '+08:00'), CONVERT_TZ(UTC_TIMESTAMP(), '+00:00', '+08:00'))")
        .bind(note_id)
        .bind(user_id)
        .bind(&req.title)
        .bind(&req.content)
        .bind(&req.category)
        .bind(&req.tags)
        .execute(&state.db)
        .await?;

    let note = sqlx::query_as::<_, Note>(
        "SELECT id, user_id, title, content, category, tags, created_at, updated_at FROM notes WHERE id = ?",
    )
    .bind(note_id)
    .fetch_one(&state.db)
    .await?;
    Ok(note)
}

pub async fn list_notes(
    state: &AppState,
    user_id: Uuid,
    tag: Option<String>,
    keyword: Option<String>,
) -> Result<Vec<Note>, NoteError> {
    let mut q = String::from("SELECT id, user_id, title, content, category, tags, created_at, updated_at FROM notes WHERE user_id = ?");
    if tag.is_some() {
        q.push_str(" AND tags LIKE ?");
    }
    if keyword.is_some() {
        q.push_str(" AND (title LIKE ? OR content LIKE ?)");
    }
    q.push_str(" ORDER BY updated_at DESC");

    let mut query = sqlx::query_as::<_, Note>(&q).bind(user_id);
    if let Some(t) = tag.as_ref() {
        query = query.bind(format!("%{}%", t));
    }
    if let Some(k) = keyword.as_ref() {
        query = query.bind(format!("%{}%", k)).bind(format!("%{}%", k));
    }

    let notes = query.fetch_all(&state.db).await?;
    Ok(notes)
}

pub async fn get_note(state: &AppState, user_id: Uuid, note_id: Uuid) -> Result<Note, NoteError> {
    let note = sqlx::query_as::<_, Note>("SELECT id, user_id, title, content, category, tags, created_at, updated_at FROM notes WHERE id = ? AND user_id = ?")
        .bind(note_id)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?;
    match note {
        Some(n) => Ok(n),
        None => Err(NoteError::NotFound),
    }
}

pub async fn update_note(
    state: &AppState,
    user_id: Uuid,
    note_id: Uuid,
    req: UpdateNoteRequest,
) -> Result<Note, NoteError> {
    // Fetch current
    let current = get_note(state, user_id, note_id).await?;
    let title = req.title.unwrap_or(current.title);
    let content = req.content.unwrap_or(current.content);
    let tags = req.tags.or(current.tags);

    sqlx::query("UPDATE notes SET title = ?, content = ?, tags = ?, updated_at = CONVERT_TZ(UTC_TIMESTAMP(), '+00:00', '+08:00') WHERE id = ? AND user_id = ?")
        .bind(&title)
        .bind(&content)
        .bind(&tags)
        .bind(note_id)
        .bind(user_id)
        .execute(&state.db)
        .await?;
    get_note(state, user_id, note_id).await
}

pub async fn delete_note(state: &AppState, user_id: Uuid, note_id: Uuid) -> Result<(), NoteError> {
    let res = sqlx::query("DELETE FROM notes WHERE id = ? AND user_id = ?")
        .bind(note_id)
        .bind(user_id)
        .execute(&state.db)
        .await?;
    if res.rows_affected() == 0 {
        return Err(NoteError::NotFound);
    }
    Ok(())
}
