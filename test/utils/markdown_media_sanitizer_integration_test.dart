import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path_provider_platform_interface/path_provider_platform_interface.dart';

import 'package:Kelivo/config/feature_flags.dart';
import 'package:Kelivo/src/rust/frb_generated.dart';
import 'package:Kelivo/utils/markdown_media_sanitizer.dart';

class _FakePathProvider extends PathProviderPlatform {
  _FakePathProvider(this._documentsPath);

  final String _documentsPath;

  @override
  Future<String?> getApplicationDocumentsPath() async => _documentsPath;
}

String _extractImageLink(String markdown) {
  final start = markdown.indexOf('](');
  if (start == -1) return markdown;
  final end = markdown.indexOf(')', start);
  if (end == -1) return markdown.substring(start + 2);
  return markdown.substring(start + 2, end);
}

Future<void> _ensureRustInitialized() async {
  if (!_rustReady) {
    await RustLib.init();
    _rustReady = true;
  }
}

bool _rustReady = false;

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  final originalProvider = PathProviderPlatform.instance;
  late Directory docsDir;

  setUp(() {
    docsDir = Directory.systemTemp.createTempSync('kelivo_markdown_docs');
    PathProviderPlatform.instance = _FakePathProvider(docsDir.path);
    FeatureFlags.setMarkdownSanitizerMode(MarkdownSanitizerMode.mock);
  });

  tearDown(() {
    if (docsDir.existsSync()) {
      docsDir.deleteSync(recursive: true);
    }
    PathProviderPlatform.instance = originalProvider;
  });

  group('MarkdownMediaSanitizer mock mode', () {
    test(
      'replaces base64 images and inlines local files using Dart path',
      () async {
        FeatureFlags.setMarkdownSanitizerMode(MarkdownSanitizerMode.mock);

        final payload = base64Encode(
          utf8.encode('mock-image-${DateTime.now().microsecondsSinceEpoch}'),
        );
        final markdown = '![mock](data:image/png;base64,$payload)';
        final replaced = await MarkdownMediaSanitizer.replaceInlineBase64Images(
          markdown,
        );

        final replacedPath = _extractImageLink(replaced);
        expect(replacedPath.contains('/images/'), isTrue);
        final replacedFile = File(replacedPath);
        expect(replacedFile.existsSync(), isTrue);

        final localImage = File('${docsDir.path}/local.png');
        localImage.createSync(recursive: true);
        localImage.writeAsBytesSync(utf8.encode('mock-local-image'));

        final localMarkdown = '![local](${localImage.path})';
        final inlined = await MarkdownMediaSanitizer.inlineLocalImagesToBase64(
          localMarkdown,
        );
        expect(inlined.startsWith('![local](data:image/png;base64,'), isTrue);
        final encoded = inlined.substring(
          '![local](data:image/png;base64,'.length,
          inlined.length - 1,
        );
        expect(encoded, base64Encode(utf8.encode('mock-local-image')));
      },
    );
  });

  group('MarkdownMediaSanitizer real mode', () {
    test('delegates to Rust implementation', () async {
      await _ensureRustInitialized();
      FeatureFlags.setMarkdownSanitizerMode(MarkdownSanitizerMode.real);

      final payload = base64Encode(
        utf8.encode('real-image-${DateTime.now().microsecondsSinceEpoch}'),
      );
      final markdown = '![real](data:image/png;base64,$payload)';
      final replaced = await MarkdownMediaSanitizer.replaceInlineBase64Images(
        markdown,
      );
      final replacedPath = _extractImageLink(replaced);
      final replacedFile = File(replacedPath);
      expect(replacedFile.existsSync(), isTrue);
      final configuredDir = Platform.environment['KELIVO_SANITIZER_IMAGE_DIR'];
      if (configuredDir != null && configuredDir.isNotEmpty) {
        expect(replacedPath.startsWith(configuredDir), isTrue);
      }
      expect(replacedPath.endsWith('.png'), isTrue);

      final localImage = File('${docsDir.path}/real-local.png');
      localImage.createSync(recursive: true);
      localImage.writeAsBytesSync(utf8.encode('real-mode-local'));

      final localMarkdown = '![local](${localImage.path})';
      final inlined = await MarkdownMediaSanitizer.inlineLocalImagesToBase64(
        localMarkdown,
      );
      expect(inlined.startsWith('![local](data:image/png;base64,'), isTrue);
      final encoded = inlined.substring(
        '![local](data:image/png;base64,'.length,
        inlined.length - 1,
      );
      expect(base64Decode(encoded), utf8.encode('real-mode-local'));
    });
  });
}
