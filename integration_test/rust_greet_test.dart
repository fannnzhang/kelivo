import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:Kelivo/src/rust/frb_generated.dart';
import 'package:Kelivo/src/rust/api/simple.dart' as api;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  setUpAll(() async {
    await RustLib.init();
  });

  test('greet returns expected string', () async {
    final res = api.greet(name: 'World');
    expect(res, 'Hello, World!');
  });
}

