BITCOIN_SOURCE_URL="https://github.com/bitlayer-org/bitcoin.git"
BITCOIN_SOURCE_BRANCH="nst_api"
CURRENT_DIR=$(pwd)
PID_FILE="$CURRENT_DIR/playground/pid_file"

if [ -f "./playground/bin/bitcoind" ] && [ -f "./playground/bin/bitcoin-cli" ]; then
    echo "Bitcoin already built. Skip building."
else 
    echo "Bitcoin not built. Building ..."
    # download bitcoin source and build
    mkdir -p ./playground/
    pushd playground
    git clone $BITCOIN_SOURCE_URL bitcoin 
    pushd bitcoin
    git checkout $BITCOIN_SOURCE_BRANCH
    cmake -B build
    pushd build
    make -j4 all
    popd
    popd
    popd

    # cp bitcoind and bitcoincli to playground
    mkdir -p ./playground/bin
    cp ./playground/bitcoin/build/bin/bitcoind ./playground/bin/
    cp ./playground/bitcoin/build/bin/bitcoin-cli ./playground/bin/
fi

data_dir="./playground/bitcoin_data"

echo "Kill bitcoin process ..."
if [ ! -f "$PID_FILE" ]; then
echo "PID file not found. Daemon may not be running."
else
    # Read the PID from the PID file
PID=$(cat "$PID_FILE")

# Check if the process is running
if ps -p "$PID" > /dev/null; then
    # Stop the daemon
    kill "$PID"
    echo "Daemon stopped with PID: $PID"
else
    echo "Daemon with PID $PID not found."
fi

# Remove the PID file
rm -f "$PID_FILE"
fi

echo "Start bitcoin ..."
mkdir -p $data_dir 
start_bitcoin_command="./playground/bin/bitcoind -datadir=$data_dir -server -listen=1 -regtest -fallbackfee=0.001 -rpcuser=admin -rpcpassword=admin -daemon -pid=$PID_FILE"
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
