use woolblocks::*;

fn main() {
    println!("Starting program");
    AllTextures
        .generate(ClientFetcher::release(), false)
        .unwrap();
    println!("Program complete");
}
