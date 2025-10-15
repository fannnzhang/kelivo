import 'frb_generated.dart';

class MockRustLibApi implements RustLibApi {
  @override
  String crateApiSimpleGreet({required String name}) {
    return 'Hello, $name!';
  }

  @override
  Future<void> crateApiSimpleInitApp() async {}
}
