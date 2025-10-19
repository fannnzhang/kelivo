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

  static const bool _useRustLlm = bool.fromEnvironment(
    'USE_RUST_LLM',
    defaultValue: _useRust,
  );

  static const bool _useRustDb = bool.fromEnvironment(
    'USE_RUST_DB',
    defaultValue: _useRust,
  );

  static MarkdownSanitizerMode markdownSanitizerMode = _useRustMarkdownSanitizer
      ? MarkdownSanitizerMode.real
      : MarkdownSanitizerMode.mock;

  static bool _runtimeUseRustLlm = _useRustLlm;
  static bool _runtimeUseRustDb = _useRustDb;

  static bool get useRustMarkdownSanitizer =>
      markdownSanitizerMode == MarkdownSanitizerMode.real;

  static bool get useRustLlm => _runtimeUseRustLlm;

  static bool get useRustDb => _runtimeUseRustDb;

  static void setMarkdownSanitizerMode(MarkdownSanitizerMode mode) {
    markdownSanitizerMode = mode;
  }

  static void setUseRustLlm(bool value) {
    _runtimeUseRustLlm = value;
  }

  static void setUseRustDb(bool value) {
    _runtimeUseRustDb = value;
  }
}
