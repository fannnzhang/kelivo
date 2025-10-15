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
}
