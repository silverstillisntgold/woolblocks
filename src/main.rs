use woolblocks::*;

fn main() -> Result<(), WoolError> {
    println!("Starting program");
    AllTextures.generate(ClientFetcher::release(), false)?;
    println!("Program complete");
    Ok(())
}
