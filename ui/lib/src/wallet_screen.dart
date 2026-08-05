import 'package:flutter/material.dart';
import 'theme.dart';
import 'orchestrator_mock.dart';

class WalletScreen extends StatefulWidget {
  const WalletScreen({super.key});

  @override
  State<WalletScreen> createState() => _WalletScreenState();
}

class _WalletScreenState extends State<WalletScreen> {
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
          Text('Wallet', style: tt.displaySmall),
          const SizedBox(height: 8),
          Text('Integrated CakeWallet Core (XMR/BTC)', style: tt.bodyMedium),
          const SizedBox(height: 32),
          
          _BalanceCard(
            currency: 'MONERO',
            symbol: 'XMR',
            balance: _mock.xmrBalance,
            address: _mock.xmrAddress,
            color: const Color(0xFFF26822), // XMR Orange
          ),
          
          const SizedBox(height: 24),
          _BalanceCard(
            currency: 'BITCOIN',
            symbol: 'BTC',
            balance: _mock.btcBalance,
            address: 'bc1qxy2kgdyvjrsqvdm...asdfg',
            color: const Color(0xFFF7931A), // BTC Gold
          ),
          
          const Spacer(),
          Row(
            children: [
              Expanded(
                child: ElevatedButton.icon(
                  onPressed: () {},
                  icon: const Icon(Icons.call_made),
                  label: const Text('SEND'),
                ),
              ),
              const SizedBox(width: 16),
              Expanded(
                child: OutlinedButton.icon(
                  onPressed: () {},
                  icon: const Icon(Icons.call_received),
                  label: const Text('RECEIVE'),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _BalanceCard extends StatelessWidget {
  const _BalanceCard({
    required this.currency,
    required this.symbol,
    required this.balance,
    required this.address,
    required this.color,
  });

  final String currency;
  final String symbol;
  final double balance;
  final String address;
  final Color  color;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Container(
                  width: 8,
                  height: 8,
                  decoration: BoxDecoration(color: color, shape: BoxShape.circle),
                ),
                const SizedBox(width: 8),
                Text(currency, style: const TextStyle(color: NyxColors.textMuted, fontSize: 11, fontWeight: FontWeight.bold, letterSpacing: 1.2)),
              ],
            ),
            const SizedBox(height: 16),
            Text('${balance.toStringAsFixed(4)} $symbol', style: const TextStyle(fontSize: 28, fontWeight: FontWeight.w600)),
            const SizedBox(height: 4),
            Text('≈ \$${(balance * (symbol == "XMR" ? 175 : 65000)).toStringAsFixed(2)} USD', 
                style: const TextStyle(color: NyxColors.textSecondary)),
            const Divider(height: 40),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Expanded(child: Text(address, style: const TextStyle(color: NyxColors.textMuted, fontSize: 12, overflow: TextOverflow.ellipsis))),
                const Icon(Icons.copy, size: 14, color: NyxColors.textMuted),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
