import 'package:flutter/foundation.dart';

enum MarkdownSanitizerMode { mock, real }

class FeatureFlags {
  FeatureFlags._();

  static const bool _useRust = bool.fromEnvironment(
    'USE_RUST',
    defaultValue: kDebugMode,
  );

  static const bool _useRustMarkdownSanitizer = bool.fromEnvironment(
    'USE_RUST_MARKDOWN_SANITIZER',
    defaultValue: _useRust,
  );

  static MarkdownSanitizerMode markdownSanitizerMode = _useRustMarkdownSanitizer
      ? MarkdownSanitizerMode.real
      : MarkdownSanitizerMode.mock;

  static bool get useRustMarkdownSanitizer =>
      markdownSanitizerMode == MarkdownSanitizerMode.real;

  static void setMarkdownSanitizerMode(MarkdownSanitizerMode mode) {
    markdownSanitizerMode = mode;
  }
}
