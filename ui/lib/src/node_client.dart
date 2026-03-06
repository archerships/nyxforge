import 'dart:convert';
import 'package:http/http.dart' as http;

/// JSON-RPC client for the local nyxforge-node binary.
/// The node runs on localhost:8888 and bridges Flutter to the P2P network.
class NodeClient {
  NodeClient({this.baseUrl = 'http://127.0.0.1:8888/rpc'});

  final String baseUrl;
  final _client = http.Client();

  /// Call a JSON-RPC method and return the `result` field.
  /// Throws [NodeException] on transport or RPC errors.
  Future<dynamic> call(String method, [Map<String, dynamic>? params]) async {
    final body = jsonEncode({'method': method, 'params': params ?? {}});

    final http.Response resp;
    try {
      resp = await _client
          .post(
            Uri.parse(baseUrl),
            headers: {'Content-Type': 'application/json'},
            body: body,
          )
          .timeout(const Duration(seconds: 10));
    } catch (e) {
      throw NodeException('Could not reach nyxforge-node at $baseUrl\n'
          'Make sure the node is running: cargo run -p nyxforge-node\n'
          'Details: $e');
    }

    if (resp.statusCode != 200) {
      throw NodeException('HTTP ${resp.statusCode}: ${resp.body}');
    }

    final json = jsonDecode(resp.body) as Map<String, dynamic>;
    if (json.containsKey('error') && json['error'] != null) {
      throw NodeException('RPC error: ${json['error']}');
    }
    return json['result'];
  }

  // ---------------------------------------------------------------------------
  // Node
  // ---------------------------------------------------------------------------

  /// Fetch the node status (version, bond count, etc.).
  Future<NodeStatus> status() async {
    final result = await call('status') as Map<String, dynamic>;
    return NodeStatus(
      version:   result['version'] as String? ?? 'unknown',
      bondCount: result['bonds']   as int?    ?? 0,
    );
  }

  // ---------------------------------------------------------------------------
  // Wallet
  // ---------------------------------------------------------------------------

  /// Create a new wallet. Returns the new addresses.
  Future<WalletAddresses> walletCreate({String passphrase = ''}) async {
    final result = await call('wallet.create', {'passphrase': passphrase})
        as Map<String, dynamic>;
    return WalletAddresses(
      xmr: result['xmr_address'] as String? ?? '',
      drk: result['drk_address'] as String? ?? '',
    );
  }

  /// Fetch wallet addresses.
  Future<WalletAddresses> walletAddresses() async {
    final result = await call('wallet.addresses') as Map<String, dynamic>;
    return WalletAddresses(
      xmr: result['xmr'] as String? ?? '',
      drk: result['drk'] as String? ?? '',
    );
  }

  /// Fetch wallet balances.
  Future<WalletBalance> walletBalances() async {
    final result = await call('wallet.balances') as Map<String, dynamic>;
    return WalletBalance(
      xmrConfirmed:   (result['xmr_confirmed']   as num?)?.toInt() ?? 0,
      xmrUnconfirmed: (result['xmr_unconfirmed'] as num?)?.toInt() ?? 0,
      drk:            (result['drk']             as num?)?.toInt() ?? 0,
    );
  }

  /// Send XMR to an address. Returns the transaction hash.
  Future<String> sendXmr(String toAddress, String amountXmr) async {
    final result = await call('wallet.send_xmr', {
          'to': toAddress,
          'amount_xmr': amountXmr,
        }) as Map<String, dynamic>;
    return result['tx_hash'] as String? ?? '';
  }

  // ---------------------------------------------------------------------------
  // Miner
  // ---------------------------------------------------------------------------

  /// Fetch miner status (hashrate, shares, running).
  Future<MinerStatus> minerStatus() async {
    final result = await call('miner.status') as Map<String, dynamic>;
    return MinerStatus(
      running:        result['running']           as bool?  ?? false,
      hashrate:       (result['hashrate']         as num?)?.toDouble() ?? 0.0,
      sharesFound:    (result['shares']           as num?)?.toInt()    ?? 0,
      xmrPendingPico: (result['xmr_pending_pico'] as num?)?.toInt()   ?? 0,
    );
  }

  /// Start mining. Optionally override the CPU thread count.
  Future<void> minerStart({int? threads}) async {
    final params = <String, dynamic>{};
    if (threads != null) params['threads'] = threads;
    await call('miner.start', params);
  }

  /// Stop mining.
  Future<void> minerStop() async {
    await call('miner.stop');
  }

  /// Change the number of mining threads (takes effect immediately if running).
  Future<void> minerSetThreads(int count) async {
    await call('miner.set_threads', {'count': count});
  }

  // ---------------------------------------------------------------------------
  // Bonds
  // ---------------------------------------------------------------------------

  /// List all bond series.
  Future<List<BondSummary>> bondList() async {
    final result = await call('bonds.list') as Map<String, dynamic>;
    final bonds = result['bonds'] as List<dynamic>? ?? [];
    return bonds.map((b) => BondSummary.fromJson(b as Map<String, dynamic>)).toList();
  }

  /// Get the current Dutch auction price for a bond (in μDRK).
  Future<int> bondAuctionPrice(String bondIdHex) async {
    final result = await call('bonds.auction_price', {'bond_id': bondIdHex})
        as Map<String, dynamic>;
    return (result['price_micro_drk'] as num?)?.toInt() ?? 0;
  }

  /// Buy `quantity` bonds at the current auction price.
  Future<BuyResult> bondBuy(String bondIdHex, int quantity) async {
    final result = await call('bonds.buy', {
      'bond_id':  bondIdHex,
      'quantity': quantity,
    }) as Map<String, dynamic>;
    return BuyResult(
      purchased:     (result['purchased']       as num?)?.toInt() ?? 0,
      priceMicroDrk: (result['price_micro_drk'] as num?)?.toInt() ?? 0,
    );
  }

  /// Propose a new bond (publishes for community review).
  Future<String> bondPropose(Map<String, dynamic> bond) async {
    final result = await call('bonds.propose', {'bond': bond})
        as Map<String, dynamic>;
    return result['bond_id'] as String? ?? '';
  }

  /// Submit bond for oracle approval.
  Future<void> bondSubmitForApproval(String bondIdHex) async {
    await call('bonds.submit_for_approval', {'bond_id': bondIdHex});
  }

  /// Accept oracle responsibility (demo: oracle key is fixed).
  Future<void> bondOracleAccept(String bondIdHex, String oracleKeyHex) async {
    await call('bonds.oracle_accept', {
      'bond_id':    bondIdHex,
      'oracle_key': oracleKeyHex,
    });
  }

  /// Lock collateral and activate a Draft bond.
  Future<void> bondIssue(String bondIdHex) async {
    await call('bonds.issue', {'bond_id': bondIdHex});
  }

  void dispose() => _client.close();
}

// ---------------------------------------------------------------------------
// Model classes
// ---------------------------------------------------------------------------

class NodeStatus {
  const NodeStatus({required this.version, required this.bondCount});
  final String version;
  final int bondCount;
}

class WalletAddresses {
  const WalletAddresses({required this.xmr, required this.drk});
  final String xmr;
  final String drk;
}

class WalletBalance {
  const WalletBalance({
    required this.xmrConfirmed,
    required this.xmrUnconfirmed,
    required this.drk,
  });

  /// Picomonero (1 XMR = 1e12 pico).
  final int xmrConfirmed;
  final int xmrUnconfirmed;

  /// μDRK (1 DRK = 1e6 μDRK).
  final int drk;

  String get xmrConfirmedDisplay =>
      (xmrConfirmed / 1e12).toStringAsFixed(6);

  String get xmrUnconfirmedDisplay =>
      (xmrUnconfirmed / 1e12).toStringAsFixed(6);

  String get drkDisplay =>
      (drk / 1e6).toStringAsFixed(6);
}

class MinerStatus {
  const MinerStatus({
    required this.running,
    required this.hashrate,
    required this.sharesFound,
    required this.xmrPendingPico,
  });

  final bool   running;
  final double hashrate;        // H/s, 60-second rolling average
  final int    sharesFound;
  final int    xmrPendingPico;  // picomonero
}

class NodeException implements Exception {
  const NodeException(this.message);
  final String message;
  @override
  String toString() => 'NodeException: $message';
}

// ---------------------------------------------------------------------------
// Bond models
// ---------------------------------------------------------------------------

/// Helper: converts a Rust [u8; 32] (JSON array of ints) to a 64-char hex string.
String bytesToHex(dynamic raw) {
  if (raw == null) return '';
  if (raw is String) return raw;          // already hex
  final list = (raw as List).cast<int>();
  return list.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
}

String _microDrkToDisplay(int micro) =>
    (micro / 1e6).toStringAsFixed(micro % 1000000 == 0 ? 0 : 4);

class AuctionInfo {
  const AuctionInfo({
    required this.startPriceMicro,
    required this.reservePriceMicro,
    required this.durationSecs,
  });

  final int startPriceMicro;
  final int reservePriceMicro;
  final int durationSecs;

  String get startDisplay   => _microDrkToDisplay(startPriceMicro);
  String get reserveDisplay => _microDrkToDisplay(reservePriceMicro);
  String get durationDays   => (durationSecs ~/ 86400).toString();

  factory AuctionInfo.fromJson(Map<String, dynamic> j) => AuctionInfo(
    startPriceMicro:   (j['start_price']   as num?)?.toInt() ?? 0,
    reservePriceMicro: (j['reserve_price'] as num?)?.toInt() ?? 0,
    durationSecs:      (j['duration_secs'] as num?)?.toInt() ?? 0,
  );
}

class GoalInfo {
  const GoalInfo({
    required this.title,
    required this.description,
    required this.dataId,
    required this.operator,
    required this.threshold,
    required this.deadline,
  });

  final String title;
  final String description;
  final String dataId;
  final String operator;
  final String threshold;
  final String deadline;

  factory GoalInfo.fromJson(Map<String, dynamic> j) => GoalInfo(
    title:       j['title']       as String? ?? '',
    description: j['description'] as String? ?? '',
    dataId:      (j['metric'] as Map<String, dynamic>?)?['data_id'] as String? ?? '',
    operator:    (j['metric'] as Map<String, dynamic>?)?['operator'] as String? ?? '',
    threshold:   (j['metric'] as Map<String, dynamic>?)?['threshold'] as String? ?? '',
    deadline:    j['deadline']    as String? ?? '',
  );

  String get deadlineShort => deadline.length >= 10 ? deadline.substring(0, 10) : deadline;
}

class BondSummary {
  const BondSummary({
    required this.id,
    required this.state,
    required this.totalSupply,
    required this.bondsRemaining,
    required this.redemptionMicroDrk,
    required this.auction,
    required this.goals,
    required this.createdAtBlock,
    this.activatedAtSecs,
  });

  final String         id;               // 64-char hex
  final String         state;
  final int            totalSupply;
  final int            bondsRemaining;
  final int            redemptionMicroDrk;
  final AuctionInfo    auction;
  final List<GoalInfo> goals;
  final int            createdAtBlock;
  final int?           activatedAtSecs;

  /// The first goal — used as the card/list headline.
  GoalInfo get primaryGoal => goals.isNotEmpty ? goals.first : const GoalInfo(
    title: '', description: '', dataId: '', operator: '', threshold: '', deadline: '');

  String get redemptionDisplay => _microDrkToDisplay(redemptionMicroDrk);

  factory BondSummary.fromJson(Map<String, dynamic> j) => BondSummary(
    id:                 bytesToHex(j['id']),
    state:              j['state']        as String? ?? 'Unknown',
    totalSupply:        (j['total_supply']   as num?)?.toInt() ?? 0,
    bondsRemaining:     (j['bonds_remaining'] as num?)?.toInt() ?? 0,
    redemptionMicroDrk: (j['redemption_value'] as num?)?.toInt() ?? 0,
    auction:            AuctionInfo.fromJson(
                            j['auction'] as Map<String, dynamic>? ?? {}),
    goals:              (j['goals'] as List<dynamic>? ?? [])
                            .map((g) => GoalInfo.fromJson(g as Map<String, dynamic>))
                            .toList(),
    createdAtBlock:     (j['created_at_block'] as num?)?.toInt() ?? 0,
    activatedAtSecs:    (j['activated_at_secs'] as num?)?.toInt(),
  );
}

class BuyResult {
  const BuyResult({required this.purchased, required this.priceMicroDrk});
  final int purchased;
  final int priceMicroDrk;
  String get priceDisplay => _microDrkToDisplay(priceMicroDrk);
}
