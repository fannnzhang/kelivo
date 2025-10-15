# FRB Build & Switch Guide

- Refresh deps: `fvm flutter pub get`
- Regenerate bindings: `flutter_rust_bridge_codegen generate`
- Quality gates:
  - `fvm dart format lib test --set-exit-if-changed`
  - `fvm dart analyze`
  - `fvm flutter test`

Run modes:
- Mock: `fvm flutter run --dart-define=USE_RUST=false`
- Real: `fvm flutter run --dart-define=USE_RUST=true`

Notes:
- Uses `flutter_rust_bridge` v2; config: `flutter_rust_bridge.yaml`.
- MCP/context7 for docs: library `/fzyzcjy/flutter_rust_bridge`.
- Configure MCP env: `MCP_CONTEXT7_URL`, `MCP_CONTEXT7_TOKEN`.

