extern crate warp_devices;

mod poh_core;

use poh_core::{DataBaseAddrs, PohCoreOps, PohCoreParam};
use warp_devices::{
    varium_c1100::VariumC1100,
    xdma::{DmaBuffer, XdmaOps},
};

const HASH_BYTES: usize = 32;

const IN_HASH: [u8; HASH_BYTES] = [
    0x01, 0xba, 0x47, 0x19, 0xc8, 0x0b, 0x6f, 0xe9, 0x11, 0xb0, 0x91, 0xa7, 0xc0, 0x51, 0x24, 0xb6,
    0x4e, 0xee, 0xce, 0x96, 0x4e, 0x09, 0xc0, 0x58, 0xef, 0x8f, 0x98, 0x05, 0xda, 0xca, 0x54, 0x6b,
];

const OUT_HASH: [u8; HASH_BYTES] = [
    0x9c, 0x82, 0x72, 0x01, 0xb9, 0x40, 0x19, 0xb4, 0x2f, 0x85, 0x70, 0x6b, 0xc4, 0x9c, 0x59, 0xff,
    0x84, 0xb5, 0x60, 0x4d, 0x11, 0xca, 0xaf, 0xb9, 0x0a, 0xb9, 0x48, 0x56, 0xc4, 0xe1, 0xdd, 0x7a,
];

impl PohCoreParam for VariumC1100 {
    const BASE_ADDR: u64 = 0x0005_0000;
}

fn main() {
    env_logger::init();

    let varium = VariumC1100::new().expect("cannot construct device");

    let mut hashes_buffer = DmaBuffer::new(HASH_BYTES);
    let mut num_iters_buffer = DmaBuffer::new(8);

    hashes_buffer.get_mut().extend_from_slice(&IN_HASH);
    num_iters_buffer
        .get_mut()
        .extend_from_slice(&1u64.to_le_bytes());

    let addrs = DataBaseAddrs {
        in_hashes_base: 0,
        num_iters_base: 4096,
        out_hashes_base: 8192,
    };

    varium.init_poh(addrs, 1).expect("init");

    // Write the inputs to the card.
    varium
        .dma_write(&hashes_buffer, addrs.in_hashes_base)
        .expect("write hashes");
    varium
        .dma_write(&num_iters_buffer, addrs.num_iters_base)
        .expect("write num_iters");

    varium.run_poh().expect("run");

    std::thread::sleep(std::time::Duration::from_secs(1));
    varium
        .dma_read(&mut hashes_buffer, addrs.out_hashes_base)
        .expect("read hashes");

    let hash = &hashes_buffer.as_slice()[0..HASH_BYTES];
    println!("output hash {}", hex::encode(hash));
    assert_eq!(hash, OUT_HASH);

    // let mut buffer = DmaBuffer::new(12288);
    // for i in 0..12288 {
    //     buffer.get_mut().push(1);
    // }
    // varium.dma_write(&buffer, 0).expect("write mem");
    // for i in 0..12288 {
    //     buffer.get_mut().push(0);
    // }
    // varium.dma_read(&mut buffer, 0).expect("read mem");

    // let mem = &buffer.as_slice()[0..12288];
    // println!("mem {}", hex::encode(mem));
}
