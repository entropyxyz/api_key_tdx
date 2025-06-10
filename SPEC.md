# API Key Service – permanent storage

Enable clients of Entropy Core to make secure API calls to third party services using API keys stored on a server-side service ("AKS" below), while maintaining privacy and integrity of the API keys and the requests/responses.

## Setup and assumptions

This document assumes the Entropy blockchain is up and running and has a working set of secret key shares distributed among a set of TDX nodes, each running `entropy-tss` inside Confidential Virtual Machines (CVMs) on untrusted hosts. The CVMs are attested, guaranteeing that all nodes can trust that all their peers run the same software. The blockchain has a registry of known CVMs, and a public API allowing users to query IP addresses and public keys for CVMs.

In addition to the threshold signing nodes, there is a set of API Key Service (AKS) nodes running `entropy-aks` inside CVMs; anyone can run AKS nodes.

The AKS nodes are also attested using TDX quotes and the blockchain API/on-chain registry can be queried for IP addresses and public keys in the same way as for the TSS case. 

The software composing the CVM for both TSS and AKS is defined ahead of time as a raw disk image and contains a minimal linux build (x86). Beyond basic OS services like networking, the CVM runs `entropy-tss` or `entropy-aks`.

Since registering on-chain requires an attestation, and the public keys are given as input data to the quote, we assume that if a node can authenticate using the public keys it registered with, it has made an attestation and can be trusted to be running an endorsed release of the service.

The API Key Service ("AKS" below) has two jobs:

1. Manage storage of secret user data, e.g. API keys.
1. Execute remote API calls on behalf of users, using their secrets to authenticate the requests, and return the results to the users.

### Assumptions

To be clear about what the underlying assumptions are for the design outline that ensues, we list them here:

- Tens of AKS nodes; certainly not thousands.
- Thousands of users, not millions.
- Full state is tens of megabytes, not gigabytes
- AKS nodes are incentivized. We assume most of them will do their best to stay available and do the work.
- Permissionless. AKS nodes will come and go willy nilly and there is no central authority that knows who they are or how reliable they are.

### Requirements

* MUST be able to deploy user's API keys to a server-side service, and relay client requests to a third party HTTP service, substituting a placeholder mark for the deployed API key.
* MUST provide an access control mechanism (ACL) allowing user-defined control over access to a particular deployed API key.
* MUST maintain privacy and integrity of deployed API keys from both the server operator and external actors.
* MUST maintain privacy and integrity of relayed HTTP requests and responses from both the server operator and external actors.
* MUST provide responses to relayed HTTP requests which are near-identical to the original response (maintaining headers, etc).
* MUST provide a simple client for testing/demonstration purposes.
* SHOULD have persistent secure storage of API keys across server restarts or upgrades.
* SHOULD have a logging mechanism.
* SHOULD have a mechanism by which anyone can independently verify that the server-side executable being run corresponds to a publicly available source code repository.

See issue [#1398](https://github.com/entropyxyz/entropy-core/issues/1398) for background and discussion.

## Secret data CRUD

Anyone on the internet can query the blockchain for the public keys and hostnames of all active AKS nodes. Using this information they can establish a secure connection to an AKS node. The untrusted host running the CVM cannot MITM this connection, as the secret keys are generated in the CVM and never stored anywhere accessible to the host.

It will be possible for clients to independently verify the on-chain attestation of a particular AKS instance before using it (pending [entropy-core#1438](https://github.com/entropyxyz/entropy-core/issues/1438) being resolved).

Users submit and query data to/from the AKS nodes using standard HTTP POST requests with encrypted and signed payloads (encrypted under the AKS node's public key and signed by the user's private key). The untrusted host executing the CVM is assumed to be able to spy on the traffic and manipulate all data going to and from the AKS node inside the CVM, but has no way to decrypt the payload.

The relevant RPC endpoints exposed by the AKS are:

- `/deploy-api-key`: adds a new secret. This is associated with a given base URL of the service it is for, and the public key of the user who signs the request body. The base URL + user's pubkey constitutes the identifier for the secret.
- `/update-secret`: updates the secret and/or ACL with secret*id `id` (*Note: not implemented yet*).
- `/delete-secret`: deletes the secret with secret*id `id` (*Note: not implemented yet*).
- `/make-request`: instructs the AKS to make a remote API call to `remote-url` using the http verb `verb`, with optional POST/PUT payload `payload` and the secret matching the public key used to sign the request payload; the secret is identified by the base URL (extracted from `remote-url`) + user's public key.

**Note:** There are two aspects of ACLs. Firstly, the "ACL proper" that defines which users' public keys are allowed to use a secret; secondly the "usage ACL", defining how a secret can be used, e.g. "only for amounts smaller than `x`" or "only between 10am and 4pm". Refer to issue [#INSERT_ISSUE_NR_HERE] for the details of how this works.

All calls to the AKS from the outside are signed and contain counter measures to replay attacks to stop a snooping cloud operator from recording traffic and replaying it at will. The API responses are encrypted to the end user's public key before they are sent back outside the AKS.

The encryption/authentication used is [the same as entropy-tss uses](https://github.com/entropyxyz/entropy-core/blob/master/crates/protocol/src/sign_and_encrypt/mod.rs) and protection is given against replay attacks ([currently using timestamps](https://github.com/entropyxyz/api_key_tdx/blob/cd96b90d1f63e0f3d75bc461e179b094d2cf400e/src/api_keys/api.rs#L81-L87) but this may change to block number or nonces).

**NOTE:** Users must do local key management and keep track of their secret signing key. A stolen secret key can be used by anyone to execute remote API calls, so it's essentially equivalent to stealing the API key in the first place. That is an important matter worthy of its own separate discussion.

## What do AKS nodes store?

To support the above scenario, the AKS nodes must keep the following data:

- User secrets + base URL.
- User ACL, one for each secret.
- Users public keys.
- Pending requests and responses.
- Additional bobs and bits, e.g. errors, logs.

If each user stores 5 secrets of max 1kb each, with 1kb of ACL data, and 1kb of public key data, we have ~15Kb per user, so supporting 1000 users/node requires ~15Mb of memory/storage. When there are network problems and the AKS cannot reach the remote services, the storage needs for pending requests/responses could spike significantly. A rate limiting feature will eventually become essential.

## State storage

The storage discussion that follows is split into two distinct pain points: the TSS/AKS encrypted local storage and, for AKS, state replication.

### State replication

TSS and AKS have different needs. For TSS, state replication is not needed at all. For AKS nodes it's the other way around: AKS nodes do need to replicate state between them.

What would it look like if AKS nodes did **not** replicate their state? Then every node is an island and when the node goes away so does its data and users must re-upload their secret data and all pending external API calls are lost. Users have no way of learning that their selected AKS node is gone so they have to stop what they are doing and fix the problem as they notice: pick a new AKS node and re-upload their secrets (they should probably revoke the old ones and issue new secrets from their API providers). Users can mitigate this somewhat by selecting multiple AKS nodes to store their secrets (and take care of updating/revoking secrets) but it still puts the onus of managing dissappearing nodes on them. The downsides are significant. No resilience, no redundancy, no scaling, no availability guarantees. An un-replicated AKS service is not decentralized, censorship resistant or available.

For AKS state replication we distinguish two cases:

1. A new AKS joining the network. Requires a full copy of the state.
1. Users making CRUD changes to state, expecting all nodes to see the changes in a reasonable timeframe (seconds?).

#### New AKS node joining the network

Flow outline:

- Generate keypairs (Account ID for signing and x25519 pair for encryption).
- Request a quote nonce from the chain using [`entropy-client::user::request_attestation`](https://docs.rs/entropy-client/0.4.0-rc.1/entropy_client/user/fn.request_attestation.html). This submits an extrinsic which causes the chain respond with an event containing a nonce which is associated with the keypair used to sign the extrinsic.
- Generate a TDX quote using the following as input data (which is signed as part of the quote body):
  - The nonce from the previous step
  - The account ID and x25519 public key 
  - ['Context'](https://github.com/entropyxyz/entropy-core/blob/32e5dcc4e8c6532968de28d3cae410ff2c332872/crates/shared/src/attestation.rs#L62) indicating that this quote is intended to be used for registering an API Key Service instance
- Submit an [`add_box` extrinsic](https://github.com/entropyxyz/entropy-core/blob/32e5dcc4e8c6532968de28d3cae410ff2c332872/pallets/outtie/src/lib.rs#L158) containing the quote, public keys and IP address and port of the instance.  
- If the chain runtime can validate the quote, this AKS instance is added to the [list of available instances](https://github.com/entropyxyz/entropy-core/blob/32e5dcc4e8c6532968de28d3cae410ff2c332872/pallets/outtie/src/lib.rs#L89)
- Ask the blockchain for [a list of known AKS nodes](https://github.com/entropyxyz/entropy-core/blob/32e5dcc4e8c6532968de28d3cae410ff2c332872/pallets/outtie/src/lib.rs#L88).
- Pick the 'closest' node to this node's account ID using the XOR metric, and make an HTTP request to that node to retrieve it's list of API keys.
- Once in sync, it lets the blockchain know that it's now `READY` but submitting an extrinsic.
- Start accepting user requests and propagate own state changes to the other nodes.

A rebooting AKS node is not too different from a new node. It must re-register on the blockchain (the node cannot know how long it has been gone for and perhaps the blockchain deleted it) and even if it probably has a decently recent copy of the state on its local disk, it must still query the other nodes to know how far behind it is. Given the small size of the state it is likely easier to simply treat a re-booting AKS node as a new node.

### State change propagation

State change synchronization is a big topic and there are many solutions out there, varying greatly in complexity and safety/resilience guarantees offered. What we suggest here is a sketch of "the simplest thing that can work". See issue [#INSERT_ISSUE_NR_HERE] for details.

- Use a "XOR proximity" metric to assign "neighbours" to AKS nodes; each AKS node tries to find `n` such neighbours, where `n` is a system parameter choosen to work well with the actual/expected number of AKS nodes.
- The lookup key is the hash of the `secret` + `pubkey` + `baseURL` (and possibly the `ACL` as well, TBD)
- When users look for an AKS node to deploy a secret to, they calculate the lookup key and choose the AKS node that is "closest".
- AKS nodes propagate own state updates to its `n` closest neighbours.
- AKS nodes propagate state updates received from other AKS nodes to `n-1` neighbours (i.e. to all except the one they received the update from)
- Eventually all nodes within `n` proximity will have received the update and stop propagating it.

See issue [#INSERT_ISSUE_NR_HERE] for more details.

## Local storage

An AKS node needs local storage for two reasons:

1. To recover quickly after a reboot. Generating fresh keypairs means that the account needs to be funded by the node operator before it can continue to operator - this means there is a human in the loop and so recovering can potentially take a very long time.
1. Store ephemeral data about in-flight requests that would be pointless to gossip to other AKS nodes. Consider an AKS node that has successfully executed a request to a remote service ("hey Coinbase, close my account and send all the coins to 0xDEADBEEF"), but the AKS node crashes before it can send the response back to the user. In this case, an AKS node with local storage could recover its state and send the response once it is back online.

### Local storage encryption

While we focus on AKS in this document, there is a slight overlap with the TSS; we briefly discuss both here.

TSS nodes **must** be able to recover their encryption key on a reboot. Without such a mechanism, the consequences for the TSS network as a whole range from bad (need to trigger a key re-share) to catastrophic (need to restart the whole network?). For AKS nodes losing access to their local storage, the consequences are less dire but given we have to build it for TSS, the same architecture can benefit AKS nodes, enabling them to recover faster from a reboot.

The amount of data stored at an AKS node is small and updates probably rare. It is fine to simply store everything in memory and implement a periodical checkpoint mechanism to flush the data to disk.

The form of the disk-based storage does not matter very much for the current conversation, as long as it is encrypted under a key known only to the AKS node itself. It could be a sqlite database or a bincode-encoded dump of the in-memory data tossed into a file and saved; it could also be a binary blob stored on the blockchain.

Without access to the current state, the AKS node is not operational and the node will not accept RPCs at all until the state is available, checked to be valid and in sync with the other nodes. As discussed above, having a local dump of the node's state can speed up recovery after a reboot.

### What about storing data in an encrypted binary blob on the blockchain?

At first glance this idea is attractive because it leverages the well-understood properties of the blockchain to replicate and distribute data trustlessly.

The downside is that each AKS node would have to make a blockchain transaction at every block of the entire state. Even if the state size is small (~15Mb), when multiplied by the number of AKS nodes it can get unwieldy very quickly. There is also the matter of synchronizing the AKS operation with the cadence of block production, which is undesirable.

There are more problems with this approach:

- If each AKS node dumps its own state on-chain, how to reconcile everyone's updates?
- Using substrate JSONRPC calls to store data blobs means needing to run reliable and available RPC node infra: costly and centralized.
- Maybe AKS nodes could be light clients on the Entropy network and avoid having to trust an RPC node to be able to read data, but is it really worth the effort? They'd still need a full node to submit their state updates over JSONRPC.

## A few words on attack scenarios

- A TDX compromise is catastrophic; the AKS node would be compromised and with it all user secrets. Game over.
- A dishonest server operator can probably pull off a successful rollback attack on local disk storage, replacing a file with an older version containing a state they like better. This is probably not catastrophic for the whole network, especially if the data stored locally is limited to just pending responses and ephemeral (meta-)data.
- Eclipse and DoS attacks are trivial; defense here will rely on the blockchain noticing that some AKS nodes are unavailable and having ways of replacing them. If the attacks are temporary this is probably not catastrophic for the network as a whole. On the other hand, all AKS nodes are open to traffic from the public internet, so it is both easy and cheap to DoS them.
- Bugs in the AKS code can easily be catastrophic. An AKS node's job is akin to that of a browser (make arbitrary http requests) and a bad bug in http response parsing could lead to data leaks. This scenario has nothing to do with data storage though.
- Network partitions, while certainly unpleasant, are probably not catastrophic. The AKS nodes can continue operating and serve requests, but they will not be able to replicate the state with the rest of the network. This is a problem for the network as a whole, but not for individual AKS nodes. When the network heals, the AKS nodes can (probably) patch up the state.
- All AKS network traffic can be watched and even if it's all encrypted, a lot can be learned about users by just analyzing source and destination IP addresses and interaction patterns. Possible mitigations are NYM-style cover traffic with each AKS node acting as a proxy hop for their peers..

### SGX Sealing API as a keyserver for AKS/TSS

This is probably a very good avenue to explore further. Currently all CPUs that support TDX also support SGX so we can probably re-utilize the infra. The SGX Sealing API has been hacked several times over the years and not all of the vulnerabilities have been definitively patched (last Intel fix was rolled out in Nov 2024). On the other hand SGX is probably roughly as secure as TDX is.

_TODO: find a proper home for the SGX conversation and extend the text with an outline of the flow._
