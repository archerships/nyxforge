import 'dart:async';
import 'package:flutter/material.dart';
import 'theme.dart';
import 'node_client.dart';

class BondMarketScreen extends StatefulWidget {
  const BondMarketScreen({super.key});

  @override
  State<BondMarketScreen> createState() => _BondMarketScreenState();
}

class _BondMarketScreenState extends State<BondMarketScreen> {
  final _client = NodeClient();
  List<BondSummary> _bonds = [];
  bool  _loading = true;
  String? _error;
  String _filter = 'All';
  final _filters = ['All', 'Active', 'Proposed', 'Draft'];

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
    setState(() { _loading = true; _error = null; });
    try {
      final bonds = await _client.bondList();
      if (mounted) setState(() { _bonds = bonds; _loading = false; });
    } on NodeException catch (e) {
      if (mounted) setState(() { _error = e.message; _loading = false; });
    }
  }

  List<BondSummary> get _filtered {
    if (_filter == 'All') return _bonds;
    return _bonds.where((b) => b.state == _filter).toList();
  }

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // ── Header bar ────────────────────────────────────────────────
        Container(
          padding: const EdgeInsets.fromLTRB(24, 20, 24, 0),
          child: Row(
            children: [
              Text('Bond Market', style: tt.titleLarge),
              const Spacer(),
              IconButton(
                onPressed: _load,
                tooltip: 'Refresh',
                icon: const Icon(Icons.refresh, size: 20, color: NyxColors.textMuted),
              ),
            ],
          ),
        ),
        Padding(
          padding: const EdgeInsets.fromLTRB(24, 8, 24, 0),
          child: Text('Browse and buy anonymous social policy bonds',
              style: tt.bodyMedium),
        ),

        // ── Filter chips ──────────────────────────────────────────────
        Padding(
          padding: const EdgeInsets.fromLTRB(20, 12, 20, 0),
          child: Wrap(
            spacing: 8,
            children: _filters.map((f) => FilterChip(
              label: Text(f),
              selected: _filter == f,
              onSelected: (_) => setState(() => _filter = f),
              selectedColor: NyxColors.accentGlow,
              labelStyle: TextStyle(
                color: _filter == f ? NyxColors.accentBright : NyxColors.textSecondary,
                fontSize: 12,
              ),
            )).toList(),
          ),
        ),

        const Divider(height: 16),

        // ── Bond list ─────────────────────────────────────────────────
        Expanded(
          child: _loading
              ? const Center(child: CircularProgressIndicator())
              : _error != null
                  ? _ErrorView(message: _error!, onRetry: _load)
                  : _filtered.isEmpty
                      ? _EmptyView(filter: _filter)
                      : ListView.builder(
                          padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
                          itemCount: _filtered.length,
                          itemBuilder: (ctx, i) => _BondCard(
                            bond: _filtered[i],
                            client: _client,
                            onRefresh: _load,
                          ),
                        ),
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Bond card
// ---------------------------------------------------------------------------

class _BondCard extends StatefulWidget {
  const _BondCard({required this.bond, required this.client, required this.onRefresh});
  final BondSummary bond;
  final NodeClient  client;
  final VoidCallback onRefresh;

  @override
  State<_BondCard> createState() => _BondCardState();
}

class _BondCardState extends State<_BondCard> {
  int?  _priceMicro;
  bool  _priceLoading = false;
  bool  _expanded = false;

  @override
  void initState() {
    super.initState();
    if (widget.bond.state == 'Active') _fetchPrice();
  }

  Future<void> _fetchPrice() async {
    setState(() => _priceLoading = true);
    try {
      final p = await widget.client.bondAuctionPrice(widget.bond.id);
      if (mounted) setState(() { _priceMicro = p; _priceLoading = false; });
    } catch (_) {
      if (mounted) setState(() => _priceLoading = false);
    }
  }

  Color _stateColor(String state) {
    switch (state) {
      case 'Active':    return NyxColors.success;
      case 'Draft':     return NyxColors.warning;
      case 'Proposed':  return NyxColors.textSecondary;
      case 'Redeemable': return NyxColors.accentBright;
      case 'Expired':   return NyxColors.danger;
      default:          return NyxColors.textMuted;
    }
  }

  @override
  Widget build(BuildContext context) {
    final b  = widget.bond;
    final tt = Theme.of(context).textTheme;
    final stateColor = _stateColor(b.state);
    final pctRemaining = b.totalSupply > 0
        ? b.bondsRemaining / b.totalSupply
        : 0.0;

    return Card(
      margin: const EdgeInsets.only(bottom: 12),
      child: InkWell(
        borderRadius: BorderRadius.circular(8),
        onTap: () => setState(() => _expanded = !_expanded),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // ── Title row ──────────────────────────────────────────
              Row(
                children: [
                  Expanded(
                    child: Text(b.primaryGoal.title,
                        style: tt.titleLarge?.copyWith(fontSize: 15),
                        maxLines: 2,
                        overflow: TextOverflow.ellipsis),
                  ),
                  const SizedBox(width: 8),
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                    decoration: BoxDecoration(
                      color: stateColor.withValues(alpha: 0.12),
                      borderRadius: BorderRadius.circular(4),
                      border: Border.all(color: stateColor.withValues(alpha: 0.4)),
                    ),
                    child: Text(b.state,
                        style: TextStyle(color: stateColor, fontSize: 11,
                            fontWeight: FontWeight.w600, letterSpacing: 0.5)),
                  ),
                ],
              ),
              const SizedBox(height: 10),

              // ── Stats row ──────────────────────────────────────────
              Row(
                children: [
                  _Stat(label: 'REDEMPTION', value: '${b.redemptionDisplay} DRK'),
                  const SizedBox(width: 24),
                  if (b.state == 'Active') ...[
                    _Stat(
                      label: 'ASK PRICE',
                      value: _priceLoading
                          ? '…'
                          : _priceMicro != null
                              ? '${(_priceMicro! / 1e6).toStringAsFixed(4)} DRK'
                              : '—',
                      valueColor: NyxColors.accentBright,
                    ),
                    const SizedBox(width: 24),
                  ],
                  _Stat(label: 'DEADLINE', value: b.primaryGoal.deadlineShort),
                ],
              ),
              const SizedBox(height: 10),

              // ── Supply bar ─────────────────────────────────────────
              Row(
                children: [
                  Expanded(
                    child: ClipRRect(
                      borderRadius: BorderRadius.circular(2),
                      child: LinearProgressIndicator(
                        value: pctRemaining,
                        backgroundColor: NyxColors.border,
                        color: NyxColors.accent,
                        minHeight: 4,
                      ),
                    ),
                  ),
                  const SizedBox(width: 8),
                  Text(
                    '${b.bondsRemaining.toStringWithCommas()} / ${b.totalSupply.toStringWithCommas()} left',
                    style: const TextStyle(color: NyxColors.textMuted, fontSize: 11),
                  ),
                ],
              ),

              // ── Expanded detail ────────────────────────────────────
              if (_expanded) ...[
                const Divider(height: 20),
                for (final (i, g) in b.goals.indexed) ...[
                  if (b.goals.length > 1)
                    Padding(
                      padding: const EdgeInsets.only(bottom: 4),
                      child: Text(
                        'Criterion ${i + 1}',
                        style: const TextStyle(
                          color: NyxColors.accentBright,
                          fontSize: 11,
                          fontWeight: FontWeight.w600,
                          letterSpacing: 0.8,
                        ),
                      ),
                    ),
                  Text(g.description,
                      style: tt.bodyMedium?.copyWith(color: NyxColors.textSecondary)),
                  const SizedBox(height: 4),
                  _KeyValue('Data ID', g.dataId),
                  _KeyValue('Condition', '${g.operator} ${g.threshold}'),
                  if (i < b.goals.length - 1) const SizedBox(height: 8),
                ],
                const SizedBox(height: 4),
                _KeyValue('Start price', '${b.auction.startDisplay} DRK'),
                _KeyValue('Reserve price', '${b.auction.reserveDisplay} DRK'),
                _KeyValue('Auction window', '${b.auction.durationDays} days'),
                _KeyValue('Bond ID', '${b.id.substring(0, 16)}…'),
              ],

              // ── Action buttons ─────────────────────────────────────
              if (b.state == 'Active') ...[
                const SizedBox(height: 12),
                Row(
                  children: [
                    ElevatedButton.icon(
                      onPressed: _priceMicro != null
                          ? () => _showBuyDialog(context)
                          : null,
                      icon: const Icon(Icons.shopping_cart_outlined, size: 16),
                      label: const Text('BUY'),
                    ),
                    const SizedBox(width: 8),
                    OutlinedButton(
                      onPressed: () => setState(() => _expanded = !_expanded),
                      child: Text(_expanded ? 'LESS' : 'DETAILS'),
                    ),
                  ],
                ),
              ] else ...[
                const SizedBox(height: 8),
                OutlinedButton(
                  onPressed: () => setState(() => _expanded = !_expanded),
                  child: Text(_expanded ? 'LESS' : 'DETAILS'),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }

  void _showBuyDialog(BuildContext context) {
    showDialog<void>(
      context: context,
      builder: (ctx) => _BuyDialog(
        bond:     widget.bond,
        priceMicro: _priceMicro!,
        client:   widget.client,
        onSuccess: () {
          widget.onRefresh();
          _fetchPrice();
        },
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Buy dialog
// ---------------------------------------------------------------------------

class _BuyDialog extends StatefulWidget {
  const _BuyDialog({
    required this.bond,
    required this.priceMicro,
    required this.client,
    required this.onSuccess,
  });
  final BondSummary  bond;
  final int          priceMicro;
  final NodeClient   client;
  final VoidCallback onSuccess;

  @override
  State<_BuyDialog> createState() => _BuyDialogState();
}

class _BuyDialogState extends State<_BuyDialog> {
  final _qtyCtrl = TextEditingController(text: '1');
  bool   _buying = false;
  String? _result;
  String? _err;

  int get _qty => int.tryParse(_qtyCtrl.text) ?? 1;
  int get _totalMicro => _qty * widget.priceMicro;

  @override
  void dispose() {
    _qtyCtrl.dispose();
    super.dispose();
  }

  Future<void> _buy() async {
    if (_qty <= 0) return;
    setState(() { _buying = true; _err = null; });
    try {
      final r = await widget.client.bondBuy(widget.bond.id, _qty);
      if (mounted) {
        setState(() { _buying = false; _result = 'Bought ${r.purchased} bond(s) at ${r.priceDisplay} DRK each.'; });
        widget.onSuccess();
      }
    } on NodeException catch (e) {
      if (mounted) setState(() { _buying = false; _err = e.message; });
    }
  }

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;
    final b  = widget.bond;
    return AlertDialog(
      backgroundColor: NyxColors.surface,
      title: Text('Buy Bonds', style: tt.titleLarge),
      content: SizedBox(
        width: 340,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(b.primaryGoal.title,
                style: tt.bodyMedium?.copyWith(color: NyxColors.textSecondary),
                maxLines: 2,
                overflow: TextOverflow.ellipsis),
            const SizedBox(height: 16),
            _KeyValue('Current price', '${(_totalMicro / 1e6 / _qty).toStringAsFixed(4)} DRK / bond'),
            _KeyValue('Available', '${b.bondsRemaining.toStringWithCommas()} bonds'),
            const SizedBox(height: 16),
            TextField(
              controller: _qtyCtrl,
              keyboardType: TextInputType.number,
              decoration: const InputDecoration(
                labelText: 'Quantity',
                suffixText: 'bonds',
              ),
              onChanged: (_) => setState(() {}),
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                const Text('Total cost:', style: TextStyle(color: NyxColors.textMuted, fontSize: 13)),
                const Spacer(),
                Text(
                  '${(_totalMicro / 1e6).toStringAsFixed(4)} DRK',
                  style: const TextStyle(color: NyxColors.accentBright,
                      fontWeight: FontWeight.w600, fontSize: 15),
                ),
              ],
            ),
            if (_result != null) ...[
              const SizedBox(height: 12),
              Text(_result!, style: const TextStyle(color: NyxColors.success)),
            ],
            if (_err != null) ...[
              const SizedBox(height: 12),
              Text(_err!, style: const TextStyle(color: NyxColors.danger, fontSize: 12)),
            ],
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('CANCEL'),
        ),
        if (_result == null)
          ElevatedButton(
            onPressed: _buying ? null : _buy,
            child: _buying
                ? const SizedBox(width: 16, height: 16,
                    child: CircularProgressIndicator(strokeWidth: 2))
                : const Text('BUY'),
          ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Small shared widgets
// ---------------------------------------------------------------------------

class _Stat extends StatelessWidget {
  const _Stat({required this.label, required this.value, this.valueColor});
  final String label;
  final String value;
  final Color? valueColor;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label, style: const TextStyle(color: NyxColors.textMuted, fontSize: 10, letterSpacing: 0.8)),
        const SizedBox(height: 2),
        Text(value, style: TextStyle(
          color: valueColor ?? NyxColors.textPrimary,
          fontSize: 13, fontWeight: FontWeight.w500,
        )),
      ],
    );
  }
}

class _KeyValue extends StatelessWidget {
  const _KeyValue(this.key_, this.value);
  final String key_;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 3),
      child: Row(
        children: [
          SizedBox(
            width: 120,
            child: Text(key_, style: const TextStyle(color: NyxColors.textMuted, fontSize: 12)),
          ),
          Expanded(
            child: Text(value,
                style: const TextStyle(color: NyxColors.textSecondary, fontSize: 12),
                overflow: TextOverflow.ellipsis),
          ),
        ],
      ),
    );
  }
}

class _ErrorView extends StatelessWidget {
  const _ErrorView({required this.message, required this.onRetry});
  final String message;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.link_off, color: NyxColors.danger, size: 40),
            const SizedBox(height: 12),
            Text(message,
                textAlign: TextAlign.center,
                style: const TextStyle(color: NyxColors.textSecondary, fontSize: 13)),
            const SizedBox(height: 16),
            OutlinedButton.icon(
              onPressed: onRetry,
              icon: const Icon(Icons.refresh, size: 16),
              label: const Text('RETRY'),
            ),
          ],
        ),
      ),
    );
  }
}

class _EmptyView extends StatelessWidget {
  const _EmptyView({required this.filter});
  final String filter;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.bar_chart, size: 48, color: NyxColors.textMuted),
          const SizedBox(height: 16),
          Text(filter == 'All' ? 'No bonds yet' : 'No $filter bonds',
              style: Theme.of(context).textTheme.titleLarge),
          const SizedBox(height: 8),
          const Text('Issue the first bond using the ISSUE tab.',
              style: TextStyle(color: NyxColors.textSecondary)),
        ],
      ),
    );
  }
}

// Extension for thousands-separator formatting
extension _IntFormat on int {
  String toStringWithCommas() {
    final s = toString();
    final buf = StringBuffer();
    for (var i = 0; i < s.length; i++) {
      if (i > 0 && (s.length - i) % 3 == 0) buf.write(',');
      buf.write(s[i]);
    }
    return buf.toString();
  }
}
