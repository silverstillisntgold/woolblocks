use woolblocks::*;

fn main() {
    println!("Starting program");
    Xbrz.generate(ClientFetcher::release(), false).unwrap();
    println!("Program complete");
}
