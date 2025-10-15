import 'dart:io';

import 'package:flutter/foundation.dart';
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

class _BenchmarkResult {
  const _BenchmarkResult({
    required this.replaceDuration,
    required this.inlineDuration,
  });

  final Duration replaceDuration;
  final Duration inlineDuration;

  double msReplace(int iterations) =>
      replaceDuration.inMicroseconds / 1000.0 / iterations;

  double msInline(int iterations) =>
      inlineDuration.inMicroseconds / 1000.0 / iterations;
}

Future<void> _ensureRustInit() async {
  if (!_rustInitialized) {
    await RustLib.init();
    _rustInitialized = true;
  }
}

bool _rustInitialized = false;

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  final originalProvider = PathProviderPlatform.instance;
  late Directory docsDir;
  late String fixtureMarkdown;

  setUpAll(() {
    docsDir = Directory.systemTemp.createTempSync('kelivo_benchmark_docs');
    PathProviderPlatform.instance = _FakePathProvider(docsDir.path);

    final raw = File(
      'test/testdata/benchmark/benchmark_markdown.md',
    ).readAsStringSync();

    final replacement = <String, String>{};
    final sourcesDir = Directory('${docsDir.path}/local_sources')
      ..createSync(recursive: true);

    for (int i = 0; i < 3; i++) {
      final file = File('${sourcesDir.path}/local_$i.png');
      final bytes = List<int>.generate(3072, (index) => (index + i) % 256);
      file.writeAsBytesSync(bytes, flush: true);
      replacement['{{LOCAL_$i}}'] = file.path;
    }

    fixtureMarkdown = replacement.entries.fold(
      raw,
      (acc, entry) => acc.replaceAll(entry.key, entry.value),
    );
  });

  tearDownAll(() {
    PathProviderPlatform.instance = originalProvider;
    if (docsDir.existsSync()) {
      docsDir.deleteSync(recursive: true);
    }
  });

  test('markdown sanitizer mock vs real benchmark', () async {
    const iterations = 5;
    final mock = await _runBenchmark(
      mode: MarkdownSanitizerMode.mock,
      markdown: fixtureMarkdown,
      docsDir: docsDir,
      iterations: iterations,
    );

    final real = await _runBenchmark(
      mode: MarkdownSanitizerMode.real,
      markdown: fixtureMarkdown,
      docsDir: docsDir,
      iterations: iterations,
    );

    final mockReplaceMs = mock.msReplace(iterations);
    final realReplaceMs = real.msReplace(iterations);
    final mockInlineMs = mock.msInline(iterations);
    final realInlineMs = real.msInline(iterations);

    // Log human readable stats for documentation capture.
    debugPrint('Benchmark iterations: $iterations');
    debugPrint('Mock replace avg (ms): ${mockReplaceMs.toStringAsFixed(2)}');
    debugPrint('Real replace avg (ms): ${realReplaceMs.toStringAsFixed(2)}');
    debugPrint('Mock inline avg (ms): ${mockInlineMs.toStringAsFixed(2)}');
    debugPrint('Real inline avg (ms): ${realInlineMs.toStringAsFixed(2)}');

    expect(
      realReplaceMs < mockReplaceMs,
      isTrue,
      reason: 'Rust replaceInlineBase64Images should be faster than Dart mock',
    );
    expect(
      realInlineMs <= mockInlineMs * 1.2,
      isTrue,
      reason:
          'Rust inlineLocalImagesToBase64 should be comparable to Dart mock',
    );
  });
}

Future<_BenchmarkResult> _runBenchmark({
  required MarkdownSanitizerMode mode,
  required String markdown,
  required Directory docsDir,
  required int iterations,
}) async {
  FeatureFlags.setMarkdownSanitizerMode(mode);
  if (mode == MarkdownSanitizerMode.real) {
    await _ensureRustInit();
  }

  final outputDir = _resolveImagesDir(docsDir, mode);

  Duration replaceDuration = Duration.zero;
  Duration inlineDuration = Duration.zero;

  for (int i = 0; i < iterations; i++) {
    if (outputDir.existsSync()) {
      outputDir.deleteSync(recursive: true);
    }
    outputDir.createSync(recursive: true);

    final replaceWatch = Stopwatch()..start();
    final replaced = await MarkdownMediaSanitizer.replaceInlineBase64Images(
      markdown,
    );
    replaceWatch.stop();
    replaceDuration += replaceWatch.elapsed;
    expect(replaced.isNotEmpty, isTrue);

    final inlineWatch = Stopwatch()..start();
    final inlined = await MarkdownMediaSanitizer.inlineLocalImagesToBase64(
      markdown,
    );
    inlineWatch.stop();
    inlineDuration += inlineWatch.elapsed;
    expect(inlined.isNotEmpty, isTrue);
  }

  return _BenchmarkResult(
    replaceDuration: replaceDuration,
    inlineDuration: inlineDuration,
  );
}

Directory _resolveImagesDir(Directory docsDir, MarkdownSanitizerMode mode) {
  if (mode == MarkdownSanitizerMode.real) {
    final envDir = Platform.environment['KELIVO_SANITIZER_IMAGE_DIR'];
    if (envDir != null && envDir.isNotEmpty) {
      return Directory(envDir);
    }
  }
  return Directory('${docsDir.path}/images');
}
