import 'package:flutter/material.dart';
import 'theme.dart';
import 'orchestrator_mock.dart';

class DashboardScreen extends StatefulWidget {
  const DashboardScreen({super.key});

  @override
  State<DashboardScreen> createState() => _DashboardScreenState();
}

class _DashboardScreenState extends State<DashboardScreen> {
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

    return SingleChildScrollView(
      padding: const EdgeInsets.all(32),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('Dashboard', style: tt.displaySmall),
          const SizedBox(height: 8),
          Text('Unified Privacy Orchestrator', style: tt.bodyMedium),
          const SizedBox(height: 32),
          
          GridView.count(
            crossAxisCount: 2,
            shrinkWrap: true,
            physics: const NeverScrollableScrollPhysics(),
            crossAxisSpacing: 24,
            mainAxisSpacing: 24,
            childAspectRatio: 1.8,
            children: [
              _DashboardCard(
                title: 'TOTAL ASSETS',
                value: '${_mock.xmrBalance.toStringAsFixed(4)} XMR',
                subtitle: '≈ \$${(_mock.xmrBalance * 175).toStringAsFixed(2)} USD',
                icon: Icons.account_balance_wallet_outlined,
                color: NyxColors.primary,
              ),
              _DashboardCard(
                title: 'MINING YIELD',
                value: '${_mock.hashrate.toStringAsFixed(1)} KH/s',
                subtitle: _mock.isMining ? 'Active: XMRig + P2Pool' : 'Miner Offline',
                icon: Icons.bolt,
                color: _mock.isMining ? NyxColors.success : NyxColors.textMuted,
              ),
              _DashboardCard(
                title: 'ACTIVE SWAPS',
                value: '${_mock.pendingSwaps} Pending',
                subtitle: 'BasicSwap: XMR/BTC',
                icon: Icons.swap_horizontal_circle_outlined,
                color: NyxColors.warning,
              ),
              _DashboardCard(
                title: 'COMMUNITY',
                value: '${_mock.unreadMessages} New',
                subtitle: 'Cwtch: Encrypted P2P',
                icon: Icons.forum_outlined,
                color: NyxColors.accentBright,
              ),
            ],
          ),
          
          const SizedBox(height: 32),
          Text('Recent Activity', style: tt.titleLarge),
          const SizedBox(height: 16),
          const _ActivityList(),
        ],
      ),
    );
  }
}

class _DashboardCard extends StatelessWidget {
  const _DashboardCard({
    required this.title,
    required this.value,
    required this.subtitle,
    required this.icon,
    required this.color,
  });

  final String title;
  final String value;
  final String subtitle;
  final IconData icon;
  final Color color;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(title, style: const TextStyle(
                  color: NyxColors.textMuted,
                  fontSize: 12,
                  fontWeight: FontWeight.w700,
                  letterSpacing: 1.2,
                )),
                Icon(icon, color: color, size: 20),
              ],
            ),
            const Spacer(),
            Text(value, style: const TextStyle(
              color: NyxColors.textPrimary,
              fontSize: 24,
              fontWeight: FontWeight.w600,
            )),
            const SizedBox(height: 4),
            Text(subtitle, style: const TextStyle(
              color: NyxColors.textSecondary,
              fontSize: 13,
            )),
          ],
        ),
      ),
    );
  }
}

class _ActivityList extends StatelessWidget {
  const _ActivityList();

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        _ActivityItem(
          icon: Icons.add_circle_outline,
          title: 'Bond Issued',
          time: '2h ago',
          desc: 'NyxForge Alpha Release Bond #402',
        ),
        _ActivityItem(
          icon: Icons.call_received,
          title: 'Incoming XMR',
          time: '5h ago',
          desc: 'Received 0.5 XMR from External Wallet',
        ),
        _ActivityItem(
          icon: Icons.security,
          title: 'Node Synced',
          time: '8h ago',
          desc: 'Local Monero node at 100% sync',
        ),
      ],
    );
  }
}

class _ActivityItem extends StatelessWidget {
  const _ActivityItem({
    required this.icon,
    required this.title,
    required this.time,
    required this.desc,
  });

  final IconData icon;
  final String title;
  final String time;
  final String desc;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 12),
      child: Row(
        children: [
          Container(
            padding: const EdgeInsets.all(10),
            decoration: BoxDecoration(
              color: NyxColors.surface,
              borderRadius: BorderRadius.circular(8),
              border: Border.all(color: NyxColors.border),
            ),
            child: Icon(icon, color: NyxColors.textSecondary, size: 18),
          ),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Text(title, style: const TextStyle(
                      color: NyxColors.textPrimary,
                      fontSize: 14,
                      fontWeight: FontWeight.w500,
                    )),
                    Text(time, style: const TextStyle(
                      color: NyxColors.textMuted,
                      fontSize: 12,
                    )),
                  ],
                ),
                const SizedBox(height: 2),
                Text(desc, style: const TextStyle(
                  color: NyxColors.textSecondary,
                  fontSize: 13,
                )),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
