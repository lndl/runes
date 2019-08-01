mod mos6502;
use mos6502::program::Program;
use mos6502::cpu::CPU;
use std::convert::TryFrom;

fn main() {
    let cpu = CPU::new();
    let bytes = [
        0xa9, 0x01,
        0x8d, 0x00, 0x02,
        0xa9, 0x05,
        0x8d, 0x01, 0x02,
        0xa9, 0x08,
        0x8d, 0x02, 0x02].to_vec();

    match Program::try_from(bytes) {
        Ok(program) => {
            println!("{:?}", program);
            println!("{:?}", cpu)
        },
        Err(s)      => println!("{}", s),
    }
}
