use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::PathBuf;

use chrono::{DateTime, SecondsFormat, Utc};
use flutter_rust_bridge::frb;
use kelivo_core::db::{
    ConversationRow, Database, DbInitReport, ImportSummary, MessageRow, Snapshot, ToolEventRow,
};
use kelivo_core::{KelivoError, KelivoResult};
use once_cell::sync::Lazy;
use parking_lot::RwLock;

static DATABASE: Lazy<RwLock<Option<Database>>> = Lazy::new(|| RwLock::new(None));

#[frb]
pub fn db_is_initialized() -> bool {
    DATABASE.read().is_some()
}

#[frb]
pub fn db_init(db_path: String) -> Result<FrbDbInfo, String> {
    let path = PathBuf::from(&db_path);
    let (db, report) = Database::open(&path).map_err(to_err_string)?;
    *DATABASE.write() = Some(db);
    Ok(report.into())
}

#[frb]
pub fn db_upsert_conversation(conversation: FrbConversation) -> Result<(), String> {
    let row = ConversationRow::try_from(conversation)?;
    with_db(|db| db.upsert_conversation(&row))
}

#[frb]
pub fn db_upsert_message(message: FrbMessage) -> Result<(), String> {
    let row = MessageRow::try_from(message)?;
    with_db(|db| db.upsert_message(&row))
}

#[frb]
pub fn db_upsert_tool_event(event: FrbToolEvent) -> Result<(), String> {
    let row = ToolEventRow::try_from(event)?;
    with_db(|db| db.upsert_tool_event(&row))
}

#[frb]
pub fn db_query_conversation(id: String) -> Result<Option<FrbConversation>, String> {
    with_db(|db| db.get_conversation(&id)).map(|opt| opt.map(FrbConversation::from))
}

#[frb]
pub fn db_query_messages(
    conversation_id: String,
    limit: i64,
    offset: i64,
) -> Result<Vec<FrbMessage>, String> {
    with_db(|db| db.get_messages(&conversation_id, limit, offset))
        .map(|rows| rows.into_iter().map(FrbMessage::from).collect())
}

#[frb]
pub fn db_query_tool_events(message_id: String) -> Result<Vec<FrbToolEvent>, String> {
    with_db(|db| db.get_tool_events(&message_id))
        .map(|rows| rows.into_iter().map(FrbToolEvent::from).collect())
}

#[frb]
pub fn db_delete_conversation(id: String) -> Result<(), String> {
    with_db(|db| db.delete_conversation(&id))
}

#[frb]
pub fn db_export_snapshot() -> Result<FrbDbSnapshot, String> {
    with_db(|db| db.export_snapshot()).map(FrbDbSnapshot::from)
}

#[frb]
pub fn db_import_snapshot(snapshot: FrbDbSnapshot) -> Result<FrbImportSummary, String> {
    let snapshot = Snapshot::try_from(snapshot)?;
    with_db(|db| db.import_snapshot(&snapshot)).map(FrbImportSummary::from)
}

#[frb]
#[derive(Debug, Clone)]
pub struct FrbDbInfo {
    pub path: String,
    pub applied_migrations: u32,
    pub version: u32,
}

#[frb]
#[derive(Debug, Clone)]
pub struct FrbConversation {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_ids: Vec<String>,
    pub is_pinned: bool,
    pub mcp_server_ids: Vec<String>,
    pub assistant_id: Option<String>,
    pub truncate_index: i32,
    pub version_selections: HashMap<String, i32>,
}

#[frb]
#[derive(Debug, Clone)]
pub struct FrbMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
    pub total_tokens: Option<i32>,
    pub is_streaming: bool,
    pub reasoning_text: Option<String>,
    pub reasoning_start_at: Option<String>,
    pub reasoning_finished_at: Option<String>,
    pub translation: Option<String>,
    pub reasoning_segments_json: Option<String>,
    pub group_id: Option<String>,
    pub version: i32,
}

#[frb]
#[derive(Debug, Clone)]
pub struct FrbToolEvent {
    pub id: String,
    pub message_id: String,
    pub name: String,
    pub payload: String,
    pub created_at: String,
}

#[frb]
#[derive(Debug, Clone)]
pub struct FrbDbSnapshot {
    pub conversations: Vec<FrbConversation>,
    pub messages: Vec<FrbMessage>,
    pub tool_events: Vec<FrbToolEvent>,
}

#[frb]
#[derive(Debug, Clone)]
pub struct FrbImportSummary {
    pub conversations: u32,
    pub messages: u32,
    pub tool_events: u32,
}

impl From<DbInitReport> for FrbDbInfo {
    fn from(value: DbInitReport) -> Self {
        Self {
            path: value.path.to_string_lossy().to_string(),
            applied_migrations: value.applied_migrations as u32,
            version: value.version,
        }
    }
}

impl From<ConversationRow> for FrbConversation {
    fn from(value: ConversationRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            created_at: format_datetime(&value.created_at),
            updated_at: format_datetime(&value.updated_at),
            message_ids: value.message_ids,
            is_pinned: value.is_pinned,
            mcp_server_ids: value.mcp_server_ids,
            assistant_id: value.assistant_id,
            truncate_index: value.truncate_index,
            version_selections: value.version_selections,
        }
    }
}

impl TryFrom<FrbConversation> for ConversationRow {
    type Error = String;

    fn try_from(value: FrbConversation) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            title: value.title,
            created_at: parse_datetime(&value.created_at)?,
            updated_at: parse_datetime(&value.updated_at)?,
            message_ids: value.message_ids,
            is_pinned: value.is_pinned,
            mcp_server_ids: value.mcp_server_ids,
            assistant_id: value.assistant_id,
            truncate_index: value.truncate_index,
            version_selections: value.version_selections,
        })
    }
}

impl From<MessageRow> for FrbMessage {
    fn from(value: MessageRow) -> Self {
        Self {
            id: value.id,
            conversation_id: value.conversation_id,
            role: value.role,
            content: value.content,
            timestamp: format_datetime(&value.timestamp),
            model_id: value.model_id,
            provider_id: value.provider_id,
            total_tokens: value.total_tokens,
            is_streaming: value.is_streaming,
            reasoning_text: value.reasoning_text,
            reasoning_start_at: value.reasoning_start_at.map(|dt| format_datetime(&dt)),
            reasoning_finished_at: value.reasoning_finished_at.map(|dt| format_datetime(&dt)),
            translation: value.translation,
            reasoning_segments_json: value.reasoning_segments_json,
            group_id: value.group_id,
            version: value.version,
        }
    }
}

impl TryFrom<FrbMessage> for MessageRow {
    type Error = String;

    fn try_from(value: FrbMessage) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            conversation_id: value.conversation_id,
            role: value.role,
            content: value.content,
            timestamp: parse_datetime(&value.timestamp)?,
            model_id: value.model_id,
            provider_id: value.provider_id,
            total_tokens: value.total_tokens,
            is_streaming: value.is_streaming,
            reasoning_text: value.reasoning_text,
            reasoning_start_at: value
                .reasoning_start_at
                .map(|text| parse_datetime(&text))
                .transpose()?,
            reasoning_finished_at: value
                .reasoning_finished_at
                .map(|text| parse_datetime(&text))
                .transpose()?,
            translation: value.translation,
            reasoning_segments_json: value.reasoning_segments_json,
            group_id: value.group_id,
            version: value.version,
        })
    }
}

impl From<ToolEventRow> for FrbToolEvent {
    fn from(value: ToolEventRow) -> Self {
        Self {
            id: value.id,
            message_id: value.message_id,
            name: value.name,
            payload: value.payload,
            created_at: format_datetime(&value.created_at),
        }
    }
}

impl TryFrom<FrbToolEvent> for ToolEventRow {
    type Error = String;

    fn try_from(value: FrbToolEvent) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            message_id: value.message_id,
            name: value.name,
            payload: value.payload,
            created_at: parse_datetime(&value.created_at)?,
        })
    }
}

impl From<Snapshot> for FrbDbSnapshot {
    fn from(value: Snapshot) -> Self {
        Self {
            conversations: value
                .conversations
                .into_iter()
                .map(FrbConversation::from)
                .collect(),
            messages: value.messages.into_iter().map(FrbMessage::from).collect(),
            tool_events: value
                .tool_events
                .into_iter()
                .map(FrbToolEvent::from)
                .collect(),
        }
    }
}

impl TryFrom<FrbDbSnapshot> for Snapshot {
    type Error = String;

    fn try_from(value: FrbDbSnapshot) -> Result<Self, Self::Error> {
        let mut conversations = Vec::with_capacity(value.conversations.len());
        for conv in value.conversations {
            conversations.push(ConversationRow::try_from(conv)?);
        }

        let mut messages = Vec::with_capacity(value.messages.len());
        for message in value.messages {
            messages.push(MessageRow::try_from(message)?);
        }

        let mut tool_events = Vec::with_capacity(value.tool_events.len());
        for event in value.tool_events {
            tool_events.push(ToolEventRow::try_from(event)?);
        }

        Ok(Self {
            conversations,
            messages,
            tool_events,
        })
    }
}

impl From<ImportSummary> for FrbImportSummary {
    fn from(value: ImportSummary) -> Self {
        Self {
            conversations: value.conversations as u32,
            messages: value.messages as u32,
            tool_events: value.tool_events as u32,
        }
    }
}

fn with_db<T, F>(f: F) -> Result<T, String>
where
    F: FnOnce(&Database) -> KelivoResult<T>,
{
    let guard = DATABASE.read();
    match guard.as_ref() {
        Some(db) => f(db).map_err(to_err_string),
        None => Err("database not initialized".to_string()),
    }
}

fn parse_datetime(value: &str) -> Result<DateTime<Utc>, String> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|err| format!("invalid datetime {value}: {err}"))
}

fn format_datetime(value: &DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn to_err_string(err: KelivoError) -> String {
    err.message().to_string()
}
