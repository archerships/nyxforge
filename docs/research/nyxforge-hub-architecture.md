# NyxForge Hub: Unified Privacy Super-App Architecture

**Status:** [ACTIVE RESEARCH: 2026-04-20]
**Objective:** Design a single, cohesive user interface (UI) that orchestrates multiple underlying sovereign privacy tools (CakeWallet, Cwtch, BasicSwap, XMRig) alongside the native NyxForge `.bounty` exchange.

---

## 1. Executive Summary
The **NyxForge Hub** is envisioned as a "Sovereign Desktop/Mobile Environment." Instead of forcing users to juggle multiple applications with varying UX standards, the Hub acts as a master orchestrator. It runs the core logic of established privacy tools as headless background daemons and renders their functionality through a unified, high-polish Flutter frontend.

This approach ensures zero context-switching for the user while maintaining the robust security guarantees of the underlying, battle-tested protocols.

---

## 2. Architectural Paradigm: The Orchestrator Pattern

The application is split into two distinct layers:
1.  **The Daemon Layer (Backend):** The raw, unmodified (or lightly wrapped) binaries and libraries of the integrated services.
2.  **The Presentation Layer (Frontend):** A unified Flutter application that communicates with the daemons via local APIs (JSON-RPC, FFI, gRPC) and state streams.

### Component Integration Map

| Underlying Service | NyxForge Hub Function | Integration Method | Core Tech |
| :--- | :--- | :--- | :--- |
| **CakeWallet Core** | Multi-currency wallet (XMR, BTC, ZEC); Fiat on-ramps. | **Dart Library Import** | Flutter/Dart |
| **Cwtch** | Encrypted P2P forums & customer support. | **FFI (Foreign Function Interface)** | `libcwtch-go` -> Dart |
| **XMRig / Gupaxx** | Background mining (XMR/Tari) for protocol security/yield. | **Process Management (Subprocess)** | C++ / JSON-RPC |
| **BasicSwap** | Decentralized DEX for atomic swaps (Fiat <-> XMR). | **Headless Daemon** | Python / JSON-RPC |
| **NyxForge Core** | Management and storage of bearer `.bounty` assets. | **Native SDK (Rust/WASM)** | AO/Tari SDK -> Dart |
| **NyxForge Exchange**| Auction and orderbook UI for `.bounty` assets. | **Native Logic + RPC** | Custom UI over AO/BasicSwap |

---

## 3. Technical Specification (Draft)

### 3.1. Frontend Framework
*   **Primary Tech:** **Flutter** (Targeting Desktop: macOS, Windows, Linux; Mobile: iOS, Android).
*   **Why Flutter?** Both CakeWallet and Cwtch already utilize Flutter, allowing for significant code reuse (especially cryptographic dart libraries and UI widgets). It provides native-like performance for complex state management across multiple daemons.

### 3.2. State Management & IPC (Inter-Process Communication)
The Hub must maintain the state of multiple independent networks simultaneously without freezing the UI.
*   **Isolates:** Heavy cryptographic operations (key derivation, signing) will run in separate Dart Isolates.
*   **Service Connectors:** The architecture relies on "Connector" classes that abstract the IPC details:
    *   `BasicSwapConnector`: Wraps HTTP POST requests to `127.0.0.1:12701/json_rpc` for atomic swap data.
    *   `MiningConnector`: Wraps HTTP GET requests to XMRig's local API (e.g., `127.0.0.1:19999/1/summary`) to render real-time hashrate graphs.
    *   `CwtchConnector`: Uses Dart FFI (`dart:ffi`) to call functions exported by `libcwtch.so`/`libcwtch.dll` (e.g., `CwtchStart()`, `SendMessage()`).

### 3.3. Unified Master Seed (BIP-39/Polyseed)
To achieve true UX unification, the user must only backup *one* mnemonic phrase.
*   **Implementation:** The Hub generates a primary seed (e.g., 24 words).
*   **Derivation:** This master seed deterministically derives the keys for:
    1.  The Monero Wallet (via CakeWallet's subaddress derivation).
    2.  The Cwtch Identity (Ed25519 keys).
    3.  The NyxForge AO/Tari Account.
    4.  The BasicSwap trading node.

### 3.4. The Universal UI/UX Design System
The Hub will utilize a strict design system (e.g., "Nyx UI") to ensure visual consistency regardless of the underlying service.
*   **Shared Components:** A custom library of buttons, inputs, modal dialogs, and data tables.
*   **Theming:** Dark-mode native, utilizing typography and spacing suited for high-density financial data.
*   **Navigation:** A permanent sidebar (Desktop) or bottom nav (Mobile) grouping tools by function:
    *   `[Wallet]` -> CakeWallet + NyxForge Asset Viewer.
    *   `[Exchange]` -> BasicSwap UI + NyxForge `.bounty` Orderbook.
    *   `[Mine]` -> XMRig control panel and profitability metrics.
    *   `[Community]` -> Cwtch-powered forums and direct support lines.

---

## 4. Implementation Phasing Strategy

### Phase 1: Core Wallet & Communication (The Foundation)
*   Integrate CakeWallet core libraries for XMR/BTC basic send/receive functionality.
*   Implement Dart FFI bindings for `libcwtch-go` to establish secure P2P messaging.
*   *Milestone:* A secure chat app that can natively send Monero between contacts.

### Phase 2: The NyxForge Asset Layer
*   Integrate the native AO/Tari SDK to parse and manage `.bounty` files.
*   Build the custom UI for viewing bounty metadata (maturity, policy target, yield).
*   *Milestone:* The Hub can securely hold and display anonymous policy bounties alongside XMR.

### Phase 3: The Exchange & Mining (The Sovereign Economy)
*   Bundle the BasicSwap python daemon and build the JSON-RPC connector for atomic swaps.
*   Bundle XMRig and build the process manager to toggle background mining.
*   *Milestone:* Users can acquire XMR natively via DEX, mine it, and trade it for `.bounty` assets, all without leaving the application.

---

## 5. Security & Risk Considerations
*   **Daemon Sandboxing:** Running multiple complex daemons (Python, Go, C++) increases the attack surface. Operating system-level sandboxing (e.g., macOS App Sandbox, Flatpak) must restrict each daemon's access to the file system.
*   **Memory Management (FFI):** Extreme care must be taken with Dart FFI memory allocation when interfacing with Go (`libcwtch`) to prevent memory leaks or segmentation faults.
*   **Binary Signing:** All bundled binaries (XMRig, BasicSwap) must be deterministically compiled and cryptographically verified upon Hub startup to prevent supply-chain attacks.
