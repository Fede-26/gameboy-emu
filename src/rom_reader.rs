// Struct to hold the ROM header
pub struct RomHeader {
    pub entry_point: [u8; 4],
    pub nintendo_logo: [u8; 48],
    pub title: [u8; 15],
    pub manufacturer_code: [u8; 4],
    pub cgb_flag: u8,
    pub new_licensee_code: [u8; 2],
    pub sgb_flag: u8,
    pub cartridge_type: u8,
    pub rom_size: u8,
    pub ram_size: u8,
    pub destination_code: u8,
    pub old_licensee_code: u8,
    pub mask_rom_version_number: u8,
    pub header_checksum: u8,
    pub global_checksum: [u8; 2],
}

impl RomHeader {
    // Function to create a new RomHeader struct from a vector of bytes that containes the ROM
    pub fn from_vec(vec: &Vec<u8>) -> RomHeader {
        let mut entry_point = [0; 4];
        let mut nintendo_logo = [0; 48];
        let mut title = [0; 15];
        let mut manufacturer_code = [0; 4];
        let mut new_licensee_code = [0; 2];
        let mut global_checksum = [0; 2];

        // Copy the data from the vector to the arrays
        entry_point.copy_from_slice(&vec[0x100..0x104]);
        nintendo_logo.copy_from_slice(&vec[0x104..0x134]);
        title.copy_from_slice(&vec[0x134..0x143]);
        manufacturer_code.copy_from_slice(&vec[0x13F..0x143]);
        new_licensee_code.copy_from_slice(&vec[0x144..0x146]);
        global_checksum.copy_from_slice(&vec[0x14E..0x150]);

        RomHeader {
            entry_point,
            nintendo_logo,
            title,
            manufacturer_code,
            cgb_flag: vec[0x143],
            new_licensee_code,
            sgb_flag: vec[0x146],
            cartridge_type: vec[0x147],
            rom_size: vec[0x148],
            ram_size: vec[0x149],
            destination_code: vec[0x14A],
            old_licensee_code: vec[0x14B],
            mask_rom_version_number: vec[0x14C],
            header_checksum: vec[0x14D],
            global_checksum,
        }
    }
}

// Enum that contains the cartridge types
#[derive(Debug)]
pub enum CartridgeType {
    RomOnly = 0x00,
    Mbc1 = 0x01,
    Mbc1Ram = 0x02,
    Mbc1RamBattery = 0x03,
    Mbc2 = 0x05,
    Mbc2Battery = 0x06,
    RomRam = 0x08,
    RomRamBattery = 0x09,
    Mmm01 = 0x0B,
    Mmm01Ram = 0x0C,
    Mmm01RamBattery = 0x0D,
    Mbc3TimerBattery = 0x0F,
    Mbc3TimerRamBattery = 0x10,
    Mbc3 = 0x11,
    //TODO: add the rest of the cartridge types
}

impl CartridgeType {
    // Function to convert a u8 to a CartridgeType
    pub fn from_u8(value: u8) -> Option<CartridgeType> {
        match value {
            0x00 => Some(CartridgeType::RomOnly),
            0x01 => Some(CartridgeType::Mbc1),
            0x02 => Some(CartridgeType::Mbc1Ram),
            0x03 => Some(CartridgeType::Mbc1RamBattery),
            0x05 => Some(CartridgeType::Mbc2),
            0x06 => Some(CartridgeType::Mbc2Battery),
            0x08 => Some(CartridgeType::RomRam),
            0x09 => Some(CartridgeType::RomRamBattery),
            0x0B => Some(CartridgeType::Mmm01),
            0x0C => Some(CartridgeType::Mmm01Ram),
            0x0D => Some(CartridgeType::Mmm01RamBattery),
            0x0F => Some(CartridgeType::Mbc3TimerBattery),
            0x10 => Some(CartridgeType::Mbc3TimerRamBattery),
            0x11 => Some(CartridgeType::Mbc3),
            _ => None,
        }
    }
}
