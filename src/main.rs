use woolblocks::*;

fn main() -> Result<(), WoolError> {
    println!("Starting program");
    AllTextures.generate(ClientFetcher::default(), true)?;
    println!("Program complete");
    Ok(())
}
