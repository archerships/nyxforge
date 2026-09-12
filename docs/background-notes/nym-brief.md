# Nym Mixnet -- Research Brief

> April 2026
> Status: Research (background, not authoritative)
> See also: `doc/03_RESEARCH/waku-brief.md`
>           `doc/03_RESEARCH/genosdb-brief.md`
>           `doc/05_TECH/oracle-spec.md`

---

## 1. Who Is Building It

Nym Technologies SA is a Swiss company founded in 2018 by Harry Halpin
(cryptographer, previously W3C), Claudia Diaz (KU Leuven, leading mixnet
researcher), and Dave Hrycyszyn. The team is heavily research-oriented --
Claudia Diaz co-authored foundational papers on anonymous communication
systems, and the Nym protocol is grounded in academic mixnet literature
dating to David Chaum's original 1981 work.

- Website: https://nym.com
- Documentation: https://nym.com/docs/developers
- GitHub: https://github.com/nymtech
- npm (WASM): https://www.npmjs.com/package/@nymproject/nym-client-wasm
- Token: NYM (incentivizes mix node operators)
- License: Apache 2.0
- Status (as of 2026): Mainnet live; NymVPN in production; developer SDKs
  in active use; WASM browser client available

Nym has received funding from Andreessen Horowitz (a16z), Tikhon Bernstam,
and others. It is a for-profit company with a public token, unlike PSE/Waku
which are non-profit / foundation-backed.

---

## 2. What the Nym Mixnet Is

Nym is a network-level privacy infrastructure. Where Tor hides which server
you are connecting to, and end-to-end encryption hides message content, Nym
hides traffic metadata: who is communicating with whom, when, how frequently,
and how much data. This is the class of attack known as traffic analysis, and
it is the most powerful tool available to global passive adversaries (ISPs,
intelligence agencies, cloud infrastructure providers).

Nym achieves this using a mixnet: messages are encrypted in layers, routed
through a sequence of mix nodes, and held and released in randomized batches
(adding cover traffic) so that an observer watching the entire network cannot
determine which input packet corresponds to which output packet.

---

## 3. How the Mixnet Works

### 3.1 Packet Structure: Sphinx

All Nym messages are encapsulated in Sphinx packets, a fixed-size cryptographic
packet format designed for mixnets. Key properties:
- Fixed size: all packets look identical to network observers, regardless of
  payload size. Long messages are split into multiple packets; short ones are
  padded. This prevents size-based traffic correlation.
- Onion encrypted: each hop's routing information is encrypted for that hop
  only. A mix node knows where to forward a packet but not where it came from
  or its final destination.
- SURB (Single-Use Reply Block): allows replies to anonymous senders without
  the sender revealing their address.

### 3.2 The Three-Layer Mix

Messages traverse exactly three layers of mix nodes, selected pseudorandomly
from the mix node registry. Each layer:
1. Receives a Sphinx packet
2. Decrypts its outer layer to learn the next-hop address
3. Holds the packet for a random delay (drawn from an exponential distribution)
4. Forwards the packet mixed in with other packets from the same batch

The random delay is the key mechanism: it breaks timing correlations. An
observer who sees a packet enter layer 1 at time T cannot determine which
packet exits layer 3 at time T+X because the delay is unpredictable.

### 3.3 Cover Traffic

To prevent traffic analysis by observing when the network is idle, Nym mix
nodes continuously send cover (dummy) traffic. Real messages are
indistinguishable from cover traffic to a network observer.

### 3.4 Anonymity Modes

NymVPN (the consumer product) exposes two modes that clarify the tradeoff:

- Fast mode (2-hop): Lower latency, weaker anonymity. Suitable for browsing
  and streaming. Routes through 2 nodes instead of 3.
- Anonymous mode (5-hop NGM): Higher latency, strong anonymity. Routes through
  a 5-hop Network Gateway Mix topology. Best for messaging, financial
  transactions, and any use case that can tolerate seconds of delay.

For NyxForge use cases (oracle attestation, bond evidence submission) the
anonymous mode is appropriate -- these are one-shot submissions, not
interactive sessions.

### 3.5 Incentive Model

Mix node operators stake NYM tokens and earn rewards proportional to the
traffic they route. This creates a market incentive for running nodes globally,
maintaining the decentralized routing infrastructure without relying on
volunteers (unlike Tor). The NYM token emission schedule extends multiple
years, providing a credible incentive for the medium term.

---

## 4. Developer SDKs

### 4.1 TypeScript SDK (browser + Node.js)

The primary integration path for NyxForge. The TypeScript SDK supports:
- `mixFetch`: a drop-in replacement for the browser `fetch()` API that routes
  HTTP requests through the Nym mixnet. Any existing fetch call can be made
  anonymous with minimal code changes.
- Mixnet client: send and receive Nym packets directly for non-HTTP use cases
- Smart contract interaction helpers (Cosmos/EVM)

Installation: `npm install @nymproject/sdk`

### 4.2 WebAssembly Client

The `@nymproject/nym-client-wasm` package is a Rust Nym client compiled to
WebAssembly, available on npm. It allows full mixnet participation from any
WebAssembly-capable runtime -- including standard browser tabs -- without
requiring the user to install any software. This was described at launch as
"the first time the full privacy powers of mixnets are available directly from
a browser window."

Installation: `npm install @nymproject/nym-client-wasm`

The WASM client is heavier than the TypeScript SDK (larger bundle size) but
provides the strongest anonymity guarantee in a browser context.

### 4.3 Rust SDK

Full-featured SDK for server-side and oracle node use. Supports async
message passing, AsyncRead/AsyncWrite streams, and client pooling for
high-throughput node operators. This is the appropriate SDK for NyxForge
oracle node daemons.

### 4.4 SOCKS5 Proxy (language-agnostic)

For applications that cannot use an SDK directly, Nym provides a standalone
SOCKS5 proxy that routes all traffic through the mixnet. Any application that
supports SOCKS5 (most do) can gain Nym anonymity without code changes.

---

## 5. Anonymity Analysis

| Property | Nym (anonymous mode) | Nym (fast mode) |
| :------- | :------------------- | :-------------- |
| Sender IP hidden from recipients | Y | Y |
| Sender IP hidden from mix nodes | Y (each node sees one hop only) | Y |
| Traffic timing analysis resistant | Y (random delays + cover traffic) | P |
| Message size analysis resistant | Y (Sphinx fixed-size packets) | Y |
| Content encrypted | Y (Sphinx onion encryption) | Y |
| Global passive adversary resistant | Y | N |
| Reply anonymity (SURB) | Y | Y |

Nym provides the strongest anonymity guarantee of any browser-accessible
library in this survey. It is specifically designed to resist global passive
adversaries -- the threat model that matters most for NyxForge oracle nodes
operating in jurisdictions where the bonded outcome is politically sensitive.

Comparison to Tor:
- Tor hides destination (which server you connect to) from your ISP
- Nym hides sender, destination, timing, and volume from everyone including
  the infrastructure operators
- Tor uses low-latency onion routing; Nym uses high-latency mixing with cover
  traffic. Tor is faster; Nym is stronger.

---

## 6. Latency Characteristics

Nym's anonymity guarantee requires latency. Specific benchmarks are not
publicly published, but independent tests in 2024-2025 report:
- Fast mode (2-hop): comparable to a VPN, seconds-scale overhead
- Anonymous mode (5-hop NGM): multiple seconds to tens of seconds per message

This makes Nym unsuitable for:
- Real-time UI interactions (form submission feedback, live funding progress)
- High-frequency oracle polling

It is suitable for:
- One-shot attestation submissions (oracle proof delivery)
- Bond evidence package uploads
- Dispute notifications (latency of seconds is acceptable)
- Any background operation where the user is not waiting for immediate
  interactive feedback

Academic research (2024-2025) is actively working on latency reduction for
mixnets (LAMP, LARMix papers). The tradeoff between latency and anonymity
is a fundamental property of mixing, not a fixable implementation detail --
stronger anonymity always requires more delay.

---

## 7. NyxForge Integration

### 7.1 Role in the Architecture

Nym is the strongest anonymity layer available for browser-based NyxForge
clients. It should be used for operations where metadata privacy matters most
and latency is acceptable.

Recommended use cases:
- Oracle node attestation delivery (judges submit TLS-Notary proofs via Nym)
- Bond evidence package upload to Arweave (routed through Nym to decouple
  the uploader's IP from the bond)
- Dispute filing (disputer's identity must not be linkable to their IP)
- NGO admin actions that must not reveal the NGO's server location

Not recommended for:
- Live backer count updates in the web UI (use Waku instead)
- Interactive checkout flow (direct HTTPS is fine; checkout metadata is not
  sensitive at the network level)

### 7.2 Example 1 -- Anonymous Oracle Attestation Delivery

After Dr. Sarah Chen generates a TLS-Notary proof for the Alzheimer's bond:

1. Her oracle client initializes a Nym client connection:
   ```typescript
   import { createNymMixnetClient } from "@nymproject/sdk";
   const client = await createNymMixnetClient();
   await client.start({ clientId: "nyxforge-oracle-chen" });
   ```

2. She constructs the attestation payload (bond ID, TLS-Notary proof JSON,
   judge signature) and serializes it.

3. She sends it to the NyxForge resolution service Nym address via anonymous
   mode:
   ```typescript
   await client.client.send({
     payload: { message: attestationBytes, mimeType: "application/octet-stream" },
     recipient: NYXFORGE_RESOLUTION_NYM_ADDRESS,
   });
   ```

4. The message traverses 5 Nym mix nodes with random delays and cover traffic.
   The resolution service receives the attestation but cannot determine
   Dr. Chen's IP address, approximate location, or timing of submission.

5. The NyxForge resolution engine verifies the Ed25519 judge signature to
   authenticate the attestation -- identity is proven cryptographically, not
   by IP or connection metadata.

### 7.3 Example 2 -- Anonymous Bond Evidence Upload

For a long-dated bond (100-year term), an archivist must submit an annual
evidence package (WHO data proof, signed report) to Arweave without revealing
their identity or location.

Using `mixFetch` (drop-in fetch replacement):

```typescript
import { mixFetch } from "@nymproject/sdk";

// Upload evidence to Arweave using mixFetch -- routed through Nym mixnet
const response = await mixFetch("https://arweave.net/tx", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify(evidencePackage),
});
```

The Arweave network (and any network observer) sees the upload originating
from a Nym exit node, not the archivist's real IP. The archivist's identity
is proven by their NyxForge signing key in the evidence package, not their
network location.

### 7.4 Example 3 -- Dispute Filing

A bondholder who disagrees with a judge verdict files a dispute without
revealing their wallet address or IP to the opposing party:

1. The bondholder uses the Nym WASM client in the NyxForge web UI (loaded
   as a WebAssembly module in the browser tab)

2. They construct a dispute message: bond ID, disputed verdict hash, their
   counter-evidence IPFS CID, and their dispute stake commitment

3. The message is sent via Nym anonymous mode to the NyxForge dispute
   contract relay

4. The relay submits the dispute to UMA OOv3 on behalf of the bondholder

5. The bondholder's IP is never associated with the dispute on-chain

### 7.5 Hybrid Architecture: Nym + Waku

The two libraries are complementary:

| Operation | Recommended layer | Rationale |
| :-------- | :---------------- | :-------- |
| Oracle attestation delivery | Nym (anonymous mode) | One-shot; metadata privacy critical |
| Bond evidence archival | Nym (anonymous mode) | One-shot; archivist location sensitive |
| Dispute filing | Nym (anonymous mode) | One-shot; bondholder identity sensitive |
| Live backer count updates | Waku | Real-time; metadata not sensitive |
| Bond campaign announcements | Waku | Broadcast; no sender privacy needed |
| Judge coordination (multi-step) | Waku | Interactive; Nym latency too high |

---

## 8. Limitations

| Limitation | Detail |
| :--------- | :------ |
| Latency | Anonymous mode introduces multi-second delays. Not suitable for interactive UI operations. |
| Bundle size | The WASM client is a compiled Rust binary (~several MB). Adds significant load time to a web app if used in the browser critical path. |
| NYM token dependency | Mix node operators are incentivized by NYM tokens. Long-term token availability (20-100 year bonds) is uncertain, similar to the UMA/LINK dependency concern. |
| Centralized exit point risk | The service provider (the NyxForge resolution endpoint) still sees message content after Nym delivery. Nym hides metadata; end-to-end encryption of the payload is still required. |
| Cover traffic costs | Nym's continuous cover traffic consumes bandwidth even when no real messages are being sent. This is a design requirement for anonymity but matters for resource-constrained oracle nodes. |
| Not P2P between clients | Nym routes messages through the mix node network to a recipient address. It is not a direct P2P channel. Two browser clients cannot directly connect via Nym -- they communicate via a service provider endpoint. |

---

## 9. Reference Links

- Nym website: https://nym.com
- Developer docs: https://nym.com/docs/developers
- TypeScript SDK: https://nym.com/docs/developers/typescript
- Rust SDK: https://nym.com/docs/developers/rust
- WASM client (npm): https://www.npmjs.com/package/@nymproject/nym-client-wasm
- Mixnet explainer: https://nym.com/mixnet
- NymVPN fast vs anonymous modes: https://support.nym.com/hc/en-us/articles/24326365096721
- GitHub: https://github.com/nymtech
- Zcash + Nym integration discussion: https://forum.zcashcommunity.com/t/the-nym-mixnet-for-network-privacy-for-zcash/46324
