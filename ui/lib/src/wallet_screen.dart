import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'theme.dart';
import 'node_client.dart';

class WalletScreen extends StatefulWidget {
  const WalletScreen({super.key});

  @override
  State<WalletScreen> createState() => _WalletScreenState();
}

class _WalletScreenState extends State<WalletScreen>
    with SingleTickerProviderStateMixin {
  final _client = NodeClient();
  late final TabController _tabs;

  WalletAddresses? _addresses;
  WalletBalance? _balance;
  bool _loading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _tabs = TabController(length: 3, vsync: this);
    _load();
  }

  @override
  void dispose() {
    _tabs.dispose();
    _client.dispose();
    super.dispose();
  }

  Future<void> _load() async {
    setState(() { _loading = true; _error = null; });
    try {
      final addr = await _client.walletAddresses();
      final bal  = await _client.walletBalances();
      if (mounted) setState(() { _addresses = addr; _balance = bal; _loading = false; });
    } on NodeException catch (e) {
      if (mounted) setState(() { _error = e.message; _loading = false; });
    }
  }

  Future<void> _createWallet() async {
    setState(() { _loading = true; _error = null; });
    try {
      await _client.walletCreate(passphrase: '');
      await _load();
    } on NodeException catch (e) {
      if (mounted) setState(() { _error = e.message; _loading = false; });
    }
  }

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;

    if (_loading) {
      return const Center(child: CircularProgressIndicator());
    }

    // No wallet yet — show create prompt.
    if (_error != null && _addresses == null) {
      return _NoWalletPane(onCreate: _createWallet);
    }

    final addr = _addresses!;
    final bal  = _balance ?? const WalletBalance(xmrConfirmed: 0, xmrUnconfirmed: 0, drk: 0);

    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Header
          Text('Wallet', style: tt.titleLarge),
          const SizedBox(height: 4),
          Text('XMR light wallet + DRK notes', style: tt.bodyMedium),
          const SizedBox(height: 20),

          // Address row
          _AddressRow(label: 'XMR', address: addr.xmr),
          const SizedBox(height: 8),
          _AddressRow(label: 'DRK', address: addr.drk),
          const SizedBox(height: 20),

          // Balance cards
          Row(
            children: [
              Expanded(child: _BalanceCard(
                label: 'Monero',
                confirmed: bal.xmrConfirmedDisplay,
                pending: bal.xmrUnconfirmedDisplay,
                unit: 'XMR',
              )),
              const SizedBox(width: 12),
              Expanded(child: _BalanceCard(
                label: 'DarkFi',
                confirmed: bal.drkDisplay,
                unit: 'DRK',
              )),
            ],
          ),
          const SizedBox(height: 20),

          // Tab bar
          TabBar(
            controller: _tabs,
            tabs: const [
              Tab(text: 'RECEIVE'),
              Tab(text: 'SEND'),
              Tab(text: 'HISTORY'),
            ],
          ),
          const Divider(height: 1),

          // Tab views
          Expanded(
            child: TabBarView(
              controller: _tabs,
              children: [
                _ReceiveTab(xmrAddress: addr.xmr, drkAddress: addr.drk),
                _SendTab(client: _client, onSent: _load),
                const _HistoryTab(),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Sub-widgets
// ---------------------------------------------------------------------------

class _NoWalletPane extends StatelessWidget {
  const _NoWalletPane({required this.onCreate});
  final VoidCallback onCreate;

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.account_balance_wallet_outlined,
              size: 48, color: NyxColors.textMuted),
          const SizedBox(height: 16),
          Text('No wallet yet', style: tt.titleLarge),
          const SizedBox(height: 8),
          Text('Create a wallet to receive XMR from mining\nand DRK from bond redemptions.',
              style: tt.bodyMedium, textAlign: TextAlign.center),
          const SizedBox(height: 24),
          ElevatedButton.icon(
            onPressed: onCreate,
            icon:  const Icon(Icons.add, size: 16),
            label: const Text('CREATE WALLET'),
          ),
        ],
      ),
    );
  }
}

class _AddressRow extends StatelessWidget {
  const _AddressRow({required this.label, required this.address});
  final String label;
  final String address;

  @override
  Widget build(BuildContext context) {
    final short = address.length > 20
        ? '${address.substring(0, 10)}…${address.substring(address.length - 10)}'
        : address;
    return Row(
      children: [
        Container(
          width: 40,
          padding: const EdgeInsets.symmetric(vertical: 2),
          alignment: Alignment.center,
          child: Text(label,
              style: const TextStyle(
                  color: NyxColors.accentBright,
                  fontSize: 11,
                  fontWeight: FontWeight.w700)),
        ),
        const SizedBox(width: 8),
        Expanded(
          child: Text(short,
              style: const TextStyle(
                  color: NyxColors.textSecondary,
                  fontSize: 12,
                  fontFamily: 'monospace')),
        ),
        IconButton(
          icon: const Icon(Icons.copy, size: 14, color: NyxColors.textMuted),
          onPressed: () {
            Clipboard.setData(ClipboardData(text: address));
            ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(content: Text('$label address copied')));
          },
          tooltip: 'Copy $label address',
          padding: EdgeInsets.zero,
          constraints: const BoxConstraints(),
        ),
      ],
    );
  }
}

class _BalanceCard extends StatelessWidget {
  const _BalanceCard({
    required this.label,
    required this.confirmed,
    this.pending,
    required this.unit,
  });
  final String label;
  final String confirmed;
  final String? pending;
  final String unit;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(label,
                style: const TextStyle(
                    color: NyxColors.textMuted, fontSize: 11, letterSpacing: 1)),
            const SizedBox(height: 8),
            RichText(
              text: TextSpan(
                style: const TextStyle(fontFamily: 'monospace'),
                children: [
                  TextSpan(
                      text: confirmed,
                      style: const TextStyle(
                          color: NyxColors.textPrimary,
                          fontSize: 18,
                          fontWeight: FontWeight.w500)),
                  TextSpan(
                      text: ' $unit',
                      style: const TextStyle(
                          color: NyxColors.textMuted, fontSize: 12)),
                ],
              ),
            ),
            if (pending != null) ...[
              const SizedBox(height: 4),
              Text('+ $pending pending',
                  style: const TextStyle(
                      color: NyxColors.warning, fontSize: 11)),
            ],
          ],
        ),
      ),
    );
  }
}

class _ReceiveTab extends StatelessWidget {
  const _ReceiveTab({required this.xmrAddress, required this.drkAddress});
  final String xmrAddress;
  final String drkAddress;

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;
    return SingleChildScrollView(
      padding: const EdgeInsets.symmetric(vertical: 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('XMR address', style: tt.bodyMedium),
          const SizedBox(height: 8),
          SelectableText(xmrAddress,
              style: const TextStyle(
                  color: NyxColors.textPrimary,
                  fontSize: 12,
                  fontFamily: 'monospace')),
          const SizedBox(height: 20),
          Text('DRK address', style: tt.bodyMedium),
          const SizedBox(height: 8),
          SelectableText(drkAddress,
              style: const TextStyle(
                  color: NyxColors.textPrimary,
                  fontSize: 12,
                  fontFamily: 'monospace')),
        ],
      ),
    );
  }
}

class _SendTab extends StatefulWidget {
  const _SendTab({required this.client, required this.onSent});
  final NodeClient client;
  final VoidCallback onSent;

  @override
  State<_SendTab> createState() => _SendTabState();
}

class _SendTabState extends State<_SendTab> {
  final _toCtrl     = TextEditingController();
  final _amountCtrl = TextEditingController();
  bool _sending = false;
  String? _err;

  @override
  void dispose() {
    _toCtrl.dispose();
    _amountCtrl.dispose();
    super.dispose();
  }

  Future<void> _send() async {
    setState(() { _sending = true; _err = null; });
    try {
      final hash = await widget.client.sendXmr(
          _toCtrl.text.trim(), _amountCtrl.text.trim());
      if (mounted) {
        ScaffoldMessenger.of(context)
            .showSnackBar(SnackBar(content: Text('Sent — tx: $hash')));
        _toCtrl.clear();
        _amountCtrl.clear();
        widget.onSent();
      }
    } on NodeException catch (e) {
      if (mounted) setState(() { _err = e.message; });
    } finally {
      if (mounted) setState(() { _sending = false; });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          TextField(
            controller: _toCtrl,
            decoration: const InputDecoration(
              labelText: 'Recipient XMR address',
              hintText: '4...',
            ),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _amountCtrl,
            keyboardType: const TextInputType.numberWithOptions(decimal: true),
            decoration: const InputDecoration(
              labelText: 'Amount (XMR)',
              hintText: '0.001',
            ),
          ),
          if (_err != null) ...[
            const SizedBox(height: 8),
            Text(_err!, style: const TextStyle(color: NyxColors.danger, fontSize: 12)),
          ],
          const SizedBox(height: 20),
          ElevatedButton(
            onPressed: _sending ? null : _send,
            child: _sending
                ? const SizedBox(width: 16, height: 16,
                    child: CircularProgressIndicator(strokeWidth: 2))
                : const Text('SEND XMR'),
          ),
        ],
      ),
    );
  }
}

class _HistoryTab extends StatelessWidget {
  const _HistoryTab();

  @override
  Widget build(BuildContext context) {
    return const Center(
      child: Text('Transaction history coming soon',
          style: TextStyle(color: NyxColors.textMuted)),
    );
  }
}
