import 'package:flutter/material.dart';
import 'theme.dart';
import 'node_client.dart';
import 'widgets/bounty_card.dart';

class BountyMarketScreen extends StatefulWidget {
  const BountyMarketScreen({super.key});

  @override
  State<BountyMarketScreen> createState() => _BountyMarketScreenState();
}

class _BountyMarketScreenState extends State<BountyMarketScreen> {
  final _client = NodeClient();
  List<Bounty> _bounties = [];
  bool _loading = true;
  String? _error;
  String _filter = 'ALL';
  final _filters = ['ALL', 'ACTIVE', 'REDEEMABLE', 'DRAFT', 'EXPIRED'];

  @override
  void initState() {
    super.initState();
    _load();
  }

  @override
  void dispose() {
    _client.dispose();
    super.dispose();
  }

  Future<void> _load() async {
    if (!mounted) return;
    setState(() { _loading = true; _error = null; });
    try {
      final bounties = await _client.bountyList();
      if (mounted) setState(() { _bounties = bounties; _loading = false; });
    } on NodeException catch (e) {
      if (mounted) setState(() { _error = e.message; _loading = false; });
    }
  }

  List<Bounty> get _filtered {
    if (_filter == 'ALL') return _bounties;
    return _bounties.where((b) => b.state.toUpperCase() == _filter).toList();
  }

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Header
        Container(
          padding: const EdgeInsets.fromLTRB(24, 32, 24, 0),
          child: Row(
            children: [
              Text('THE VAULT', style: tt.displaySmall?.copyWith(fontSize: 24)),
              const Spacer(),
              IconButton(
                onPressed: _load,
                icon: const Icon(Icons.refresh, color: NyxColors.textMuted),
              ),
            ],
          ),
        ),
        const Padding(
          padding: EdgeInsets.symmetric(horizontal: 24, vertical: 8),
          child: Text(
            'Your anonymous bearer bounty instruments',
            style: TextStyle(color: NyxColors.textSecondary),
          ),
        ),

        // Filter chips
        Padding(
          padding: const EdgeInsets.fromLTRB(20, 16, 20, 0),
          child: Wrap(
            spacing: 8,
            children: _filters.map((f) => ChoiceChip(
              label: Text(f),
              selected: _filter == f,
              onSelected: (val) { if (val) setState(() => _filter = f); },
              selectedColor: NyxColors.accentGlow,
              labelStyle: TextStyle(
                color: _filter == f ? NyxColors.accentBright : NyxColors.textMuted,
                fontSize: 11,
                fontWeight: FontWeight.bold,
              ),
            )).toList(),
          ),
        ),

        const SizedBox(height: 16),
        const Divider(color: NyxColors.border, height: 1),

        // List
        Expanded(
          child: _loading
              ? const Center(child: CircularProgressIndicator())
              : _error != null
                  ? Center(child: Text(_error!, style: const TextStyle(color: NyxColors.danger)))
                  : ListView.builder(
                      padding: const EdgeInsets.all(24),
                      itemCount: _filtered.length,
                      itemBuilder: (ctx, i) {
                        final b = _filtered[i];
                        return BountyCard(
                          title: b.title,
                          state: b.state,
                          amount: b.amount,
                          currency: b.currency,
                          deadline: b.deadline,
                          progress: b.progress,
                          oracleQuorum: b.oracleQuorum,
                          oracleTotal: b.oracleTotal,
                          onTap: () {
                            // TODO: Navigate to details
                          },
                          onAction: b.state.toUpperCase() == 'REDEEMABLE'
                              ? () {
                                  // TODO: Execute redemption
                                }
                              : null,
                        );
                      },
                    ),
        ),
      ],
    );
  }
}
