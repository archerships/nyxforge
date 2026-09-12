# Waku -- Research Brief

> April 2026
> Status: Research (background, not authoritative)
> See also: `doc/03_RESEARCH/nym-brief.md`
>           `doc/03_RESEARCH/genosdb-brief.md`
>           `doc/05_TECH/oracle-spec.md`

---

## 1. Who Is Building It

Waku is developed by the Waku team within the Logos collective, which also
includes Status (the encrypted messaging app) and Codex (decentralized
storage). The Waku team is funded by the Logos treasury and operates as an
independent research and engineering group. Core researchers publish through
Vac (Verifiable Autonomous Computation), the Logos research arm.

- Website: https://waku.org
- Documentation: https://docs.waku.org
- GitHub: https://github.com/waku-org
- Research: https://vac.dev
- Language: Nim (nwaku, the reference node), Go (go-waku), JavaScript
  (js-waku for browsers and Node.js)
- License: MIT / Apache 2.0
- Status (as of 2025-2026): Production-ready base layer; RLN spam protection
  approaching mainnet; mix protocol (sender anonymity) in proof-of-concept

Waku was originally the messaging layer inside Status, extracted and
generalized into a standalone protocol. It is the primary P2P messaging
layer for several Web3 applications including Status, The Graph, and
WalletConnect push notifications.

---

## 2. What Waku Is

Waku is a family of modular P2P communication protocols designed for
decentralized applications that need censorship-resistant, privacy-preserving
messaging without running their own infrastructure. It operates as a shared
service network -- any application can publish to or subscribe from the
network without deploying dedicated nodes, though running a node contributes
to decentralization.

Core design goals:
- Platform agnostic: runs in browsers, mobile devices, desktop, and servers
- Resource-constrained friendly: dedicated protocols for offline and
  low-bandwidth clients
- Censorship resistant: no central server; peer discovery via multiple
  mechanisms
- Privacy preserving: separation of routing and application layers; sender
  anonymity via mix protocol (in development)
- Spam resistant: Rate Limiting Nullifier (RLN) relay approaching mainnet

---

## 3. Protocol Stack

Waku is layered on top of libp2p, adding application-level protocols for
messaging, storage, and filtering that libp2p does not provide.

### 3.1 Transport

WebSocket (browser) and TCP (native nodes). Browser clients connect to the
Waku network via WebSocket connections to relay nodes -- they do not establish
direct WebRTC connections to other browser peers, so IP addresses of browser
clients are not exposed to other application peers (only to the relay nodes
they connect through).

### 3.2 Core Protocols

Relay (GossipSub)
: The base publish-subscribe protocol. Messages are gossiped across a mesh
  of relay nodes using content topics (named channels). The sender's identity
  is not revealed to subscribers. This is the foundation for all other
  protocols.

Store
: Allows resource-constrained or offline clients to retrieve missed messages
  from archive nodes when they come back online. Critical for oracle nodes
  that may be intermittently connected.

Filter
: A light-client protocol that lets a browser client subscribe to specific
  content topics without receiving all relay traffic. The filter node forwards
  only matching messages to the light client.

Light Push
: Allows a browser client to publish messages without running a full relay
  node, by pushing through a service node. Reduces browser resource
  requirements.

### 3.3 Spam Protection: RLN Relay

Rate Limiting Nullifiers (RLN) is a ZK-based spam prevention mechanism. Each
node registers a stake on-chain and receives a credential. Per epoch (e.g.
per second), the credential allows sending up to N messages. Exceeding the
rate limit reveals the node's private key, enabling automatic stake slashing.

RLN uses ZK-SNARKs so the credential can be verified without revealing the
node's identity. This gives anonymous spam protection: the network knows the
rate limit was not exceeded, but not who sent the message. RLN was approaching
mainnet deployment as of mid-2025.

### 3.4 Sender Anonymity: Mix Protocol (in development)

Waku's base relay does not provide strong sender anonymity -- a relay node
that sees your messages can infer that you are the sender (or close to the
origin). The Waku team published a specification in early 2025 for integrating
libp2p-mix into Waku, which adds onion-routed message delivery:

1. The sender wraps the message in multiple layers of encryption (one per
   hop) -- similar to Tor's onion routing
2. The message traverses a configured number of mix nodes, each stripping
   one layer
3. The final mix node delivers the message to the relay network
4. Observers cannot link the sender to the message

A proof-of-concept implementation was in progress as of early 2026. This is
not yet production-ready but represents the direction for strong sender
anonymity in Waku.

### 3.5 Peer Discovery

Waku uses three complementary peer discovery mechanisms:
- Peer exchange: fast but Sybil-vulnerable (bootstrapping only)
- Rendezvous protocol: register and find peers by content topic
- Discv5: decentralized, Sybil-resistant, slower (steady-state)

---

## 4. Browser Support (js-waku)

The `@waku/sdk` npm package provides full Waku integration for browsers and
Node.js. As of 2025, js-waku supports:
- WebSocket transport to relay nodes (no direct P2P between browser tabs)
- Light Push for publishing messages
- Filter for subscribing to content topics
- Store for retrieving historical messages
- Peer exchange for discovery

Browser clients are light clients -- they connect to relay nodes rather than
participating in the relay mesh directly. This means browser peers do not
expose their IP addresses to other application-layer peers; only the relay
nodes they connect to see their IP.

Installation: `npm install @waku/sdk`

Basic usage pattern:
```javascript
import { createLightNode, waitForRemotePeer } from "@waku/sdk";

const node = await createLightNode({ defaultBootstrap: true });
await waitForRemotePeer(node);

// Subscribe to a content topic
const decoder = createDecoder("/nyxforge/bounty-attestation/1/proto");
await node.filter.subscribe([decoder], (msg) => {
  console.log("Received attestation:", msg.payload);
});

// Publish a message
const encoder = createEncoder({ contentTopic: "/nyxforge/bounty-attestation/1/proto" });
await node.lightPush.send(encoder, { payload: attestationBytes });
```

---

## 5. Anonymity Analysis

| Property | Waku (base relay) | Waku + Mix (future) |
| :------- | :---------------- | :------------------ |
| Browser IP hidden from app peers | Y | Y |
| Browser IP hidden from relay nodes | N | P (onion routing) |
| Message sender unlinkable | P (relay mesh dilution) | Y |
| Message content hidden | Y (encrypted payloads) | Y |
| Traffic pattern analysis resistant | N | Y |
| Spam protection without identity | P (RLN approaching mainnet) | Y |

Current state: Waku's base relay is pseudonymous, not anonymous. Browser
clients' IPs are hidden from other application peers (because all traffic
goes through relay nodes) but are visible to the relay nodes they connect
through. A malicious or subpoenaed relay node can identify which IP published
which message.

Future state with mix protocol: sender anonymity becomes strong -- relay nodes
cannot link a message to its originating IP.

---

## 6. NyxForge Integration

### 6.1 Role in the Architecture

Waku is the best current-state option for the NyxForge P2P messaging layer
for non-critical-path messages where some latency is acceptable and strong
sender anonymity is desired. Primary candidates:

- Bounty attestation delivery: judges submit TLS-Notary proofs and signed
  verdicts via a Waku content topic watched by the resolution engine
- Oracle coordination: oracle nodes coordinate on which data sources to
  fetch and share proof results
- Dispute notifications: backers are notified when a dispute is opened
  against a bounty they hold
- NGO campaign announcements: new bounty campaigns broadcast to subscribers

### 6.2 Content Topic Design

Waku uses named content topics as channels. Proposed NyxForge topic structure:

```
/nyxforge/bounty-attestation/1/proto    -- judge attestation submissions
/nyxforge/bounty-dispute/1/proto        -- dispute openings and responses
/nyxforge/campaign-announce/1/proto   -- new NGO campaign broadcasts
/nyxforge/oracle-coord/1/proto        -- oracle node coordination
```

Topics are public -- anyone can subscribe. Message payloads should be
encrypted with the bounty's public key so only authorized parties can read them.

### 6.3 Example: Oracle Attestation Submission

After a judge generates a TLS-Notary proof for the Alzheimer's bounty:

1. Judge's oracle client constructs an attestation message:
   - Bounty ID: alzheimer-cure-2045
   - TLS-Notary proof: (compressed JSON, ~15 KB)
   - Judge signature (Ed25519)
   - Timestamp

2. Message is encrypted with the bounty's resolution public key and published
   via Light Push to `/nyxforge/bounty-attestation/1/proto`

3. The NyxForge resolution engine (subscribing via Filter) receives the
   message and queues it for verification

4. The judge's IP is not visible to the resolution engine -- only to the
   Waku relay nodes that forwarded the message

5. Once 2-of-3 matching attestations arrive, the resolution engine triggers
   the PTLC unlock

### 6.4 Example: Dispute Notification to Backers

When a disputer challenges a bounty verdict:

1. The dispute contract (or resolution relayer) publishes a dispute message
   to `/nyxforge/bounty-dispute/1/proto`

2. The message includes the bounty ID, disputer's bounty stake amount, and the
   dispute claim (plain text)

3. All backers subscribed to that content topic receive the notification
   via their js-waku Filter subscription in the web UI

4. Backers can choose to participate in the dispute (via UMA or Reality.eth)
   or wait for automatic resolution

---

## 7. Limitations

| Limitation | Detail |
| :--------- | :------ |
| Relay node IP visibility | Base relay nodes see the IPs of clients connecting to them. Mitigated by mix protocol (not yet production). |
| No built-in payload encryption | Message payload encryption must be implemented at the application layer. Waku encrypts transport but not content. |
| Light client dependency on relay nodes | Browser clients depend on relay nodes for message delivery. If all relay nodes censor a content topic, light clients cannot publish. |
| Mix protocol not yet mainnet | Strong sender anonymity requires the mix protocol, which is in proof-of-concept stage as of early 2026. |
| Message size limits | Waku messages have a default max size of 1 MB. TLS-Notary proofs (10-100 KB) fit comfortably; larger evidence packages may need to be split or stored externally (Arweave) with a hash in the message. |
| No persistent identity | Waku does not provide a built-in identity layer. Application-level key management is required for judge and oracle node signing keys. |

---

## 8. Reference Links

- Waku website: https://waku.org
- js-waku SDK: https://docs.waku.org/guides/js-waku
- Protocol overview: https://docs.waku.org/learn/concepts/protocols
- RLN Relay explainer: https://blog.waku.org/explanation-series-rln-relay/
- Privacy and anonymity analysis: https://vac.dev/rlog/wakuv2-relay-anon/
- Mix protocol spec (raw): https://forum.research.logos.co/t/mix-implementation-in-javascript-and-js-waku/563
- GitHub: https://github.com/waku-org
