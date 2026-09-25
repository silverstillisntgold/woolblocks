use woolblocks::*;

fn main() -> Result<(), WoolError> {
    println!("Starting program");
    //SingleTexture::new("white_wool.png").generate(ClientFetcher::release(), true)?;
    AllTextures.generate(ClientFetcher::release(), false)?;
    println!("Program complete");
    Ok(())
}
