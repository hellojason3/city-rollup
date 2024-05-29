pub const BLOCK_GROTH16_ENCODED_VERIFIER_DATA: [[u8; 80]; 6] = [
  hex_literal::hex!("b14314e4591e346d10a5e3ff6f27593e02d6c911bcfda1cdb554b29fa57863d6448d623f2dd9c3e3f6037840b522d48a4c7e92a74ff2275b2c23f2d8a07e265e8d4ace7748d37cfdab5139dd8e22ed72"),
  hex_literal::hex!("9c06800675aa1e198ad2f2e07370338ad768918f786556e92955f09a82b3987cf138d978096f8ba1d7d309cb230b97afa01ae7e52cec6d4154bc82fb38b5418bc0847c7b309db151b70b294c904ca62d"),
  hex_literal::hex!("dd39aa59fdf20b2fd02903d1f3a8b08bb6eec58bc6fdfcf87d37441d3ae6ea8fc0c9949c6859905000a83aebe0aad9b550d672c9c3849a7ce5cad295939c11c96daaf36db518ff802ebb4b36e3715515"),
  hex_literal::hex!("6aa989ee7392f2b64aceed795188b47df2dbbf3863e56bd59b2f0bea2c8fe03777d9c28d55ac2e1ccf4c4618f5383e062fdae7da1e4a4d87532e44ee3ef62eaa80e5990ed959f97e20c5b7e00d1080e1"),
  hex_literal::hex!("1991e77d0f38c0e925c51a8db4ceda19085a90ec39cb7fd747e8becb6ae6fac36ebf56694349ec7513a2af85d2241ab7ec6d8f7d42de14067efa2160d3cb71059388044478c3b8ddcb64bc53f1fd0464"),
  hex_literal::hex!("7d8805b159f0333feff9a1d4b7c0d969dcec8f82d61b18cfe83b9a6175d17203b394331b26f61899d73efe55d5b5a2de21d44cdb0fe2829bba8a195aa8700981cdb45bb357f278903a047cbd37a63285"),
];

// note: BLOCK_VERIFIER_DATA_0_SHA_256_HASH = sha256(BLOCK_VERIFIER_DATA[0])
pub const BLOCK_GROTH16_ENCODED_VERIFIER_DATA_0_SHA_256_HASH: [u8; 32] =
    hex_literal::hex!("f6ca27dd0a90211176f366fa360f99dd27d1d25fc44e11eb663bfdce80967154");
