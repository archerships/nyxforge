import 'dart:async';
import 'dart:io' show Platform;

import 'package:flutter/material.dart';
import 'theme.dart';
import 'node_client.dart';

class MineScreen extends StatefulWidget {
  const MineScreen({super.key});

  @override
  State<MineScreen> createState() => _MineScreenState();
}

class _MineScreenState extends State<MineScreen> {
  final _client = NodeClient();
  MinerStatus? _status;
  String? _error;
  Timer? _pollTimer;
  int _threads = 1;
  final int _maxThreads = _safeProcessorCount();

  @override
  void initState() {
    super.initState();
    _threads = 1;
    _poll();
    _pollTimer = Timer.periodic(const Duration(seconds: 5), (_) => _poll());
  }

  @override
  void dispose() {
    _pollTimer?.cancel();
    _client.dispose();
    super.dispose();
  }

  Future<void> _poll() async {
    try {
      final s = await _client.minerStatus();
      if (mounted) setState(() { _status = s; _error = null; });
    } on NodeException catch (e) {
      if (mounted) setState(() { _error = e.message; });
    }
  }

  Future<void> _start() async {
    try {
      await _client.minerStart(threads: _threads);
      await _poll();
    } on NodeException catch (e) {
      if (mounted) setState(() { _error = e.message; });
    }
  }

  Future<void> _stop() async {
    try {
      await _client.minerStop();
      await _poll();
    } on NodeException catch (e) {
      if (mounted) setState(() { _error = e.message; });
    }
  }

  Future<void> _setThreads(int n) async {
    setState(() { _threads = n; });
    try {
      await _client.minerSetThreads(n);
    } on NodeException catch (e) {
      if (mounted) setState(() { _error = e.message; });
    }
  }

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;
    final running = _status?.running ?? false;

    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Title
          Text('Mine', style: tt.titleLarge),
          const SizedBox(height: 4),
          Text('Earn XMR via P2Pool mini CPU mining', style: tt.bodyMedium),
          const SizedBox(height: 24),

          // Status banner
          _StatusBanner(
            running: running,
            onStart: _start,
            onStop: _stop,
          ),
          const SizedBox(height: 24),

          // Hashrate display
          if (_status != null) ...[
            _HashrateDisplay(hashrate: _status!.hashrate),
            const SizedBox(height: 24),

            // Stats row
            _StatsRow(
              sharesFound: _status!.sharesFound,
              xmrPendingPico: _status!.xmrPendingPico,
              connected: running,
            ),
            const SizedBox(height: 24),
          ],

          // Thread slider
          _ThreadSlider(
            value: _threads,
            max: _maxThreads,
            onChanged: _setThreads,
          ),
          const SizedBox(height: 24),

          // Error display
          if (_error != null) ...[
            _ErrorBanner(message: _error!),
            const SizedBox(height: 16),
          ],

          // Info card
          const _InfoCard(),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Sub-widgets
// ---------------------------------------------------------------------------

class _StatusBanner extends StatelessWidget {
  const _StatusBanner({
    required this.running,
    required this.onStart,
    required this.onStop,
  });
  final bool running;
  final VoidCallback onStart;
  final VoidCallback onStop;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      decoration: BoxDecoration(
        color: running
            ? const Color(0xFF0A1F12)
            : NyxColors.surfaceHigh,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
            color: running ? NyxColors.success : NyxColors.border,
            width: 1),
      ),
      child: Row(
        children: [
          Container(
            width: 8, height: 8,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: running ? NyxColors.success : NyxColors.textMuted,
            ),
          ),
          const SizedBox(width: 10),
          Text(
            running ? 'Mining' : 'Stopped',
            style: TextStyle(
              color: running ? NyxColors.success : NyxColors.textSecondary,
              fontWeight: FontWeight.w600,
              fontSize: 14,
            ),
          ),
          const Spacer(),
          running
              ? OutlinedButton.icon(
                  onPressed: onStop,
                  icon: const Icon(Icons.stop, size: 14),
                  label: const Text('STOP'),
                  style: OutlinedButton.styleFrom(
                      foregroundColor: NyxColors.danger,
                      side: const BorderSide(color: NyxColors.danger)),
                )
              : ElevatedButton.icon(
                  onPressed: onStart,
                  icon: const Icon(Icons.bolt, size: 14),
                  label: const Text('START'),
                ),
        ],
      ),
    );
  }
}

class _HashrateDisplay extends StatelessWidget {
  const _HashrateDisplay({required this.hashrate});
  final double hashrate;

  String get _formatted {
    if (hashrate >= 1000) {
      return '${(hashrate / 1000).toStringAsFixed(2)} kH/s';
    }
    return '${hashrate.toStringAsFixed(1)} H/s';
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text('Hashrate (60s avg)',
            style: TextStyle(color: NyxColors.textMuted, fontSize: 11,
                letterSpacing: 1)),
        const SizedBox(height: 4),
        Text(
          _formatted,
          style: const TextStyle(
            color: NyxColors.accentBright,
            fontSize: 36,
            fontWeight: FontWeight.w300,
            fontFamily: 'monospace',
            letterSpacing: 1,
          ),
        ),
      ],
    );
  }
}

class _StatsRow extends StatelessWidget {
  const _StatsRow({
    required this.sharesFound,
    required this.xmrPendingPico,
    required this.connected,
  });
  final int sharesFound;
  final int xmrPendingPico;
  final bool connected;

  String get _pendingDisplay {
    final xmr = xmrPendingPico / 1e12;
    return xmr.toStringAsFixed(6);
  }

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        _StatChip(label: 'Shares found', value: '$sharesFound'),
        const SizedBox(width: 12),
        _StatChip(label: 'Pending', value: '$_pendingDisplay XMR'),
        const SizedBox(width: 12),
        _StatChip(
          label: 'P2Pool',
          value: connected ? 'Connected' : 'Offline',
          valueColor: connected ? NyxColors.success : NyxColors.textMuted,
        ),
      ],
    );
  }
}

class _StatChip extends StatelessWidget {
  const _StatChip({required this.label, required this.value, this.valueColor});
  final String label;
  final String value;
  final Color? valueColor;

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: Container(
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: NyxColors.surfaceHigh,
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: NyxColors.border),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(label,
                style: const TextStyle(
                    color: NyxColors.textMuted, fontSize: 10, letterSpacing: 0.5)),
            const SizedBox(height: 4),
            Text(value,
                style: TextStyle(
                    color: valueColor ?? NyxColors.textPrimary,
                    fontSize: 13,
                    fontWeight: FontWeight.w500)),
          ],
        ),
      ),
    );
  }
}

class _ThreadSlider extends StatelessWidget {
  const _ThreadSlider({
    required this.value,
    required this.max,
    required this.onChanged,
  });
  final int value;
  final int max;
  final ValueChanged<int> onChanged;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            const Text('CPU threads',
                style: TextStyle(color: NyxColors.textMuted,
                    fontSize: 11, letterSpacing: 1)),
            const Spacer(),
            Text('$value / $max',
                style: const TextStyle(
                    color: NyxColors.textSecondary, fontSize: 12)),
          ],
        ),
        Slider(
          value: value.toDouble(),
          min: 1,
          max: max.toDouble(),
          divisions: max > 1 ? max - 1 : 1,
          label: '$value',
          onChanged: (v) => onChanged(v.round()),
        ),
      ],
    );
  }
}

class _ErrorBanner extends StatelessWidget {
  const _ErrorBanner({required this.message});
  final String message;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: const Color(0xFF1A0F0F),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: NyxColors.danger),
      ),
      child: Row(
        children: [
          const Icon(Icons.warning_amber, color: NyxColors.danger, size: 14),
          const SizedBox(width: 8),
          Expanded(child: Text(message,
              style: const TextStyle(
                  color: NyxColors.textSecondary, fontSize: 12))),
        ],
      ),
    );
  }
}

class _InfoCard extends StatelessWidget {
  const _InfoCard();

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Row(
              children: [
                Icon(Icons.info_outline, color: NyxColors.accentBright, size: 16),
                SizedBox(width: 8),
                Text('How it works',
                    style: TextStyle(
                        color: NyxColors.accentBright,
                        fontWeight: FontWeight.w600,
                        fontSize: 13)),
              ],
            ),
            const SizedBox(height: 10),
            Text(
              'Mining uses RandomX on your CPU to earn Monero (XMR) via P2Pool mini, '
              'a decentralised mining pool with no registration required.\n\n'
              'Payouts go directly to your wallet address on the P2Pool sidechain. '
              'Shares are typically found within minutes on modern hardware.\n\n'
              'DarkFi merge-mining will be enabled automatically when available, '
              'earning DRK at no extra CPU cost.',
              style: tt.bodySmall,
            ),
          ],
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

int _safeProcessorCount() {
  try {
    return Platform.numberOfProcessors.clamp(1, 64);
  } catch (_) {
    return 4;
  }
}
