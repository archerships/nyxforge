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
