# API Key Service – permanent storage

## Setup and assumptions

This document assumes the Entropy blockchain is up and running and has a working set of secret key shares distributed among a set of TDX nodes, each running `entropy-tss` inside Confidential Virtual Machines (CVMs) on untrusted hosts. The CVMs are attested, guaranteeing that all nodes can trust that all their peers run the same software. The blockchain has a registry of known CVMs, and a public API allowing users to query IP addresses and public keys for CVMs.

In addition to the threshold signing nodes, there is a set of API Key Service (AKS) nodes running `entropy-aks` inside CVMs; anyone can run AKS nodes.

The AKS nodes are also attested and the blockchain API/on-chain registry can be queried for IP addresses and public keys in the same way as for the TSS case.

The software composing the CVM for both TSS and AKS is defined ahead of time as a raw disk image and contains a minimal linux build (x86). Beyond basic OS services like networking, the CVM runs `entropy-tss` or `entropy-aks`.

_TODO: Loop back to this section and describe how TSS and AKS interact (if they do)._

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

## Secret data CRUD

Anyone on the internet can query the blockchain for the public key and hostname of an AKS node. Using this information they can establish a secure connection to the AKS node. The untrusted host running the CVM cannot MITM this connection.

_TODO: Spell out the reasons why this is secure and how it works in detail, e.g. how the CVM attestation is verified by the user, why this on-chain pubkey registry is sane and what the difference is between this setup and standard TLS certs. I guess the deeper question is: what does the blockchain do for us here?_

~~The blockchain has privileged access (encrypted and authenticated) to a small set of RPC endpoints that allows validator nodes to send secrets, e.g. API keys, on user's behalf to the CVMs running the `entropy-aks` daemon ("AKS" for short). The secret data submitted by users is encrypted in-flight and opaque to validators and the on-chain transaction data cannot be decrypted/inspected. Secrets only ever reside inside the CVM, running in the TDX enclave.~~

Users submit and query data to/from the AKS nodes using standard http requests with encrypted payloads.

_TODO: HTTP requests? Or websockets? Or either? The untrusted cloud operator can inspect the traffic of course, and can spy on metdata (like user's IP). Or how does this work? Is the client-to-AKS transport mechanism specc'd somewhere?_

The relevant RPC endpoints exposed by the AKS are:

- `POST /secret?data=&acl=`: adds a new secret with an appropriate ACL; returns a `secret_id`.
- `PUT /secret?id=&data=&acl`: updates the secret and/or ACL with secret_id `id`.
- `DELETE /secret?id`: deletes the secret with secret_id `id`.
- `GET /make-request?id=&remote-url=&verb=&payload=`: instructs the AKS to make a remote API call to `remote-url` using the http verb `verb` and the secret matching the `id` to send the payload in `payload`.

*NOTE*: `secret_id`s should be unique and **random**, given that knowledge of a `secret_id` allows anyone to make remote API calls on behalf of the user.

_TODO: Use correct endpoints, from the spec doc._

_TODO: Describe the ACL format and how it works in detail._

_TODO: Can users provide any `remote-url` like this or is it limited to a set of whitelisted URLs? Stored where? In the ACL? Or separate?_

~~All calls to the AKS from the outside are signed and contain counter measures to replay attacks (likely a block number and/or a random nonce). The API responses are encrypted to the end user's public key before they are send back outside the AKS.~~

_TODO: None of the above is correct I believe?_

All calls to the AKS contain counter measures to replay attacks, to stop a snooping cloud operator from recording traffic and replaying it at will. *TODO: spec counter measures in detail.*

_TODO: Spec the replay attack counter measures in detail. Given this all happens outside the chain, using the block number is pointless._

**NOTE:** Users must do local key management and keep track of their `secret_id`s. A stolen `secret_id` can be used by anyone to execute remote API calls, so it's essentially equivalent to stealing the API key in the first place. That is an important matter worthy of its own separate discussion.

## What do AKS nodes store?

To support the above scenario, the AKS nodes must keep the following data:

- User secrets
- User ACL, one for each remote rpc
- ~~Users public keys~~
- ~~List of current validators, i.e. the set of outside nodes allowed to do secrets CRUD and their public keys.~~
- Pending requests and responses.
- Additional bobs and bits, e.g. errors~~, info about next epoch, current time and current block number~~.

If each user stores 5 secrets of max 1kb each, with 1kb of ACL data~~, and 1kb of public key data~~, we have ~10Kb per user, so supporting 1000 users/node requires ~10Mb of memory/storage. When there are network problems and the AKS cannot reach the remote services, the storage needs for pending requests/responses could spike significantly. A rate limiting feature will eventually become essential.

~~Secrets, ACLs, public keys and epoch info must be replicated to all AKS nodes so that they can all process user requests and ensure resiliance and availability.~~

## Storage options

The storage discussion that follows is split into two distinct requirements: the local storage encryption key on the one hand; and state replication on the other.

### Local storage encryption

While we focus on AKS in this document, there is a slight overlap with the TSS; we briefly discuss both here.

When it comes to encrypted local storage, TSS nodes **must** be able to recover their encryption key on a reboot.

Without such a mechanism, the consequences for the TSS network as a whole range from bad (need to trigger a key re-share) to catastrophic (need to restart the whole network?). For AKS nodes losing access to their local storage the consequences are less dire but given we have to build it for TSS, the same architecture can benefit AKS nodes, enabling them to recover from a reboot.

### State replication

When it comes to state replication the needs for TSS and AKS diverge. For TSS it's not needed at all, but AKS nodes should probably replicate state between them.

**NOTE**: We need a decision on this: to replicate or not to replicate.

If AKS nodes do **not** replicate their state every node is an island. When it goes away so does its data and users must re-upload their secret data. Users have no way of knowing that their selected AKS node is gone so they have to stop what they are doing and fix the problem right away: pick a new AKS node and re-upload their secrets (they should probably revoke the old ones and issue new secrets from their API providers). Users can mitigate this somewhat by selecting multiple AKS nodes to store their secrets (and take care of updating/revoking secrets) but it still puts the onus of managing dissappearing nodes on them.

In the scenario where AKS nodes **do** replicate the state between them we need to distinguish two cases:

1. A new AKS joining the network. Requires a full copy of the state.
1. Users making CRUD changes to state, expecting all nodes to see the changes in a reasonable timeframe (seconds?).

*TODO: Given AKS nodes replicate their state, is it still useful for AKS nodes to encrypt their local storage with a unique key? You break one, you get the full state and all user secrets. Why not use the same key then?*

#### New AKS node joining the network

Flow outline:
- Register on the blockchain as a new AKS node. *TODO: spec this in detail.*
- Ask the blockchain for a list of known AKS nodes.
- Pick `max(3, total_nodes)` and query them for a hash of their state.
- Compare the hashes received and if they match, pick one of the nodes and request a full copy of its state.
- Download&validate the state copy *TODO: what kind of validation is required here?*
- Confident that any new state can be reconciled with the state copy it downloaded, the new AKS node joins the state replication protocol and receives the last updates.
- Once in sync, it lets the blockchain know that it's `READY`
- Start accepting user requests and propagate own state changes to the other nodes.

A rebooting AKS node is not too different from a new node. It must re-register on the blockchain (the node cannot know how long it has been gone for, perhaps the blockchain deleted it) and even if it probably has a decently recent copy of the state on its local disk, it must still query the other nodes to know how far behind it is. Given the small size of the state it is likely easier to simply treat a re-booting AKS node as a new node.

### State change propagation

_TODO: This is pretty tricky, not sure I can wing a write-up of a decent state change propagation protocol._


––> Meeting notes, June 4th, 2025

### Local storage encryption key

- TSS **must** a way to recover the storage key
- AKS **may** benefit from a recovery feature

### Replication

#### Assumptions

- tens of AKS nodes, certainly not thousands
- thousands of users, not millions
- full state is tens of megabytes, not gigabytes
- AKS nodes are incentivized and we assume that most of them will do their best to stay up and do the work
- AKS nodes will come and go willy nilly and there is no central authority that knows who they are or how reliable they are

- TSS **may** be useful? For some data but definitely **not** for key shares!
- AKS **should** be replicated
  - Actually: replication for AKS is not required given that the secrets stored all have their own repudiation/replacement mechanism from the secret issue (e.g. you can go to Coinbase and get a new key), so an AKS node going down and/or dissapearing is not catastrophic.
  - If however the team feels replication is a must then here we do it:
    - TODO: describe how to do it
    - Nodes joining: they need a full copy of the state
    - Users adding/changing data: how do changes propagate to other nodes?
    - Best would be an out-of-the-box solution that can
      1.  runs in TDX
      1.  small overhead
      1.  multi-master replication ready
      1.  is this Redis? is this SQLITE + something? Is it Postgres? Is it some nifty pubsub solution? RESEARCH topic
    - …if no such ready made solution can be found we suggest:
      1.  AKS stores data in an append-only file (encrypted)
      1.  on request (by new nodes), provide full snapshots of local storage
      1.  changes to existing data are gossiped as they happen
      1.  implement automatic merging of changes

Replication for AKS is probably a bit easier to implement if we can rely on a secret key storage solution like SGX Seal API, but does not require it.

### SGX Sealing API as a keyserver for AKS/TSS

–––> END Meeting notes, June 4th, 2025

_TODO: From here on out, most of the text is probably incorrect as I was assuming some sort of replication was planned._

The amount of data stored is so small that the most performant implementation is to simply store everything in memory and implement a periodical checkpointing mechanism to save the data to disk.

The form of the disk-based storage does not matter very much for the current conversation, as long as it is encrypted under a key known only to the AKS node itself. It could be a sqlite database or a bincode-encoded dump of the in-memory data tossed into a file and saved; it could also be a binary blob stored on the blockchain.

Without access to the current state the AKS node is not operational and the node will not accept RPCs at all until the state is available, checked to be valid and in sync with the other nodes.

### Store data in an encrypted binary blob on the blockchain

At first glance this option is attractive because it leverages the well-understood properties of the blockchain to replicate and distribute data trustlessly.

The downside is that each AKS node would have to make a blockchain transaction at every block of the entire state. Even if the state size is small (~10Mb), when multiplied by the number of AKS nodes it can get unwieldy very quickly. There is also the matter of synchronizing the AKS operation with the cadence of block production, which is undesirable.

There are many more problems with this approach (e.g. how to reconcile everyone's updates?).

### Rely on replication between AKS nodes

The AKS nodes will need a mechanism to ensure they all have the same secrets, ACLs etc so they will already be replicating most of the state between them (TODO: IS THERE AN ACTUAL PLAN FOR THIS WRITTEN DOWN?).
This mechanism relies on attestation that guarantees that all the nodes run the same software and know where to find each other.

If the assumptions above about the state size are correct, the replication mechanism wouldn't have to be very sophisticated. The state is small enough to be sent over the wire in a single message. New AKS nodes joining the network would need this anyway.

### Store data in a file on disk

There is a scenario where data about in-flight operations becomes crucial, and having a local data store could be important. Consider a AKS node that has successfully executed a request to a remote service ("hey Coinbase, close my account and send all the coins to 0xDEADBEEF"), but the AKS node crashes before it can send the response back to the user.
In this case, the AKS node would need to be able to recover its state and send the response once it is back online. It would be awkward to include pending responses in the replicated state, as they are not relevant to other nodes and would only bloat the state unnecessarily.

Local, disk-based storage could be useful here.

The scenario described is less of a problem for outgoing pending requests. If the AKS node crashes after receiving the payload to forward, but before sending it, the user would see a timeout/disconnect error but their state would not be invalid and they could simply try again later (or with a different AKS node).

## State encryption key(s)

So far we have established that the AKS nodes need to store a small amount of data _somewhere_, and that the data must be encrypted. The encryption key must not be known to the outside world, including the server operators and blockchain validators.

Furthermore, AKS nodes must be able to communicate among themselves so they can replicate the state. The communication between AKS nodes is based Noise + websockets. New nodes joining the network must have a way to get a copy of the complete state.

In conclusion we have the following requirements:

- A rebooting AKS node is not different from a new AKS node joining the network, modulo pending responses.
- Thanks to attestation and TDX-local Noise protocol encryption, AKS nodes have a secure channel among themselves, private to all external actors.
- It is likely useful to have a local disk-based storage in place in addition to replication, to store pending responses and other ephemeral data. The encryption key for this local data can be safely be the same for all AKS nodes and be included in the state replication message payload.

## Attack scenarios

- A TDX compromise is catastrophic; the AKS node would be compromised and with it all user secrets. Game over.
- A dishonest server operator can probably pull off a successful rollback attack on local disk storage, replacing a file with an older version containing a state they like better. This is probably not catastrophic for the whole network, especially if data stored locally is limited to just pending responses and ephemeral (meta-)data.
- Eclipse and DoS attacks are trivial; defense here will rely on the blockchain noticing that AKS nodes are unavailable and having ways of replacing them. If the attacks are temporary this is probably not catastrophic for the network as a whole.
- Bugs in the AKS code can easily be catastrophic. An AKS node's job is akin to that of a browser: make arbitrary http requests and a bad bug in http response parsing could lead to data leaks. This scenario has nothing to do with data storage though.
- Network partitions, while certainly unpleasant, are probably not catastrophic. The AKS nodes will be able to continue operating and serving requests, but they will not be able to replicate the state with the rest of the network. This is a problem for the network as a whole, but not for individual AKS nodes and when the network heals, the AKS nodes can probably patch up the state for most of the data.

## Unclear

- Replicate state between AKS nodes or not.
  - If state is not replicated, how can the service scale? When one AKS node is "full", how can it's secrets be sharded to other nodes?
  - Eclipse attacks become much more disruptive to users. Same for DoS and partitions: the service effectively goes down.
- Local persistent storage is either a useful extra or critical, depending on the decision wrt replication.
  - Local persistent storage is useless without at least replicating the encryption key across all/most AKS ndoes.
  - Is there any point in Shamir Secret Sharing (or other fancy crypto) of each AKS node's storage encryption key?
    - Yes, because it would allow the system to survive a corrupt TDX enclave. SGX teaches us that many attacks are possible only with physical access to the machine running the enclave, so even if TDX is broken, it's not a given that all AKS nodes will be compromised immediately.
    - No, not really: a corrupted TDX enclave can just pretend that "Hey fellow AKS nodes, evil AWS killed my disk and I had to reboot, can y'all send me the key again please?".
  - Do all AKS nodes need to store all the secrets?
    - Yes, because the service needs to be able to scale and the secrets need to be replicated. Also: what is even the point of the AKS service if it's not decentralized, censorship resistant and available?
    - No, because the service can scale by adding more AKS nodes and manually sharding the secrets among them. When nodes die or are upgraded, users have to manually re-upload their secrets.
