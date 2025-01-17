use city_macros::const_concat_arrays;

use super::{
    BLOCK_GROTH16_ENCODED_VERIFIER_DATA, BLOCK_GROTH16_ENCODED_VERIFIER_DATA_0_SHA_256_HASH,
};
// set GROTH16_DISABLED_DEV_MODE = true in development ONLY, this disables the groth16 verifier for debugging circuits
pub const GROTH16_DISABLED_DEV_MODE: bool = false;


// DATA INSTRUCTIONS
const OP_PUSHBYTES_32: u8 = 0x20;
const OP_PUSHDATA1: u8 = 0x4c;

// Utility Instructions
const OP_SWAP: u8 = 0x7c;
const OP_DUP: u8 = 0x76;
const OP_SHA256: u8 = 0xa8;
const OP_EQUALVERIFY: u8 = 0x88;
const OP_1: u8 = 0x51;
const OP_2DROP: u8 = 0x6d;
const OP_NOP: u8 = 0x61;
// Action Instructions
const OP_0NOTEQUAL: u8 = 0x92;
pub const OP_CHECKGROTH16VERIFY_NOP: u8 = OP_0NOTEQUAL;
// note: OP_CHECKGROTH16VERIFY is 0xb3, but 0x61 is OP_NOP and can be used for testing without verifying proofs
pub const OP_CHECKGROTH16VERIFY: u8 = if GROTH16_DISABLED_DEV_MODE {
    OP_NOP
}else{
    0xb3
};

pub const GENESIS_STATE_HASH: [u8; 32] = [202, 236, 137, 190, 220, 171, 60, 231, 7, 152, 26, 111, 168, 109, 39, 184, 123, 44, 10, 115, 47, 238, 227, 113, 122, 173, 221, 103, 40, 135, 124, 0];

//  size = 3 + 1 + 32 + 1 + 5*(2+80) + 9 = 456
const STANDARD_BLOCK_SCRIPT_BODY: [u8; 456] = city_macros::const_concat_arrays!(
    [OP_SWAP, OP_DUP, OP_SHA256],
    [OP_PUSHBYTES_32],
    BLOCK_GROTH16_ENCODED_VERIFIER_DATA_0_SHA_256_HASH,
    [OP_EQUALVERIFY],
    [OP_PUSHDATA1, 80],
    BLOCK_GROTH16_ENCODED_VERIFIER_DATA[1],
    [OP_PUSHDATA1, 80],
    BLOCK_GROTH16_ENCODED_VERIFIER_DATA[2],
    [OP_PUSHDATA1, 80],
    BLOCK_GROTH16_ENCODED_VERIFIER_DATA[3],
    [OP_PUSHDATA1, 80],
    BLOCK_GROTH16_ENCODED_VERIFIER_DATA[4],
    [OP_PUSHDATA1, 80],
    BLOCK_GROTH16_ENCODED_VERIFIER_DATA[5],
    [
        OP_1,
        OP_CHECKGROTH16VERIFY, // OP_ACTION
        OP_2DROP,
        OP_2DROP,
        OP_2DROP,
        OP_2DROP,
        OP_2DROP,
        OP_2DROP,
        OP_1
    ]
);

// size = 3 + 1 + 32 + 1 + 5*(2+80) + 9 = 456
const GENESIS_BLOCK_SCRIPT_BODY: [u8; 456] = city_macros::const_concat_arrays!(
    [OP_SWAP, OP_DUP, OP_SHA256],
    [OP_PUSHBYTES_32],
    BLOCK_GROTH16_ENCODED_VERIFIER_DATA_0_SHA_256_HASH,
    [OP_EQUALVERIFY],
    [OP_PUSHDATA1, 80],
    BLOCK_GROTH16_ENCODED_VERIFIER_DATA[1],
    [OP_PUSHDATA1, 80],
    BLOCK_GROTH16_ENCODED_VERIFIER_DATA[2],
    [OP_PUSHDATA1, 80],
    BLOCK_GROTH16_ENCODED_VERIFIER_DATA[3],
    [OP_PUSHDATA1, 80],
    BLOCK_GROTH16_ENCODED_VERIFIER_DATA[4],
    [OP_PUSHDATA1, 80],
    BLOCK_GROTH16_ENCODED_VERIFIER_DATA[5],
    [
        OP_1,
        OP_CHECKGROTH16VERIFY_NOP, // OP_ACTION
        OP_2DROP,
        OP_2DROP,
        OP_2DROP,
        OP_2DROP,
        OP_2DROP,
        OP_2DROP,
        OP_1
    ]
);

pub const GENESIS_BLOCK_SCRIPT_TEMPLATE: [u8; 489] =
    const_concat_arrays!([OP_PUSHBYTES_32], [0u8; 32], GENESIS_BLOCK_SCRIPT_BODY);

pub const STANDARD_BLOCK_SCRIPT_TEMPLATE: [u8; 489] =
    const_concat_arrays!([OP_PUSHBYTES_32], [0u8; 32], STANDARD_BLOCK_SCRIPT_BODY);
