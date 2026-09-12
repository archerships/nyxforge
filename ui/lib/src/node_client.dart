import 'dart:convert';
import 'package:http/http.dart' as http;

/// JSON-RPC client for the local nyxforge-node binary (Bearer Bounty MVP).
class NodeClient {
  NodeClient({this.baseUrl = 'http://127.0.0.1:8888/rpc'});

  final String baseUrl;
  final _client = http.Client();

  Future<dynamic> call(String method, [Map<String, dynamic>? params]) async {
    final body = jsonEncode({'method': method, 'params': params ?? {}});
    try {
      final resp = await _client.post(
        Uri.parse(baseUrl),
        headers: {'Content-Type': 'application/json'},
        body: body,
      ).timeout(const Duration(seconds: 10));

      if (resp.statusCode != 200) {
        throw NodeException('HTTP ${resp.statusCode}: ${resp.body}');
      }

      final json = jsonDecode(resp.body) as Map<String, dynamic>;
      if (json.containsKey('error') && json['error'] != null) {
        throw NodeException('RPC error: ${json['error']}');
      }
      return json['result'];
    } catch (e) {
      throw NodeException('Connection error: $e');
    }
  }

  Future<NodeStatus> status() async {
    final result = await call('status') as Map<String, dynamic>;
    return NodeStatus(
      version: result['version'] ?? '0.1.0',
      bountyCount: result['bounties'] ?? 0,
    );
  }

  Future<List<Bounty>> bountyList() async {
    final result = await call('bounties.list') as Map<String, dynamic>;
    final bounties = result['bounties'] as List<dynamic>? ?? [];
    return bounties.map((b) => Bounty.fromJson(b as Map<String, dynamic>)).toList();
  }

  Future<Bounty> bountyGet(String file) async {
    final result = await call('bounties.get', {'file': file}) as Map<String, dynamic>;
    return Bounty.fromJson(result);
  }

  // ── Stubs for Wallet and Miner (to be implemented in Phase 5) ──

  Future<WalletAddresses> walletCreate({String passphrase = ''}) async =>
      const WalletAddresses(xmr: '5...dummy', drk: '0x...dummy');

  Future<WalletAddresses> walletAddresses() async => 
      const WalletAddresses(xmr: '5...dummy', drk: '0x...dummy');

  Future<WalletBalance> walletBalances() async =>
      const WalletBalance(xmrConfirmed: 1000000000000, xmrUnconfirmed: 0, drk: 5000000);

  Future<String> sendXmr(String to, String amount) async => 'dummy_tx_hash';

  Future<MinerStatus> minerStatus() async =>
      const MinerStatus(running: false, hashrate: 0.0, sharesFound: 0, xmrPendingPico: 0);

  Future<void> minerStart({int? threads}) async {}
  Future<void> minerStop() async {}
  Future<void> minerSetThreads(int n) async {}

  // ── Stubs for Bounty Lifecycle ──

  Future<String> bountyPropose(Map<String, dynamic> bounty) async => 'dummy_bounty_id';
  Future<void> bountySubmitForApproval(String id) async {}
  Future<void> bountyOracleAccept(String id, String key) async {}
  Future<void> bountyIssue(String id) async {}
  Future<int> bountyAuctionPrice(String id) async => 1000000;
  Future<dynamic> bountyBuy(String id, int qty) async => null;

  void dispose() => _client.close();
}

class NodeStatus {
  const NodeStatus({required this.version, required this.bountyCount});
  final String version;
  final int bountyCount;
}

class WalletAddresses {
  const WalletAddresses({required this.xmr, required this.drk});
  final String xmr;
  final String drk;
}

class WalletBalance {
  const WalletBalance({required this.xmrConfirmed, required this.xmrUnconfirmed, required this.drk});
  final int xmrConfirmed;
  final int xmrUnconfirmed;
  final int drk;

  String get xmrConfirmedDisplay => (xmrConfirmed / 1e12).toStringAsFixed(6);
  String get xmrUnconfirmedDisplay => (xmrUnconfirmed / 1e12).toStringAsFixed(6);
  String get drkDisplay => (drk / 1e6).toStringAsFixed(6);
}

class MinerStatus {
  const MinerStatus({required this.running, required this.hashrate, required this.sharesFound, required this.xmrPendingPico});
  final bool running;
  final double hashrate;
  final int sharesFound;
  final int xmrPendingPico;
}

class Bounty {
  Bounty({
    required this.file,
    required this.seriesId,
    required this.serial,
    required this.state,
    required this.currency,
    required this.amount,
    required this.title,
    required this.description,
    required this.deadline,
    required this.expiry,
    required this.oracleQuorum,
    required this.oracleTotal,
    this.progress = 0.0,
  });

  final String file;
  final String seriesId;
  final int serial;
  final String state;
  final String currency;
  final double amount;
  final String title;
  final String description;
  final DateTime deadline;
  final DateTime expiry;
  final int oracleQuorum;
  final int oracleTotal;
  final double progress;

  factory Bounty.fromJson(Map<String, dynamic> j) {
    return Bounty(
      file: j['file'] ?? '',
      seriesId: j['series_id'] ?? '',
      serial: j['serial'] ?? 0,
      state: j['state'] ?? 'DRAFT',
      currency: j['currency'] ?? 'xmr',
      amount: double.tryParse(j['amount']?.toString() ?? '0.0') ?? 0.0,
      title: j['title'] ?? '',
      description: j['description'] ?? '',
      deadline: DateTime.tryParse(j['deadline'] ?? '') ?? DateTime.now(),
      expiry: DateTime.tryParse(j['expiry'] ?? '') ?? DateTime.now(),
      oracleQuorum: j['quorum'] ?? 0,
      oracleTotal: (j['oracle_panel'] as List?)?.length ?? 0,
      progress: 0.42, // Mock progress for UI demo
    );
  }
}

class NodeException implements Exception {
  const NodeException(this.message);
  final String message;
  @override
  String toString() => 'NodeException: $message';
}
