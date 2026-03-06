import 'package:flutter/material.dart';
import 'theme.dart';
import 'node_client.dart';

// ---------------------------------------------------------------------------
// Fixed demo oracle key (matches ORACLE_KEY_A = [0x22u8; 32])
// ---------------------------------------------------------------------------
const _demoOracleKeyHex =
    '2222222222222222222222222222222222222222222222222222222222222222';

// ---------------------------------------------------------------------------
// Per-criterion state
// ---------------------------------------------------------------------------

class _CriterionState {
  _CriterionState({
    String title    = '',
    String desc     = '',
    String dataId   = 'us.hud.pit_count.unsheltered',
    String operator = 'LessThan',
    String threshold = '50000',
    String deadline = '2030-01-01',
  })  : titleCtrl    = TextEditingController(text: title),
        descCtrl     = TextEditingController(text: desc),
        dataIdCtrl   = TextEditingController(text: dataId),
        threshCtrl   = TextEditingController(text: threshold),
        deadlineCtrl = TextEditingController(text: deadline),
        operatorValue = operator;

  final TextEditingController titleCtrl;
  final TextEditingController descCtrl;
  final TextEditingController dataIdCtrl;
  final TextEditingController threshCtrl;
  final TextEditingController deadlineCtrl;
  String operatorValue;

  bool get isValid =>
      titleCtrl.text.trim().isNotEmpty &&
      threshCtrl.text.trim().isNotEmpty &&
      deadlineCtrl.text.trim().isNotEmpty;

  Map<String, dynamic> toJson() => {
    'title':       titleCtrl.text.trim(),
    'description': descCtrl.text.trim(),
    'metric': {
      'data_id':     dataIdCtrl.text.trim(),
      'operator':    operatorValue,
      'threshold':   threshCtrl.text.trim(),
      'aggregation': null,
    },
    'evidence_format': null,
    'deadline': '${deadlineCtrl.text.trim()}T00:00:00Z',
  };

  void dispose() {
    titleCtrl.dispose();
    descCtrl.dispose();
    dataIdCtrl.dispose();
    threshCtrl.dispose();
    deadlineCtrl.dispose();
  }
}

// ---------------------------------------------------------------------------
// Screen
// ---------------------------------------------------------------------------

class IssueBondScreen extends StatefulWidget {
  const IssueBondScreen({super.key});

  @override
  State<IssueBondScreen> createState() => _IssueBondScreenState();
}

class _IssueBondScreenState extends State<IssueBondScreen> {
  final _client = NodeClient();

  // Step 0 = goal, 1 = economics, 2 = oracle, 3 = review
  int _step = 0;

  // ── Criteria (one or more; AND semantics) ────────────────────────
  final List<_CriterionState> _criteria = [_CriterionState()];

  // ── Economics fields ─────────────────────────────────────────────
  final _supplyCtrl    = TextEditingController(text: '1000');
  final _redemCtrl     = TextEditingController(text: '10');
  final _startPCtrl    = TextEditingController(text: '1');
  final _reservePCtrl  = TextEditingController(text: '1');
  final _durationCtrl  = TextEditingController(text: '7');

  // ── Oracle fields ─────────────────────────────────────────────────
  final _oracleKeyCtrl = TextEditingController(text: _demoOracleKeyHex);
  final _stakeCtrl     = TextEditingController(text: '100');

  // ── Submission state ──────────────────────────────────────────────
  bool    _submitting = false;
  String? _error;
  String? _bondIdHex;   // set on success

  // ── Navigation ───────────────────────────────────────────────────

  bool _canNext() {
    switch (_step) {
      case 0:
        return _criteria.isNotEmpty && _criteria.every((c) => c.isValid);
      case 1:
        final supply = int.tryParse(_supplyCtrl.text) ?? 0;
        final redem  = int.tryParse(_redemCtrl.text)  ?? 0;
        final startP = int.tryParse(_startPCtrl.text) ?? 0;
        final resP   = int.tryParse(_reservePCtrl.text) ?? 0;
        final dur    = int.tryParse(_durationCtrl.text) ?? 0;
        return supply > 0 && redem >= 0 && startP > 0 && resP >= 0 &&
               resP <= startP && dur > 0;
      case 2:
        return _oracleKeyCtrl.text.trim().length == 64;
      default:
        return true;
    }
  }

  void _next() {
    if (_step < 3) {
      setState(() { _step++; _error = null; });
    } else {
      _submit();
    }
  }

  void _back() {
    if (_step > 0) setState(() { _step--; _error = null; });
  }

  // ── Bond construction ─────────────────────────────────────────────

  /// Placeholder ID — the node recomputes the canonical blake3 ID server-side
  /// in bonds.propose, so this value is overwritten before storage.
  List<int> _fakeBondId() => List<int>.filled(32, 0);

  Map<String, dynamic> _buildBond() {
    final issuer     = List<int>.filled(32, 0x11);
    final oracleKey  = _hexToBytes(_oracleKeyCtrl.text.trim());
    final supply     = int.parse(_supplyCtrl.text);
    final redem      = int.parse(_redemCtrl.text) * 1000000;      // whole → μDRK
    final startP     = int.parse(_startPCtrl.text) * 1000000;
    final reserveP   = int.parse(_reservePCtrl.text) * 1000000;
    final durSecs    = int.parse(_durationCtrl.text) * 86400;
    final stake      = int.parse(_stakeCtrl.text) * 1000000;

    return {
      'id':              _fakeBondId(),
      'issuer':          issuer,
      'total_supply':    supply,
      'redemption_value': redem,
      'auction': {
        'start_price':   startP,
        'reserve_price': reserveP,
        'duration_secs': durSecs,
      },
      'bonds_remaining':   supply,
      'activated_at_secs': null,
      'state':             'Draft',
      'goals': _criteria.map((c) => c.toJson()).toList(),
      'oracle': {
        'quorum':          1,
        'oracle_keys':     [oracleKey],
        'required_stake':  stake,
        'slash_fraction':  '0.5',
      },
      'verification': {
        'attestation_threshold': 1,
        'challenge_period_secs': 86400,
        'dao_override_allowed':  false,
      },
      'created_at_block': 0,
      'return_address':   issuer,
    };
  }

  List<int> _hexToBytes(String hex) =>
      List.generate(hex.length ~/ 2, (i) => int.parse(hex.substring(i*2, i*2+2), radix: 16));

  // ── Submission ────────────────────────────────────────────────────

  Future<void> _submit() async {
    setState(() { _submitting = true; _error = null; });
    try {
      final bond = _buildBond();

      // propose → submit → oracle accept → issue
      final bondId = await _client.bondPropose(bond);
      await _client.bondSubmitForApproval(bondId);
      await _client.bondOracleAccept(bondId, _demoOracleKeyHex);
      await _client.bondIssue(bondId);

      if (mounted) setState(() { _submitting = false; _bondIdHex = bondId; });
    } on NodeException catch (e) {
      if (mounted) setState(() { _submitting = false; _error = e.message; });
    }
  }

  void _reset() {
    for (final c in _criteria) { c.dispose(); }
    setState(() {
      _step = 0;
      _bondIdHex = null;
      _error = null;
      _criteria.clear();
      _criteria.add(_CriterionState());
    });
  }

  // ── Build ─────────────────────────────────────────────────────────

  @override
  void dispose() {
    for (final c in _criteria) { c.dispose(); }
    for (final c in [_supplyCtrl, _redemCtrl, _startPCtrl,
                     _reservePCtrl, _durationCtrl, _oracleKeyCtrl, _stakeCtrl]) {
      c.dispose();
    }
    _client.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (_bondIdHex != null) return _SuccessView(bondId: _bondIdHex!, onIssueAnother: _reset);

    final tt = Theme.of(context).textTheme;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // ── Header ────────────────────────────────────────────────
        Padding(
          padding: const EdgeInsets.fromLTRB(24, 20, 24, 4),
          child: Text('Issue Bond', style: tt.titleLarge),
        ),
        Padding(
          padding: const EdgeInsets.fromLTRB(24, 0, 24, 0),
          child: Text('Define a social goal, set auction parameters, and go live.',
              style: tt.bodyMedium),
        ),

        // ── Step indicator ────────────────────────────────────────
        Padding(
          padding: const EdgeInsets.fromLTRB(24, 16, 24, 0),
          child: _StepIndicator(current: _step, labels: const ['Goal', 'Economics', 'Oracle', 'Review']),
        ),

        const Divider(height: 20),

        // ── Step body ─────────────────────────────────────────────
        Expanded(
          child: SingleChildScrollView(
            padding: const EdgeInsets.fromLTRB(24, 0, 24, 24),
            child: _buildStep(),
          ),
        ),

        // ── Error ─────────────────────────────────────────────────
        if (_error != null)
          Padding(
            padding: const EdgeInsets.fromLTRB(24, 0, 24, 8),
            child: Text(_error!,
                style: const TextStyle(color: NyxColors.danger, fontSize: 12)),
          ),

        // ── Navigation buttons ────────────────────────────────────
        Padding(
          padding: const EdgeInsets.fromLTRB(24, 0, 24, 20),
          child: Row(
            children: [
              if (_step > 0)
                OutlinedButton(
                  onPressed: _submitting ? null : _back,
                  child: const Text('BACK'),
                ),
              const Spacer(),
              ElevatedButton(
                onPressed: (_canNext() && !_submitting) ? _next : null,
                child: _submitting
                    ? const SizedBox(width: 18, height: 18,
                        child: CircularProgressIndicator(strokeWidth: 2))
                    : Text(_step == 3 ? 'ISSUE BOND' : 'NEXT'),
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildStep() {
    switch (_step) {
      case 0: return _GoalStep(
          criteria: _criteria,
          onChanged: () => setState(() {}),
          onAddCriterion: () => setState(() => _criteria.add(_CriterionState())),
          onRemoveCriterion: (i) => setState(() {
            _criteria[i].dispose();
            _criteria.removeAt(i);
          }),
        );
      case 1: return _EconomicsStep(
          supplyCtrl: _supplyCtrl, redemCtrl: _redemCtrl,
          startPCtrl: _startPCtrl, reservePCtrl: _reservePCtrl,
          durationCtrl: _durationCtrl,
          onChanged: () => setState(() {}));
      case 2: return _OracleStep(
          oracleKeyCtrl: _oracleKeyCtrl, stakeCtrl: _stakeCtrl,
          onChanged: () => setState(() {}));
      case 3: return _ReviewStep(
          criteria: _criteria,
          supply: _supplyCtrl.text, redemption: _redemCtrl.text,
          startPrice: _startPCtrl.text, reservePrice: _reservePCtrl.text,
          duration: _durationCtrl.text,
          oracleKey: _oracleKeyCtrl.text);
      default: return const SizedBox.shrink();
    }
  }
}

// ---------------------------------------------------------------------------
// Step pages
// ---------------------------------------------------------------------------

class _GoalStep extends StatelessWidget {
  const _GoalStep({
    required this.criteria,
    required this.onChanged,
    required this.onAddCriterion,
    required this.onRemoveCriterion,
  });

  final List<_CriterionState> criteria;
  final VoidCallback           onChanged;
  final VoidCallback           onAddCriterion;
  final ValueChanged<int>      onRemoveCriterion;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        for (final (i, c) in criteria.indexed) ...[
          if (criteria.length > 1) ...[
            const SizedBox(height: 8),
            Row(
              children: [
                Text(
                  'Criterion ${i + 1}',
                  style: const TextStyle(
                    color: NyxColors.accentBright,
                    fontSize: 12,
                    fontWeight: FontWeight.w700,
                    letterSpacing: 1.2,
                  ),
                ),
                const Spacer(),
                if (criteria.length > 1)
                  IconButton(
                    icon: const Icon(Icons.remove_circle_outline,
                        size: 18, color: NyxColors.danger),
                    tooltip: 'Remove criterion',
                    onPressed: () => onRemoveCriterion(i),
                    padding: EdgeInsets.zero,
                    constraints: const BoxConstraints(),
                  ),
              ],
            ),
          ] else
            _Section('Social Goal'),
          _Field('Criterion title', c.titleCtrl,
              hint: 'e.g. Global malaria deaths below 100k by 2040',
              onChanged: onChanged),
          _Field('Description (optional)', c.descCtrl,
              hint: 'Measurement methodology and context',
              maxLines: 3, onChanged: onChanged),
          _Section('Metric'),
          _Field('Data source ID', c.dataIdCtrl,
              hint: 'e.g. who.malaria.annual_deaths', onChanged: onChanged),
          const SizedBox(height: 12),
          _OperatorPicker(
            value: c.operatorValue,
            onChanged: (v) { c.operatorValue = v; onChanged(); },
          ),
          const SizedBox(height: 12),
          _Field('Threshold value', c.threshCtrl,
              hint: 'e.g. 100000',
              keyboardType: TextInputType.number, onChanged: onChanged),
          _Section('Deadline'),
          _Field('Deadline (YYYY-MM-DD)', c.deadlineCtrl,
              hint: '2040-01-01', onChanged: onChanged),
          if (i < criteria.length - 1) const Divider(height: 24),
        ],
        const SizedBox(height: 12),
        OutlinedButton.icon(
          onPressed: onAddCriterion,
          icon: const Icon(Icons.add, size: 16),
          label: const Text('ADD CRITERION'),
        ),
      ],
    );
  }
}

class _EconomicsStep extends StatelessWidget {
  const _EconomicsStep({
    required this.supplyCtrl, required this.redemCtrl,
    required this.startPCtrl, required this.reservePCtrl,
    required this.durationCtrl, required this.onChanged,
  });

  final TextEditingController supplyCtrl, redemCtrl, startPCtrl, reservePCtrl, durationCtrl;
  final VoidCallback onChanged;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _Section('Supply & Redemption'),
        _Field('Total bond supply', supplyCtrl, hint: '10000', keyboardType: TextInputType.number, suffix: 'bonds', onChanged: onChanged),
        _Field('Redemption value per bond', redemCtrl, hint: '10', keyboardType: TextInputType.number, suffix: 'DRK', onChanged: onChanged),
        _Section('Dutch Auction'),
        const Text(
          'Price starts at the starting price and falls linearly to the reserve over the auction window.',
          style: TextStyle(color: NyxColors.textMuted, fontSize: 12),
        ),
        const SizedBox(height: 12),
        _Field('Starting price', startPCtrl, hint: '5', keyboardType: TextInputType.number, suffix: 'DRK', onChanged: onChanged),
        _Field('Reserve (floor) price', reservePCtrl, hint: '1', keyboardType: TextInputType.number, suffix: 'DRK', onChanged: onChanged),
        _Field('Auction duration', durationCtrl, hint: '7', keyboardType: TextInputType.number, suffix: 'days', onChanged: onChanged),
      ],
    );
  }
}

class _OracleStep extends StatelessWidget {
  const _OracleStep({required this.oracleKeyCtrl, required this.stakeCtrl, required this.onChanged});
  final TextEditingController oracleKeyCtrl, stakeCtrl;
  final VoidCallback onChanged;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _Section('Oracle Network'),
        const Text(
          'Oracles are independent nodes that verify the goal has been met. '
          'The demo oracle key is pre-filled.',
          style: TextStyle(color: NyxColors.textMuted, fontSize: 12),
        ),
        const SizedBox(height: 12),
        _Field('Oracle public key (hex)', oracleKeyCtrl,
            hint: '64 hex characters', onChanged: onChanged),
        _Field('Required oracle stake', stakeCtrl, hint: '100',
            keyboardType: TextInputType.number, suffix: 'DRK', onChanged: onChanged),
      ],
    );
  }
}

class _ReviewStep extends StatelessWidget {
  const _ReviewStep({
    required this.criteria,
    required this.supply, required this.redemption,
    required this.startPrice, required this.reservePrice, required this.duration,
    required this.oracleKey,
  });

  final List<_CriterionState> criteria;
  final String supply, redemption, startPrice, reservePrice, duration, oracleKey;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _Section('Goal'),
        for (final (i, c) in criteria.indexed) ...[
          if (criteria.length > 1)
            Padding(
              padding: const EdgeInsets.only(top: 4, bottom: 2),
              child: Text('Criterion ${i + 1}',
                  style: const TextStyle(color: NyxColors.textMuted, fontSize: 12)),
            ),
          _ReviewRow('Title',    c.titleCtrl.text),
          if (c.descCtrl.text.isNotEmpty)
            _ReviewRow('Description', c.descCtrl.text),
          _ReviewRow('Metric', '${c.dataIdCtrl.text}  ${c.operatorValue}  ${c.threshCtrl.text}'),
          _ReviewRow('Deadline', c.deadlineCtrl.text),
          if (i < criteria.length - 1) const SizedBox(height: 4),
        ],
        _Section('Economics'),
        _ReviewRow('Supply',      '$supply bonds'),
        _ReviewRow('Redemption',  '$redemption DRK per bond'),
        _ReviewRow('Start price', '$startPrice DRK'),
        _ReviewRow('Reserve',     '$reservePrice DRK'),
        _ReviewRow('Auction',     '$duration days'),
        _Section('Oracle'),
        _ReviewRow('Oracle key',  '${oracleKey.substring(0, 16)}…'),
        const SizedBox(height: 16),
        Container(
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            color: NyxColors.accentGlow,
            borderRadius: BorderRadius.circular(6),
            border: Border.all(color: NyxColors.accent),
          ),
          child: const Row(
            children: [
              Icon(Icons.info_outline, color: NyxColors.accentBright, size: 16),
              SizedBox(width: 8),
              Expanded(
                child: Text(
                  'In demo mode the bond goes live immediately (oracle auto-accepts, no ZK proof required).',
                  style: TextStyle(color: NyxColors.textSecondary, fontSize: 12),
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Success view
// ---------------------------------------------------------------------------

class _SuccessView extends StatelessWidget {
  const _SuccessView({required this.bondId, required this.onIssueAnother});
  final String bondId;
  final VoidCallback onIssueAnother;

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(40),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              width: 72, height: 72,
              decoration: BoxDecoration(
                color: NyxColors.success.withValues(alpha: 0.12),
                shape: BoxShape.circle,
                border: Border.all(color: NyxColors.success, width: 2),
              ),
              child: const Icon(Icons.check, color: NyxColors.success, size: 38),
            ),
            const SizedBox(height: 20),
            Text('Bond Live!', style: tt.titleLarge?.copyWith(color: NyxColors.success)),
            const SizedBox(height: 8),
            Text('Your bond is now active on the network.',
                style: tt.bodyMedium, textAlign: TextAlign.center),
            const SizedBox(height: 16),
            SelectableText(
              bondId,
              style: const TextStyle(
                color: NyxColors.textMuted, fontSize: 11,
                fontFamily: 'monospace', letterSpacing: 0.5,
              ),
            ),
            const SizedBox(height: 32),
            Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                OutlinedButton(
                  onPressed: onIssueAnother,
                  child: const Text('ISSUE ANOTHER'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Shared form widgets
// ---------------------------------------------------------------------------

class _Section extends StatelessWidget {
  const _Section(this.title);
  final String title;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(top: 20, bottom: 10),
      child: Text(title,
          style: const TextStyle(
            color: NyxColors.accentBright,
            fontSize: 12,
            fontWeight: FontWeight.w700,
            letterSpacing: 1.4,
          )),
    );
  }
}

class _Field extends StatelessWidget {
  const _Field(this.label, this.controller, {
    this.hint = '', this.maxLines = 1,
    this.keyboardType, this.suffix, required this.onChanged,
  });

  final String label;
  final TextEditingController controller;
  final String hint;
  final int    maxLines;
  final TextInputType? keyboardType;
  final String? suffix;
  final VoidCallback onChanged;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: TextField(
        controller:   controller,
        maxLines:     maxLines,
        keyboardType: keyboardType,
        onChanged:    (_) => onChanged(),
        decoration: InputDecoration(
          labelText: label,
          hintText:  hint,
          suffixText: suffix,
        ),
      ),
    );
  }
}

class _OperatorPicker extends StatelessWidget {
  const _OperatorPicker({required this.value, required this.onChanged});
  final String value;
  final ValueChanged<String> onChanged;

  static const _ops = [
    ('LessThan',           '< (less than)'),
    ('LessThanOrEqual',    '≤ (less than or equal)'),
    ('GreaterThan',        '> (greater than)'),
    ('GreaterThanOrEqual', '≥ (greater than or equal)'),
    ('Equal',              '= (equal)'),
  ];

  @override
  Widget build(BuildContext context) {
    return DropdownButtonFormField<String>(
      initialValue: value,
      decoration: const InputDecoration(labelText: 'Operator'),
      dropdownColor: NyxColors.surfaceHigh,
      items: _ops.map((op) => DropdownMenuItem(
        value: op.$1,
        child: Text(op.$2,
            style: const TextStyle(color: NyxColors.textPrimary, fontSize: 14)),
      )).toList(),
      onChanged: (v) { if (v != null) onChanged(v); },
    );
  }
}

class _ReviewRow extends StatelessWidget {
  const _ReviewRow(this.label, this.value);
  final String label, value;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 5),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 120,
            child: Text(label,
                style: const TextStyle(color: NyxColors.textMuted, fontSize: 13)),
          ),
          Expanded(
            child: Text(value,
                style: const TextStyle(color: NyxColors.textPrimary, fontSize: 13)),
          ),
        ],
      ),
    );
  }
}

class _StepIndicator extends StatelessWidget {
  const _StepIndicator({required this.current, required this.labels});
  final int current;
  final List<String> labels;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: List.generate(labels.length, (i) {
        final done    = i < current;
        final active  = i == current;
        final color   = done || active ? NyxColors.accentBright : NyxColors.textMuted;
        return Expanded(
          child: Row(
            children: [
              // Circle
              Container(
                width: 24, height: 24,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: active
                      ? NyxColors.accent
                      : done
                          ? NyxColors.success
                          : NyxColors.surfaceHigh,
                  border: Border.all(
                    color: active ? NyxColors.accentBright
                        : done ? NyxColors.success
                        : NyxColors.border,
                  ),
                ),
                child: Center(
                  child: done
                      ? const Icon(Icons.check, size: 13, color: Colors.white)
                      : Text('${i + 1}',
                          style: TextStyle(
                            color: color, fontSize: 11,
                            fontWeight: FontWeight.w600,
                          )),
                ),
              ),
              const SizedBox(width: 4),
              Expanded(
                child: Text(labels[i],
                    style: TextStyle(
                      color: color, fontSize: 11,
                      fontWeight: active ? FontWeight.w600 : FontWeight.normal,
                    ),
                    overflow: TextOverflow.ellipsis),
              ),
              // Connector line (not after last)
              if (i < labels.length - 1)
                Expanded(
                  child: Container(height: 1, color: NyxColors.border),
                ),
            ],
          ),
        );
      }),
    );
  }
}
