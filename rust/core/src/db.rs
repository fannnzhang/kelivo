use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{DateTime, SecondsFormat, Utc};
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::{KelivoError, KelivoResult};

const MIGRATIONS: &[(u32, &str)] = &[(
    1,
    r#"
        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            message_ids TEXT NOT NULL,
            is_pinned INTEGER NOT NULL,
            mcp_server_ids TEXT NOT NULL,
            assistant_id TEXT,
            truncate_index INTEGER NOT NULL,
            version_selections TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            model_id TEXT,
            provider_id TEXT,
            total_tokens INTEGER,
            is_streaming INTEGER NOT NULL,
            reasoning_text TEXT,
            reasoning_start_at TEXT,
            reasoning_finished_at TEXT,
            translation TEXT,
            reasoning_segments_json TEXT,
            group_id TEXT,
            version INTEGER NOT NULL,
            FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS tool_events (
            id TEXT PRIMARY KEY,
            message_id TEXT NOT NULL,
            name TEXT NOT NULL,
            payload TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(message_id) REFERENCES messages(id) ON DELETE CASCADE
        );
    "#,
)];

#[derive(Clone)]
pub struct Database {
    path: PathBuf,
    conn: Arc<Mutex<Connection>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DbInitReport {
    pub path: PathBuf,
    pub applied_migrations: usize,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConversationRow {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_ids: Vec<String>,
    pub is_pinned: bool,
    pub mcp_server_ids: Vec<String>,
    pub assistant_id: Option<String>,
    pub truncate_index: i32,
    pub version_selections: HashMap<String, i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageRow {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
    pub total_tokens: Option<i32>,
    pub is_streaming: bool,
    pub reasoning_text: Option<String>,
    pub reasoning_start_at: Option<DateTime<Utc>>,
    pub reasoning_finished_at: Option<DateTime<Utc>>,
    pub translation: Option<String>,
    pub reasoning_segments_json: Option<String>,
    pub group_id: Option<String>,
    pub version: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolEventRow {
    pub id: String,
    pub message_id: String,
    pub name: String,
    pub payload: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub conversations: Vec<ConversationRow>,
    pub messages: Vec<MessageRow>,
    pub tool_events: Vec<ToolEventRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportSummary {
    pub conversations: usize,
    pub messages: usize,
    pub tool_events: usize,
}

impl Database {
    pub fn open(path: &Path) -> KelivoResult<(Self, DbInitReport)> {
        let mut conn = Connection::open(path).map_err(|err| {
            KelivoError::new(format!("failed to open database {}: {err}", path.display()))
        })?;

        conn.pragma_update(None, "foreign_keys", &1)
            .map_err(|err| KelivoError::new(format!("failed to enable foreign keys: {err}")))?;
        conn.pragma_update(None, "busy_timeout", &5_000)
            .map_err(|err| KelivoError::new(format!("failed to set busy_timeout: {err}")))?;

        let (applied_migrations, version) = run_migrations(&mut conn)?;
        let db = Self {
            path: path.to_path_buf(),
            conn: Arc::new(Mutex::new(conn)),
        };

        let report = DbInitReport {
            path: db.path.clone(),
            applied_migrations,
            version,
        };
        Ok((db, report))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn upsert_conversation(&self, row: &ConversationRow) -> KelivoResult<()> {
        self.with_conn(|conn| upsert_conversation_inner(conn, row))
    }

    pub fn upsert_message(&self, row: &MessageRow) -> KelivoResult<()> {
        self.with_conn(|conn| upsert_message_inner(conn, row))
    }

    pub fn upsert_tool_event(&self, row: &ToolEventRow) -> KelivoResult<()> {
        self.with_conn(|conn| upsert_tool_event_inner(conn, row))
    }

    pub fn get_conversation(&self, id: &str) -> KelivoResult<Option<ConversationRow>> {
        self.with_conn(|conn| get_conversation_inner(conn, id))
    }

    pub fn get_messages(
        &self,
        conversation_id: &str,
        limit: i64,
        offset: i64,
    ) -> KelivoResult<Vec<MessageRow>> {
        self.with_conn(|conn| get_messages_inner(conn, conversation_id, limit, offset))
    }

    pub fn get_tool_events(&self, message_id: &str) -> KelivoResult<Vec<ToolEventRow>> {
        self.with_conn(|conn| get_tool_events_inner(conn, message_id))
    }

    pub fn delete_conversation(&self, id: &str) -> KelivoResult<()> {
        self.with_conn(|conn| {
            conn.execute("DELETE FROM conversations WHERE id = ?1", params![id])
                .map_err(|err| KelivoError::new(format!("failed to delete conversation: {err}")))?;
            Ok(())
        })
    }

    pub fn import_snapshot(&self, snapshot: &Snapshot) -> KelivoResult<ImportSummary> {
        self.with_conn(|conn| {
            let tx = conn.transaction().map_err(|err| {
                KelivoError::new(format!("failed to start import transaction: {err}"))
            })?;

            tx.execute("DELETE FROM tool_events", params![])
                .map_err(|err| KelivoError::new(format!("failed to clear tool_events: {err}")))?;
            tx.execute("DELETE FROM messages", params![])
                .map_err(|err| KelivoError::new(format!("failed to clear messages: {err}")))?;
            tx.execute("DELETE FROM conversations", params![])
                .map_err(|err| KelivoError::new(format!("failed to clear conversations: {err}")))?;

            for conv in &snapshot.conversations {
                upsert_conversation_inner(&tx, conv)?;
            }
            for message in &snapshot.messages {
                upsert_message_inner(&tx, message)?;
            }
            for event in &snapshot.tool_events {
                upsert_tool_event_inner(&tx, event)?;
            }

            tx.commit()
                .map_err(|err| KelivoError::new(format!("failed to commit import: {err}")))?;

            Ok(ImportSummary {
                conversations: snapshot.conversations.len(),
                messages: snapshot.messages.len(),
                tool_events: snapshot.tool_events.len(),
            })
        })
    }

    pub fn export_snapshot(&self) -> KelivoResult<Snapshot> {
        self.with_conn(|conn| {
            let conversations = conn
                .prepare(
                    r#"
                        SELECT id, title, created_at, updated_at, message_ids, is_pinned,
                               mcp_server_ids, assistant_id, truncate_index, version_selections
                        FROM conversations
                    "#,
                )
                .map_err(|err| {
                    KelivoError::new(format!("failed to prepare conversation export: {err}"))
                })?
                .query_map(params![], |row| build_conversation_row(row))
                .map_err(|err| KelivoError::new(format!("failed to export conversations: {err}")))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|err| {
                    KelivoError::new(format!("failed to collect conversations: {err}"))
                })?;

            let messages = conn
                .prepare(
                    r#"
                        SELECT id, conversation_id, role, content, timestamp, model_id, provider_id,
                               total_tokens, is_streaming, reasoning_text, reasoning_start_at,
                               reasoning_finished_at, translation, reasoning_segments_json,
                               group_id, version
                        FROM messages
                        ORDER BY timestamp ASC
                    "#,
                )
                .map_err(|err| {
                    KelivoError::new(format!("failed to prepare message export: {err}"))
                })?
                .query_map(params![], |row| build_message_row(row))
                .map_err(|err| KelivoError::new(format!("failed to export messages: {err}")))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|err| KelivoError::new(format!("failed to collect messages: {err}")))?;

            let tool_events = conn
                .prepare(
                    r#"
                        SELECT id, message_id, name, payload, created_at
                        FROM tool_events
                        ORDER BY created_at ASC
                    "#,
                )
                .map_err(|err| {
                    KelivoError::new(format!("failed to prepare tool event export: {err}"))
                })?
                .query_map(params![], |row| build_tool_event_row(row))
                .map_err(|err| KelivoError::new(format!("failed to export tool events: {err}")))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|err| KelivoError::new(format!("failed to collect tool events: {err}")))?;

            Ok(Snapshot {
                conversations,
                messages,
                tool_events,
            })
        })
    }

    fn with_conn<T, F>(&self, f: F) -> KelivoResult<T>
    where
        F: FnOnce(&mut Connection) -> KelivoResult<T>,
    {
        let mut guard = self.conn.lock();
        f(&mut *guard)
    }
}

fn run_migrations(conn: &mut Connection) -> KelivoResult<(usize, u32)> {
    let current_version: u32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|err| KelivoError::new(format!("failed to read user_version: {err}")))?;

    let mut applied = 0usize;
    let mut latest_version = current_version;

    for (target_version, sql) in MIGRATIONS {
        if *target_version <= current_version {
            continue;
        }

        let tx = conn.transaction().map_err(|err| {
            KelivoError::new(format!("failed to open migration transaction: {err}"))
        })?;
        tx.execute_batch(sql).map_err(|err| {
            KelivoError::new(format!(
                "failed to execute migration {target_version}: {err}"
            ))
        })?;
        tx.pragma_update(None, "user_version", target_version)
            .map_err(|err| KelivoError::new(format!("failed to bump user_version: {err}")))?;
        tx.commit().map_err(|err| {
            KelivoError::new(format!(
                "failed to commit migration {target_version}: {err}"
            ))
        })?;
        applied += 1;
        latest_version = *target_version;
    }

    Ok((applied, latest_version))
}

fn upsert_conversation_inner(conn: &Connection, row: &ConversationRow) -> KelivoResult<()> {
    let message_ids = serde_json::to_string(&row.message_ids)
        .map_err(|err| KelivoError::new(format!("failed to serialize message_ids: {err}")))?;
    let mcp_server_ids = serde_json::to_string(&row.mcp_server_ids)
        .map_err(|err| KelivoError::new(format!("failed to serialize mcp_server_ids: {err}")))?;
    let version_selections = serde_json::to_string(&row.version_selections).map_err(|err| {
        KelivoError::new(format!("failed to serialize version_selections: {err}"))
    })?;

    conn.execute(
        r#"
            INSERT INTO conversations (
                id, title, created_at, updated_at, message_ids, is_pinned,
                mcp_server_ids, assistant_id, truncate_index, version_selections
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6,
                ?7, ?8, ?9, ?10
            )
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                created_at = excluded.created_at,
                updated_at = excluded.updated_at,
                message_ids = excluded.message_ids,
                is_pinned = excluded.is_pinned,
                mcp_server_ids = excluded.mcp_server_ids,
                assistant_id = excluded.assistant_id,
                truncate_index = excluded.truncate_index,
                version_selections = excluded.version_selections
        "#,
        params![
            row.id,
            row.title,
            format_datetime(&row.created_at),
            format_datetime(&row.updated_at),
            message_ids,
            bool_to_int(row.is_pinned),
            mcp_server_ids,
            row.assistant_id,
            row.truncate_index,
            version_selections
        ],
    )
    .map_err(|err| KelivoError::new(format!("failed to upsert conversation: {err}")))?;
    Ok(())
}

fn upsert_message_inner(conn: &Connection, row: &MessageRow) -> KelivoResult<()> {
    conn.execute(
        r#"
            INSERT INTO messages (
                id, conversation_id, role, content, timestamp, model_id,
                provider_id, total_tokens, is_streaming, reasoning_text,
                reasoning_start_at, reasoning_finished_at, translation,
                reasoning_segments_json, group_id, version
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6,
                ?7, ?8, ?9, ?10,
                ?11, ?12, ?13,
                ?14, ?15, ?16
            )
            ON CONFLICT(id) DO UPDATE SET
                conversation_id = excluded.conversation_id,
                role = excluded.role,
                content = excluded.content,
                timestamp = excluded.timestamp,
                model_id = excluded.model_id,
                provider_id = excluded.provider_id,
                total_tokens = excluded.total_tokens,
                is_streaming = excluded.is_streaming,
                reasoning_text = excluded.reasoning_text,
                reasoning_start_at = excluded.reasoning_start_at,
                reasoning_finished_at = excluded.reasoning_finished_at,
                translation = excluded.translation,
                reasoning_segments_json = excluded.reasoning_segments_json,
                group_id = excluded.group_id,
                version = excluded.version
        "#,
        params![
            row.id,
            row.conversation_id,
            row.role,
            row.content,
            format_datetime(&row.timestamp),
            row.model_id,
            row.provider_id,
            row.total_tokens,
            bool_to_int(row.is_streaming),
            row.reasoning_text,
            row.reasoning_start_at
                .map(|value| format_datetime(&value)),
            row.reasoning_finished_at
                .map(|value| format_datetime(&value)),
            row.translation,
            row.reasoning_segments_json,
            row.group_id,
            row.version
        ],
    )
    .map_err(|err| KelivoError::new(format!("failed to upsert message: {err}")))?;
    Ok(())
}

fn upsert_tool_event_inner(conn: &Connection, row: &ToolEventRow) -> KelivoResult<()> {
    conn.execute(
        r#"
            INSERT INTO tool_events (
                id, message_id, name, payload, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(id) DO UPDATE SET
                message_id = excluded.message_id,
                name = excluded.name,
                payload = excluded.payload,
                created_at = excluded.created_at
        "#,
        params![
            row.id,
            row.message_id,
            row.name,
            row.payload,
            format_datetime(&row.created_at)
        ],
    )
    .map_err(|err| KelivoError::new(format!("failed to upsert tool event: {err}")))?;
    Ok(())
}

fn get_conversation_inner(conn: &Connection, id: &str) -> KelivoResult<Option<ConversationRow>> {
    conn.query_row(
        r#"
            SELECT id, title, created_at, updated_at, message_ids, is_pinned,
                   mcp_server_ids, assistant_id, truncate_index, version_selections
            FROM conversations
            WHERE id = ?1
        "#,
        params![id],
        |row| build_conversation_row(row),
    )
    .optional()
    .map_err(|err| KelivoError::new(format!("failed to query conversation: {err}")))
}

fn get_messages_inner(
    conn: &Connection,
    conversation_id: &str,
    limit: i64,
    offset: i64,
) -> KelivoResult<Vec<MessageRow>> {
    let mut stmt = conn
        .prepare(
            r#"
                SELECT id, conversation_id, role, content, timestamp, model_id, provider_id,
                       total_tokens, is_streaming, reasoning_text, reasoning_start_at,
                       reasoning_finished_at, translation, reasoning_segments_json,
                       group_id, version
                FROM messages
                WHERE conversation_id = ?1
                ORDER BY timestamp ASC
                LIMIT ?2 OFFSET ?3
            "#,
        )
        .map_err(|err| KelivoError::new(format!("failed to prepare message query: {err}")))?;

    let rows = stmt
        .query_map(params![conversation_id, limit, offset], |row| {
            build_message_row(row)
        })
        .map_err(|err| KelivoError::new(format!("failed to iterate messages: {err}")))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| KelivoError::new(format!("failed to collect messages: {err}")))
}

fn get_tool_events_inner(conn: &Connection, message_id: &str) -> KelivoResult<Vec<ToolEventRow>> {
    let mut stmt = conn
        .prepare(
            r#"
                SELECT id, message_id, name, payload, created_at
                FROM tool_events
                WHERE message_id = ?1
                ORDER BY created_at ASC
            "#,
        )
        .map_err(|err| KelivoError::new(format!("failed to prepare tool event query: {err}")))?;

    let rows = stmt
        .query_map(params![message_id], |row| build_tool_event_row(row))
        .map_err(|err| KelivoError::new(format!("failed to iterate tool events: {err}")))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| KelivoError::new(format!("failed to collect tool events: {err}")))
}

fn build_conversation_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ConversationRow> {
    let message_ids_json: String = row.get(4)?;
    let message_ids: Vec<String> = serde_json::from_str(&message_ids_json).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(err))
    })?;

    let mcp_server_json: String = row.get(6)?;
    let mcp_server_ids: Vec<String> = serde_json::from_str(&mcp_server_json).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(err))
    })?;

    let version_json: String = row.get(9)?;
    let version_selections: HashMap<String, i32> =
        serde_json::from_str(&version_json).map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(err))
        })?;

    Ok(ConversationRow {
        id: row.get(0)?,
        title: row.get(1)?,
        created_at: parse_datetime(&row.get::<_, String>(2)?).map_err(to_sql_error)?,
        updated_at: parse_datetime(&row.get::<_, String>(3)?).map_err(to_sql_error)?,
        message_ids,
        is_pinned: int_to_bool(row.get(5)?),
        mcp_server_ids,
        assistant_id: row.get(7)?,
        truncate_index: row.get(8)?,
        version_selections,
    })
}

fn build_message_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MessageRow> {
    Ok(MessageRow {
        id: row.get(0)?,
        conversation_id: row.get(1)?,
        role: row.get(2)?,
        content: row.get(3)?,
        timestamp: parse_datetime(&row.get::<_, String>(4)?).map_err(to_sql_error)?,
        model_id: row.get(5)?,
        provider_id: row.get(6)?,
        total_tokens: row.get(7)?,
        is_streaming: int_to_bool(row.get(8)?),
        reasoning_text: row.get(9)?,
        reasoning_start_at: optional_datetime(row.get(10)?).map_err(to_sql_error)?,
        reasoning_finished_at: optional_datetime(row.get(11)?).map_err(to_sql_error)?,
        translation: row.get(12)?,
        reasoning_segments_json: row.get(13)?,
        group_id: row.get(14)?,
        version: row.get(15)?,
    })
}

fn build_tool_event_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ToolEventRow> {
    Ok(ToolEventRow {
        id: row.get(0)?,
        message_id: row.get(1)?,
        name: row.get(2)?,
        payload: row.get(3)?,
        created_at: parse_datetime(&row.get::<_, String>(4)?).map_err(to_sql_error)?,
    })
}

fn bool_to_int(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

fn int_to_bool(value: i64) -> bool {
    value != 0
}

fn format_datetime(value: &DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn parse_datetime(value: &str) -> KelivoResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|err| KelivoError::new(format!("invalid datetime value {value}: {err}")))
}

fn optional_datetime(value: Option<String>) -> KelivoResult<Option<DateTime<Utc>>> {
    match value {
        Some(text) => parse_datetime(&text).map(Some),
        None => Ok(None),
    }
}

fn to_sql_error(err: KelivoError) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_conversation(id: &str) -> ConversationRow {
        ConversationRow {
            id: id.to_string(),
            title: "Sample conversation".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            message_ids: vec![],
            is_pinned: false,
            mcp_server_ids: vec!["kelivo".to_string()],
            assistant_id: Some("assistant-1".to_string()),
            truncate_index: -1,
            version_selections: HashMap::new(),
        }
    }

    fn sample_message(conversation_id: &str, id: &str) -> MessageRow {
        MessageRow {
            id: id.to_string(),
            conversation_id: conversation_id.to_string(),
            role: "user".to_string(),
            content: "Hello world".to_string(),
            timestamp: Utc::now(),
            model_id: Some("gpt-4o".to_string()),
            provider_id: Some("openai".to_string()),
            total_tokens: Some(32),
            is_streaming: false,
            reasoning_text: None,
            reasoning_start_at: None,
            reasoning_finished_at: None,
            translation: None,
            reasoning_segments_json: None,
            group_id: Some("group-1".to_string()),
            version: 0,
        }
    }

    #[test]
    fn opens_database_and_runs_migrations() {
        let dir = tempdir().expect("temp dir");
        let db_path = dir.path().join("kelivo.db");
        let (db, report) = Database::open(&db_path).expect("database open");
        assert_eq!(db.path(), db_path.as_path());
        assert_eq!(report.version, 1);
    }

    #[test]
    fn upserts_and_queries_conversation() {
        let dir = tempdir().expect("temp dir");
        let db_path = dir.path().join("kelivo.db");
        let (db, _) = Database::open(&db_path).expect("database");

        let mut conv = sample_conversation("conv-1");
        conv.message_ids = vec!["msg-1".to_string()];
        conv.is_pinned = true;
        conv.version_selections.insert("group-1".to_string(), 2);

        db.upsert_conversation(&conv).expect("upsert conversation");

        let fetched = db
            .get_conversation("conv-1")
            .expect("query conversation")
            .expect("conversation exists");
        assert_eq!(fetched.id, conv.id);
        assert_eq!(fetched.message_ids, conv.message_ids);
        assert!(fetched.is_pinned);
        assert_eq!(fetched.version_selections.get("group-1"), Some(&2));
    }

    #[test]
    fn upserts_and_queries_messages() {
        let dir = tempdir().expect("temp dir");
        let db_path = dir.path().join("kelivo.db");
        let (db, _) = Database::open(&db_path).expect("database");

        let conv = sample_conversation("conv-1");
        db.upsert_conversation(&conv).expect("upsert conv");

        let message = sample_message("conv-1", "msg-1");
        db.upsert_message(&message).expect("upsert message");

        let messages = db.get_messages("conv-1", 10, 0).expect("query messages");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, message.id);
    }

    #[test]
    fn imports_and_exports_snapshot() {
        let dir = tempdir().expect("temp dir");
        let db_path = dir.path().join("kelivo.db");
        let (db, _) = Database::open(&db_path).expect("database");

        let mut conv = sample_conversation("conv-1");
        conv.message_ids = vec!["msg-1".to_string()];

        let message = sample_message("conv-1", "msg-1");
        let tool_event = ToolEventRow {
            id: "event-1".to_string(),
            message_id: "msg-1".to_string(),
            name: "search".to_string(),
            payload: "{\"status\":\"ok\"}".to_string(),
            created_at: Utc::now(),
        };

        let snapshot = Snapshot {
            conversations: vec![conv.clone()],
            messages: vec![message.clone()],
            tool_events: vec![tool_event.clone()],
        };

        let summary = db.import_snapshot(&snapshot).expect("import snapshot");
        assert_eq!(summary.conversations, 1);
        assert_eq!(summary.messages, 1);
        assert_eq!(summary.tool_events, 1);

        let exported = db.export_snapshot().expect("export snapshot");
        assert_eq!(exported.conversations.len(), 1);
        assert_eq!(exported.messages.len(), 1);
        assert_eq!(exported.tool_events.len(), 1);
        assert_eq!(exported.conversations[0].id, conv.id);
        assert_eq!(exported.messages[0].id, message.id);
        assert_eq!(exported.tool_events[0].id, tool_event.id);
    }
}
