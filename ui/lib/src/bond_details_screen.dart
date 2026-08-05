import 'package:flutter/material.dart';
import 'theme.dart';
import 'node_client.dart';

// ---------------------------------------------------------------------------
// Screen: Bond Details & Lifecycle Management
// ---------------------------------------------------------------------------

class BondDetailsScreen extends StatefulWidget {
  final String bondId;
  const BondDetailsScreen({super.key, required this.bondId});

  @override
  State<BondDetailsScreen> createState() => _BondDetailsScreenState();
}

class _BondDetailsScreenState extends State<BondDetailsScreen> {
  final _client = NodeClient();
  bool _loading = true;
  String? _error;
  Map<String, dynamic>? _bond;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() { _loading = true; _error = null; });
    try {
      // In a real app, this would fetch the full BondV2 object
      final bond = await _client.bondGet(widget.bondId);
      if (mounted) setState(() { _bond = bond; _loading = false; });
    } on NodeException catch (e) {
      if (mounted) setState(() { _error = e.message; _loading = false; });
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) return const Scaffold(body: Center(child: CircularProgressIndicator()));
    if (_error != null) return Scaffold(body: Center(child: Text(_error!)));
    if (_bond == null) return const Scaffold(body: Center(child: Text('Bond not found')));

    final tt = Theme.of(context).textTheme;
    final state = _bond!['state'] as String;

    return Scaffold(
      appBar: AppBar(
        title: Text('Bond Detail', style: tt.titleMedium),
        backgroundColor: NyxColors.background,
        elevation: 0,
      ),
      body: SingleChildScrollView(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // ── Header: Basic Meta ────────────────────────────────
            _HeaderSection(bond: _bond!),

            const Divider(height: 1, color: NyxColors.border),

            // ── State-Specific Body ──────────────────────────────
            Padding(
              padding: const EdgeInsets.all(24),
              child: _buildStateView(state),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildStateView(String state) {
    switch (state) {
      case 'Proposed':    return _ProposedView(bond: _bond!);
      case 'Active':      return _ActiveView(bond: _bond!);
      case 'Challenge':   return _ChallengeView(bond: _bond!);
      case 'Maintenance': return _MaintenanceView(bond: _bond!);
      case 'Redeemable':  return _RedeemableView(bond: _bond!);
      case 'Settled':     return _SettledView(bond: _bond!);
      case 'Expired':     return _ExpiredView(bond: _bond!);
      default:            return Text('Unknown State: $state');
    }
  }
}

// ---------------------------------------------------------------------------
// Header Section
// ---------------------------------------------------------------------------

class _HeaderSection extends StatelessWidget {
  final Map<String, dynamic> bond;
  const _HeaderSection({required this.bond});

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;
    final goals = bond['goals'] as List;
    final goal = goals.first;

    return Container(
      padding: const EdgeInsets.fromLTRB(24, 12, 24, 24),
      color: NyxColors.surface,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _StateBadge(state: bond['state']),
          const SizedBox(height: 12),
          Text(goal['title'], style: tt.displaySmall?.copyWith(fontSize: 22)),
          const SizedBox(height: 8),
          Text(goal['description'], style: tt.bodyMedium?.copyWith(color: NyxColors.textSecondary)),
          const SizedBox(height: 16),
          Row(
            children: [
              _MetaItem(label: 'Expiry', value: goal['deadline'].split('T')[0]),
              const SizedBox(width: 24),
              _MetaItem(label: 'Supply', value: '${bond['total_supply']} units'),
            ],
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Lifecycle Views
// ---------------------------------------------------------------------------

class _ProposedView extends StatelessWidget {
  final Map<String, dynamic> bond;
  const _ProposedView({required this.bond});

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _SectionTitle('Community Discussion'),
        const Text('SPBs require public buy-in. Share suggestions or concerns anonymously.',
            style: TextStyle(color: NyxColors.textMuted, fontSize: 13)),
        const SizedBox(height: 16),
        _CommentTile(author: '0xabc…', body: 'The metric data source should be NASA, not NOAA.', date: '2h ago'),
        _CommentTile(author: '0xdef…', body: 'Is the 200-year yield sufficient for oracle relay?', date: '1d ago'),
        const SizedBox(height: 24),
        SizedBox(
          width: double.infinity,
          child: ElevatedButton(
            onPressed: () {},
            child: const Text('BACK THIS PROPOSAL'),
          ),
        ),
      ],
    );
  }
}

class _ActiveView extends StatelessWidget {
  final Map<String, dynamic> bond;
  const _ActiveView({required this.bond});

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _SectionTitle('Robotic Monitoring'),
        Container(
          height: 120,
          width: double.infinity,
          decoration: BoxDecoration(
            color: NyxColors.background,
            borderRadius: BorderRadius.circular(8),
            border: Border.all(color: NyxColors.border),
          ),
          child: const Center(child: Text('Live Metric Graph [WIP]', style: TextStyle(color: NyxColors.textMuted))),
        ),
        const SizedBox(height: 24),
        _SectionTitle('Social Goal Progress'),
        _ReviewRow('Current Value', '412.5 ppm'),
        _ReviewRow('Goal Threshold', '< 350.0 ppm'),
        const SizedBox(height: 32),
        const Text('This is a Hybrid bond. Any anonymous actor can assert the goal has been met by posting a bond.',
            style: TextStyle(color: NyxColors.textMuted, fontSize: 12)),
        const SizedBox(height: 12),
        SizedBox(
          width: double.infinity,
          child: OutlinedButton(
            onPressed: () {},
            child: const Text('ASSERT GOAL MET'),
          ),
        ),
      ],
    );
  }
}

class _ChallengeView extends StatelessWidget {
  final Map<String, dynamic> bond;
  const _ChallengeView({required this.bond});

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Container(
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            color: NyxColors.danger.withValues(alpha: 0.1),
            borderRadius: BorderRadius.circular(8),
            border: Border.all(color: NyxColors.danger),
          ),
          child: const Column(
            children: [
              Row(
                children: [
                  Icon(Icons.warning_amber_rounded, color: NyxColors.danger),
                  SizedBox(width: 12),
                  Text('OUTCOME DISPUTED', style: TextStyle(color: NyxColors.danger, fontWeight: FontWeight.bold)),
                ],
              ),
              SizedBox(height: 8),
              Text('An actor has challenged the robotic oracle reading. The Optimistic Game is now live.',
                  style: TextStyle(color: NyxColors.textPrimary, fontSize: 13)),
            ],
          ),
        ),
        const SizedBox(height: 24),
        _SectionTitle('The Optimistic Game'),
        _ReviewRow('Current Stake', '2.5 XMR'),
        _ReviewRow('Challenge Closes', '22h 14m'),
        const SizedBox(height: 32),
        SizedBox(
          width: double.infinity,
          child: ElevatedButton(
            style: ElevatedButton.styleFrom(backgroundColor: NyxColors.danger, foregroundColor: Colors.white),
            onPressed: () {},
            child: const Text('DOUBLE STAKE TO CHALLENGE'),
          ),
        ),
        const SizedBox(height: 12),
        const Center(child: Text('Escalates to Jury if challenged again.', style: TextStyle(color: NyxColors.textMuted, fontSize: 11))),
      ],
    );
  }
}

class _MaintenanceView extends StatelessWidget {
  final Map<String, dynamic> bond;
  const _MaintenanceView({required this.bond});

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _SectionTitle('Stability Endowment'),
        const Text('This bond is in maintenance mode, paying stability dividends from XMR yield.',
            style: TextStyle(color: NyxColors.textMuted, fontSize: 13)),
        const SizedBox(height: 20),
        Container(
          padding: const EdgeInsets.all(20),
          decoration: BoxDecoration(
            color: NyxColors.accentGlow,
            borderRadius: BorderRadius.circular(12),
          ),
          child: Column(
            children: [
              const Text('Endowment Principal', style: TextStyle(color: NyxColors.textSecondary, fontSize: 12)),
              Text('10,000 XMR', style: Theme.of(context).textTheme.headlineMedium?.copyWith(color: NyxColors.accentBright)),
              const Divider(height: 32),
              const Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text('Yield Accrued (2026)', style: TextStyle(color: NyxColors.textMuted)),
                  Text('154.2 XMR', style: TextStyle(color: NyxColors.success)),
                ],
              ),
            ],
          ),
        ),
        const SizedBox(height: 24),
        _SectionTitle('Dividend History'),
        _DividendTile(amount: '1.2 XMR', date: 'Mar 01, 2026', status: 'PAID'),
        _DividendTile(amount: '1.2 XMR', date: 'Feb 01, 2026', status: 'PAID'),
      ],
    );
  }
}

class _RedeemableView extends StatelessWidget {
  final Map<String, dynamic> bond;
  const _RedeemableView({required this.bond});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        const Icon(Icons.verified_outlined, color: NyxColors.success, size: 64),
        const SizedBox(height: 16),
        const Text('GOAL ACHIEVED', style: TextStyle(color: NyxColors.success, fontSize: 24, fontWeight: FontWeight.w300, letterSpacing: 2)),
        const SizedBox(height: 8),
        const Text('The oracle consensus has confirmed the criteria were met. Payout is available.',
            textAlign: TextAlign.center, style: TextStyle(color: NyxColors.textSecondary)),
        const SizedBox(height: 40),
        Container(
          padding: const EdgeInsets.all(24),
          decoration: BoxDecoration(
            border: Border.all(color: NyxColors.border),
            borderRadius: BorderRadius.circular(12),
          ),
          child: Column(
            children: [
              const Text('REDEEMABLE VALUE', style: TextStyle(color: NyxColors.textMuted, fontSize: 12)),
              const Text('12.5 XMR', style: TextStyle(color: NyxColors.textPrimary, fontSize: 32, fontWeight: FontWeight.bold)),
              const Text('Per Bond Note', style: TextStyle(color: NyxColors.textMuted, fontSize: 11)),
              const SizedBox(height: 24),
              SizedBox(
                width: double.infinity,
                child: ElevatedButton(
                  onPressed: () {},
                  child: const Text('GENERATE BURN PROOF & CLAIM'),
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

class _SettledView extends StatelessWidget {
  final Map<String, dynamic> bond;
  const _SettledView({required this.bond});

  @override
  Widget build(BuildContext context) {
    return const Center(child: Text('Bond Settled. Payouts complete.', style: TextStyle(color: NyxColors.textMuted)));
  }
}

class _ExpiredView extends StatelessWidget {
  final Map<String, dynamic> bond;
  const _ExpiredView({required this.bond});

  @override
  Widget build(BuildContext context) {
    return const Center(child: Text('Bond Expired. Collateral returned to issuer.', style: TextStyle(color: NyxColors.textMuted)));
  }
}

// ---------------------------------------------------------------------------
// Shared Component Widgets
// ---------------------------------------------------------------------------

class _StateBadge extends StatelessWidget {
  final String state;
  const _StateBadge({required this.state});

  @override
  Widget build(BuildContext context) {
    Color c = NyxColors.accent;
    if (state == 'Active') c = NyxColors.success;
    if (state == 'Challenge') c = NyxColors.danger;
    if (state == 'Redeemable') c = NyxColors.success;

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: c.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: c.withValues(alpha: 0.5)),
      ),
      child: Text(state.toUpperCase(),
          style: TextStyle(color: c, fontSize: 10, fontWeight: FontWeight.bold, letterSpacing: 1)),
    );
  }
}

class _MetaItem extends StatelessWidget {
  final String label, value;
  const _MetaItem({required this.label, required this.value});

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label, style: const TextStyle(color: NyxColors.textMuted, fontSize: 11)),
        Text(value, style: const TextStyle(color: NyxColors.textPrimary, fontSize: 14)),
      ],
    );
  }
}

class _SectionTitle extends StatelessWidget {
  final String title;
  const _SectionTitle(this.title);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Text(title.toUpperCase(),
          style: const TextStyle(color: NyxColors.accentBright, fontSize: 12, fontWeight: FontWeight.bold, letterSpacing: 1.5)),
    );
  }
}

class _ReviewRow extends StatelessWidget {
  final String label, value;
  const _ReviewRow(this.label, this.value);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: const TextStyle(color: NyxColors.textMuted)),
          Text(value, style: const TextStyle(color: NyxColors.textPrimary)),
        ],
      ),
    );
  }
}

class _CommentTile extends StatelessWidget {
  final String author, body, date;
  const _CommentTile({required this.author, required this.body, required this.date});

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(color: NyxColors.background, borderRadius: BorderRadius.circular(6), border: Border.all(color: NyxColors.border)),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(author, style: const TextStyle(color: NyxColors.accentBright, fontSize: 11)),
              Text(date, style: const TextStyle(color: NyxColors.textMuted, fontSize: 10)),
            ],
          ),
          const SizedBox(height: 6),
          Text(body, style: const TextStyle(color: NyxColors.textSecondary, fontSize: 13)),
        ],
      ),
    );
  }
}

class _DividendTile extends StatelessWidget {
  final String amount, date, status;
  const _DividendTile({required this.amount, required this.date, required this.status});

  @override
  Widget build(BuildContext context) {
    return ListTile(
      contentPadding: EdgeInsets.zero,
      title: Text(amount, style: const TextStyle(color: NyxColors.textPrimary)),
      subtitle: Text(date, style: const TextStyle(color: NyxColors.textMuted, fontSize: 12)),
      trailing: Text(status, style: const TextStyle(color: NyxColors.success, fontSize: 10, fontWeight: FontWeight.bold)),
    );
  }
}
