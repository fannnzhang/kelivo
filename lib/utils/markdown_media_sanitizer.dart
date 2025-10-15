import 'dart:convert';
import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:path_provider/path_provider.dart';
import 'package:uuid/uuid.dart';

import '../config/feature_flags.dart';
import '../src/rust/api/markdown_sanitizer.dart' as rust_api;

class MarkdownMediaSanitizer {
  MarkdownMediaSanitizer._();

  static Future<String> replaceInlineBase64Images(String markdown) async {
    if (!FeatureFlags.useRustMarkdownSanitizer) {
      return _fallbackReplaceInlineBase64Images(markdown);
    }

    try {
      return await rust_api.replaceInlineBase64Images(markdown: markdown);
    } catch (err, stack) {
      debugPrint(
        '[MarkdownMediaSanitizer] Rust replaceInlineBase64Images failed: $err',
      );
      debugPrintStack(stackTrace: stack);
      return _fallbackReplaceInlineBase64Images(markdown);
    }
  }

  static Future<String> inlineLocalImagesToBase64(String markdown) async {
    if (!FeatureFlags.useRustMarkdownSanitizer) {
      return _fallbackInlineLocalImagesToBase64(markdown);
    }

    try {
      return await rust_api.inlineLocalImagesToBase64(markdown: markdown);
    } catch (err, stack) {
      debugPrint(
        '[MarkdownMediaSanitizer] Rust inlineLocalImagesToBase64 failed: $err',
      );
      debugPrintStack(stackTrace: stack);
      return _fallbackInlineLocalImagesToBase64(markdown);
    }
  }

  static Future<String> _fallbackReplaceInlineBase64Images(String markdown) {
    return _DartMarkdownMediaSanitizer.replaceInlineBase64Images(markdown);
  }

  static Future<String> _fallbackInlineLocalImagesToBase64(String markdown) {
    return _DartMarkdownMediaSanitizer.inlineLocalImagesToBase64(markdown);
  }
}

class _DartMarkdownMediaSanitizer {
  static final Uuid _uuid = const Uuid();
  static final RegExp _imgRe = RegExp(
    r'!\[[^\]]*\]\((data:image/[a-zA-Z0-9.+-]+;base64,[a-zA-Z0-9+/=\r\n]+)\)',
    multiLine: true,
  );

  static Future<String> replaceInlineBase64Images(String markdown) async {
    if (!markdown.contains('data:image')) return markdown;

    final matches = _imgRe.allMatches(markdown).toList();
    if (matches.isEmpty) return markdown;

    final docs = await getApplicationDocumentsDirectory();
    final dir = Directory('${docs.path}/images');
    if (!await dir.exists()) {
      await dir.create(recursive: true);
    }

    final sb = StringBuffer();
    int last = 0;
    for (final m in matches) {
      sb.write(markdown.substring(last, m.start));
      final dataUrl = m.group(1)!;
      String ext = _extFromMime(_mimeOf(dataUrl));

      final b64Index = dataUrl.indexOf('base64,');
      if (b64Index < 0) {
        sb.write(markdown.substring(m.start, m.end));
        last = m.end;
        continue;
      }
      final payload = dataUrl.substring(b64Index + 7);

      final normalized = payload.replaceAll('\n', '');
      final bytes = await compute(_decodeBase64, normalized);

      final digest = _uuid.v5(Uuid.NAMESPACE_URL, normalized);
      final file = File('${dir.path}/img_$digest.$ext');
      if (!await file.exists()) {
        await file.writeAsBytes(bytes, flush: true);
      }

      final replaced = markdown
          .substring(m.start, m.end)
          .replaceFirst(dataUrl, file.path);
      sb.write(replaced);
      last = m.end;
    }
    sb.write(markdown.substring(last));
    return sb.toString();
  }

  static Future<String> inlineLocalImagesToBase64(String markdown) async {
    if (!(markdown.contains('![') && markdown.contains(']('))) return markdown;

    final re = RegExp(r'!\[[^\]]*\]\(([^)]+)\)', multiLine: true);
    final matches = re.allMatches(markdown).toList();
    if (matches.isEmpty) return markdown;

    final sb = StringBuffer();
    int last = 0;
    for (final m in matches) {
      sb.write(markdown.substring(last, m.start));
      final url = (m.group(1) ?? '').trim();
      final isRemote = url.startsWith('http://') || url.startsWith('https://');
      final isData = url.startsWith('data:');
      final isFileUri = url.startsWith('file://');
      final isLikelyLocalPath =
          (!isRemote && !isData) &&
          (isFileUri || url.startsWith('/') || url.contains(':'));

      if (!isLikelyLocalPath) {
        sb.write(markdown.substring(m.start, m.end));
        last = m.end;
        continue;
      }

      try {
        var path = url;
        if (isFileUri) {
          path = url.replaceFirst('file://', '');
        }
        final fixed = path;
        final f = File(fixed);
        if (!f.existsSync()) {
          sb.write(markdown.substring(m.start, m.end));
          last = m.end;
          continue;
        }
        final bytes = await f.readAsBytes();
        final b64 = base64Encode(bytes);
        final mime = _guessMimeFromPath(fixed);
        final dataUrl = 'data:$mime;base64,$b64';
        final replaced = markdown
            .substring(m.start, m.end)
            .replaceFirst(url, dataUrl);
        sb.write(replaced);
      } catch (_) {
        sb.write(markdown.substring(m.start, m.end));
      }
      last = m.end;
    }
    sb.write(markdown.substring(last));
    return sb.toString();
  }

  static String _guessMimeFromPath(String path) {
    final lower = path.toLowerCase();
    if (lower.endsWith('.jpg') || lower.endsWith('.jpeg')) return 'image/jpeg';
    if (lower.endsWith('.png')) return 'image/png';
    if (lower.endsWith('.webp')) return 'image/webp';
    if (lower.endsWith('.gif')) return 'image/gif';
    return 'image/png';
  }

  static List<int> _decodeBase64(String b64) =>
      base64Decode(b64.replaceAll('\n', ''));

  static String _mimeOf(String dataUrl) {
    try {
      final start = dataUrl.indexOf(':');
      final semi = dataUrl.indexOf(';');
      if (start >= 0 && semi > start) {
        return dataUrl.substring(start + 1, semi);
      }
    } catch (_) {}
    return 'image/png';
  }

  static String _extFromMime(String mime) {
    switch (mime.toLowerCase()) {
      case 'image/jpeg':
      case 'image/jpg':
        return 'jpg';
      case 'image/webp':
        return 'webp';
      case 'image/gif':
        return 'gif';
      case 'image/png':
      default:
        return 'png';
    }
  }
}
