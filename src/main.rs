mod utils;

use bitcoin::{
    absolute,
    blockdata::{
        script::Builder,
        transaction::{OutPoint, Transaction, TxOut},
    },
    sighash::SighashCache,
    taproot::LeafVersion,
    Amount, TapLeafHash,
};
use bitcoincore_rpc::{
    jsonrpc::{self, simple_http},
    Auth, Client, RawTx as _, RpcApi,
};
use clap::Parser;
use log::{debug, error, info, warn, LevelFilter};
use simple_logger::SimpleLogger;
use std::time::Duration;
use utils::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Bitcoin Core RPC URL (e.g., http://127.0.0.1:18443)
    #[arg(short, long)]
    endpoint: String,

    /// Bitcoin Core RPC username
    #[arg(short, long)]
    user: String,

    /// Bitcoin Core RPC password
    #[arg(short, long)]
    password: String,

    /// Script size in kilobytes (e.g., 500 for 500KB)
    #[arg(short, long, default_value_t = 500)]
    script_size_kb: u64,

    /// Name of Bitcoin RPC
    #[arg(short, long, default_value = "sendnsttransaction")]
    rpc_name: String,
}

fn main() {
    // Initialize the logger with the INFO level filter.
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let args = Args::parse();

    let url = &args.endpoint;
    let user = &args.user;
    let password = &args.password;
    let script_size_kb = args.script_size_kb;

    pub const FEE_AMOUNT: u64 = 2621549 * 3;

    let rpc = Client::new(
        url,
        Auth::UserPass(String::from(user), String::from(password)),
    )
    .unwrap();

    let amount = 10000000 as u64;
    let alice = SignerInfo::new();

    let mut script = Builder::new();
    let script_size = script_size_kb * 1024;
    for _ in 0..script_size / 2 {
        script = script
            .push_int(1)
            .push_opcode(bitcoin::opcodes::all::OP_DROP);
    }

    let script_bytes = script.into_script();
    info!("the byte size of script {}", script_bytes.as_bytes().len());
    let tapinfo = create_taproot_address(vec![script_bytes]);

    let txid = rpc
        .send_to_address(
            &tapinfo.address,
            Amount::from_sat(amount),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("send tx failed");

    let tx_result = rpc.get_transaction(&txid, None).expect("error");
    debug!("deposit tx detail {:?}", tx_result.details);

    let input = Input {
        outpoint: OutPoint {
            txid,
            vout: tx_result.details[0].vout,
        },
        _amount: Amount::from_sat(amount),
    };

    let output = TxOut {
        value: Amount::from_sat(amount - FEE_AMOUNT),
        script_pubkey: alice.address.script_pubkey(),
    };

    let mut tx = Transaction {
        version: bitcoin::transaction::Version(2),
        lock_time: absolute::LockTime::ZERO,
        input: vec![generate_default_tx_in(&input)],
        output: vec![output],
    };

    let mut sig_hash_cache = SighashCache::new(&tx);
    let sighash = sig_hash_cache
        .taproot_script_spend_signature_hash(
            0,
            &bitcoin::sighash::Prevouts::All(&vec![TxOut {
                value: Amount::from_sat(amount),
                script_pubkey: tapinfo.address.clone().script_pubkey(),
            }]),
            TapLeafHash::from_script(
                tapinfo.scripts[0].clone().as_script(),
                LeafVersion::TapScript,
            ),
            bitcoin::sighash::TapSighashType::Default,
        )
        .expect("error");

    let sig = alice.sign_schnorr(sighash);
    let spend_control_block = tapinfo
        .taproot_spend_info
        .control_block(&(
            tapinfo.scripts[0].clone(),
            bitcoin::taproot::LeafVersion::TapScript,
        ))
        .expect("error");

    tx.input[0].witness.push(sig);
    tx.input[0].witness.push(tapinfo.scripts[0].clone());
    tx.input[0].witness.push(&spend_control_block.serialize());

    let mut builder = simple_http::Builder::new()
        .url(url)
        .expect("invalid rpc info");
    builder = builder
        .auth(user, Some(password))
        .timeout(Duration::from_secs(100));
    let transport = jsonrpc::Client::with_transport(builder.build());
    let btc_client = Client::from_jsonrpc(transport);

    let block_count = 1;
    let _ = btc_client.generate_to_address(block_count as u64, &alice.address);
    info!("send tx {}", tx.txid());

    // Send the transaction using the `sendrawtransaction` RPC call.
    let txid_res = btc_client.call::<bitcoin::Txid>("sendrawtransaction", &[tx.raw_hex().into()]);
    match txid_res {
        Ok(txid) => {
            warn!("Transaction sent: {} by sendrawtransaction, this transaction is not a non-standard transaction", txid);
        }
        Err(e_) => {
            info!(
                "Error sending transaction by sendrawtransaction: {:?}, txid: {}",
                e_,
                tx.txid()
            );
        }
    }

    // Send the transaction using the `sendnsttransaction` RPC call.
    let txid_res = btc_client.call::<bitcoin::Txid>(&args.rpc_name, &[tx.raw_hex().into()]);
    match txid_res {
        Ok(txid) => {
            info!(
                "Transaction sent: {} by {}, this transaction is a non-standard transaction",
                txid, args.rpc_name
            );
            println!("\n ✅ the RPC {} passed the testing tool", args.rpc_name);
        }
        Err(e_) => {
            error!(
                "Error sending transaction by {}: {:?}, txid: {}",
                args.rpc_name,
                e_,
                tx.txid()
            );
            println!(
                "\n ❌ the RPC {} didn't pass the testing tool",
                args.rpc_name
            );
            std::process::exit(1);
        }
    }
}
