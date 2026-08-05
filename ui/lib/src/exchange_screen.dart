import 'package:flutter/material.dart';
import 'theme.dart';

class ExchangeScreen extends StatelessWidget {
  const ExchangeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;

    return Padding(
      padding: const EdgeInsets.all(32),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('Exchange', style: tt.displaySmall),
          const SizedBox(height: 8),
          Text('BasicSwap DEX + NyxForge Orderbook', style: tt.bodyMedium),
          const SizedBox(height: 48),
          
          const Center(
            child: Column(
              children: [
                Icon(Icons.swap_horizontal_circle_outlined, size: 64, color: NyxColors.textMuted),
                SizedBox(height: 16),
                Text('BasicSwap Daemon Integration Pending', 
                    style: TextStyle(color: NyxColors.textSecondary, fontSize: 16)),
                SizedBox(height: 8),
                Text('Ensure BasicSwap is installed and running on port 12701',
                    style: TextStyle(color: NyxColors.textMuted, fontSize: 13)),
              ],
            ),
          ),
          
          const Spacer(),
          SizedBox(
            width: double.infinity,
            child: OutlinedButton(
              onPressed: () {},
              child: const Text('MANUAL CONNECTION SETUP'),
            ),
          ),
        ],
      ),
    );
  }
}
