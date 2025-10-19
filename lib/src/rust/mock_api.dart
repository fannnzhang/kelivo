import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:Kelivo/src/rust/api/backup.dart';
import 'package:Kelivo/src/rust/api/db.dart';
import 'package:Kelivo/src/rust/api/llm_types.dart';
import 'package:Kelivo/src/rust/frb_generated.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

class MockRustLibApi implements RustLibApi {
  @override
  Future<Uint8List> crateApiBackupCreateBackupZip({
    required List<BackupZipEntryInput> entries,
  }) async {
    final payload = entries
        .map(
          (entry) => <String, Object?>{
            'path': entry.path,
            'isDir': entry.isDir,
            'data': entry.isDir ? '' : base64Encode(entry.data),
          },
        )
        .toList(growable: false);
    return Uint8List.fromList(utf8.encode(jsonEncode(payload)));
  }

  @override
  Future<List<BackupZipEntry>> crateApiBackupExtractBackupZip({
    required List<int> bytes,
  }) async {
    try {
      final decoded = utf8.decode(bytes);
      final payload = (jsonDecode(decoded) as List)
          .cast<Map<String, dynamic>>();
      return payload
          .map((map) {
            final isDir = (map['isDir'] as bool?) ?? false;
            final encoded = (map['data'] as String?) ?? '';
            final data = isDir
                ? Uint8List(0)
                : Uint8List.fromList(base64Decode(encoded));
            return BackupZipEntry(
              path: map['path'] as String? ?? '',
              data: data,
              isDir: isDir,
            );
          })
          .toList(growable: false);
    } catch (error, stack) {
      print('mock extract backup zip error: $error\n$stack');
      rethrow;
    }
  }

  @override
  Future<List<WebDavEntry>> crateApiBackupParseWebdavPropfind({
    required String xml,
    required String baseHref,
  }) async {
    return const <WebDavEntry>[];
  }

  @override
  Future<String> crateApiGoogleAuthCreateGoogleAuthJwt({
    required String clientEmail,
    required String privateKeyPem,
    required String tokenUri,
    required List<String> scopes,
  }) async {
    return 'mock-jwt';
  }

  @override
  String crateApiSimpleGreet({required String name}) {
    return 'Hello, $name!';
  }

  @override
  Future<void> crateApiSimpleInitApp() async {}

  @override
  Future<String> crateApiMarkdownSanitizerReplaceInlineBase64Images({
    required String markdown,
  }) async {
    return markdown;
  }

  @override
  Future<String> crateApiMarkdownSanitizerInlineLocalImagesToBase64({
    required String markdown,
  }) async {
    return markdown;
  }

  @override
  Future<String> crateApiDocumentParserExtractTextFromDocx({
    required String path,
  }) => _readFileAsString(path);

  @override
  Future<String> crateApiDocumentParserExtractTextFromPdf({
    required String path,
  }) => _readFileAsString(path);

  @override
  Future<String> crateApiDocumentParserReadTextFallback({
    required String path,
  }) => _readFileAsString(path);

  Future<String> _readFileAsString(String path) async {
    try {
      final file = File(path);
      if (!await file.exists()) {
        return '[[mock missing file: $path]]';
      }
      final bytes = await file.readAsBytes();
      return utf8.decode(bytes, allowMalformed: true);
    } catch (error) {
      return '[[mock read error: $error]]';
    }
  }

  @override
  Future<void> crateApiDbDbDeleteConversation({required String id}) async {}

  @override
  Future<FrbDbSnapshot> crateApiDbDbExportSnapshot() async {
    return const FrbDbSnapshot(
      conversations: <FrbConversation>[],
      messages: <FrbMessage>[],
      toolEvents: <FrbToolEvent>[],
    );
  }

  @override
  Future<FrbImportSummary> crateApiDbDbImportSnapshot({
    required FrbDbSnapshot snapshot,
  }) async {
    return const FrbImportSummary(conversations: 0, messages: 0, toolEvents: 0);
  }

  @override
  Future<FrbDbInfo> crateApiDbDbInit({required String dbPath}) async {
    return FrbDbInfo(path: dbPath, appliedMigrations: 0, version: 1);
  }

  @override
  Future<bool> crateApiDbDbIsInitialized() async {
    return true;
  }

  @override
  Future<FrbConversation?> crateApiDbDbQueryConversation({
    required String id,
  }) async {
    return null;
  }

  @override
  Future<List<FrbMessage>> crateApiDbDbQueryMessages({
    required String conversationId,
    required PlatformInt64 limit,
    required PlatformInt64 offset,
  }) async {
    return const <FrbMessage>[];
  }

  @override
  Future<List<FrbToolEvent>> crateApiDbDbQueryToolEvents({
    required String messageId,
  }) async {
    return const <FrbToolEvent>[];
  }

  @override
  Future<void> crateApiDbDbUpsertConversation({
    required FrbConversation conversation,
  }) async {}

  @override
  Future<void> crateApiDbDbUpsertMessage({required FrbMessage message}) async {}

  @override
  Future<void> crateApiDbDbUpsertToolEvent({
    required FrbToolEvent event,
  }) async {}

  @override
  Stream<String> crateApiLlmLlmChatStream({required FrbChatRequest request}) {
    return Stream.value('mock stream not implemented');
  }

  @override
  Future<bool> crateApiLlmLlmCancel({required String requestId}) async {
    return true;
  }

  @override
  Future<FrbChatResponse> crateApiLlmLlmChat({
    required FrbChatRequest request,
  }) async {
    return FrbChatResponse(
      requestId: request.requestId,
      providerId: request.provider ?? 'mock-provider',
      model: request.model,
      outputText: 'mock response',
      usage: const FrbChatUsage(
        promptTokens: 0,
        completionTokens: 0,
        totalTokens: 0,
      ),
    );
  }
}
