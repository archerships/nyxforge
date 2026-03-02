import 'package:flutter/material.dart';
import 'theme.dart';
import 'node_client.dart';
import 'wallet_screen.dart';
import 'mine_screen.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  int _selectedIndex = 0;

  static const _destinations = [
    NavigationRailDestination(
      icon:  Icon(Icons.search),
      label: Text('BONDS'),
    ),
    NavigationRailDestination(
      icon:  Icon(Icons.add_circle_outline),
      label: Text('ISSUE'),
    ),
    NavigationRailDestination(
      icon:  Icon(Icons.account_balance_wallet_outlined),
      label: Text('WALLET'),
    ),
    NavigationRailDestination(
      icon:  Icon(Icons.bolt),
      label: Text('MINE'),
    ),
    NavigationRailDestination(
      icon:  Icon(Icons.settings_outlined),
      label: Text('NODE'),
    ),
  ];

  static const _screens = [
    _BondListPlaceholder(),
    _IssuePlaceholder(),
    WalletScreen(),
    MineScreen(),
    _NodeStatusScreen(),
  ];

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Row(
        children: [
          NavigationRail(
            selectedIndex:    _selectedIndex,
            onDestinationSelected: (i) => setState(() => _selectedIndex = i),
            labelType:        NavigationRailLabelType.all,
            leading:          const _SidebarLogo(),
            destinations:     _destinations,
            minWidth:         72,
          ),
          const VerticalDivider(width: 1),
          Expanded(
            child: _screens[_selectedIndex],
          ),
        ],
      ),
    );
  }
}

// -- Sidebar logo -------------------------------------------------------------

class _SidebarLogo extends StatelessWidget {
  const _SidebarLogo();

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 24),
      child: Column(
        children: [
          Container(
            width: 40,
            height: 40,
            decoration: BoxDecoration(
              color:        NyxColors.accentGlow,
              borderRadius: BorderRadius.circular(8),
              border:       Border.all(color: NyxColors.accent, width: 1.5),
            ),
            child: const Icon(Icons.hub, color: NyxColors.accentBright, size: 22),
          ),
          const SizedBox(height: 6),
          const Text(
            'NYX',
            style: TextStyle(
              color:       NyxColors.accentBright,
              fontSize:    10,
              fontWeight:  FontWeight.w700,
              letterSpacing: 2,
            ),
          ),
        ],
      ),
    );
  }
}

// -- Placeholder screens ------------------------------------------------------

class _BondListPlaceholder extends StatelessWidget {
  const _BondListPlaceholder();

  @override
  Widget build(BuildContext context) {
    return _PlaceholderScreen(
      icon:     Icons.bar_chart,
      title:    'Bond Market',
      subtitle: 'Browse and trade anonymous social policy bonds',
      chips:    const ['All Bonds', 'Environment', 'Health', 'Housing', 'Education'],
    );
  }
}

class _IssuePlaceholder extends StatelessWidget {
  const _IssuePlaceholder();

  @override
  Widget build(BuildContext context) {
    return _PlaceholderScreen(
      icon:     Icons.add_circle_outline,
      title:    'Issue Bond',
      subtitle: 'Define a social goal and issue bonds backed by collateral',
      chips:    const ['Define Goal', 'Set Oracle', 'Lock Collateral', 'Publish'],
    );
  }
}

class _PlaceholderScreen extends StatelessWidget {
  const _PlaceholderScreen({
    required this.icon,
    required this.title,
    required this.subtitle,
    required this.chips,
  });

  final IconData icon;
  final String   title;
  final String   subtitle;
  final List<String> chips;

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 48, color: NyxColors.textMuted),
          const SizedBox(height: 16),
          Text(title,    style: tt.titleLarge),
          const SizedBox(height: 8),
          Text(subtitle, style: tt.bodyMedium),
          const SizedBox(height: 24),
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: chips
                .map((c) => Chip(label: Text(c)))
                .toList(),
          ),
          const SizedBox(height: 32),
          OutlinedButton(
            onPressed: () {},
            child: const Text('COMING SOON'),
          ),
        ],
      ),
    );
  }
}

// -- Node status screen -------------------------------------------------------

class _NodeStatusScreen extends StatefulWidget {
  const _NodeStatusScreen();

  @override
  State<_NodeStatusScreen> createState() => _NodeStatusScreenState();
}

class _NodeStatusScreenState extends State<_NodeStatusScreen> {
  final _client = NodeClient();
  NodeStatus? _status;
  String?     _error;
  bool        _loading = false;

  @override
  void initState() {
    super.initState();
    _fetchStatus();
  }

  Future<void> _fetchStatus() async {
    setState(() { _loading = true; _error = null; });
    try {
      final s = await _client.status();
      if (mounted) setState(() { _status = s; _loading = false; });
    } on NodeException catch (e) {
      if (mounted) setState(() { _error = e.message; _loading = false; });
    }
  }

  @override
  void dispose() {
    _client.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;

    return Padding(
      padding: const EdgeInsets.all(32),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('Node Status', style: tt.titleLarge),
          const SizedBox(height: 4),
          Text('Local nyxforge-node connection', style: tt.bodyMedium),
          const SizedBox(height: 24),

          if (_loading)
            const Center(child: CircularProgressIndicator()),

          if (_error != null)
            _ErrorCard(message: _error!, onRetry: _fetchStatus),

          if (_status != null) ...[
            _StatusRow(label: 'Status',    value: 'Connected',
                       valueColor: NyxColors.success),
            _StatusRow(label: 'Version',   value: _status!.version),
            _StatusRow(label: 'Known bonds', value: '${_status!.bondCount}'),
            _StatusRow(label: 'RPC endpoint',
                       value: 'http://127.0.0.1:8888/rpc'),
          ],

          const Spacer(),
          OutlinedButton.icon(
            onPressed: _fetchStatus,
            icon:  const Icon(Icons.refresh, size: 16),
            label: const Text('REFRESH'),
          ),
        ],
      ),
    );
  }
}

class _StatusRow extends StatelessWidget {
  const _StatusRow({required this.label, required this.value, this.valueColor});
  final String label;
  final String value;
  final Color? valueColor;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Row(
        children: [
          SizedBox(
            width: 140,
            child: Text(label,
                style: const TextStyle(color: NyxColors.textMuted, fontSize: 13)),
          ),
          Text(value,
              style: TextStyle(
                color:      valueColor ?? NyxColors.textPrimary,
                fontSize:   13,
                fontWeight: FontWeight.w500,
              )),
        ],
      ),
    );
  }
}

class _ErrorCard extends StatelessWidget {
  const _ErrorCard({required this.message, required this.onRetry});
  final String   message;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return Card(
      color: const Color(0xFF1A0F0F),
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(8),
        side: const BorderSide(color: NyxColors.danger, width: 1),
      ),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Row(
              children: [
                Icon(Icons.link_off, color: NyxColors.danger, size: 16),
                SizedBox(width: 8),
                Text('Node offline',
                    style: TextStyle(color: NyxColors.danger,
                        fontWeight: FontWeight.w600)),
              ],
            ),
            const SizedBox(height: 8),
            Text(message,
                style: const TextStyle(
                    color: NyxColors.textSecondary, fontSize: 12)),
            const SizedBox(height: 12),
            TextButton(
              onPressed: onRetry,
              child: const Text('Retry'),
            ),
          ],
        ),
      ),
    );
  }
}
