# Non-Standard Transaction RPC Testing Tool

This tool is designed to test and compare the functionality of the standard transaction RPC (`sendrawtransaction`) and the non-standard transaction RPC (`sendnsttransaction`) within Bitcoin Core.

## An Example Modification to Bitcoin Core

Bitlayer provides an example modification to Bitcoin Core ((https://github.com/bitlayer-org/bitcoin/tree/nst_api)), introducing the `sendnsttransaction` API by changing a minimal amount of code. This new API allows broadcasting non-standard transactions, similar to the widely used `sendrawtransaction` API, but with relaxed restrictions on transaction standards.

## Functionality

This tool performs the following steps to verify the availability of the non-standard transaction RPC:

1.  **Create a Large-Script Address:** Generates a Bitcoin address with a significant script size.
2.  **Send Funds:** Sends 1,000,000 satoshis (sats) to the generated address using RPC.
3.  **Consume with a Non-Standard Transaction:**
    - Attempts to consume the funds from the address using `sendrawtransaction`, which is expected to fail due to the transaction being non-standard.
    - Attempts to consume the same funds using `sendnsttransaction`, which is expected to succeed.

## Usage

### Steps

1.  **Start a Local Regtest Network (Optional):**

- If you don't have an existing Bitcoin regtest network, you can run the `./setup_bitcoin_example.sh` script to start one.
- `./setup_bitcoin_example.sh` will download our example modification of Bitcoin core, build and run, so this regtest network has added a `sendnsttransaction` RPC.

2.  **Run the Testing Tool:**

- Clone the repository: `git clone git@github.com:bitlayer-org/NST-Tool.git`
- Navigate to the directory: `cd NST-Tool`
- Run the test: `cargo run -- --endpoint http://127.0.0.1:18443 --user admin --password admin`

, or

- Download the release binary `wget https://github.com/bitlayer-org/NST-Tool/releases/download/v0.1/nst-tool && chmod +x nst-tool`
- Run the test: `./nst-tool --endpoint http://127.0.0.1:18443 --user admin --password admin`

3.  **Stop Nework:**

    - Use `pkill bitcoin` to stop network.

### Command-Line Options

```
Usage: nst-tool [OPTIONS] --endpoint <ENDPOINT> --user <USER> --password <PASSWORD>

Options:
  -e, --endpoint <ENDPOINT>
          Bitcoin Core RPC URL (e.g., http://127.0.0.1:18443)
  -u, --user <USER>
          Bitcoin Core RPC username
  -p, --password <PASSWORD>
          Bitcoin Core RPC password
  -s, --script-size-kb <SCRIPT_SIZE_KB>
          Script size in kilobytes (e.g., 500 for 500KB) [default: 500]
  -r, --rpc-name <RPC_NAME>
          Name of Bitcoin RPC [default: sendnsttransaction]
  -h, --help
          Print help
  -V, --version
          Print version
```

Examples:

```
# test 1MB bytes transaction
cargo run -- --endpoint http://127.0.0.1:18443 --user admin --password admin --script-size-kb 1000
# test `nonstdtx` RPC
cargo run -- --endpoint http://127.0.0.1:18443 --user admin --password admin --rpc-name nonstdtx
```

## License

This repository is licensed under the Apache 2.0 license.
