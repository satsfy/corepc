# OpenRPC Bitcoin Core Specs

To generate OpenRPC specs for a new Bitcoin Core version, use LLM to backport PR [34683](https://github.com/bitcoin/bitcoin/pull/34683) into newer version, then run command `bitcoin-cli getopenrpcinfo` to generate the spec.

For older versions before v31, the PR was backported. The backport get progressively less trustworthy every version down. Down to version v26, no workarounds were needed (the PR cherry-picked cleanly onto the release tag, generally safe), but between v25 and v18 progressively more problems to work around were encountered. v17, in particular, is entirely AI-based and best-effort basing itself on v18 because a standardize RPC doc didn't exist for that version. Regardless, all these specs pass integration tests, which is the quality bar we seek.

The branches that generated each version can be found on:

Bitcoin Core Version 31.0: https://github.com/satsfy/bitcoin/tree/31.x-openrpc-backport
Bitcoin Core Version 30.2: https://github.com/satsfy/bitcoin/tree/30.x-openrpc-backport
Bitcoin Core Version 29.2: https://github.com/satsfy/bitcoin/tree/29.x-openrpc-backport
Bitcoin Core Version 28.2: https://github.com/satsfy/bitcoin/tree/28.x-openrpc-backport
Bitcoin Core Version 27.2: https://github.com/satsfy/bitcoin/tree/27.x-openrpc-backport
Bitcoin Core Version 26.2: https://github.com/satsfy/bitcoin/tree/26.x-openrpc-backport
Bitcoin Core Version 25.2: https://github.com/satsfy/bitcoin/tree/25.x-openrpc-backport
Bitcoin Core Version 24.2: https://github.com/satsfy/bitcoin/tree/24.x-openrpc-backport
Bitcoin Core Version 23.2: https://github.com/satsfy/bitcoin/tree/23.x-openrpc-backport
Bitcoin Core Version 22.2: https://github.com/satsfy/bitcoin/tree/22.x-openrpc-backport
Bitcoin Core Version 0.21.2: https://github.com/satsfy/bitcoin/tree/0.21.x-openrpc-backport
Bitcoin Core Version 0.20.2: https://github.com/satsfy/bitcoin/tree/0.20.x-openrpc-backport
Bitcoin Core Version 0.19.2: https://github.com/satsfy/bitcoin/tree/0.19.x-openrpc-backport
Bitcoin Core Version 0.18.2: https://github.com/satsfy/bitcoin/tree/0.18.x-openrpc-backport
Bitcoin Core Version 0.17.2: https://github.com/satsfy/bitcoin/tree/0.17.x-openrpc-backport.
