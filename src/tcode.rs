#[derive(Debug)]
pub enum Tcode {
    TcodeNull,
    TcodeOwn,
    TcodeDbr,
    TcodeIbr,
    TcodeErr,
    TcodeProgTraceSync,
    TcodeDbrSync,
    TcodeIbrSync,
    TcodeFull,
    TcodeIbrHist,
    TcodeIbrHistSync,
    TcodeRbr,
    TcodeProgTraceCorr
}

impl From<u8> for Tcode {
    fn from(value: u8) -> Self {
        match value {
            2 => Tcode::TcodeOwn,
            3 => Tcode::TcodeDbr,
            4 => Tcode::TcodeIbr,
            8 => Tcode::TcodeErr,
            9 => Tcode::TcodeProgTraceSync,
            10 => Tcode::TcodeDbrSync,
            11 => Tcode::TcodeIbrSync,
            27 => Tcode::TcodeFull,
            28 => Tcode::TcodeIbrHist,
            29 => Tcode::TcodeIbrHistSync,
            30 => Tcode::TcodeRbr,
            33 => Tcode::TcodeProgTraceCorr,
            _ => panic!("Invalid Tcode value: {}", value),
        }
    }
}