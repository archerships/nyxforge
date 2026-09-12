# GenosDB -- Research Brief

> April 2026
> Status: Research (background, not authoritative)
> See also: `doc/03_RESEARCH/oracle-network-comparison.md`
>           `doc/05_TECH/architecture.md`

---

## 1. Who Is Building It

GenosDB (GDB) is developed by Esteban Fernandez Perez (estebanrfp), an
independent open-source developer. It is a solo-led project with community
contributions, not backed by a VC-funded company or foundation.

- Website: https://genosdb.com
- GitHub: https://github.com/estebanrfp/gdb
- npm: https://www.npmjs.com/package/genosdb
- Language: Pure JavaScript (browser and Node.js)
- License: MIT
- Status (as of early 2026): Active development; production-ready for
  non-privacy-critical use cases

GenosDB was built explicitly as a reaction to GunDB's limitations --
specifically GunDB's lack of structured conflict resolution, weak security
model, and poor scalability beyond small peer counts. It draws on lessons
from GunDB, OrbitDB, and Automerge.

---

## 2. What GenosDB Is

GenosDB is a distributed, real-time, peer-to-peer graph database that runs
entirely in the browser or Node.js with no central server required. Data is
replicated across peers using WebRTC for transport, CRDT-based conflict
resolution for consistency, and OPFS (Origin Private File System) for local
persistence.

The key design goals:
- Zero server dependency: works offline and syncs when peers reconnect
- Zero-trust security: every operation is cryptographically signed and
  verified; no peer is trusted by default
- Scalability: cellular mesh topology reduces connection complexity from
  O(N^2) to O(N) for large peer sets
- Developer ergonomics: MongoDB-style queries, reactive data subscriptions,
  single-file HTML deployments possible

---

## 3. Technical Architecture

### 3.1 Data Model

GenosDB uses a graph structure with nodes and edges. Nodes hold arbitrary
JSON data; edges express typed relationships between nodes. Queries use a
MongoDB-style API with support for sorting, pagination, and recursive multi-
hop graph traversal via the `$edge` operator.

### 3.2 Storage

Data is persisted locally using the browser's Origin Private File System
(OPFS), giving near-native disk performance for offline storage. All data is
serialized with MessagePack (binary, compact) and compressed with Pako
(Deflate). Storage I/O runs in a dedicated Web Worker to avoid blocking the
main thread. Where OPFS is unavailable (older browsers), it falls back to
IndexedDB.

### 3.3 Networking (GenosRTC)

Peer connections use WebRTC for data transport, with Nostr relays handling
the signaling handshake (exchanging SDP offers/answers). GenosDB ships with
a built-in list of stable Nostr relays that is kept updated.

Named data channels support JSON, strings, and binary payloads. The protocol
also supports direct audio/video streaming between peers and cross-tab sync
via BroadcastChannel within the same browser session.

For networks of 100+ peers, the optional Cellular Mesh overlay organizes
peers into logical "cells" with designated bridge nodes, cutting connection
complexity from O(N^2) to O(N) while maintaining efficient message
propagation.

### 3.4 Conflict Resolution

Conflict resolution uses Hybrid Logical Clocks (HLC), combining physical
wall-clock time with logical counters. The merge strategy is Last-Write-Wins
(LWW), with safeguards against clock manipulation attacks. This gives
eventual consistency across peers without a central coordinator.

### 3.5 Security Model

GenosDB's security is built on a Zero-Trust Security Manager (SM), activated
via `{ sm: true }` at initialization. The SM inspects every incoming
operation and rejects anything not backed by a valid cryptographic signature.

Identity: every peer is identified by an Ethereum address backed by a
private key. Private keys can be protected by WebAuthn (biometrics, hardware
keys) or a mnemonic phrase. Every action is signed with the peer's key,
giving authenticity (the message came from the claimed sender) and integrity
(the message was not altered in transit).

RBAC: three role tiers are defined -- superadmin (hard-coded constitutional
authority), manager (delegated operational permissions), and guest (minimal
access). The SM validates operations against its own rulebook server-side;
client-side role claims are ignored.

Two-tiered verification: the SM performs a constitutional check (against an
immutable hard-coded superadmin list) and a public record check (against
verifiably granted roles in the database). This prevents privilege escalation
even under network partitions.

---

## 4. Anonymity and Privacy Analysis

This is the most important section for NyxForge evaluation.

### 4.1 Peer identity: pseudonymous, not anonymous

Every GenosDB peer is identified by an Ethereum address. This is pseudonymous
-- no real name is attached -- but it is a persistent, linkable identifier.
A peer signing with the same key across sessions is recognizable as the same
entity. There is no built-in mechanism for unlinkable or ephemeral identity.

Using a fresh keypair per session breaks persistent linkability but loses any
accumulated role grants or reputation, and does not hide IP addresses.

### 4.2 IP address exposure

WebRTC connections expose peers' IP addresses during the ICE negotiation
phase. Both the local IP and the public IP (obtained via STUN) are included
in the SDP offer exchanged through the Nostr signaling relay. This means
peers can see each other's IP addresses during connection establishment
unless additional measures are taken.

Mitigations available but not built in:
- Routing WebRTC through a TURN relay hides the true IP but introduces a
  trusted relay operator
- Routing signaling traffic through Tor hides the IP at the signaling stage
  but WebRTC itself will still reveal the local network IP unless WebRTC
  IP leak prevention is configured in the browser
- Using a VPN introduces a trusted third party

### 4.3 Data privacy

Data stored in GenosDB nodes is not encrypted at rest by default. End-to-end
encryption is available via password-based initialization (`{ password: "..." }`),
which encrypts the data before it is stored or transmitted. However this is
opt-in and the encryption model is not documented in detail -- it is unclear
whether it uses authenticated encryption or what the key derivation scheme is.

There is no zero-knowledge layer. Peers who are granted read access see the
actual data, not a proof about the data.

### 4.4 Metadata privacy

Even with data encryption enabled, the graph structure is visible to peers:
which nodes exist, how many edges they have, when they were last updated, and
which peer keys wrote them. For a bond system this leaks significant
information: who is transacting with whom, when, and at what frequency.

### 4.5 Summary

| Property | GenosDB default | With mitigations |
| :------- | :-------------- | :--------------- |
| Peer identity unlinkable | N | Partial (fresh keypair per session) |
| IP addresses hidden | N | Partial (VPN / TURN relay) |
| Data encrypted at rest | N | Y (opt-in password init) |
| Data encrypted in transit | Y (WebRTC DTLS) | Y |
| Graph metadata hidden | N | N |
| ZK proofs / selective disclosure | N | N |

---

## 5. Current Use Cases

GenosDB is used primarily in browser-native collaborative applications that
need real-time sync without a server:

- Collaborative whiteboards and editors (built in a single HTML file)
- Multiplayer browser games with shared state
- Offline-first web apps replacing Firebase with no backend
- P2P audio/video applications (direct WebRTC streams)
- Decentralized chat and messaging apps using Nostr for peer discovery

These use cases share a profile: real-time, short-lived sessions, low
sensitivity data, developer-friendly. They are not privacy-critical
applications.

---

## 6. Comparison to Alternatives

| Database | Identity model | IP privacy | ZK support | Anonymity fit |
| :------- | :------------- | :--------- | :--------- | :------------ |
| GenosDB  | Ethereum address (persistent) | None built-in | None | Poor |
| GunDB    | Public key (persistent) | None built-in | None | Poor |
| OrbitDB  | IPFS peer ID (persistent) | None built-in | None | Poor |
| Automerge | No identity | None | None | No auth at all |
| DarkFi   | Stealth address (anonymous) | Tor-native | Full ZK | Excellent |
| Nostr (P2P layer only) | Public key (pseudonymous) | None built-in | None | Poor |

GenosDB is the most developer-friendly of the browser-native P2P databases
but is the weakest on privacy of any option considered for NyxForge.

---

## 7. Fit Analysis for NyxForge

### Potential roles

GenosDB could serve as a lightweight P2P sync layer for non-sensitive bond
metadata in a public-facing NyxForge deployment where privacy is traded for
developer convenience. Specific candidates:

- Syncing the bond campaign public listing (title, terms hash, progress,
  deadline) across NGO admin nodes
- P2P real-time backer count updates on the landing page
- Collaborative editing of draft bond terms before publication

### Hard blockers for privacy-sensitive use

GenosDB must not be used for:

- Syncing backer identity data or return address information
- Transmitting or storing .bond files or PTLC parameters
- Any data that links a specific donor to a specific bond
- Oracle attestation records

### Recommended verdict

GenosDB is not a fit for NyxForge's core data layer given the anonymity and
privacy requirements. It could be considered for a narrow, explicitly public-
facing metadata layer (campaign listings, aggregate funding progress) where
no privacy is expected and developer speed is the priority.

For the P2P node layer of the full NyxForge architecture, DarkFi's anonymous
peer model is the correct long-run target. For a nearer-term anonymous P2P
layer, libp2p over Tor or I2P is more appropriate than GenosDB.

---

## 8. Reference Links

- GenosDB website: https://genosdb.com
- GitHub: https://github.com/estebanrfp/gdb
- Technical features: https://genosdb.com/technical-features-of-genosdb-gdb-307fe8cc6618
- Cellular mesh design: https://genosdb.com/genosdb-cellular-mesh-solving-p2p-network-scalability-0b3ddd0874e9
- Distributed trust model: https://github.com/estebanrfp/gdb/wiki/GenosDB-Distributed-Trust-Model
- P2P protocol architecture: https://dev.to/estebanrfp/designing-a-next-generation-p2p-protocol-architecture-for-genosdb-525j
- Nostr signaling integration: https://genosdb.com/genosdb-nostr-decentralized-data
- P2P database comparison (2026): https://genosdb.com/popular-p2p-distributed-databases
