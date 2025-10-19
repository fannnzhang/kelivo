import 'dart:async';

import 'package:Kelivo/src/rust/api/db.dart' as rust_db;

class RustDbBridge {
  RustDbBridge._();

  static final RustDbBridge instance = RustDbBridge._();

  bool _initialized = false;
  String? _dbPath;

  bool get isInitialized => _initialized;
  String? get path => _dbPath;

  Future<void> init(String dbPath) async {
    if (_initialized && _dbPath == dbPath) {
      return;
    }
    await rust_db.dbInit(dbPath: dbPath);
    _initialized = true;
    _dbPath = dbPath;
  }

  Future<void> upsertConversation(rust_db.FrbConversation conversation) async {
    if (!_initialized) return;
    await rust_db.dbUpsertConversation(conversation: conversation);
  }

  Future<void> upsertMessage(rust_db.FrbMessage message) async {
    if (!_initialized) return;
    await rust_db.dbUpsertMessage(message: message);
  }

  Future<void> upsertToolEvent(rust_db.FrbToolEvent event) async {
    if (!_initialized) return;
    await rust_db.dbUpsertToolEvent(event: event);
  }

  Future<void> deleteConversation(String conversationId) async {
    if (!_initialized) return;
    await rust_db.dbDeleteConversation(id: conversationId);
  }

  Future<rust_db.FrbConversation?> getConversation(String conversationId) async {
    if (!_initialized) return null;
    return rust_db.dbQueryConversation(id: conversationId);
  }

  Future<List<rust_db.FrbMessage>> getMessages(
    String conversationId, {
    int limit = 256,
    int offset = 0,
  }) async {
    if (!_initialized) return const <rust_db.FrbMessage>[];
    return rust_db.dbQueryMessages(
      conversationId: conversationId,
      limit: limit,
      offset: offset,
    );
  }

  Future<rust_db.FrbDbSnapshot> exportSnapshot() async {
    if (!_initialized) {
      return const rust_db.FrbDbSnapshot(
        conversations: <rust_db.FrbConversation>[],
        messages: <rust_db.FrbMessage>[],
        toolEvents: <rust_db.FrbToolEvent>[],
      );
    }
    return rust_db.dbExportSnapshot();
  }

  Future<void> importSnapshot(rust_db.FrbDbSnapshot snapshot) async {
    if (!_initialized) return;
    await rust_db.dbImportSnapshot(snapshot: snapshot);
  }
}
