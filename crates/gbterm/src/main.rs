use gb_core::cartridge::Cartridge;

fn main() {
    let path = std::env::args().nth(1).unwrap();

    let cartridge = Cartridge::from_rom_file(path).unwrap();

    println!("ROM Title: {}", cartridge.header().title);
    println!("Actual ROM size: {}", cartridge.rom_len());
    println!("Header ROM size: {}", cartridge.header().rom_size);
    println!("Checksum matches: {}", cartridge.header_checksum_match());
}
