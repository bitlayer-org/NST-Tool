# A Tool for Non-Standard Transaction RPC

## Start a local network (Optional)

Bitlayer offers a useful tool to

```
make -B build
cd build
make all
cd bin
mkdir bitcoin_data
./bitcoind -datadir=./bitcoin_data -blocksonly -server -port=8332 -regtest
```
