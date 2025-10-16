import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'api/backup.dart';
import 'frb_generated.dart';

class MockRustLibApi implements RustLibApi {
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
        .toList();
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
      return payload.map((map) {
        final isDir = (map['isDir'] as bool?) ?? false;
        final data = isDir
            ? Uint8List(0)
            : Uint8List.fromList(base64Decode((map['data'] as String?) ?? ''));
        return BackupZipEntry(
          path: map['path'] as String? ?? '',
          data: data,
          isDir: isDir,
        );
      }).toList();
    } catch (_) {
      return const <BackupZipEntry>[];
    }
  }

  @override
  Future<List<WebDavEntry>> crateApiBackupParseWebdavPropfind({
    required String xml,
    required String baseUrl,
  }) async {
    return const <WebDavEntry>[];
  }

  Future<String> _readFileAsString(String path) async {
    try {
      final file = File(path);
      if (!await file.exists()) {
        return '[[mock missing file: $path]]';
      }
      final bytes = await file.readAsBytes();
      return utf8.decode(bytes, allowMalformed: true);
    } catch (err) {
      return '[[mock read error: $err]]';
    }
  }
}
