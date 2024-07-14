#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use k256::Secp256k1;
use stylus_sdk::{
    alloy_primitives::{address, keccak256, Address, FixedBytes, U256},
    alloy_sol_types::SolValue,
    stylus_proc::{entrypoint, external, sol_storage},
};

sol_storage! {
    #[entrypoint]
    struct Verifier {}
}

const VERIFIER: Address = address!("49Dc27B14CfEe893e4AC9E47984Ca6B2Dccd7A2E");

#[external]
impl Verifier {
    pub fn verify_proof(
        favorite_count: U256,
        post_id: U256,
        full_text: String,
        signature_a: FixedBytes<32>,
        signature_b: FixedBytes<32>,
        recid: u8,
    ) -> bool {
        let raw_data = (favorite_count, post_id, &full_text).abi_encode();
        let recovered = Address::from_raw_public_key(
            &(VerifyingKey::recover_from_prehash(
                keccak256(raw_data).as_slice(),
                &Signature::from_slice(
                    [signature_a.as_slice(), signature_b.as_slice()]
                        .concat()
                        .as_slice(),
                )
                .unwrap(),
                RecoveryId::from_byte(recid).unwrap(),
            )
            .unwrap()
            .to_sec1_bytes()),
        );
        return recovered == VERIFIER;
    }
}
