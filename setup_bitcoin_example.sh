# todo: download bitcoin and run it automatically

# cp bitcoind and bitcoincli to /playground

data_dir="./playground/bitcoin_data"

echo "Kill bitcoin process ..."
pkill bitcoind
pkill bitcoind

echo "Start bitcoin ..."
mkdir -p $data_dir 
start_bitcoin_command="./playground/bin/bitcoind -datadir=$data_dir -server -listen=1 -regtest -fallbackfee=0.001 -rpcuser=admin -rpcpassword=admin -daemon"
echo "Execute $start_bitcoin_command"
/bin/bash -c "$start_bitcoin_command"

echo "Sleep 10s"
for i in {1..10} 
do 
    sleep 1s
    printf "."
done
echo ""


echo "Loading default wallet ..."
create_wallet_command="./playground/bin/bitcoin-cli -chain=regtest -datadir=$data_dir -rpcuser=admin -rpcpassword=admin createwallet default"
echo "Execute $create_wallet_command"
eval $create_wallet_command

load_wallet_command="./playground/bin/bitcoin-cli -chain=regtest -datadir=$data_dir -rpcuser=admin -rpcpassword=admin loadwallet default"
echo "Execute $load_wallet_command"
eval $load_wallet_command

set -e

echo "Generating a block every 5s seconds. Press [CTRL+C] to stop.."

address_command="./playground/bin/bitcoin-cli -chain=regtest -datadir=$data_dir -rpcuser=admin -rpcpassword=admin getnewaddress"
echo "Execute $address_command"
address=`/bin/bash -c "$address_command"`

init_generate_command="./playground/bin/bitcoin-cli -chain=regtest -datadir=$data_dir -rpcuser=admin -rpcpassword=admin generatetoaddress 100 $address"
echo "Execute $init_generate_command"

eval $init_generate_command
generate_command="./playground/bin/bitcoin-cli -chain=regtest -datadir=$data_dir -rpcuser=admin -rpcpassword=admin generatetoaddress 1 $address"
echo "Execute $generate_command"

while :
do
        echo "Generating a new block `date '+%d/%m/%Y %H:%M:%S'` ..."
        eval $generate_command
        sleep 5s
done