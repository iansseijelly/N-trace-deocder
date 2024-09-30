#[derive(Debug)]
pub enum Btype {
    Bindirect = 0,  // indirect branch
    Btrap = 1,      // trap
    Bexception = 2, // exception
    Binterrupt = 3  // interrupt
}

impl From<u8> for Btype {
    fn from(value: u8) -> Self {
        match value {
            0 => Btype::Bindirect,
            1 => Btype::Btrap,
            2 => Btype::Bexception,
            3 => Btype::Binterrupt,
            _ => panic!("Invalid Btype value: {}", value),
        }
    }
}