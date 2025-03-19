# Testing Tool for Non-Standard Transaction RPC

This tool is used for testing non-standard transaction API.

## A example of new API-change bitcoin code

Bitlayer offers a example to modify bitcoin-core, changing XX lines of code. [link]

the `sendnsttransaction` API is very similar with the wide-used API `sendrawtransaction`, except `sendnsttransaction` accept non-standard transaction.

## Check the Avaibility of API for Non-Standard Transaction RPC

This tool will firstly create a address, which contains a large script size.

then, use api to send 10000000 sats to the address.

final, consume this address through a non-standard transaction.

The final step will contains two try, the first one is sending transaction to `sendrawtransaction` which will fail, and the second try is sending transaction to `sendnsttransaction` which will success.

## Usage

The usage of this tool is below.

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
  -h, --help
          Print help
  -V, --version
          Print version
```

### STEP 1: Start a Local Regtest Network (Optional)

`./setup_bitcoin_example.sh` will start a local regtest network to test.

### STEP 2: Use testing tool to check the avalibility of API.

```
git clone git@github.com:bitlayer-org/NST-Tool.git
cd NST-Tool
cargo run -- --endpoint http://127.0.0.1:18443 --user admin --password admin
```

## License
