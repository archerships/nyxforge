import 'package:flutter/material.dart';
import 'theme.dart';

class CommunityScreen extends StatelessWidget {
  const CommunityScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;

    return Padding(
      padding: const EdgeInsets.all(32),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('Community', style: tt.displaySmall),
          const SizedBox(height: 8),
          Text('Encrypted P2P Forums via Cwtch', style: tt.bodyMedium),
          const SizedBox(height: 48),
          
          const Expanded(
            child: Center(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(Icons.forum_outlined, size: 64, color: NyxColors.textMuted),
                  SizedBox(height: 16),
                  Text('Cwtch FFI Bridge Initializing', 
                      style: TextStyle(color: NyxColors.textSecondary, fontSize: 16)),
                  SizedBox(height: 8),
                  Text('P2P networking requires libcwtch-go',
                      style: TextStyle(color: NyxColors.textMuted, fontSize: 13)),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
