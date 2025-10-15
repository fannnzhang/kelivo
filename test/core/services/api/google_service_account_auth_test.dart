import 'dart:convert';
import 'dart:io';

import 'package:Kelivo/core/services/api/google_service_account_auth.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('GoogleServiceAccountAuth.getAccessToken', () {
    late HttpServer server;
    late Uri tokenUri;

    setUp(() async {
      server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
      tokenUri = Uri.parse(
        'http://${server.address.host}:${server.port}/token',
      );
      server.listen((request) async {
        final body = await utf8.decoder.bind(request).join();
        final params = Uri.splitQueryString(body);
        expect(
          params['grant_type'],
          equals('urn:ietf:params:oauth:grant-type:jwt-bearer'),
          reason: 'grant_type should be JWT bearer',
        );
        final assertion = params['assertion'];
        expect(assertion, isNotEmpty);
        final segments = assertion!.split('.');
        expect(segments.length, 3);

        final payload = _decodeJwtPayload(segments[1]);
        expect(payload['aud'], tokenUri.toString());
        expect(payload['iss'], _testClientEmail);

        request.response.statusCode = 200;
        request.response.headers.contentType = ContentType.json;
        request.response.write(
          jsonEncode({'access_token': 'mock-token', 'expires_in': 3600}),
        );
        await request.response.close();
      });
    });

    tearDown(() async {
      await server.close(force: true);
    });

    test('returns access token using injected JWT factory', () async {
      final creds = GoogleServiceAccountCredentials(
        clientEmail: _testClientEmail,
        privateKey: 'unused-testing-key',
        tokenUri: tokenUri.toString(),
      );

      final token = await GoogleServiceAccountAuth.getAccessToken(
        creds,
        scopes: const ['https://www.googleapis.com/auth/cloud-platform'],
        createJwt: _stubJwtFactory,
      );

      expect(token, 'mock-token');
    });
  });
}

Map<String, dynamic> _decodeJwtPayload(String segment) {
  final normalized = base64Url.normalize(segment);
  final bytes = base64Url.decode(normalized);
  return json.decode(utf8.decode(bytes)) as Map<String, dynamic>;
}

Future<String> _stubJwtFactory(
  GoogleServiceAccountCredentials creds,
  List<String> scopes,
) async {
  const header = {'alg': 'RS256', 'typ': 'JWT'};
  final payload = {
    'iss': creds.clientEmail,
    'scope': scopes.join(' '),
    'aud': creds.tokenUri,
    'iat': 1,
    'exp': 3601,
  };
  final encodedHeader = _encodeJson(header);
  final encodedPayload = _encodeJson(payload);
  const signature = 'stub-signature';
  return '$encodedHeader.$encodedPayload.$signature';
}

String _encodeJson(Map<String, dynamic> jsonObject) {
  final jsonString = json.encode(jsonObject);
  return base64Url.encode(utf8.encode(jsonString)).replaceAll('=', '');
}

const String _testClientEmail = 'test-service@example.iam.gserviceaccount.com';
