import 'dart:convert';
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import '../../../utils/brand_assets.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:flutter/gestures.dart';
import 'package:provider/provider.dart';
import 'package:file_picker/file_picker.dart';
import '../../../icons/lucide_adapter.dart';
import '../../../core/providers/settings_provider.dart';
import '../../../core/providers/model_provider.dart';
import '../../../core/providers/assistant_provider.dart';
import '../../model/widgets/model_detail_sheet.dart';
import '../../model/widgets/model_select_sheet.dart';
import '../widgets/share_provider_sheet.dart';
import 'package:flutter_slidable/flutter_slidable.dart';
import '../../../l10n/app_localizations.dart';
import '../../../shared/widgets/snackbar.dart';
import '../../../shared/widgets/ios_checkbox.dart';
import '../../../shared/widgets/ios_switch.dart';
import 'multi_key_manager_page.dart';
import 'provider_network_page.dart';
import '../../../core/services/haptics.dart';

class ProviderDetailPage extends StatefulWidget {
  const ProviderDetailPage({super.key, required this.keyName, required this.displayName});
  final String keyName;
  final String displayName;

  @override
  State<ProviderDetailPage> createState() => _ProviderDetailPageState();
}

class _ProviderDetailPageState extends State<ProviderDetailPage> {
  final PageController _pc = PageController();
  int _index = 0;
  late ProviderConfig _cfg;
  late ProviderKind _kind;
  final _nameCtrl = TextEditingController();
  final _keyCtrl = TextEditingController();
  final _baseCtrl = TextEditingController();
  final _pathCtrl = TextEditingController();
  // Google Vertex AI extras
  final _locationCtrl = TextEditingController();
  final _projectCtrl = TextEditingController();
  final _saJsonCtrl = TextEditingController();
  bool _enabled = true;
  bool _useResp = false; // openai
  bool _vertexAI = false; // google
  bool _showApiKey = false; // toggle visibility
  bool _multiKeyEnabled = false; // single/multi key mode
  // network proxy (per provider)
  bool _proxyEnabled = false;
  final _proxyHostCtrl = TextEditingController();
  final _proxyPortCtrl = TextEditingController(text: '8080');
  final _proxyUserCtrl = TextEditingController();
  final _proxyPassCtrl = TextEditingController();

  @override
  void initState() {
    super.initState();
    final settings = context.read<SettingsProvider>();
    _cfg = settings.getProviderConfig(widget.keyName, defaultName: widget.displayName);
    _kind = ProviderConfig.classify(widget.keyName, explicitType: _cfg.providerType);
    _enabled = _cfg.enabled;
    _nameCtrl.text = _cfg.name;
    _keyCtrl.text = _cfg.apiKey;
    _baseCtrl.text = _cfg.baseUrl;
    _pathCtrl.text = _cfg.chatPath ?? '/chat/completions';
    _useResp = _cfg.useResponseApi ?? false;
    _vertexAI = _cfg.vertexAI ?? false;
    _locationCtrl.text = _cfg.location ?? '';
    _projectCtrl.text = _cfg.projectId ?? '';
    _saJsonCtrl.text = _cfg.serviceAccountJson ?? '';
    // proxy
    _proxyEnabled = _cfg.proxyEnabled ?? false;
    _proxyHostCtrl.text = _cfg.proxyHost ?? '';
    _proxyPortCtrl.text = _cfg.proxyPort ?? '8080';
    _proxyUserCtrl.text = _cfg.proxyUsername ?? '';
    _proxyPassCtrl.text = _cfg.proxyPassword ?? '';
    _multiKeyEnabled = _cfg.multiKeyEnabled ?? false;
  }

  @override
  void dispose() {
    _pc.dispose();
    _nameCtrl.dispose();
    _keyCtrl.dispose();
    _baseCtrl.dispose();
    _pathCtrl.dispose();
    _locationCtrl.dispose();
    _projectCtrl.dispose();
    _saJsonCtrl.dispose();
    _proxyHostCtrl.dispose();
    _proxyPortCtrl.dispose();
    _proxyUserCtrl.dispose();
    _proxyPassCtrl.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;
    final l10n = AppLocalizations.of(context)!;
    bool _isUserAdded(String key) {
      const fixed = {
        'KelivoIN', 'OpenAI', 'Gemini', 'SiliconFlow', 'OpenRouter',
        'DeepSeek', 'Tensdaq', 'Aliyun', 'Zhipu AI', 'Claude', 'Grok', 'ByteDance',
      };
      return !fixed.contains(key);
    }

    return Scaffold(
      appBar: AppBar(
        leading: Tooltip(
          message: l10n.settingsPageBackButton,
          child: _TactileIconButton(
            icon: Lucide.ArrowLeft,
            color: cs.onSurface,
            semanticLabel: l10n.settingsPageBackButton,
            size: 22,
            onTap: () => Navigator.of(context).maybePop(),
          ),
        ),
        title: Row(
          children: [
            _BrandAvatar(
              name: (_nameCtrl.text.isEmpty ? widget.displayName : _nameCtrl.text),
              size: 22,
            ),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                _nameCtrl.text.isEmpty ? widget.displayName : _nameCtrl.text,
                style: const TextStyle(fontSize: 16),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
            ),
          ],
        ),
        actions: [
          Tooltip(
            message: l10n.providerDetailPageTestButton,
            child: _TactileIconButton(
              icon: Lucide.HeartPulse,
              color: cs.onSurface,
              semanticLabel: l10n.providerDetailPageTestButton,
              size: 22,
              onTap: _openTestDialog,
            ),
          ),
          Tooltip(
            message: l10n.providerDetailPageShareTooltip,
            child: _TactileIconButton(
              icon: Lucide.Share2,
              color: cs.onSurface,
              semanticLabel: l10n.providerDetailPageShareTooltip,
              size: 22,
              onTap: () async {
                await showShareProviderSheet(context, widget.keyName);
              },
            ),
          ),
          if (_isUserAdded(widget.keyName))
            Tooltip(
              message: l10n.providerDetailPageDeleteProviderTooltip,
              child: _TactileIconButton(
                icon: Lucide.Trash2,
                color: cs.error,
                semanticLabel: l10n.providerDetailPageDeleteProviderTooltip,
                size: 22,
                onTap: () async {
                  final confirm = await showDialog<bool>(
                    context: context,
                    builder: (ctx) => AlertDialog(
                      title: Text(l10n.providerDetailPageDeleteProviderTitle),
                      content: Text(l10n.providerDetailPageDeleteProviderContent),
                      actions: [
                        TextButton(onPressed: () => Navigator.of(ctx).pop(false), child: Text(l10n.providerDetailPageCancelButton)),
                        TextButton(onPressed: () => Navigator.of(ctx).pop(true), child: Text(l10n.providerDetailPageDeleteButton, style: const TextStyle(color: Colors.red))),
                      ],
                    ),
                  );
                  if (confirm == true) {
                    // Clear assistant-level model selections that reference this provider
                    try {
                      final ap = context.read<AssistantProvider>();
                      for (final a in ap.assistants) {
                        if (a.chatModelProvider == widget.keyName) {
                          await ap.updateAssistant(a.copyWith(clearChatModel: true));
                        }
                      }
                    } catch (_) {}

                    // Remove provider config and related selections/pins
                    await context.read<SettingsProvider>().removeProviderConfig(widget.keyName);
                    if (!mounted) return;
                    Navigator.of(context).maybePop();
                    showAppSnackBar(
                      context,
                      message: l10n.providerDetailPageProviderDeletedSnackbar,
                      type: NotificationType.success,
                    );
                }
              },
            ),
          ),
          const SizedBox(width: 12),
        ],
      ),
      body: PageView(
        controller: _pc,
        onPageChanged: (i) => setState(() => _index = i),
        children: [
          _buildConfigTab(context, cs, l10n),
          _buildModelsTab(context, cs, l10n),
        ],
      ),
      bottomNavigationBar: SafeArea(
        top: false,
        child: Padding(
          padding: const EdgeInsets.fromLTRB(12, 6, 12, 10),
          child: _BottomTabs(
            index: _index,
            leftIcon: Lucide.Settings2,
            leftLabel: l10n.providerDetailPageConfigTab,
            rightIcon: Lucide.Boxes,
            rightLabel: l10n.providerDetailPageModelsTab,
            onSelect: (i) {
              setState(() => _index = i);
              _pc.animateToPage(
                i,
                duration: const Duration(milliseconds: 220),
                curve: Curves.easeOutCubic,
              );
            },
          ),
        ),
      ),
    );
  }

  Widget _buildConfigTab(BuildContext context, ColorScheme cs, AppLocalizations l10n) {
    return ListView(
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 16),
      children: [
        if (widget.keyName.toLowerCase() == 'kelivoin') ...[
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            decoration: BoxDecoration(
              color: cs.primary.withOpacity(0.08),
              borderRadius: BorderRadius.circular(10),
              border: Border.all(color: cs.primary.withOpacity(0.35)),
            ),
            child: Text.rich(
              TextSpan(
                text: 'Powered by ',
                style: TextStyle(color: cs.onSurface.withOpacity(0.8)),
                children: [
                  TextSpan(
                    text: 'Pollinations AI',
                    style: TextStyle(
                      color: cs.primary,
                      fontWeight: FontWeight.w700,
                    ),
                    recognizer: TapGestureRecognizer()
                      ..onTap = () async {
                        final uri = Uri.parse('https://pollinations.ai');
                        try {
                          final ok = await launchUrl(uri, mode: LaunchMode.externalApplication);
                          if (!ok) {
                            await launchUrl(uri);
                          }
                        } catch (_) {
                          await launchUrl(uri);
                        }
                      },
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 12),
        ],
        if (widget.keyName.toLowerCase() == 'tensdaq') ...[
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            decoration: BoxDecoration(
              color: cs.primary.withOpacity(0.08),
              borderRadius: BorderRadius.circular(10),
              border: Border.all(color: cs.primary.withOpacity(0.35)),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  '革命性竞价 AI MaaS 平台，价格由市场供需决定，告别高成本固定定价。',
                  style: TextStyle(color: cs.onSurface.withOpacity(0.8)),
                ),
                const SizedBox(height: 6),
                Text.rich(
                  TextSpan(
                    text: '官网：',
                    style: TextStyle(color: cs.onSurface.withOpacity(0.8)),
                    children: [
                      TextSpan(
                        text: 'https://dashboard.x-aio.com',
                        style: TextStyle(color: cs.primary, fontWeight: FontWeight.w700),
                        recognizer: TapGestureRecognizer()
                          ..onTap = () async {
                            final uri = Uri.parse('https://dashboard.x-aio.com');
                            try {
                              final ok = await launchUrl(uri, mode: LaunchMode.externalApplication);
                              if (!ok) {
                                await launchUrl(uri);
                              }
                            } catch (_) {
                              await launchUrl(uri);
                            }
                          },
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 12),
        ],
        // 顶部管理分组标题
        Text(l10n.providerDetailPageManageSectionTitle, style: TextStyle(fontSize: 13, color: cs.onSurface.withOpacity(0.8))),
        const SizedBox(height: 6),
        // Top iOS-style section card for key settings
        _iosSectionCard(children: [
          if (widget.keyName.toLowerCase() != 'kelivoin') _providerKindRow(context),
          _iosRow(
            context,
            label: l10n.providerDetailPageEnabledTitle,
            trailing: IosSwitch(value: _enabled, onChanged: (v) { setState(() => _enabled = v); _save(); }),
          ),
          _iosRow(
            context,
            label: l10n.providerDetailPageMultiKeyModeTitle,
            trailing: IosSwitch(value: _multiKeyEnabled, onChanged: (v) { setState(() => _multiKeyEnabled = v); _save(); }),
          ),
          if (_multiKeyEnabled)
          _TactileRow(
            onTap: () async {
              await Navigator.of(context).push(
                MaterialPageRoute(
                  builder: (_) => MultiKeyManagerPage(
                    providerKey: widget.keyName,
                    providerDisplayName: widget.displayName,
                  ),
                ),
              );
              if (mounted) setState(() {});
            },
            builder: (pressed) {
              final base = Theme.of(context).colorScheme.onSurface;
              final isDark = Theme.of(context).brightness == Brightness.dark;
              final target = pressed ? (Color.lerp(base, isDark ? Colors.black : Colors.white, 0.55) ?? base) : base;
              return TweenAnimationBuilder<Color?>(
                tween: ColorTween(end: target),
                duration: const Duration(milliseconds: 220),
                curve: Curves.easeOutCubic,
                builder: (context, color, _) {
                  final c = color ?? base;
                  return Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
                    child: Row(
                      children: [
                        Expanded(child: Text(l10n.providerDetailPageManageKeysButton, style: TextStyle(fontSize: 15, color: c))),
                        Icon(Lucide.ChevronRight, size: 16, color: c),
                      ],
                    ),
                  );
                },
              );
            },
          ),
          if (_kind == ProviderKind.openai)
            _iosRow(
              context,
              label: l10n.providerDetailPageResponseApiTitle,
              trailing: IosSwitch(value: _useResp, onChanged: (v) { setState(() => _useResp = v); _save(); }),
            ),
          if (_kind == ProviderKind.google)
            _iosRow(
              context,
              label: l10n.providerDetailPageVertexAiTitle,
              trailing: IosSwitch(value: _vertexAI, onChanged: (v) { setState(() => _vertexAI = v); _save(); }),
            ),
          _TactileRow(
            onTap: () async {
              await Navigator.of(context).push(
                MaterialPageRoute(
                  builder: (_) => ProviderNetworkPage(
                    providerKey: widget.keyName,
                    providerDisplayName: widget.displayName,
                  ),
                ),
              );
              final settings = context.read<SettingsProvider>();
              final latest = settings.getProviderConfig(widget.keyName, defaultName: widget.displayName);
              setState(() {
                _proxyEnabled = latest.proxyEnabled ?? false;
                _proxyHostCtrl.text = latest.proxyHost ?? '';
                _proxyPortCtrl.text = latest.proxyPort ?? '8080';
              });
            },
            builder: (pressed) {
              final cs2 = Theme.of(context).colorScheme;
              final base = cs2.onSurface;
              final isDark = Theme.of(context).brightness == Brightness.dark;
              final target = pressed ? (Color.lerp(base, isDark ? Colors.black : Colors.white, 0.55) ?? base) : base;
              return TweenAnimationBuilder<Color?>(
                tween: ColorTween(end: target),
                duration: const Duration(milliseconds: 220),
                curve: Curves.easeOutCubic,
                builder: (context, color, _) {
                  final c = color ?? base;
                  return Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
                    child: Row(
                      children: [
                        Expanded(child: Text(l10n.providerDetailPageNetworkTab, style: TextStyle(fontSize: 15, color: c))),
                        Icon(Lucide.ChevronRight, size: 16, color: c),
                      ],
                    ),
                  );
                },
              );
            },
          ),
        ]),
        const SizedBox(height: 12),
        _inputRow(
          context,
          label: l10n.providerDetailPageNameLabel,
          controller: _nameCtrl,
          hint: widget.displayName,
          enabled: widget.keyName.toLowerCase() != 'kelivoin',
          onChanged: (_) => _save(),
        ),
        const SizedBox(height: 12),
        if (!(_kind == ProviderKind.google && _vertexAI)) ...[
          if (widget.keyName.toLowerCase() != 'kelivoin' && !_multiKeyEnabled) ...[
            _inputRow(
              context,
              label: 'API Key',
              controller: _keyCtrl,
              hint: l10n.providerDetailPageApiKeyHint,
              obscure: !_showApiKey,
              suffix: IconButton(
                tooltip: _showApiKey ? l10n.providerDetailPageHideTooltip : l10n.providerDetailPageShowTooltip,
                icon: Icon(_showApiKey ? Lucide.EyeOff : Lucide.Eye, color: cs.onSurface.withOpacity(0.7), size: 18),
                onPressed: () => setState(() => _showApiKey = !_showApiKey),
              ),
              onChanged: (_) => _save(),
            ),
            const SizedBox(height: 12),
          ],
          _inputRow(
            context,
            label: 'API Base URL',
            controller: _baseCtrl,
            hint: ProviderConfig.defaultsFor(widget.keyName, displayName: widget.displayName).baseUrl,
            enabled: widget.keyName.toLowerCase() != 'kelivoin',
            onChanged: (_) => _save(),
          ),
        ],
        if (_kind == ProviderKind.openai && widget.keyName.toLowerCase() != 'kelivoin' && !_useResp) ...[
          const SizedBox(height: 12),
          _inputRow(
            context,
            label: l10n.providerDetailPageApiPathLabel,
            controller: _pathCtrl,
            enabled: widget.keyName.toLowerCase() != 'openai' && widget.keyName.toLowerCase() != 'tensdaq',
            hint: '/chat/completions',
            onChanged: (_) => _save(),
          ),
        ],
        if (_kind == ProviderKind.google) ...[
          const SizedBox(height: 12),
          if (_vertexAI) ...[
            const SizedBox(height: 12),
            _inputRow(context, label: l10n.providerDetailPageLocationLabel, controller: _locationCtrl, hint: 'us-central1', onChanged: (_) => _save()),
            const SizedBox(height: 12),
            _inputRow(context, label: l10n.providerDetailPageProjectIdLabel, controller: _projectCtrl, hint: 'my-project-id', onChanged: (_) => _save()),
            const SizedBox(height: 12),
            _multilineRow(
              context,
              label: l10n.providerDetailPageServiceAccountJsonLabel,
              controller: _saJsonCtrl,
              hint: '{\n  "type": "service_account", ...\n}',
              actions: [
                TextButton.icon(
                  onPressed: _importServiceAccountJson,
                  icon: Icon(Lucide.Upload, size: 16),
                  label: Text(l10n.providerDetailPageImportJsonButton),
                ),
              ],
              onChanged: (_) => _save(),
            ),
          ],
        ],
        const SizedBox(height: 12),
      ],
    );
  }

  Widget _buildModelsTab(BuildContext context, ColorScheme cs, AppLocalizations l10n) {
    final cfg = context.watch<SettingsProvider>().providerConfigs[widget.keyName];
    if (cfg == null) {
      // Provider has been removed; avoid recreating it via getProviderConfig.
      return Center(
        child: Text(l10n.providerDetailPageProviderRemovedMessage, style: TextStyle(color: cs.onSurface.withOpacity(0.7))),
      );
    }
    final models = cfg.models;
    return Stack(
      children: [
        if (models.isEmpty)
          Center(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(l10n.providerDetailPageNoModelsTitle, style: TextStyle(fontSize: 18, color: cs.onSurface)),
                const SizedBox(height: 6),
                Text(
                  l10n.providerDetailPageNoModelsSubtitle,
                  style: TextStyle(fontSize: 13, color: cs.primary),
                  textAlign: TextAlign.center,
                ),
              ],
            ),
          )
        else
          ReorderableListView.builder(
            padding: const EdgeInsets.fromLTRB(16, 16, 16, 100),
            itemCount: models.length,
            onReorder: (oldIndex, newIndex) async {
              if (newIndex > oldIndex) newIndex -= 1;
              final id = models[oldIndex];
              final list = List<String>.from(models);
              final item = list.removeAt(oldIndex);
              list.insert(newIndex, item);
              setState(() {});
              await context.read<SettingsProvider>().setProviderConfig(
                widget.keyName,
                cfg.copyWith(models: list),
              );
            },
            proxyDecorator: (child, index, animation) {
              return AnimatedBuilder(
                animation: animation,
                builder: (context, _) {
                  final t = Curves.easeOutBack.transform(animation.value);
                  return Transform.scale(
                    scale: 0.98 + 0.02 * t,
                    child: Material(
                      elevation: 8 * t,
                      color: Colors.transparent,
                      borderRadius: BorderRadius.circular(12),
                      child: child,
                    ),
                  );
                },
                child: child,
              );
            },
            itemBuilder: (c, i) {
              final id = models[i];
              final cs = Theme.of(context).colorScheme;
              return KeyedSubtree(
                key: ValueKey('reorder-model-$id'),
                child: ReorderableDelayedDragStartListener(
                  index: i,
                  child: Slidable(
                    key: ValueKey('model-$id'),
                    endActionPane: ActionPane(
                      motion: const StretchMotion(),
                      extentRatio: 0.42,
                      children: [
                        CustomSlidableAction(
                          autoClose: true,
                          backgroundColor: Colors.transparent,
                          child: Container(
                            width: double.infinity,
                            height: double.infinity,
                            decoration: BoxDecoration(
                              color: Theme.of(context).brightness == Brightness.dark ? cs.error.withOpacity(0.22) : cs.error.withOpacity(0.14),
                              borderRadius: BorderRadius.circular(12),
                              border: Border.all(color: cs.error.withOpacity(0.35)),
                            ),
                            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                            alignment: Alignment.center,
                            child: FittedBox(
                              fit: BoxFit.scaleDown,
                              child: Row(
                                mainAxisSize: MainAxisSize.min,
                                children: [
                                  Icon(Lucide.Trash2, color: cs.error, size: 18),
                                  const SizedBox(width: 6),
                                  Text(l10n.providerDetailPageDeleteModelButton, style: TextStyle(color: cs.error, fontWeight: FontWeight.w700)),
                                ],
                              ),
                            ),
                          ),
                          onPressed: (_) async {
                            final ok = await showDialog<bool>(
                              context: context,
                              builder: (dctx) => AlertDialog(
                                backgroundColor: cs.surface,
                                title: Text(l10n.providerDetailPageConfirmDeleteTitle),
                                content: Text(l10n.providerDetailPageConfirmDeleteContent),
                                actions: [
                                  TextButton(onPressed: () => Navigator.of(dctx).pop(false), child: Text(l10n.providerDetailPageCancelButton)),
                                  TextButton(onPressed: () => Navigator.of(dctx).pop(true), child: Text(l10n.providerDetailPageDeleteButton)),
                                ],
                              ),
                            );
                            if (ok != true) return;
                            final settings = context.read<SettingsProvider>();
                            final old = settings.getProviderConfig(widget.keyName, defaultName: widget.displayName);
                            final prevList = List<String>.from(old.models);
                            final prevOverrides = Map<String, dynamic>.from(old.modelOverrides);
                            final removeIndex = prevList.indexOf(id);
                            final newList = prevList.where((e) => e != id).toList();
                            final newOverrides = Map<String, dynamic>.from(prevOverrides)..remove(id);
                            await settings.setProviderConfig(widget.keyName, old.copyWith(models: newList, modelOverrides: newOverrides));
                            if (!mounted) return;
                            showAppSnackBar(
                              context,
                              message: l10n.providerDetailPageModelDeletedSnackbar,
                              type: NotificationType.info,
                              actionLabel: l10n.providerDetailPageUndoButton,
                              onAction: () {
                                Future(() async {
                                  final cfg2 = context.read<SettingsProvider>().getProviderConfig(widget.keyName, defaultName: widget.displayName);
                                  final restoredList = List<String>.from(cfg2.models);
                                  if (!restoredList.contains(id)) {
                                    if (removeIndex >= 0 && removeIndex <= restoredList.length) {
                                      restoredList.insert(removeIndex, id);
                                    } else {
                                      restoredList.add(id);
                                    }
                                  }
                                  final restoredOverrides = Map<String, dynamic>.from(cfg2.modelOverrides);
                                  if (!restoredOverrides.containsKey(id) && prevOverrides.containsKey(id)) {
                                    restoredOverrides[id] = prevOverrides[id];
                                  }
                                  await settings.setProviderConfig(widget.keyName, cfg2.copyWith(models: restoredList, modelOverrides: restoredOverrides));
                                });
                              },
                            );
                          },
                        ),
                      ],
                    ),
                    child: _ModelCard(providerKey: widget.keyName, modelId: id),
                  ),
                ),
              );
            },
          ),
        Positioned(
          left: 0,
          right: 0,
          bottom: 12 + MediaQuery.of(context).padding.bottom,
          child: Center(
            child: Container(
              decoration: BoxDecoration(
                // Solid color: dark theme uses an opaque lightened surface; light uses input-like gray
                color: Theme.of(context).brightness == Brightness.dark
                    ? Color.alphaBlend(Colors.white.withOpacity(0.12), cs.surface)
                    : const Color(0xFFF2F3F5),
                borderRadius: BorderRadius.circular(999),
              ),
              padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  _TactileRow(
                    pressedScale: 0.97,
                    haptics: false,
                    onTap: () => _showModelPicker(context),
                    builder: (pressed) {
                      return Container(
                        decoration: BoxDecoration(
                          borderRadius: BorderRadius.circular(999),
                          border: Border.all(color: cs.primary.withOpacity(0.35)),
                        ),
                        padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 10),
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Icon(Lucide.Boxes, size: 20, color: cs.primary),
                            const SizedBox(width: 8),
                            Text(l10n.providerDetailPageFetchModelsButton, style: TextStyle(color: cs.primary, fontSize: 14, fontWeight: FontWeight.w600)),
                          ],
                        ),
                      );
                    },
                  ),
                  const SizedBox(width: 10),
                  _TactileRow(
                    pressedScale: 0.97,
                    haptics: false,
                    onTap: () async {
                      await showCreateModelSheet(context, providerKey: widget.keyName);
                    },
                    builder: (pressed) {
                      return Container(
                        decoration: BoxDecoration(
                          color: cs.primary.withOpacity(0.12),
                          borderRadius: BorderRadius.circular(999),
                        ),
                        padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 10),
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Icon(Lucide.Plus, size: 20, color: cs.primary),
                            const SizedBox(width: 8),
                            Text(l10n.providerDetailPageAddNewModelButton, style: TextStyle(color: cs.primary, fontSize: 14)),
                          ],
                        ),
                      );
                    },
                  ),
                ],
              ),
            ),
          ),
        ),
      ],
    );
  }

  // Legacy network tab removed (replaced by ProviderNetworkPage)

  Widget _switchRow({required IconData icon, required String title, required bool value, required ValueChanged<bool> onChanged}) {
    final cs = Theme.of(context).colorScheme;
    return Row(
      children: [
        Container(
          width: 36,
          height: 36,
          decoration: BoxDecoration(color: cs.primary.withOpacity(0.08), borderRadius: BorderRadius.circular(10)),
          alignment: Alignment.center,
          margin: const EdgeInsets.only(right: 12),
          child: Icon(icon, size: 20, color: cs.primary),
        ),
        Expanded(child: Text(title, style: const TextStyle(fontSize: 15))),
        IosSwitch(value: value, onChanged: onChanged),
      ],
    );
  }

  Widget _inputRow(BuildContext context, {required String label, required TextEditingController controller, String? hint, bool obscure = false, bool enabled = true, Widget? suffix, ValueChanged<String>? onChanged}) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final cs = Theme.of(context).colorScheme;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label, style: TextStyle(fontSize: 13, color: cs.onSurface.withOpacity(0.8))),
        const SizedBox(height: 6),
        TextField(
          controller: controller,
          obscureText: obscure,
          enabled: enabled,
          onChanged: onChanged,
          decoration: InputDecoration(
            hintText: hint,
            filled: true,
            fillColor: isDark ? Colors.white10 : const Color(0xFFF2F3F5),
            border: OutlineInputBorder(borderRadius: BorderRadius.circular(12), borderSide: BorderSide(color: Colors.transparent)),
            enabledBorder: OutlineInputBorder(borderRadius: BorderRadius.circular(12), borderSide: BorderSide(color: Colors.transparent)),
            focusedBorder: OutlineInputBorder(borderRadius: BorderRadius.circular(12), borderSide: BorderSide(color: cs.primary.withOpacity(0.4))),
            suffixIcon: suffix,
          ),
        ),
      ],
    );
  }

  Widget _checkboxRow(BuildContext context, {required String title, required bool value, required ValueChanged<bool> onChanged}) {
    final cs = Theme.of(context).colorScheme;
    return InkWell(
      onTap: () => onChanged(!value),
      child: Row(
        children: [
          // iOS-style circular checkbox
          IosCheckbox(value: value, onChanged: onChanged),
          Text(title, style: TextStyle(fontSize: 14, color: cs.onSurface)),
        ],
      ),
    );
  }

  // --- iOS style helpers (consistent with MultiKeyManagerPage) ---

  Widget _iosSectionCard({required List<Widget> children}) {
    final theme = Theme.of(context);
    final cs = theme.colorScheme;
    final isDark = theme.brightness == Brightness.dark;
    final Color base = cs.surface;
    final Color bg = isDark ? Color.lerp(base, Colors.white, 0.06)! : Color.lerp(base, Colors.white, 0.92)!;
    return Container(
      decoration: BoxDecoration(
        color: bg,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: cs.outlineVariant.withOpacity(isDark ? 0.08 : 0.06), width: 0.6),
        boxShadow: [
          if (!isDark) BoxShadow(color: Colors.black.withOpacity(0.02), blurRadius: 6, offset: const Offset(0, 1)),
        ],
      ),
      clipBehavior: Clip.antiAlias,
      child: Column(children: children),
    );
  }

  Widget _iosRow(
    BuildContext context, {
    required String label,
    Widget? trailing,
    GestureTapCallback? onTap,
  }) {
    final cs = Theme.of(context).colorScheme;
    return _TactileRow(
      onTap: onTap,
      builder: (pressed) {
        final isDark = Theme.of(context).brightness == Brightness.dark;
        final base = cs.onSurface;
        final target = pressed ? (Color.lerp(base, isDark ? Colors.black : Colors.white, 0.55) ?? base) : base;
        return TweenAnimationBuilder<Color?>(
          tween: ColorTween(end: target),
          duration: const Duration(milliseconds: 220),
          curve: Curves.easeOutCubic,
          builder: (context, color, _) {
            final c = color ?? base;
            return Padding(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
              child: Row(
                children: [
                  Expanded(child: Text(label, style: TextStyle(fontSize: 15, color: c))),
                  if (trailing != null) trailing,
                ],
              ),
            );
          },
        );
      },
    );
  }

  

  Widget _providerKindRow(BuildContext context) {
    String labelFor(ProviderKind k) {
      switch (k) {
        case ProviderKind.google:
          return 'Gemini';
        case ProviderKind.claude:
          return 'Claude';
        case ProviderKind.openai:
        default:
          return 'OpenAI';
      }
    }
    return _TactileRow(
      onTap: _showProviderKindSheet,
      builder: (pressed) {
        final cs = Theme.of(context).colorScheme;
        final base = cs.onSurface;
        final isDark = Theme.of(context).brightness == Brightness.dark;
        final target = pressed ? (Color.lerp(base, isDark ? Colors.black : Colors.white, 0.55) ?? base) : base;
        return TweenAnimationBuilder<Color?>(
          tween: ColorTween(end: target),
          duration: const Duration(milliseconds: 220),
          curve: Curves.easeOutCubic,
          builder: (context, color, _) {
            final c = color ?? base;
            return Padding(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
              child: Row(
                children: [
                  Expanded(child: Text(AppLocalizations.of(context)!.settingsPageProviders, style: TextStyle(fontSize: 15, color: c))),
                  Text(labelFor(_kind), style: TextStyle(fontSize: 15, color: c)),
                  const SizedBox(width: 6),
                  Icon(Lucide.ChevronRight, size: 16, color: c),
                ],
              ),
            );
          },
        );
      },
    );
  }

  Future<void> _showProviderKindSheet() async {
    final cs = Theme.of(context).colorScheme;
    final selected = await showModalBottomSheet<ProviderKind>(
      context: context,
      backgroundColor: cs.surface,
      shape: const RoundedRectangleBorder(borderRadius: BorderRadius.vertical(top: Radius.circular(16))),
      builder: (ctx) {
        return SafeArea(
          top: false,
          child: Padding(
            padding: const EdgeInsets.fromLTRB(16, 12, 16, 16),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Container(
                  width: 40,
                  height: 4,
                  decoration: BoxDecoration(color: cs.onSurface.withOpacity(0.2), borderRadius: BorderRadius.circular(999)),
                ),
                const SizedBox(height: 12),
                _providerKindTile(ctx, ProviderKind.openai, label: 'OpenAI'),
                _providerKindTile(ctx, ProviderKind.google, label: 'Gemini'),
                _providerKindTile(ctx, ProviderKind.claude, label: 'Claude'),
              ],
            ),
          ),
        );
      },
    );
    if (selected != null) {
      setState(() => _kind = selected);
      await _save();
    }
  }

  Widget _providerKindTile(BuildContext ctx, ProviderKind k, {required String label}) {
    final cs = Theme.of(ctx).colorScheme;
    final selected = _kind == k;
    return _TactileRow(
      pressedScale: 1.00,
      haptics: false,
      onTap: () => Navigator.of(ctx).pop(k),
      builder: (pressed) {
        final base = cs.onSurface;
        final isDark = Theme.of(ctx).brightness == Brightness.dark;
        final target = pressed ? (Color.lerp(base, isDark ? Colors.black : Colors.white, 0.55) ?? base) : base;
        return TweenAnimationBuilder<Color?>(
          tween: ColorTween(end: target),
          duration: const Duration(milliseconds: 200),
          curve: Curves.easeOutCubic,
          builder: (context, color, _) {
            final c = color ?? base;
            return Padding(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
              child: Row(
                children: [
                  Expanded(child: Text(label, style: TextStyle(fontSize: 15, color: c))),
                  if (selected) Icon(Icons.check, color: cs.primary),
                ],
              ),
            );
          },
        );
      },
    );
  }


  Future<void> _save() async {
    final settings = context.read<SettingsProvider>();
    final old = settings.getProviderConfig(widget.keyName, defaultName: widget.displayName);
    String projectId = _projectCtrl.text.trim();
    if ((_kind == ProviderKind.google) && _vertexAI && projectId.isEmpty) {
      try {
        final obj = jsonDecode(_saJsonCtrl.text) as Map<String, dynamic>;
        projectId = (obj['project_id'] as String?)?.trim() ?? '';
      } catch (_) {}
    }
    final updated = old.copyWith(
      enabled: _enabled,
      name: _nameCtrl.text.trim().isEmpty ? widget.displayName : _nameCtrl.text.trim(),
      apiKey: _keyCtrl.text.trim(),
      baseUrl: _baseCtrl.text.trim(),
      providerType: _kind,  // Save the selected provider type
      chatPath: _kind == ProviderKind.openai ? _pathCtrl.text.trim() : old.chatPath,
      useResponseApi: _kind == ProviderKind.openai ? _useResp : old.useResponseApi,
      vertexAI: _kind == ProviderKind.google ? _vertexAI : old.vertexAI,
      location: _kind == ProviderKind.google ? _locationCtrl.text.trim() : old.location,
      projectId: _kind == ProviderKind.google ? projectId : old.projectId,
      serviceAccountJson: _kind == ProviderKind.google ? _saJsonCtrl.text.trim() : old.serviceAccountJson,
      multiKeyEnabled: _multiKeyEnabled,
      // preserve models and modelOverrides and proxy fields implicitly via copyWith
    );
    await settings.setProviderConfig(widget.keyName, updated);
    if (!mounted) return;
    // Silent auto-save (no snackbar) for immediate-save UX
  }

  Widget _multilineRow(
    BuildContext context, {
    required String label,
    required TextEditingController controller,
    String? hint,
    List<Widget>? actions,
    ValueChanged<String>? onChanged,
  }) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final cs = Theme.of(context).colorScheme;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Expanded(child: Text(label, style: TextStyle(fontSize: 13, color: cs.onSurface.withOpacity(0.8)))),
            if (actions != null) ...actions,
          ],
        ),
        const SizedBox(height: 6),
        TextField(
          controller: controller,
          maxLines: 8,
          minLines: 4,
          onChanged: onChanged,
          decoration: InputDecoration(
            hintText: hint,
            filled: true,
            alignLabelWithHint: true,
            fillColor: isDark ? Colors.white10 : const Color(0xFFF2F3F5),
            border: OutlineInputBorder(borderRadius: BorderRadius.circular(12), borderSide: BorderSide(color: Colors.transparent)),
            enabledBorder: OutlineInputBorder(borderRadius: BorderRadius.circular(12), borderSide: BorderSide(color: Colors.transparent)),
            focusedBorder: OutlineInputBorder(borderRadius: BorderRadius.circular(12), borderSide: BorderSide(color: cs.primary.withOpacity(0.4))),
          ),
        ),
      ],
    );
  }

  Future<void> _importServiceAccountJson() async {
    try {
      // Lazy import to avoid hard dependency errors in web
      // ignore: avoid_dynamic_calls
      // ignore: import_of_legacy_library_into_null_safe
      // Using file_picker which is already in pubspec
      // import placed at top-level of this file
      final picker = await _pickJsonFile();
      if (picker == null) return;
      _saJsonCtrl.text = picker;
      // Auto-fill projectId if available
      try {
        final obj = jsonDecode(_saJsonCtrl.text) as Map<String, dynamic>;
        final pid = (obj['project_id'] as String?)?.trim();
        if ((pid ?? '').isNotEmpty && _projectCtrl.text.trim().isEmpty) {
          _projectCtrl.text = pid!;
        }
      } catch (_) {}
      if (mounted) {
        setState(() {});
        await _save();
      }
    } catch (e) {
      if (!mounted) return;
      showAppSnackBar(
        context,
        message: '$e',
        type: NotificationType.error,
      );
    }
  }

  Future<String?> _pickJsonFile() async {
    try {
      final result = await FilePicker.platform.pickFiles(
        type: FileType.custom,
        allowedExtensions: const ['json'],
        allowMultiple: false,
      );
      if (result == null || result.files.isEmpty) return null;
      final file = result.files.single;
      final path = file.path;
      if (path == null) return null;
      final text = await File(path).readAsString();
      return text;
    } catch (e) {
      return null;
    }
  }

  Future<void> _openTestDialog() async {
    await showDialog(
      context: context,
      barrierDismissible: false,
      builder: (ctx) => _ConnectionTestDialog(providerKey: widget.keyName, providerDisplayName: widget.displayName),
    );
  }

  // _saveNetwork moved to ProviderNetworkPage

  Widget _buildProviderTypeSelector(BuildContext context, ColorScheme cs, AppLocalizations l10n) {
    return Row(
      children: [
        Expanded(
          child: GestureDetector(
            onTap: () {
              setState(() {
                _kind = ProviderKind.openai;
              });
            },
            child: Container(
              padding: const EdgeInsets.symmetric(vertical: 12, horizontal: 8),
              decoration: BoxDecoration(
                color: _kind == ProviderKind.openai 
                    ? cs.primary.withOpacity(0.15) 
                    : Theme.of(context).brightness == Brightness.dark 
                        ? Colors.white10 
                        : const Color(0xFFF7F7F9),
                borderRadius: BorderRadius.circular(12),
                border: Border.all(
                  color: _kind == ProviderKind.openai 
                      ? cs.primary.withOpacity(0.5) 
                      : cs.outlineVariant.withOpacity(0.2),
                  width: _kind == ProviderKind.openai ? 2 : 1,
                ),
              ),
              child: Column(
                children: [
                  Text(
                    'OpenAI',
                    style: TextStyle(
                      fontSize: 13,
                      fontWeight: _kind == ProviderKind.openai ? FontWeight.w600 : FontWeight.w500,
                      color: _kind == ProviderKind.openai ? cs.primary : cs.onSurface,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
        const SizedBox(width: 8),
        Expanded(
          child: GestureDetector(
            onTap: () {
              setState(() {
                _kind = ProviderKind.google;
              });
            },
            child: Container(
              padding: const EdgeInsets.symmetric(vertical: 12, horizontal: 8),
              decoration: BoxDecoration(
                color: _kind == ProviderKind.google 
                    ? cs.primary.withOpacity(0.15) 
                    : Theme.of(context).brightness == Brightness.dark 
                        ? Colors.white10 
                        : const Color(0xFFF7F7F9),
                borderRadius: BorderRadius.circular(12),
                border: Border.all(
                  color: _kind == ProviderKind.google 
                      ? cs.primary.withOpacity(0.5) 
                      : cs.outlineVariant.withOpacity(0.2),
                  width: _kind == ProviderKind.google ? 2 : 1,
                ),
              ),
              child: Column(
                children: [
                  Text(
                    'Gemini',
                    style: TextStyle(
                      fontSize: 13,
                      fontWeight: _kind == ProviderKind.google ? FontWeight.w600 : FontWeight.w500,
                      color: _kind == ProviderKind.google ? cs.primary : cs.onSurface,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
        const SizedBox(width: 8),
        Expanded(
          child: GestureDetector(
            onTap: () {
              setState(() {
                _kind = ProviderKind.claude;
              });
            },
            child: Container(
              padding: const EdgeInsets.symmetric(vertical: 12, horizontal: 8),
              decoration: BoxDecoration(
                color: _kind == ProviderKind.claude 
                    ? cs.primary.withOpacity(0.15) 
                    : Theme.of(context).brightness == Brightness.dark 
                        ? Colors.white10 
                        : const Color(0xFFF7F7F9),
                borderRadius: BorderRadius.circular(12),
                border: Border.all(
                  color: _kind == ProviderKind.claude 
                      ? cs.primary.withOpacity(0.5) 
                      : cs.outlineVariant.withOpacity(0.2),
                  width: _kind == ProviderKind.claude ? 2 : 1,
                ),
              ),
              child: Column(
                children: [
                  Text(
                    'Claude',
                    style: TextStyle(
                      fontSize: 13,
                      fontWeight: _kind == ProviderKind.claude ? FontWeight.w600 : FontWeight.w500,
                      color: _kind == ProviderKind.claude ? cs.primary : cs.onSurface,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ],
    );
  }

  Future<void> _showModelPicker(BuildContext context) async {
    final cs = Theme.of(context).colorScheme;
    final settings = context.read<SettingsProvider>();
    final cfg = settings.getProviderConfig(widget.keyName, defaultName: widget.displayName);
    final controller = TextEditingController();
    List<dynamic> items = const [];
    bool loading = true;
    String error = '';
    // Collapsed state per group in the selector dialog
    final Map<String, bool> collapsed = <String, bool>{};

    await showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: cs.surface,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(16)),
      ),
      builder: (ctx) {
        return StatefulBuilder(builder: (ctx, setLocal) {
          final l10n = AppLocalizations.of(ctx)!;
          Future<void> _load() async {
            try {
              final list = await ProviderManager.listModels(cfg);
              setLocal(() {
                items = list;
                loading = false;
              });
            } catch (e) {
              setLocal(() {
                items = const [];
                loading = false;
                error = '$e';
              });
            }
          }

          if (loading) {
            // kick off loading once
            Future.microtask(_load);
          }

          final selected = settings.getProviderConfig(widget.keyName, defaultName: widget.displayName).models.toSet();
          final query = controller.text.trim().toLowerCase();
          final filtered = <ModelInfo>[
            for (final m in items)
              if (m is ModelInfo && (query.isEmpty || m.id.toLowerCase().contains(query) || m.displayName.toLowerCase().contains(query))) m
          ];

          String _groupFor(ModelInfo m) {
            final id = m.id.toLowerCase();
            // Embeddings first
            if (m.type == ModelType.embedding || id.contains('embedding') || id.contains('embed')) {
              return l10n.providerDetailPageEmbeddingsGroupTitle;
            }
            // OpenAI families
            // if (RegExp(r'gpt-4o|gpt-4\.1|gpt-4|gpt4').hasMatch(id)) return 'GPT-4';
            if (id.contains('gpt') || RegExp(r'(^|[^a-z])o[134]').hasMatch(id)) return 'GPT';
            // if (RegExp(r'(^|[^a-z])o[134]').hasMatch(id)) return zhLocal ? 'o 系列' : 'o Series';
            // if (id.contains('gpt-5')) return 'GPT-5';
            // Google Gemini
            if (id.contains('gemini-2.0')) return 'Gemini 2.0';
            if (id.contains('gemini-2.5')) return 'Gemini 2.5';
            if (id.contains('gemini-1.5')) return 'Gemini 1.5';
            if (id.contains('gemini')) return 'Gemini';
            // Anthropic Claude
            if (id.contains('claude-3.5')) return 'Claude 3.5';
            if (id.contains('claude-3')) return 'Claude 3';
            if (id.contains('claude-4')) return 'Claude 4';
            if (id.contains('claude-sonnet')) return 'Claude Sonnet';
            if (id.contains('claude-opus')) return 'Claude Opus';
            // Others by vendor keyword
            if (id.contains('deepseek')) return 'DeepSeek';
            if (RegExp(r'qwen|qwq|qvq|dashscope').hasMatch(id)) return 'Qwen';
            if (RegExp(r'doubao|ark|volc').hasMatch(id)) return 'Doubao';
            if (id.contains('glm') || id.contains('zhipu')) return 'GLM';
            if (id.contains('mistral')) return 'Mistral';
            if (id.contains('grok') || id.contains('xai')) return 'Grok';
            return l10n.providerDetailPageOtherModelsGroupTitle;
          }

          final Map<String, List<ModelInfo>> grouped = {};
          for (final m in filtered) {
            final g = _groupFor(m);
            (grouped[g] ??= []).add(m);
          }
          final groupKeys = grouped.keys.toList()..sort((a, b) => a.toLowerCase().compareTo(b.toLowerCase()));

          return SafeArea(
            top: false,
            child: AnimatedPadding(
              duration: const Duration(milliseconds: 180),
              curve: Curves.easeOutCubic,
              padding: EdgeInsets.only(bottom: MediaQuery.of(ctx).viewInsets.bottom),
              child: DraggableScrollableSheet(
                expand: false,
                initialChildSize: 0.7,
                maxChildSize: 0.8,
                minChildSize: 0.4,
                builder: (c, scrollController) {
                  final bottomPadding = MediaQuery.of(ctx).padding.bottom + 16;
                  return Column(
                    children: [
                      const SizedBox(height: 8),
                      Container(width: 40, height: 4, decoration: BoxDecoration(color: cs.onSurface.withOpacity(0.2), borderRadius: BorderRadius.circular(999))),
                      const SizedBox(height: 12),
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        child: TextField(
                          controller: controller,
                          onChanged: (_) => setLocal(() {}),
                          decoration: InputDecoration(
                            hintText: l10n.providerDetailPageFilterHint,
                            filled: true,
                            fillColor: Theme.of(ctx).brightness == Brightness.dark ? Colors.white10 : const Color(0xFFF2F3F5),
                            border: OutlineInputBorder(borderRadius: BorderRadius.circular(12), borderSide: BorderSide(color: Colors.transparent)),
                            enabledBorder: OutlineInputBorder(borderRadius: BorderRadius.circular(12), borderSide: BorderSide(color: Colors.transparent)),
                            focusedBorder: OutlineInputBorder(borderRadius: BorderRadius.circular(12), borderSide: BorderSide(color: cs.primary.withOpacity(0.4))),
                          ),
                        ),
                      ),
                      const SizedBox(height: 8),
                      Expanded(
                        child: loading
                            ? const Center(child: CircularProgressIndicator())
                          : error.isNotEmpty
                              ? Center(child: Text(error, style: TextStyle(color: cs.error)))
                              : ListView(
                                  controller: scrollController,
                                  padding: EdgeInsets.only(bottom: bottomPadding),
                                  children: [
                                    for (final g in groupKeys) ...[
                                      // Group header with actions
                                      Padding(
                                        padding: const EdgeInsets.fromLTRB(12, 6, 12, 6),
                                        child: _TactileRow(
                                          pressedScale: 0.98,
                                          haptics: false,
                                          onTap: () => setLocal(() {
                                            collapsed[g] = !(collapsed[g] == true);
                                          }),
                                          builder: (_) {
                                            return Container(
                                              decoration: BoxDecoration(
                                                color: Theme.of(context).brightness == Brightness.dark
                                                    ? Colors.white10
                                                    : const Color(0xFFF2F3F5),
                                                borderRadius: BorderRadius.circular(12),
                                              ),
                                              child: Padding(
                                                padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 6),
                                                child: Row(
                                                  children: [
                                                  SizedBox(
                                                    width: 28,
                                                    child: Center(
                                                      child: AnimatedRotation(
                                                        turns: (collapsed[g] == true) ? 0.0 : 0.25,
                                                        duration: const Duration(milliseconds: 220),
                                                        curve: Curves.easeOutCubic,
                                                        child: Icon(
                                                          Lucide.ChevronRight,
                                                          size: 20,
                                                          color: cs.onSurface.withOpacity(0.7),
                                                        ),
                                                      ),
                                                    ),
                                                  ),
                                                  const SizedBox(width: 16),
                                                  Expanded(
                                                    child: Text(
                                                      g,
                                                      style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w600),
                                                      maxLines: 1,
                                                      overflow: TextOverflow.ellipsis,
                                                    ),
                                                  ),
                                                  const SizedBox(width: 8),
                                                  Builder(builder: (ctx2) {
                                                    final allAdded = grouped[g]!.every((m) => selected.contains(m.id));
                                                    return IconButton(
                                                      padding: EdgeInsets.zero,
                                                      constraints: const BoxConstraints(minWidth: 48, minHeight: 40),
                                                      tooltip: allAdded ? l10n.providerDetailPageRemoveGroupTooltip : l10n.providerDetailPageAddGroupTooltip,
                                                      icon: Icon(allAdded ? Lucide.Minus : Lucide.Plus, size: 24, color: allAdded ? cs.onSurface.withValues(alpha: 0.7) : cs.onSurface.withValues(alpha: 0.7)),
                                                      onPressed: () async {
                                                        final old = settings.getProviderConfig(widget.keyName, defaultName: widget.displayName);
                                                        if (allAdded) {
                                                          final toRemove = grouped[g]!.map((m) => m.id).toSet();
                                                          final list = old.models.where((id) => !toRemove.contains(id)).toList();
                                                          await settings.setProviderConfig(widget.keyName, old.copyWith(models: list));
                                                        } else {
                                                          final toAdd = grouped[g]!.where((m) => !selected.contains(m.id)).map((m) => m.id).toList();
                                                          if (toAdd.isEmpty) return;
                                                          final set = old.models.toSet()..addAll(toAdd);
                                                          await settings.setProviderConfig(widget.keyName, old.copyWith(models: set.toList()));
                                                        }
                                                        setLocal(() {});
                                                      },
                                                    );
                                                  }),
                                                ],
                                              ),
                                              ));

                                          },
                                        ),
                                      ),
                                      AnimatedSize(
                                        duration: const Duration(milliseconds: 220),
                                        curve: Curves.easeOutCubic,
                                        child: (collapsed[g] == true)
                                            ? const SizedBox.shrink()
                                            : Column(
                                                children: [
                                                  for (final m in grouped[g]!)
                                                    Builder(builder: (c2) {
                                            final added = selected.contains(m.id);
                                            return Padding(
                                              padding: const EdgeInsets.fromLTRB(12, 6, 12, 6),
                                              child: _TactileRow(
                                                pressedScale: 0.98,
                                                haptics: false,
                                                onTap: () {},
                                                builder: (_) {
                                                  return Container(
                                                    decoration: BoxDecoration(
                                                      borderRadius: BorderRadius.circular(12),
                                                    ),
                                                    child: Padding(
                                                      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
                                                      child: Row(
                                                        children: [
                                                          SizedBox(
                                                            width: 28,
                                                            child: Center(child: _BrandAvatar(name: m.id, size: 28)),
                                                          ),
                                                          const SizedBox(width: 16),
                                                          Expanded(
                                                            child: Column(
                                                              crossAxisAlignment: CrossAxisAlignment.start,
                                                              children: [
                                                                Text(m.displayName, style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w600), maxLines: 1, overflow: TextOverflow.ellipsis),
                                                                const SizedBox(height: 4),
                                                                _modelTagWrap(context, m),
                                                              ],
                                                            ),
                                                          ),
                                                          const SizedBox(width: 8),
                                                          IconButton(
                                                            padding: EdgeInsets.zero,
                                                            constraints: const BoxConstraints(minWidth: 48, minHeight: 40),
                                                            onPressed: () async {
                                                              final old = settings.getProviderConfig(widget.keyName, defaultName: widget.displayName);
                                                              final list = old.models.toList();
                                                              if (added) {
                                                                list.removeWhere((e) => e == m.id);
                                                              } else {
                                                                list.add(m.id);
                                                              }
                                                              await settings.setProviderConfig(widget.keyName, old.copyWith(models: list));
                                                              setLocal(() {});
                                                            },
                                                            icon: Icon(added ? Lucide.Minus : Lucide.Plus, size: 24, color: added ? cs.onSurface.withValues(alpha: 0.7) : cs.onSurface.withValues(alpha: 0.7)),
                                                          ),
                                                        ],
                                                      ),
                                                    ),
                                                  );
                                                },
                                              ),
                                            );
                                          }),
                                                ],
                                              ),
                                      ),
                                    ],
                                  ],
                                ),
                    ),
                    ],
                  );
                },
              ),
            ),
          );
        });
      },
    );
  }

  Widget _capPill(BuildContext context, IconData icon, String label) {
    final cs = Theme.of(context).colorScheme;
    return Container(
      decoration: BoxDecoration(color: cs.primary.withOpacity(0.10), borderRadius: BorderRadius.circular(999)),
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      child: Row(mainAxisSize: MainAxisSize.min, children: [
        Icon(icon, size: 12, color: cs.primary),
        const SizedBox(width: 4),
        Text(label, style: TextStyle(fontSize: 11, color: cs.primary)),
      ]),
    );
  }
}

Widget _buildDismissBg(BuildContext context, {required bool alignStart}) {
  final cs = Theme.of(context).colorScheme;
  final l10n = AppLocalizations.of(context)!;
  return Container(
    decoration: BoxDecoration(
      color: cs.error.withOpacity(0.12),
      borderRadius: BorderRadius.circular(12),
    ),
    padding: const EdgeInsets.symmetric(horizontal: 16),
    alignment: alignStart ? Alignment.centerLeft : Alignment.centerRight,
    child: Row(
      mainAxisAlignment: alignStart ? MainAxisAlignment.start : MainAxisAlignment.end,
      children: [
        Icon(Lucide.Trash2, color: cs.error, size: 20),
        const SizedBox(width: 6),
        Text(
          l10n.providerDetailPageDeleteText,
          style: TextStyle(color: cs.error, fontWeight: FontWeight.w600),
        ),
      ],
    ),
  );
}

class _ModelCard extends StatelessWidget {
  const _ModelCard({required this.providerKey, required this.modelId});
  final String providerKey;
  final String modelId;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final cs = Theme.of(context).colorScheme;
    return _TactileRow(
      pressedScale: 0.98,
      haptics: false,
      onTap: () {},
      builder: (pressed) {
        return Container(
          decoration: BoxDecoration(
            color: cs.surface,
            borderRadius: BorderRadius.circular(12),
          ),
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            child: Row(
              children: [
                _BrandAvatar(name: modelId, size: 28),
                const SizedBox(width: 10),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(_displayName(context), style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w600)),
                      const SizedBox(height: 4),
                      _modelTagWrap(context, _effective(context)),
                    ],
                  ),
                ),
                _TactileIconButton(
                  icon: Lucide.Settings2,
                  color: cs.onSurface.withOpacity(0.7),
                  size: 18,
                  semanticLabel: l10n.providerDetailPageEditTooltip,
                  haptics: false,
                  onTap: () async {
                    await showModelDetailSheet(context, providerKey: providerKey, modelId: modelId);
                  },
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  ModelInfo _infer(String id) {
    // build a minimal ModelInfo and let registry infer
    return ModelRegistry.infer(ModelInfo(id: id, displayName: id));
  }

  ModelInfo _effective(BuildContext context) {
    final base = _infer(modelId);
    final cfg = context.watch<SettingsProvider>().getProviderConfig(providerKey);
    final ov = cfg.modelOverrides[modelId] as Map?;
    if (ov == null) return base;
    ModelType? type;
    final t = (ov['type'] as String?) ?? '';
    if (t == 'embedding') type = ModelType.embedding; else if (t == 'chat') type = ModelType.chat;
    List<Modality>? input;
    if (ov['input'] is List) {
      input = [
        for (final e in (ov['input'] as List)) (e.toString() == 'image' ? Modality.image : Modality.text)
      ];
    }
    List<Modality>? output;
    if (ov['output'] is List) {
      output = [
        for (final e in (ov['output'] as List)) (e.toString() == 'image' ? Modality.image : Modality.text)
      ];
    }
    List<ModelAbility>? abilities;
    if (ov['abilities'] is List) {
      abilities = [
        for (final e in (ov['abilities'] as List)) (e.toString() == 'reasoning' ? ModelAbility.reasoning : ModelAbility.tool)
      ];
    }
    return base.copyWith(
      displayName: (ov['name'] as String?)?.isNotEmpty == true ? ov['name'] as String : base.displayName,
      type: type ?? base.type,
      input: input ?? base.input,
      output: output ?? base.output,
      abilities: abilities ?? base.abilities,
    );
  }

  String _displayName(BuildContext context) {
    final cfg = context.watch<SettingsProvider>().getProviderConfig(providerKey);
    final ov = cfg.modelOverrides[modelId] as Map?;
    if (ov != null) {
      final n = (ov['name'] as String?)?.trim();
      if (n != null && n.isNotEmpty) return n;
    }
    return modelId;
  }

  Widget _pill(BuildContext context, IconData icon, String label) {
    final cs = Theme.of(context).colorScheme;
    return Container(
      decoration: BoxDecoration(color: cs.primary.withOpacity(0.10), borderRadius: BorderRadius.circular(999)),
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      child: Row(mainAxisSize: MainAxisSize.min, children: [
        Icon(icon, size: 12, color: cs.primary),
        const SizedBox(width: 4),
        Text(label, style: TextStyle(fontSize: 11, color: cs.primary)),
      ]),
    );
  }
}

class _ConnectionTestDialog extends StatefulWidget {
  const _ConnectionTestDialog({required this.providerKey, required this.providerDisplayName});
  final String providerKey;
  final String providerDisplayName;

  @override
  State<_ConnectionTestDialog> createState() => _ConnectionTestDialogState();
}

enum _TestState { idle, loading, success, error }

class _ConnectionTestDialogState extends State<_ConnectionTestDialog> {
  String? _selectedModelId;
  _TestState _state = _TestState.idle;
  String _errorMessage = '';

  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;
    final l10n = AppLocalizations.of(context)!;
    final title = l10n.providerDetailPageTestConnectionTitle;
    final canTest = _selectedModelId != null && _state != _TestState.loading;
    return Dialog(
      backgroundColor: cs.surface,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
      insetPadding: const EdgeInsets.symmetric(horizontal: 24, vertical: 24),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 520),
        child: Padding(
          padding: const EdgeInsets.fromLTRB(18, 18, 18, 12),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              Center(child: Text(title, style: const TextStyle(fontSize: 18, fontWeight: FontWeight.w700))),
              const SizedBox(height: 16),
              _buildBody(context, cs, l10n),
              const SizedBox(height: 16),
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  TextButton(onPressed: () => Navigator.of(context).pop(), child: Text(l10n.providerDetailPageCancelButton)),
                  const SizedBox(width: 8),
                  TextButton(
                    onPressed: canTest ? _doTest : null,
                    style: TextButton.styleFrom(foregroundColor: canTest ? cs.primary : cs.onSurface.withOpacity(0.4)),
                    child: Text(l10n.providerDetailPageTestButton),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }

  

  Widget _buildBody(BuildContext context, ColorScheme cs, AppLocalizations l10n) {
    switch (_state) {
      case _TestState.idle:
        return _buildIdle(context, cs, l10n);
      case _TestState.loading:
        return _buildLoading(context, cs, l10n);
      case _TestState.success:
        return _buildResult(context, cs, l10n, success: true, message: l10n.providerDetailPageTestSuccessMessage);
      case _TestState.error:
        return _buildResult(context, cs, l10n, success: false, message: _errorMessage);
    }
  }

  Widget _buildIdle(BuildContext context, ColorScheme cs, AppLocalizations l10n) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        if (_selectedModelId == null)
          TextButton(
            onPressed: _pickModel,
            child: Text(l10n.providerDetailPageSelectModelButton),
          )
        else
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              _BrandAvatar(name: _selectedModelId!, size: 24),
              const SizedBox(width: 8),
              Flexible(
                child: Text(
                  _selectedModelId!,
                  style: const TextStyle(fontWeight: FontWeight.w600),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              const SizedBox(width: 10),
              TextButton(onPressed: _pickModel, child: Text(l10n.providerDetailPageChangeButton)),
            ],
          ),
      ],
    );
  }

  Widget _buildLoading(BuildContext context, ColorScheme cs, AppLocalizations l10n) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        if (_selectedModelId != null)
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              _BrandAvatar(name: _selectedModelId!, size: 24),
              const SizedBox(width: 8),
              Flexible(
                child: Text(
                  _selectedModelId!,
                  style: const TextStyle(fontWeight: FontWeight.w600),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ],
          ),
        const SizedBox(height: 16),
        const LinearProgressIndicator(minHeight: 4),
        const SizedBox(height: 12),
        Text(l10n.providerDetailPageTestingMessage, style: TextStyle(color: cs.onSurface.withOpacity(0.7))),
      ],
    );
  }

  Widget _buildResult(BuildContext context, ColorScheme cs, AppLocalizations l10n, {required bool success, required String message}) {
    final color = success ? Colors.green : cs.error;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        if (_selectedModelId != null)
          _TactileRow(
            pressedScale: 0.98,
            haptics: false,
            onTap: _pickModel,
            builder: (_) {
              return Padding(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  mainAxisSize: MainAxisSize.max,
                  children: [
                    _BrandAvatar(name: _selectedModelId!, size: 24),
                    const SizedBox(width: 8),
                    Expanded(
                      child: Text(
                        _selectedModelId!,
                        style: const TextStyle(fontWeight: FontWeight.w600),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        textAlign: TextAlign.center,
                      ),
                    ),
                    const SizedBox(width: 8),
                    Icon(Lucide.ChevronDown, size: 16, color: cs.onSurface.withOpacity(0.7)),
                  ],
                ),
              );
            },
          ),
        const SizedBox(height: 14),
        Text(message, style: TextStyle(color: color, fontSize: 14, fontWeight: FontWeight.w600)),
      ],
    );
  }

  Future<void> _pickModel() async {
    final selected = await showModelPickerForTest(context, widget.providerKey, widget.providerDisplayName);
    if (selected != null) {
      setState(() {
        _selectedModelId = selected;
        _state = _TestState.idle;
        _errorMessage = '';
      });
    }
  }

  Future<void> _doTest() async {
    if (_selectedModelId == null) return;
    setState(() {
      _state = _TestState.loading;
      _errorMessage = '';
    });
    try {
      final cfg = context.read<SettingsProvider>().getProviderConfig(widget.providerKey, defaultName: widget.providerDisplayName);
      await ProviderManager.testConnection(cfg, _selectedModelId!);
      if (!mounted) return;
      setState(() => _state = _TestState.success);
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _state = _TestState.error;
        _errorMessage = e.toString();
      });
    }
  }
}

Future<String?> showModelPickerForTest(BuildContext context, String providerKey, String providerDisplayName) async {
  final cs = Theme.of(context).colorScheme;
  final settings = context.read<SettingsProvider>();
  final cfg = settings.getProviderConfig(providerKey, defaultName: providerDisplayName);
  final sel = await showModelSelector(context, limitProviderKey: providerKey);
  return sel?.modelId;
}

ModelInfo _effectiveFor(BuildContext context, String providerKey, String providerDisplayName, ModelInfo base) {
  final cfg = context.read<SettingsProvider>().getProviderConfig(providerKey, defaultName: providerDisplayName);
  final ov = cfg.modelOverrides[base.id] as Map?;
  if (ov == null) return base;
  ModelType? type;
  final t = (ov['type'] as String?) ?? '';
  if (t == 'embedding') type = ModelType.embedding; else if (t == 'chat') type = ModelType.chat;
  List<Modality>? input;
  if (ov['input'] is List) {
    input = [
      for (final e in (ov['input'] as List)) (e.toString() == 'image' ? Modality.image : Modality.text)
    ];
  }
  List<Modality>? output;
  if (ov['output'] is List) {
    output = [
      for (final e in (ov['output'] as List)) (e.toString() == 'image' ? Modality.image : Modality.text)
    ];
  }
  List<ModelAbility>? abilities;
  if (ov['abilities'] is List) {
    abilities = [
      for (final e in (ov['abilities'] as List)) (e.toString() == 'reasoning' ? ModelAbility.reasoning : ModelAbility.tool)
    ];
  }
  return base.copyWith(
    type: type ?? base.type,
    input: input ?? base.input,
    output: output ?? base.output,
    abilities: abilities ?? base.abilities,
  );
}


// Using flutter_slidable for reliable swipe actions with confirm + undo.

Widget _modelTagWrap(BuildContext context, ModelInfo m) {
  final cs = Theme.of(context).colorScheme;
  final l10n = AppLocalizations.of(context)!;
  final isDark = Theme.of(context).brightness == Brightness.dark;
  List<Widget> chips = [];
  // type tag
  chips.add(Container(
    decoration: BoxDecoration(
      color: isDark ? cs.primary.withOpacity(0.25) : cs.primary.withOpacity(0.15),
      borderRadius: BorderRadius.circular(999),
      border: Border.all(color: cs.primary.withOpacity(0.2), width: 0.5),
    ),
    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
    child: Text(m.type == ModelType.chat ? l10n.modelSelectSheetChatType : l10n.modelSelectSheetEmbeddingType, style: TextStyle(fontSize: 11, color: isDark ? cs.primary : cs.primary.withOpacity(0.9), fontWeight: FontWeight.w500)),
  ));
  // modality tag capsule
  chips.add(Container(
    decoration: BoxDecoration(
      color: isDark ? cs.tertiary.withOpacity(0.25) : cs.tertiary.withOpacity(0.15),
      borderRadius: BorderRadius.circular(999),
      border: Border.all(color: cs.tertiary.withOpacity(0.2), width: 0.5),
    ),
    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
    child: Row(mainAxisSize: MainAxisSize.min, children: [
      for (final mod in m.input)
        Padding(
          padding: const EdgeInsets.only(right: 2),
          child: Icon(mod == Modality.text ? Lucide.Type : Lucide.Image, size: 12, color: isDark ? cs.tertiary : cs.tertiary.withOpacity(0.9)),
        ),
      Icon(Lucide.ChevronRight, size: 12, color: isDark ? cs.tertiary : cs.tertiary.withOpacity(0.9)),
      for (final mod in m.output)
        Padding(
          padding: const EdgeInsets.only(left: 2),
          child: Icon(mod == Modality.text ? Lucide.Type : Lucide.Image, size: 12, color: isDark ? cs.tertiary : cs.tertiary.withOpacity(0.9)),
        ),
    ]),
  ));
  // abilities capsules (icon-only)
  for (final ab in m.abilities) {
    if (ab == ModelAbility.tool) {
      chips.add(Container(
        decoration: BoxDecoration(
          color: isDark ? cs.primary.withOpacity(0.25) : cs.primary.withOpacity(0.15),
          borderRadius: BorderRadius.circular(999),
          border: Border.all(color: cs.primary.withOpacity(0.2), width: 0.5),
        ),
        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3),
        child: Icon(Lucide.Hammer, size: 12, color: isDark ? cs.primary : cs.primary.withOpacity(0.9)),
      ));
    } else if (ab == ModelAbility.reasoning) {
      chips.add(Container(
        decoration: BoxDecoration(
          color: isDark ? cs.secondary.withOpacity(0.3) : cs.secondary.withOpacity(0.18),
          borderRadius: BorderRadius.circular(999),
          border: Border.all(color: cs.secondary.withOpacity(0.25), width: 0.5),
        ),
        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3),
        child: SvgPicture.asset('assets/icons/deepthink.svg', width: 12, height: 12, colorFilter: ColorFilter.mode(isDark ? cs.secondary : cs.secondary.withOpacity(0.9), BlendMode.srcIn)),
      ));
    }
  }
  return Wrap(spacing: 6, runSpacing: 6, crossAxisAlignment: WrapCrossAlignment.center, children: chips);
}


// Legacy page-based implementations removed in favor of swipeable PageView tabs.


class _BrandAvatar extends StatelessWidget {
  const _BrandAvatar({required this.name, this.size = 20});
  final String name;
  final double size;


  bool _preferMonochromeWhite(String n) {
    final k = n.toLowerCase();
    if (RegExp(r'openai|gpt|o\d').hasMatch(k)) return true;
    if (RegExp(r'grok|xai').hasMatch(k)) return true;
    return false;
  }

  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final asset = BrandAssets.assetForName(name);
    final lower = name.toLowerCase();
    final bool _mono = isDark && (RegExp(r'openai|gpt|o\\d').hasMatch(lower) || RegExp(r'grok|xai').hasMatch(lower) || RegExp(r'openrouter').hasMatch(lower));
    return CircleAvatar(
      radius: size / 2,
      backgroundColor: isDark ? Colors.white10 : cs.primary.withOpacity(0.1),
      child: asset == null
          ? Text(name.isNotEmpty ? name.characters.first.toUpperCase() : '?',
              style: TextStyle(color: cs.primary, fontSize: size * 0.5, fontWeight: FontWeight.w700))
          : (asset.endsWith('.svg')
              ? SvgPicture.asset(
                  asset,
                  width: size * 0.7,
                  height: size * 0.7,
                  colorFilter: _mono ? const ColorFilter.mode(Colors.white, BlendMode.srcIn) : null,
                )
              : Image.asset(
                  asset,
                  width: size * 0.7,
                  height: size * 0.7,
                  fit: BoxFit.contain,
                  color: _mono ? Colors.white : null,
                  colorBlendMode: _mono ? BlendMode.srcIn : null,
                )),
    );
  }
}

// Top-level tactile row used by iOS-style lists here
class _TactileRow extends StatefulWidget {
  const _TactileRow({required this.builder, this.onTap, this.pressedScale = 1.00, this.haptics = true});
  final Widget Function(bool pressed) builder;
  final VoidCallback? onTap;
  final double pressedScale;
  final bool haptics;
  @override
  State<_TactileRow> createState() => _TactileRowState();
}

// Icon-only tactile button for AppBar (no ripple, slight press scale)
class _TactileIconButton extends StatefulWidget {
  const _TactileIconButton({
    required this.icon,
    required this.color,
    required this.onTap,
    this.onLongPress,
    this.semanticLabel,
    this.size = 22,
    this.haptics = true,
  });

  final IconData icon;
  final Color color;
  final VoidCallback onTap;
  final VoidCallback? onLongPress;
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
    final icon = Icon(widget.icon, size: widget.size, color: _pressed ? pressColor : base, semanticLabel: widget.semanticLabel);

    return Semantics(
      button: true,
      label: widget.semanticLabel,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTapDown: (_) => setState(() => _pressed = true),
        onTapUp: (_) => setState(() => _pressed = false),
        onTapCancel: () => setState(() => _pressed = false),
        onTap: () {
          if (widget.haptics) Haptics.light();
          widget.onTap();
        },
        onLongPress: widget.onLongPress == null
            ? null
            : () {
                if (widget.haptics) Haptics.medium();
                widget.onLongPress!.call();
              },
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

class _TactileRowState extends State<_TactileRow> {
  bool _pressed = false;
  void _setPressed(bool v) {
    if (_pressed != v) setState(() => _pressed = v);
  }
  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTapDown: widget.onTap == null ? null : (_) => _setPressed(true),
      onTapUp: widget.onTap == null ? null : (_) => _setPressed(false),
      onTapCancel: widget.onTap == null ? null : () => _setPressed(false),
      onTap: widget.onTap == null
          ? null
          : () {
              if (widget.haptics) Haptics.soft();
              widget.onTap!.call();
            },
      child: AnimatedScale(
        scale: _pressed ? widget.pressedScale : 1.0,
        duration: const Duration(milliseconds: 110),
        curve: Curves.easeOutCubic,
        child: widget.builder(_pressed),
      ),
    );
  }
}

// Bottom tactile tabs (two items) without ripple
class _BottomTabs extends StatelessWidget {
  const _BottomTabs({
    required this.index,
    required this.leftIcon,
    required this.leftLabel,
    required this.rightIcon,
    required this.rightLabel,
    required this.onSelect,
  });
  final int index;
  final IconData leftIcon;
  final String leftLabel;
  final IconData rightIcon;
  final String rightLabel;
  final ValueChanged<int> onSelect;

  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;
    final isDark = Theme.of(context).brightness == Brightness.dark;
    return Container(
      decoration: BoxDecoration(
        color: isDark ? Colors.transparent : cs.surface,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(color: cs.outlineVariant.withOpacity(isDark ? 0.18 : 0.12), width: 0.8),
      ),
      padding: const EdgeInsets.symmetric(vertical: 6, horizontal: 8),
      child: Row(
        children: [
          Expanded(child: _BottomTabItem(icon: leftIcon, label: leftLabel, selected: index == 0, onTap: () => onSelect(0))),
          Expanded(child: _BottomTabItem(icon: rightIcon, label: rightLabel, selected: index == 1, onTap: () => onSelect(1))),
        ],
      ),
    );
  }
}

class _BottomTabItem extends StatefulWidget {
  const _BottomTabItem({required this.icon, required this.label, required this.selected, required this.onTap});
  final IconData icon;
  final String label;
  final bool selected;
  final VoidCallback onTap;

  @override
  State<_BottomTabItem> createState() => _BottomTabItemState();
}

class _BottomTabItemState extends State<_BottomTabItem> {
  bool _pressed = false;

  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;
    final baseColor = cs.onSurface.withOpacity(0.7);
    final selColor = cs.primary;
    final target = widget.selected ? selColor : baseColor;

    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTapDown: (_) => setState(() => _pressed = true),
      onTapUp: (_) => setState(() => _pressed = false),
      onTapCancel: () => setState(() => _pressed = false),
      onTap: () {
        Haptics.soft();
        widget.onTap();
      },
      child: TweenAnimationBuilder<Color?>(
        tween: ColorTween(end: target),
        duration: const Duration(milliseconds: 220),
        curve: Curves.easeOutCubic,
        builder: (context, color, _) {
          final c = color ?? baseColor;
          return AnimatedScale(
            scale: _pressed ? 0.95 : 1.0,
            duration: const Duration(milliseconds: 110),
            curve: Curves.easeOutCubic,
            child: Padding(
              padding: const EdgeInsets.symmetric(vertical: 6, horizontal: 6),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(widget.icon, size: 20, color: c),
                  const SizedBox(height: 4),
                  AnimatedDefaultTextStyle(
                    duration: const Duration(milliseconds: 200),
                    curve: Curves.easeOutCubic,
                    style: TextStyle(fontSize: 12, fontWeight: FontWeight.w600, color: c),
                    child: Text(widget.label, maxLines: 1, overflow: TextOverflow.ellipsis),
                  ),
                ],
              ),
            ),
          );
        },
      ),
    );
  }
}
