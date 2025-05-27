
# API Key Service

This is a service for secure deployment of HTTP API keys. It is designed to be run in a Confidential 
Virtual Machine using Intel TDX, and uses on-chain attestation provided by the Entropy network.

Users can upload their API keys for an HTTP service, and then make requests which are forwarded to
that service, substituting a placeholder with their API key. 

Client requests are authenticated with an sr25519 signature from the user. The server is authenticated
using both sr25519 signing and x25519 encryption with a on-chain attested public keys.

You can run a test server without TDX by starting without enabling the `production` feature (which is
disabled the default):

```
cargo run --chain-endpoint ws://localhost:9944
``` 

There is also a client CLI. To use it you need the public encryption (x25519) key of the server which
you can get by doing:
    
```
curl http://localhost:3001/info
```

Then you can deploy an API key like so, substituting <PUBLIC KEY OF SERVER> with the x25519 public key
from the output of the previous command. 

```
cargo run -p entropy-api-key-service-client -- --mnemonic //Alice --service-x25519-public-key <PUBLIC KEY OF SERVER> deploy-api-key my-secret-api-key https://api.thecatapi.com
```
