use woolblocks::*;

fn main() {
    println!("Starting program");
    PixelBlocks
        .generate(ClientFetcher::release(), false)
        .unwrap();
    println!("Program complete");
}
