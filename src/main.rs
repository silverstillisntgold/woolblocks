use woolblocks::*;

fn main() -> Result<(), WoolError> {
    println!("Starting program");
    AllTextures.generate(ClientFetcher::release(), true)?;
    println!("Program complete");
    Ok(())
}
