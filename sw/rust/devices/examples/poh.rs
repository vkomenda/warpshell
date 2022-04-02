extern crate warp_devices;

mod poh_core;

use poh_core::{DataBaseAddrs, PohCoreOps, PohCoreParam};
use warp_devices::{
    varium_c1100::VariumC1100,
    xdma::{DmaBuffer, XdmaOps},
};

const HASH_BYTES: usize = 32;

const ONE_HASH: [u8; HASH_BYTES] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
];

const HASH_OF_ONE_HASH: [u8; HASH_BYTES] = [
    0xec, 0x49, 0x16, 0xdd, 0x28, 0xfc, 0x4c, 0x10, 0xd7, 0x8e, 0x28, 0x7c, 0xa5, 0xd9, 0xcc, 0x51,
    0xee, 0x1a, 0xe7, 0x3c, 0xbf, 0xde, 0x08, 0xc6, 0xb3, 0x73, 0x24, 0xcb, 0xfa, 0xac, 0x8b, 0xc5,
];

const IN_HASH: [u8; HASH_BYTES] = [
    0x01, 0xba, 0x47, 0x19, 0xc8, 0x0b, 0x6f, 0xe9, 0x11, 0xb0, 0x91, 0xa7, 0xc0, 0x51, 0x24, 0xb6,
    0x4e, 0xee, 0xce, 0x96, 0x4e, 0x09, 0xc0, 0x58, 0xef, 0x8f, 0x98, 0x05, 0xda, 0xca, 0x54, 0x6b,
];

const OUT_HASH: [u8; HASH_BYTES] = [
    0x9c, 0x82, 0x72, 0x01, 0xb9, 0x40, 0x19, 0xb4, 0x2f, 0x85, 0x70, 0x6b, 0xc4, 0x9c, 0x59, 0xff,
    0x84, 0xb5, 0x60, 0x4d, 0x11, 0xca, 0xaf, 0xb9, 0x0a, 0xb9, 0x48, 0x56, 0xc4, 0xe1, 0xdd, 0x7a,
];

const NUM_HASHES: usize = 18;

impl PohCoreParam for VariumC1100 {
    const BASE_ADDR: u64 = 0x0005_0000;
}

fn main() {
    env_logger::init();

    let varium = VariumC1100::new().expect("cannot construct device");

    let mut hashes_buffer = DmaBuffer::new(NUM_HASHES);
    let mut num_iters_buffer = DmaBuffer::new(NUM_HASHES);
    let mut in_hash_le = IN_HASH.clone();
    in_hash_le.reverse();
    let mut out_hash_le = OUT_HASH.clone();
    out_hash_le.reverse();

    for i in 0..NUM_HASHES {
        hashes_buffer.get_mut().extend_from_slice(&in_hash_le);
        num_iters_buffer
            .get_mut()
            .extend_from_slice(&1u64.to_le_bytes());
    }

    let addrs = DataBaseAddrs {
        in_hashes_base: 0,
        num_iters_base: 4096,
        out_hashes_base: 8192,
    };

    println!("Init...");
    varium.init_poh(addrs, NUM_HASHES as u32).expect("init");

    // Write the inputs to the card.
    varium
        .dma_write(&hashes_buffer, addrs.in_hashes_base)
        .expect("write hashes");
    varium
        .dma_write(&num_iters_buffer, addrs.num_iters_base)
        .expect("write num_iters");

    println!("input hashes {}", hex::encode(hashes_buffer.get()));
    println!("Run...");
    varium.run_poh().expect("run");

    println!("Return...");
    varium
        .dma_read(&mut hashes_buffer, addrs.out_hashes_base)
        .expect("read hashes");

    println!("output hashes {}", hex::encode(hashes_buffer.get()));
    for i in 0..NUM_HASHES {
        let hash = &hashes_buffer.as_slice()[HASH_BYTES * i..HASH_BYTES * (i + 1)];
        println!("{}. got {}", i, hex::encode(hash));
        assert_eq!(hash, out_hash_le);
    }
    println!("Success!");
}
