import 'frb_generated.dart';

class MockRustLibApi implements RustLibApi {
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
}
