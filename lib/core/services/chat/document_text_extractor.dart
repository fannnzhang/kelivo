import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

import '../../../src/rust/api/document_parser.dart' as document_parser;
import '../../../utils/sandbox_path_resolver.dart';

class DocumentTextExtractor {
  static const _unsupportedDocMessage =
      '[[DOC format (.doc) not supported for text extraction]]';

  static Future<String> extract({
    required String path,
    required String mime,
  }) async {
    final fixedPath = SandboxPathResolver.fix(path);
    try {
      switch (mime) {
        case 'application/pdf':
          return await document_parser.extractTextFromPdf(path: fixedPath);
        case 'application/vnd.openxmlformats-officedocument.wordprocessingml.document':
        case 'application/vnd.ms-word.document.macroEnabled.12':
          return await document_parser.extractTextFromDocx(path: fixedPath);
        case 'application/msword':
          return _unsupportedDocMessage;
        default:
          return await document_parser.readTextFallback(path: fixedPath);
      }
    } on FrbException catch (err) {
      return '[[Rust document parser error: $err]]';
    } catch (err) {
      return '[[Failed to read file: $err]]';
    }
  }
}
