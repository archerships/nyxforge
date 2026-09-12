import 'package:flutter/material.dart';
import '../theme.dart';

/// A high-fidelity Bond Card component for the Bearer Bond MVP.
class BondCard extends StatelessWidget {
  const BondCard({
    super.key,
    required this.title,
    required this.state,
    required this.amount,
    required this.currency,
    required this.deadline,
    required this.progress,
    required this.oracleQuorum,
    required this.oracleTotal,
    this.onTap,
    this.onAction,
  });

  final String title;
  final String state;
  final double amount;
  final String currency;
  final DateTime deadline;
  final double progress; // 0.0 to 1.0
  final int oracleQuorum;
  final int oracleTotal;
  final VoidCallback? onTap;
  final VoidCallback? onAction;

  Color get _stateColor {
    switch (state.toUpperCase()) {
      case 'ACTIVE':
        return NyxColors.primary;
      case 'REDEEMABLE':
      case 'SETTLED':
        return NyxColors.success;
      case 'DRAFT':
        return NyxColors.warning;
      case 'EXPIRED':
      case 'RECLAIMED':
        return NyxColors.danger;
      default:
        return NyxColors.textMuted;
    }
  }

  @override
  Widget build(BuildContext context) {
    final daysRemaining = deadline.difference(DateTime.now()).inDays;
    final isRedeemable = state.toUpperCase() == 'REDEEMABLE';

    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: onTap,
        child: Container(
          margin: const EdgeInsets.only(bottom: 16),
          decoration: BoxDecoration(
            color: NyxColors.surface,
            borderRadius: BorderRadius.circular(12),
            border: Border.all(
              color: isRedeemable ? NyxColors.success : NyxColors.border,
              width: isRedeemable ? 2 : 1,
            ),
            boxShadow: [
              if (isRedeemable)
                BoxShadow(
                  color: NyxColors.success.withOpacity(0.1),
                  blurRadius: 12,
                  spreadRadius: 2,
                ),
            ],
          ),
          child: Padding(
            padding: const EdgeInsets.all(20),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Header: State and Menu
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                      decoration: BoxDecoration(
                        color: _stateColor.withOpacity(0.1),
                        borderRadius: BorderRadius.circular(4),
                        border: Border.all(color: _stateColor.withOpacity(0.4)),
                      ),
                      child: Text(
                        state.toUpperCase(),
                        style: TextStyle(
                          color: _stateColor,
                          fontSize: 11,
                          fontWeight: FontWeight.bold,
                          letterSpacing: 1,
                        ),
                      ),
                    ),
                    const Icon(Icons.more_vert, color: NyxColors.textMuted, size: 20),
                  ],
                ),
                const SizedBox(height: 16),

                // Title
                Text(
                  title,
                  style: const TextStyle(
                    color: NyxColors.textPrimary,
                    fontSize: 18,
                    fontWeight: FontWeight.w600,
                  ),
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                ),
                const SizedBox(height: 8),
                const Divider(color: NyxColors.border),
                const SizedBox(height: 12),

                // Primary Stats
                Row(
                  children: [
                    _StatBlock(
                      label: 'AMOUNT',
                      value: '${amount.toStringAsFixed(1)} ${currency.toUpperCase()}',
                    ),
                    const Spacer(),
                    _StatBlock(
                      label: 'DEADLINE',
                      value: '${deadline.year}-${deadline.month.toString().padLeft(2, '0')}-${deadline.day.toString().padLeft(2, '0')}',
                    ),
                  ],
                ),
                const SizedBox(height: 16),

                // Progress Bar (for quantitative)
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Text(
                      'GOAL PROGRESS: ${(progress * 100).toInt()}%',
                      style: const TextStyle(color: NyxColors.textSecondary, fontSize: 11),
                    ),
                    Text(
                      '${daysRemaining}d remaining',
                      style: const TextStyle(color: NyxColors.textMuted, fontSize: 11),
                    ),
                  ],
                ),
                const SizedBox(height: 8),
                ClipRRect(
                  borderRadius: BorderRadius.circular(4),
                  child: LinearProgressIndicator(
                    value: progress,
                    backgroundColor: NyxColors.secondary,
                    color: _stateColor,
                    minHeight: 8,
                  ),
                ),
                const SizedBox(height: 16),

                // Footer: Oracles and Action
                Row(
                  children: [
                    const Text(
                      'ORACLES:',
                      style: TextStyle(color: NyxColors.textMuted, fontSize: 10, fontWeight: FontWeight.bold),
                    ),
                    const SizedBox(width: 8),
                    for (int i = 0; i < oracleTotal; i++)
                      Padding(
                        padding: const EdgeInsets.only(right: 4),
                        child: Icon(
                          Icons.circle,
                          size: 10,
                          color: i < oracleQuorum ? NyxColors.success : NyxColors.textMuted,
                        ),
                      ),
                    Text(
                      ' ($oracleQuorum/$oracleTotal Quorum)',
                      style: const TextStyle(color: NyxColors.textMuted, fontSize: 10),
                    ),
                    const Spacer(),
                    if (isRedeemable)
                      ElevatedButton(
                        onPressed: onAction,
                        style: ElevatedButton.styleFrom(
                          backgroundColor: NyxColors.success,
                          foregroundColor: Colors.white,
                          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
                          textStyle: const TextStyle(fontSize: 12, fontWeight: FontWeight.bold),
                        ),
                        child: const Text('REDEEM NOW'),
                      ),
                  ],
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _StatBlock extends StatelessWidget {
  const _StatBlock({required this.label, required this.value});
  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
          style: const TextStyle(color: NyxColors.textMuted, fontSize: 10, fontWeight: FontWeight.bold, letterSpacing: 1),
        ),
        const SizedBox(height: 4),
        Text(
          value,
          style: const TextStyle(color: NyxColors.textPrimary, fontSize: 14, fontWeight: FontWeight.w500),
        ),
      ],
    );
  }
}
