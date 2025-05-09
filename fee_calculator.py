from time import sleep
import requests
import json, sys

TOKEN = "7f71aec516fb4306e3385f5c46e75b475483d29c0721d44f352817316030080c"
TOKEN = "7f71aec516fb4306e3385f5c46e75b475483d29c0721d44f352817316030080c1"
CHAINUP_API = "https://docs.chainupcloud.com/blockchain-api/bitcoin/json-rpc-methods"


def get_blockchain_info():
    """
    Fetches blockchain information from the UniSat API.
    """
    response = requests.get(
        "https://open-api.unisat.io/v1/indexer/blockchain/info",
        headers={
            "Authorization": TOKEN,
            "Accept": "*/*",
        },
    )

    data = response.json()
    return data


def get_tx_info(txid: str):
    """
    Fetches transaction information from the UniSat API.
    """
    response = requests.get(
        f"https://open-api.unisat.io/v1/indexer/tx/{txid}",
        headers={"Authorization": TOKEN, "Accept": "*/*"},
    )

    data = response.json()
    return data


def get_block_info(block_height: int):
    """
    Fetches block information from the UniSat API.
    """
    response = requests.get(
        f"https://open-api.unisat.io/v1/indexer/block/{block_height}/txs?cursor=0&size=10000",
        headers={"Authorization": TOKEN, "Accept": "*/*"},
    )

    data = response.json()
    return data


# curl -sSL "https://mempool.space/api/block/{block_hash}/raw"
def get_raw_block_info(block_hash: str):
    """
    Fetches raw block information from the UniSat API.
    """
    response = requests.get(
        f"https://mempool.space/api/block/{block_hash}/raw",
        stream=True,
    )
    return response.content


# curl -sSL "https://mempool.space/api/tx/15e10745f15593a899cef391191bdd3d7c12412cc4696b7bcb669d0feadc8521"
def get_transaction_fee_and_weight(txid: str) -> tuple:
    """
    Fetches transaction fee information from the UniSat API.
    """
    response = requests.get(
        f"https://mempool.space/api/tx/{txid}",
    )

    # print("response", response.content)
    fee = response.json()["fee"]
    weight = response.json()["weight"]
    return (fee, weight)


# example: python3 fee_calculator.py 2dcbac5acc30028260dad6edbf574c3b98c4bbbb09182cfe8b7efd4ce8d90c9a 3
if __name__ == "__main__":
    # args read from command line
    # example: txid = "2dcbac5acc30028260dad6edbf574c3b98c4bbbb09182cfe8b7efd4ce8d90c9a"
    try:
        txid = sys.argv[1]
        # example: r = 3
        r = int(sys.argv[2])
    except IndexError:
        print(f"Usage: python3 {sys.argv[0]} <txid> <r>")
        print(
            f"Example: python3 {sys.argv[0]} 2dcbac5acc30028260dad6edbf574c3b98c4bbbb09182cfe8b7efd4ce8d90c9a 3"
        )
        sys.exit(1)

    print("Blockchain Information:", get_blockchain_info())
    get_tx_info_result = get_tx_info(txid)
    height = get_tx_info_result["data"]["height"]
    block_hash = get_tx_info_result["data"]["blkid"]
    idx = get_tx_info_result["data"]["idx"]
    print(
        f"Transaction Information for {txid}: at idx {idx} of {height} block {block_hash}"
    )

    raw_block = get_raw_block_info(block_hash)
    # save raw_block to file
    with open(f"{block_hash}.bin", "wb") as f:
        f.write(raw_block)

    sleep(2)
    block_info = get_block_info(height)
    tx_num = len(block_info["data"])

    # load fee_rate_set from file
    fee_rate_set = {}
    try:
        with open(f"{block_hash}_fee_rate_set.json", "r") as f:
            fee_rate_set = json.load(f)
        print(f"Loaded fee_rate_set from file, {len(fee_rate_set)} / {tx_num} entities")
    except FileNotFoundError:
        print("fee_rate_set file not found, creating a new one")

    # start from 1, remove coin base transaction
    for i in range(1, tx_num):
        id = block_info["data"][i]["txid"]
        if id in fee_rate_set:
            # print(f"Transaction {i}/{tx_num}: {id} already in fee_rate_set")
            continue

        sleep(0.1)
        print(f"Transaction {i}/{tx_num}: {id}", end=", ")

        # get transaction fee and weight
        try:
            (fee, weight) = get_transaction_fee_and_weight(id)
        except Exception as e:
            print(f"Error: {e}")
            continue
        vb_size = weight / 4
        fee_rate = fee / vb_size
        print(
            f"fee: {fee}, weight: {weight}, vb: {vb_size}, fee_rate: {fee_rate}",
        )
        fee_rate_set[id] = fee_rate

    # save fee_rate_set to file
    with open(f"{block_hash}_fee_rate_set.json", "w") as f:
        json.dump(fee_rate_set, f)

    assert fee_rate_set[txid] != 0, "fee_rate_set is empty"
    del fee_rate_set[txid]  # remove the non-standard transaction
    assert len(fee_rate_set) > 0, "fee_rate_set is empty"
    assert len(fee_rate_set) == tx_num - 2, "re-run the script to get all fee_rate_set"

    # get the median of fee_rate_set
    fee_rate_list = list(fee_rate_set.values())
    fee_rate_list.sort()
    median_fee_rate = fee_rate_list[len(fee_rate_list) // 2]
    print(f"Median fee rate: {median_fee_rate}")

    # get txid vbyte
    (fee, weight) = get_transaction_fee_and_weight(txid)
    tx_vb = weight / 4
    print(
        f"Transaction {txid} fee: {fee}, weight: {weight}, vb: {tx_vb}, fee_rate: {fee / tx_vb}"
    )

    # need to pay offchain
    offchain_pay = r * tx_vb * median_fee_rate - fee
    assert (
        offchain_pay > 0
    ), f"offchain_pay {offchain_pay} is negative, don't need to pay offchain"
    print(f"Offchain pay: {offchain_pay} for tx {txid} with {r} multiple rate")
