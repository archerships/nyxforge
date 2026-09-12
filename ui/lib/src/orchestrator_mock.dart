import 'dart:async';
import 'dart:math';
import 'package:flutter/material.dart';

/// Unified Mock Orchestrator for NyxForge Hub.
/// Provides simulated data for CakeWallet, Cwtch, BasicSwap, and XMRig.
class OrchestratorMock extends ChangeNotifier {
  OrchestratorMock._() {
    _startSimulation();
  }
  static final instance = OrchestratorMock._();

  final _random = Random();

  // -- Wallet State (CakeWallet) --
  double xmrBalance = 4.2183;
  double btcBalance = 0.0125;
  double drkBalance = 150.0;
  String xmrAddress = "44AFFq5kSiGBo3SxpCbcWjLxXvG2W5HnS78L49564";

  // -- Mining State (XMRig) --
  bool   isMining = true;
  double hashrate = 12.4; // KH/s
  int    shares   = 1024;
  int    threads  = 8;

  // -- Exchange State (BasicSwap) --
  int pendingSwaps = 1;
  List<String> activePairs = ["XMR/BTC", "XMR/ETH"];

  // -- Community State (Cwtch) --
  int unreadMessages = 3;
  bool isCwtchOnline = true;

  void _startSimulation() {
    Timer.periodic(const Duration(seconds: 5), (timer) {
      // Simulate hashrate fluctuation
      if (isMining) {
        hashrate += (_random.nextDouble() - 0.5) * 0.5;
        if (hashrate < 0) hashrate = 0;
        if (_random.nextDouble() > 0.8) shares++;
      }

      // Randomly change balance slightly
      if (_random.nextDouble() > 0.95) {
        xmrBalance += 0.0001;
      }

      notifyListeners();
    });
  }

  void toggleMining() {
    isMining = !isMining;
    if (!isMining) hashrate = 0;
    else hashrate = 12.4;
    notifyListeners();
  }
}
