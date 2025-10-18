import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../../core/providers/settings_provider.dart';
import '../../../icons/lucide_adapter.dart';
import '../../scan/pages/qr_scan_page.dart';
import 'package:image_picker/image_picker.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import '../../../l10n/app_localizations.dart';
import '../../../shared/widgets/snackbar.dart';
import '../../../core/services/haptics.dart';
import '../../../shared/widgets/ios_tile_button.dart';

class _ImportResult {
  final String key;
  final ProviderConfig cfg;
  _ImportResult(this.key, this.cfg);
}

List<_ImportResult> _decodeChatBoxJson(BuildContext context, String s) {
  final settings = context.read<SettingsProvider>();
  final Map<String, dynamic> obj = jsonDecode(s) as Map<String, dynamic>;
  final providers = (obj['providers'] as Map?)?.map((k, v) => MapEntry(k.toString(), v)) ?? {};
  final out = <_ImportResult>[];
  String uniqueKey(String prefix, String display) {
    final existing = settings.providerConfigs.keys.toSet();
    // If display equals prefix, generate '<prefix> - <n>' starting at 1
    if (display.toLowerCase() == prefix.toLowerCase()) {
      int i = 1;
      String candidate = '$prefix - $i';
      while (existing.contains(candidate)) {
        i++;
        candidate = '$prefix - $i';
      }
      return candidate;
    }
    // Otherwise prefer '<prefix> - <display>', then add ' (n)'
    String base = '$prefix - $display';
    if (!existing.contains(base)) return base;
    int i = 2;
    String candidate = '$base ($i)';
    while (existing.contains(candidate)) {
      i++;
      candidate = '$base ($i)';
    }
    return candidate;
  }
  // OpenAI
  final openai = providers['openai'] as Map?;
  if (openai != null) {
    final apiKey = (openai['apiKey'] ?? '').toString();
    final baseUrl = (openai['baseUrl'] ?? '').toString();
    final providedName = (openai['name'] ?? '').toString();
    if (apiKey.trim().isNotEmpty) {
      final name = providedName.isNotEmpty ? providedName : 'OpenAI-Compat';
      final key = uniqueKey('OpenAI', name);
      final cfg = ProviderConfig(
        id: key,
        enabled: true,
        name: name,
        apiKey: apiKey,
        baseUrl: baseUrl.isNotEmpty ? baseUrl : 'https://api.openai.com/v1',
        providerType: ProviderKind.openai,
        chatPath: '/chat/completions',
        useResponseApi: false,
        models: const [],
        modelOverrides: const {},
        proxyEnabled: false,
        proxyHost: '',
        proxyPort: '8080',
        proxyUsername: '',
        proxyPassword: '',
      );
      out.add(_ImportResult(key, cfg));
    }
  }
  // Claude
  final claude = providers['claude'] as Map?;
  if (claude != null) {
    final apiKey = (claude['apiKey'] ?? '').toString();
    final baseUrl = (claude['baseUrl'] ?? '').toString();
    final providedName = (claude['name'] ?? '').toString();
    if (apiKey.trim().isNotEmpty) {
      final name = providedName.isNotEmpty ? providedName : 'Claude';
      final key = uniqueKey('Claude', name);
      final cfg = ProviderConfig(
        id: key,
        enabled: true,
        name: name,
        apiKey: apiKey,
        baseUrl: baseUrl.isNotEmpty ? baseUrl : 'https://api.anthropic.com/v1',
        providerType: ProviderKind.claude,
        models: const [],
        modelOverrides: const {},
        proxyEnabled: false,
        proxyHost: '',
        proxyPort: '8080',
        proxyUsername: '',
        proxyPassword: '',
      );
      out.add(_ImportResult(key, cfg));
    }
  }
  // Gemini
  final gemini = providers['gemini'] as Map?;
  if (gemini != null) {
    final apiKey = (gemini['apiKey'] ?? '').toString();
    final providedName = (gemini['name'] ?? '').toString();
    if (apiKey.trim().isNotEmpty) {
      final name = providedName.isNotEmpty ? providedName : 'Gemini';
      final key = uniqueKey('Google', name);
      final cfg = ProviderConfig(
        id: key,
        enabled: true,
        name: name,
        apiKey: apiKey,
        baseUrl: 'https://generativelanguage.googleapis.com/v1beta',
        providerType: ProviderKind.google,
        vertexAI: false,
        location: '',
        projectId: '',
        models: const [],
        modelOverrides: const {},
        proxyEnabled: false,
        proxyHost: '',
        proxyPort: '8080',
        proxyUsername: '',
        proxyPassword: '',
      );
      out.add(_ImportResult(key, cfg));
    }
  }
  return out;
}

_ImportResult _decodeSingle(BuildContext context, String s) {
  if (!s.trim().startsWith('ai-provider:v1:')) {
    throw FormatException('Invalid prefix');
  }
  final base64Str = s.trim().substring('ai-provider:v1:'.length);
  final jsonStr = utf8.decode(base64Decode(base64Str));
  final obj = jsonDecode(jsonStr) as Map<String, dynamic>;
  final type = (obj['type'] ?? '').toString();
  final name = (obj['name'] ?? '').toString();
  final apiKey = (obj['apiKey'] ?? '').toString();
  final baseUrl = (obj['baseUrl'] ?? '').toString();

  final settings = context.read<SettingsProvider>();
  String uniqueKey(String prefix, String display) {
    final existing = settings.providerConfigs.keys.toSet();
    if (display.toLowerCase() == prefix.toLowerCase()) {
      int i = 1;
      String candidate = '$prefix - $i';
      while (existing.contains(candidate)) {
        i++;
        candidate = '$prefix - $i';
      }
      return candidate;
    }
    String base = '$prefix - $display';
    if (!existing.contains(base)) return base;
    int i = 2;
    String candidate = '$base ($i)';
    while (existing.contains(candidate)) {
      i++;
      candidate = '$base ($i)';
    }
    return candidate;
  }

  if (type == 'openai-compat') {
    final key = uniqueKey('OpenAI', name.isEmpty ? 'OpenAI' : name);
    final cfg = ProviderConfig(
      id: key,
      enabled: true,
      name: name.isEmpty ? 'OpenAI' : name,
      apiKey: apiKey,
      baseUrl: baseUrl.isNotEmpty ? baseUrl : 'https://api.openai.com/v1',
      providerType: ProviderKind.openai,
      chatPath: '/chat/completions',
      useResponseApi: false,
      models: const [],
      modelOverrides: const {},
      proxyEnabled: false,
      proxyHost: '',
      proxyPort: '8080',
      proxyUsername: '',
      proxyPassword: '',
    );
    return _ImportResult(key, cfg);
  } else if (type == 'google') {
    final key = uniqueKey('Google', name.isEmpty ? 'Google' : name);
    final cfg = ProviderConfig(
      id: key,
      enabled: true,
      name: name.isEmpty ? 'Google' : name,
      apiKey: apiKey,
      baseUrl: 'https://generativelanguage.googleapis.com/v1beta',
      providerType: ProviderKind.google,
      vertexAI: false,
      location: '',
      projectId: '',
      models: const [],
      modelOverrides: const {},
      proxyEnabled: false,
      proxyHost: '',
      proxyPort: '8080',
      proxyUsername: '',
      proxyPassword: '',
    );
    return _ImportResult(key, cfg);
  } else if (type == 'claude') {
    final key = uniqueKey('Claude', name.isEmpty ? 'Claude' : name);
    final cfg = ProviderConfig(
      id: key,
      enabled: true,
      name: name.isEmpty ? 'Claude' : name,
      apiKey: apiKey,
      baseUrl: baseUrl.isNotEmpty ? baseUrl : 'https://api.anthropic.com/v1',
      providerType: ProviderKind.claude,
      models: const [],
      modelOverrides: const {},
      proxyEnabled: false,
      proxyHost: '',
      proxyPort: '8080',
      proxyUsername: '',
      proxyPassword: '',
    );
    return _ImportResult(key, cfg);
  } else {
    throw FormatException('Unknown provider type: $type');
  }
}

Future<void> showImportProviderSheet(BuildContext context) async {
  final cs = Theme.of(context).colorScheme;
  final controller = TextEditingController();
  await showModalBottomSheet<void>(
    context: context,
    isScrollControlled: true,
    backgroundColor: cs.surface,
    shape: const RoundedRectangleBorder(
      borderRadius: BorderRadius.vertical(top: Radius.circular(16)),
    ),
    builder: (ctx) {
      final l10n = AppLocalizations.of(ctx)!;
      return SafeArea(
        top: false,
        child: AnimatedPadding(
          duration: const Duration(milliseconds: 180),
          curve: Curves.easeOutCubic,
          padding: () {
            final mq = MediaQuery.of(ctx);
            return EdgeInsets.fromLTRB(
              16,
              10,
              16,
              10 + mq.padding.bottom + mq.viewInsets.bottom,
            );
          }(),
          child: FractionallySizedBox(
            heightFactor: 0.55,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Center(child: Container(width: 40, height: 4, decoration: BoxDecoration(color: cs.onSurface.withOpacity(0.2), borderRadius: BorderRadius.circular(999)))),
                const SizedBox(height: 12),
                // iOS-style header: centered title with left/right actions
                SizedBox(
                  height: 36,
                  child: Stack(
                    children: [
                      Align(
                        alignment: Alignment.center,
                        child: Text(
                          l10n.importProviderSheetTitle,
                          style: const TextStyle(fontSize: 18, fontWeight: FontWeight.w600),
                        ),
                      ),
                      Align(
                        alignment: Alignment.centerLeft,
                        child: _TactileIconButton(
                          icon: Lucide.Camera,
                          color: cs.onSurface,
                          size: 22,
                          semanticLabel: l10n.importProviderSheetScanQrTooltip,
                          onTap: () async {
                            final code = await Navigator.of(ctx).push<String>(
                              MaterialPageRoute(builder: (_) => const QrScanPage()),
                            );
                            if (code == null || code.isEmpty) return;
                            try {
                              final settings = ctx.read<SettingsProvider>();
                              final results = <_ImportResult>[];
                              // Support combined multi-provider QR content: newline-separated share strings or JSON
                              final parts = code.split(RegExp(r'\r?\n+')).map((e)=>e.trim()).where((e)=>e.isNotEmpty).toList();
                              if (parts.length > 1) {
                                for (final p in parts) {
                                  try {
                                    if (p.startsWith('ai-provider:v1:')) {
                                      results.add(_decodeSingle(ctx, p));
                                    } else if (p.startsWith('{')) {
                                      results.addAll(_decodeChatBoxJson(ctx, p));
                                    }
                                  } catch (_) {}
                                }
                                if (results.isEmpty) {
                                  throw const FormatException('Unsupported format');
                                }
                              } else {
                                final p = parts.first;
                                if (p.startsWith('ai-provider:v1:')) {
                                  results.add(_decodeSingle(ctx, p));
                                } else if (p.startsWith('{')) {
                                  results.addAll(_decodeChatBoxJson(ctx, p));
                                } else {
                                  throw const FormatException('Unsupported format');
                                }
                              }
                              for (final r in results) {
                                await settings.setProviderConfig(r.key, r.cfg);
                                final order = List<String>.of(settings.providersOrder);
                                order.remove(r.key);
                                order.insert(0, r.key);
                                await settings.setProvidersOrder(order);
                              }
                              Navigator.of(ctx).pop();
                              showAppSnackBar(
                                context,
                                message: l10n.importProviderSheetImportSuccessMessage(results.length),
                                type: NotificationType.success,
                              );
                            } catch (e) {
                              showAppSnackBar(
                                ctx,
                                message: l10n.importProviderSheetImportFailedMessage(e.toString()),
                                type: NotificationType.error,
                              );
                            }
                          },
                        ),
                      ),
                      Align(
                        alignment: Alignment.centerRight,
                        child: _TactileIconButton(
                          icon: Lucide.Image,
                          color: cs.onSurface,
                          size: 22,
                          semanticLabel: l10n.importProviderSheetFromGalleryTooltip,
                          onTap: () async {
                            try {
                              // pick from gallery and analyze
                              final picker = ImagePicker();
                              final img = await picker.pickImage(source: ImageSource.gallery);
                              if (img == null) return;
                              final scanner = MobileScannerController();
                              final result = await scanner.analyzeImage(img.path);
                              String? code;
                              if (result != null) {
                                try {
                                  // dynamic access to barcodes for compatibility
                                  final bars = (result as dynamic).barcodes as List?;
                                  if (bars != null) {
                                    for (final b in bars) {
                                      final v = (b as dynamic).rawValue as String?;
                                      if (v != null && v.isNotEmpty) { code = v; break; }
                                    }
                                  }
                                } catch (_) {}
                              }
                              if (code == null || code!.isEmpty) throw 'QR not detected';
                              final settings = ctx.read<SettingsProvider>();
                              final results = <_ImportResult>[];
                              final parts = code!.split(RegExp(r'\r?\n+')).map((e)=>e.trim()).where((e)=>e.isNotEmpty).toList();
                              if (parts.length > 1) {
                                for (final p in parts) {
                                  try {
                                    if (p.startsWith('ai-provider:v1:')) {
                                      results.add(_decodeSingle(ctx, p));
                                    } else if (p.startsWith('{')) {
                                      results.addAll(_decodeChatBoxJson(ctx, p));
                                    }
                                  } catch (_) {}
                                }
                                if (results.isEmpty) throw 'Unsupported content';
                              } else {
                                final p = parts.first;
                                if (p.startsWith('ai-provider:v1:')) {
                                  results.add(_decodeSingle(ctx, p));
                                } else if (p.startsWith('{')) {
                                  results.addAll(_decodeChatBoxJson(ctx, p));
                                } else {
                                  throw 'Unsupported content';
                                }
                              }
                              for (final r in results) {
                                await settings.setProviderConfig(r.key, r.cfg);
                                final order = List<String>.of(settings.providersOrder);
                                order.remove(r.key);
                                order.insert(0, r.key);
                                await settings.setProvidersOrder(order);
                              }
                              Navigator.of(ctx).pop();
                              showAppSnackBar(
                                context,
                                message: l10n.importProviderSheetImportSuccessMessage(results.length),
                                type: NotificationType.success,
                              );
                            } catch (e) {
                              showAppSnackBar(
                                ctx,
                                message: l10n.importProviderSheetImportFailedMessage(e.toString()),
                                type: NotificationType.error,
                              );
                            }
                          },
                        ),
                      ),
                    ],
                  ),
                ),
                const SizedBox(height: 8),
                Flexible(
                  fit: FlexFit.loose,
                  child: ListView(
                    shrinkWrap: true,
                    physics: const NeverScrollableScrollPhysics(),
                    padding: const EdgeInsets.only(bottom: 8),
                    children: [
                      TextField(
                        controller: controller,
                        maxLines: 10,
                        decoration: InputDecoration(
                          hintText: l10n.importProviderSheetDescription,
                          filled: true,
                          fillColor: Theme.of(ctx).brightness == Brightness.dark ? Colors.white10 : Colors.white,
                          border: OutlineInputBorder(
                            borderRadius: BorderRadius.circular(12),
                            borderSide: BorderSide(color: cs.outlineVariant.withOpacity(0.4)),
                          ),
                          enabledBorder: OutlineInputBorder(
                            borderRadius: BorderRadius.circular(12),
                            borderSide: BorderSide(color: cs.outlineVariant.withOpacity(0.4)),
                          ),
                          focusedBorder: OutlineInputBorder(
                            borderRadius: BorderRadius.circular(12),
                            borderSide: BorderSide(color: cs.primary.withOpacity(0.5)),
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
                Row(
                  children: [
                    Expanded(
                      child: IosTileButton(
                        icon: Lucide.X,
                        label: l10n.importProviderSheetCancelButton,
                        onTap: () {
                          Haptics.light();
                          FocusScope.of(ctx).unfocus();
                          WidgetsBinding.instance.addPostFrameCallback((_) {
                            if (Navigator.of(ctx).canPop()) {
                              Navigator.of(ctx).maybePop();
                            }
                          });
                        },
                      ),
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: IosTileButton(
                        icon: Lucide.Import,
                        label: l10n.importProviderSheetImportButton,
                        onTap: () async {
                          final raw = controller.text.trim();
                          if (raw.isEmpty) return;
                          try {
                            final settings = ctx.read<SettingsProvider>();
                            final results = <_ImportResult>[];
                            // Support multi-line input where each non-empty line is a share string or JSON
                            final lines = raw.split(RegExp(r'\r?\n'))
                                .map((e) => e.trim())
                                .where((e) => e.isNotEmpty)
                                .toList();
                            if (lines.length > 1) {
                              for (final line in lines) {
                                try {
                                  if (line.startsWith('ai-provider:v1:')) {
                                    results.add(_decodeSingle(ctx, line));
                                  } else if (line.startsWith('{')) {
                                    results.addAll(_decodeChatBoxJson(ctx, line));
                                  }
                                } catch (_) {
                                  // skip invalid line
                                }
                              }
                              if (results.isEmpty) {
                                throw const FormatException('No valid lines');
                              }
                            } else {
                              final text = lines.first;
                              if (text.startsWith('ai-provider:v1:')) {
                                results.add(_decodeSingle(ctx, text));
                              } else if (text.startsWith('{')) {
                                results.addAll(_decodeChatBoxJson(ctx, text));
                              } else {
                                throw const FormatException('Unsupported format');
                              }
                            }
                            for (final r in results) {
                              await settings.setProviderConfig(r.key, r.cfg);
                              // Put to front
                              final order = List<String>.of(settings.providersOrder);
                              order.remove(r.key);
                              order.insert(0, r.key);
                              await settings.setProvidersOrder(order);
                            }
                            Navigator.of(ctx).pop();
                            showAppSnackBar(
                              context,
                              message: l10n.importProviderSheetImportSuccessMessage(results.length),
                              type: NotificationType.success,
                            );
                          } catch (e) {
                            showAppSnackBar(
                              ctx,
                              message: l10n.importProviderSheetImportFailedMessage(e.toString()),
                              type: NotificationType.error,
                            );
                          }
                        },
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
      );
    },
  );
}

// Local iOS-like tactile icon button (no ripple, light haptics)
class _TactileIconButton extends StatefulWidget {
  const _TactileIconButton({
    required this.icon,
    required this.color,
    required this.onTap,
    this.semanticLabel,
    this.size = 22,
    this.haptics = true,
  });

  final IconData icon;
  final Color color;
  final VoidCallback onTap;
  final String? semanticLabel;
  final double size;
  final bool haptics;

  @override
  State<_TactileIconButton> createState() => _TactileIconButtonState();
}

class _TactileIconButtonState extends State<_TactileIconButton> {
  bool _pressed = false;
  @override
  Widget build(BuildContext context) {
    final base = widget.color;
    final pressColor = base.withOpacity(0.7);
    final icon = Icon(
      widget.icon,
      size: widget.size,
      color: _pressed ? pressColor : base,
      semanticLabel: widget.semanticLabel,
    );

    return Semantics(
      button: true,
      label: widget.semanticLabel,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTapDown: (_) => setState(() => _pressed = true),
        onTapUp: (_) => setState(() => _pressed = false),
        onTapCancel: () => setState(() => _pressed = false),
        onTap: () { if (widget.haptics) Haptics.light(); widget.onTap(); },
        child: AnimatedScale(
          scale: _pressed ? 0.95 : 1.0,
          duration: const Duration(milliseconds: 100),
          curve: Curves.easeOut,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 6),
            child: icon,
          ),
        ),
      ),
    );
  }
}
