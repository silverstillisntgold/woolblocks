use woolblocks::*;

fn main() {
    println!("Starting program");
    Xbrz.generate(ClientFetcher::release(), true).unwrap();
    println!("Program complete");
}
