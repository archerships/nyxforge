import 'package:flutter/material.dart';
import 'theme.dart';
import 'orchestrator_mock.dart';

class MineScreen extends StatefulWidget {
  const MineScreen({super.key});

  @override
  State<MineScreen> createState() => _MineScreenState();
}

class _MineScreenState extends State<MineScreen> {
  final _mock = OrchestratorMock.instance;

  @override
  void initState() {
    super.initState();
    _mock.addListener(_onStateChange);
  }

  @override
  void dispose() {
    _mock.removeListener(_onStateChange);
    super.dispose();
  }

  void _onStateChange() {
    if (mounted) setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;

    return Padding(
      padding: const EdgeInsets.all(32),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('Mining Control', style: tt.displaySmall),
          const SizedBox(height: 8),
          Text('XMRig + P2Pool Orchestration', style: tt.bodyMedium),
          const SizedBox(height: 32),
          
          Card(
            child: Padding(
              padding: const EdgeInsets.all(24),
              child: Column(
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text('STATUS', style: tt.labelLarge),
                          const SizedBox(height: 4),
                          Text(_mock.isMining ? 'ACTIVE' : 'STOPPED', 
                              style: TextStyle(
                                color: _mock.isMining ? NyxColors.success : NyxColors.danger,
                                fontSize: 24,
                                fontWeight: FontWeight.bold,
                              )),
                        ],
                      ),
                      Switch(
                        value: _mock.isMining,
                        onChanged: (v) => _mock.toggleMining(),
                        activeColor: NyxColors.success,
                      ),
                    ],
                  ),
                  const Divider(height: 48),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceAround,
                    children: [
                      _StatBox(label: 'HASHRATE', value: '${_mock.hashrate.toStringAsFixed(2)} KH/s'),
                      _StatBox(label: 'SHARES', value: '${_mock.shares}'),
                      _StatBox(label: 'THREADS', value: '${_mock.threads}'),
                    ],
                  ),
                ],
              ),
            ),
          ),
          
          const SizedBox(height: 32),
          Text('Protocol Settings', style: tt.titleLarge),
          const SizedBox(height: 16),
          const _MiningSettings(),
        ],
      ),
    );
  }
}

class _StatBox extends StatelessWidget {
  const _StatBox({required this.label, required this.value});
  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text(label, style: const TextStyle(color: NyxColors.textMuted, fontSize: 11, fontWeight: FontWeight.bold)),
        const SizedBox(height: 4),
        Text(value, style: const TextStyle(color: NyxColors.textPrimary, fontSize: 18, fontWeight: FontWeight.w500)),
      ],
    );
  }
}

class _MiningSettings extends StatelessWidget {
  const _MiningSettings();

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        _SettingTile(label: 'Mining Pool', value: 'p2pool.nyxforge.com:3333'),
        _SettingTile(label: 'Payout Address', value: '44AFFq5k...49564'),
        _SettingTile(label: 'Algorithm', value: 'RandomX'),
      ],
    );
  }
}

class _SettingTile extends StatelessWidget {
  const _SettingTile({required this.label, required this.value});
  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 12),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: const TextStyle(color: NyxColors.textSecondary)),
          Text(value, style: const TextStyle(color: NyxColors.textPrimary, fontWeight: FontWeight.w500)),
        ],
      ),
    );
  }
}
