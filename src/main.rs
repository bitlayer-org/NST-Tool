mod utils;

use bitcoin::{
    absolute,
    blockdata::{
        script::Builder,
        transaction::{OutPoint, Transaction, TxOut},
    },
    sighash::SighashCache,
    taproot::LeafVersion,
    Amount,
    Network::*,
    TapLeafHash,
};
use bitcoincore_rpc::{Auth, Client, RawTx as _, RpcApi};
use clap::Parser;
use log::{error, info, warn, LevelFilter};
use simple_logger::SimpleLogger;
use std::io::Write;
use std::str::FromStr;
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

    #[arg(short, long, default_value_t = Regtest)]
    network: bitcoin::Network,

    /// Script size in kilobytes (e.g., 500 for 500KB)
    #[arg(short, long, default_value_t = 500)]
    script_size_kb: u64,

    /// Name of Bitcoin RPC
    #[arg(short, long, default_value = "sendnsttransaction")]
    rpc_name: String,

    /// Dry run
    #[arg(short, long, default_value = "false")]
    dry_run: bool,

    /// Deposit Txid (only use when dry run is enabled)
    #[arg(short, long)]
    txid: Option<String>,

    // Deposit Vout (only use when dry run is enabled)
    #[arg(short, long)]
    vout: Option<u32>,
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
    let alice = SignerInfo::new(args.network);

    // generate a script with the given size
    let mut script = Builder::new();
    let script_size = script_size_kb * 1024;
    for _ in 0..script_size / 2 {
        script = script
            .push_int(1)
            .push_opcode(bitcoin::opcodes::all::OP_DROP);
    }

    let script_bytes = script.into_script();
    info!("the byte size of script {}", script_bytes.as_bytes().len());
    let tapinfo = create_taproot_address(vec![script_bytes], args.network);
    info!("deposit addresss: {:?}", tapinfo.address);

    // check if dry run is enabled
    let (txid, vout) = if args.dry_run {
        info!("Dry run enabled, not sending transaction.");
        if args.txid.is_none() || args.vout.is_none() {
            error!("Please provide deposit txid and vout for the dry run.");
            std::process::exit(1);
        }
        let txid = bitcoin::Txid::from_str(&args.txid.unwrap()).expect("invalid txid");
        (txid, args.vout.unwrap())
    } else {
        // send to the taproot address
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
        info!("txid of deposit transaction {:?}", tx_result.details);
        (txid, tx_result.details[0].vout)
    };

    // create a transaction to spend the deposit
    let input = Input {
        outpoint: OutPoint { txid, vout: vout },
        _amount: Amount::from_sat(amount),
    };

    let output = TxOut {
        value: Amount::from_sat(amount - FEE_AMOUNT),
        script_pubkey: alice.address().script_pubkey(),
    };

    let mut tx = Transaction {
        version: bitcoin::transaction::Version(2),
        lock_time: absolute::LockTime::ZERO,
        input: vec![generate_default_tx_in(&input)],
        output: vec![output],
    };

    // sign the transaction
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

    if args.dry_run {
        info!("Dry run enabled, not sending transaction.");
        info!("write raw of tx {} to raw_tx.txt", tx.txid());
        write!(
            std::fs::File::create("raw_tx.txt").unwrap(),
            "{}",
            tx.raw_hex()
        )
        .expect("write raw tx failed");
        return;
    }

    // send the transaction
    let block_count = 1;
    let _ = rpc.generate_to_address(block_count as u64, &alice.address());
    info!("send tx {}", tx.txid());

    // Send the transaction using the `sendrawtransaction` RPC call.
    let txid_res = rpc.call::<bitcoin::Txid>("sendrawtransaction", &[tx.raw_hex().into()]);
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
    let txid_res = rpc.call::<bitcoin::Txid>(&args.rpc_name, &[tx.raw_hex().into()]);
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
