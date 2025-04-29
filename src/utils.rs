use bitcoin::{
    blockdata::transaction::OutPoint,
    key::{rand::rngs::OsRng, Keypair, PrivateKey, PublicKey},
    script::ScriptBuf,
    secp256k1::{All, Message, Secp256k1, SecretKey, XOnlyPublicKey},
    sighash::TapSighash,
    taproot::{TaprootBuilder, TaprootSpendInfo},
    Address, Amount,
};
use bitcoincore_rpc::jsonrpc::serde_json;
use std::str::FromStr;

/// Taproot information
#[derive(Clone)]
pub struct TaprootInfo {
    pub address: Address,
    pub scripts: Vec<ScriptBuf>,
    pub taproot_spend_info: TaprootSpendInfo,
}

/// Create a taproot address
pub fn create_taproot_address(scripts: Vec<ScriptBuf>, network: bitcoin::Network) -> TaprootInfo {
    build_taptree_with_script(scripts, network)
}

/// Build a taproot tree with a script
pub fn build_taptree_with_script(
    scripts: Vec<ScriptBuf>,
    network: bitcoin::Network,
) -> TaprootInfo {
    let internal_key = XOnlyPublicKey::from_str(
        "93c7378d96518a75448821c4f7c8f4bae7ce60f804d03d1f0628dd5dd0f5de51",
    )
    .unwrap();
    let taproot_spend_info = TaprootBuilder::new()
        .add_leaf(0, scripts[0].clone())
        .expect("unable to add leaf")
        .finalize(&Secp256k1::new(), internal_key)
        .expect("unable to finalize");
    let address = Address::p2tr_tweaked(taproot_spend_info.output_key(), network);
    TaprootInfo {
        address: address.clone(),
        scripts,
        taproot_spend_info,
    }
}

/// Generate a default transaction input
pub fn generate_default_tx_in(input: &Input) -> bitcoin::TxIn {
    bitcoin::TxIn {
        previous_output: input.outpoint,
        script_sig: ScriptBuf::new(),
        sequence: bitcoin::Sequence::MAX,
        witness: bitcoin::Witness::new(),
    }
}

/// Transaction input
pub struct Input {
    pub outpoint: OutPoint,
    pub _amount: Amount,
}

/// Signer info
pub struct SignerInfo {
    secp: Secp256k1<All>,
    _pk: PublicKey,
    _sk: SecretKey,
    keypair: Keypair,
    address: Address,
}

impl SignerInfo {
    fn generate_signer_info(
        sk: SecretKey,
        secp: Secp256k1<All>,
        network: bitcoin::Network,
    ) -> Self {
        let private_key = PrivateKey::new(sk, network);
        println!(
            "generate private key: {:?}, network: {:?}",
            serde_json::to_string(&private_key).unwrap(),
            network
        );
        let keypair = Keypair::from_secret_key(&secp, &sk);
        let (_, _parity) = XOnlyPublicKey::from_keypair(&keypair);
        let pubkey = PublicKey::from_private_key(&secp, &private_key);
        let address = Address::p2wpkh(&pubkey, network).unwrap();
        SignerInfo {
            _pk: private_key.public_key(&secp),
            secp,
            _sk: sk,
            keypair,
            address,
        }
    }

    pub fn new(network: bitcoin::Network) -> Self {
        let rng = &mut OsRng;
        let secp: Secp256k1<All> = Secp256k1::new();
        let (sk, _) = secp.generate_keypair(rng);

        Self::generate_signer_info(sk, secp, network)
    }

    pub fn sign_schnorr(&self, hash: TapSighash) -> Vec<u8> {
        let msg = Message::from_digest_slice(&hash[..]).expect("should be TapSighash");
        let signature = self.secp.sign_schnorr(&msg, &self.keypair);
        signature.serialize().to_vec()
    }

    pub fn address(&self) -> Address {
        self.address.clone()
    }
}
