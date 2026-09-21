use woolblocks::*;

fn main() {
    let client_fetcher = ClientFetcher::default();
    AllTextures.generate(client_fetcher, true).unwrap();
}
