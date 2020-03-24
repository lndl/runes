use std::fmt;

use mos6502::cpu::CPU;

#[derive(Debug)]
enum Opcode {
    ADC, AND, ASL, AXS, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS, CLC,
    CLD, CLI, CLV, CMP, CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JMP, JSR,
    LAX, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA, PLP, ROL, ROR, RTI, RTS,
    SAX, SBC, SEC, SED, SEI, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA, ___
}

pub enum AddressingMode {
    Implicit,
    Abs { l: u8, h: u8 }, AbsX { l: u8, h: u8 }, AbsY { l: u8, h: u8 },
    ZeroPage { o: u8 }, ZeroPageX { o: u8 }, ZeroPageY { o: u8 },
    Relative { o: i8 },
    AccA, AccX, AccY, AccSP,
    Indirect { l: u8, h: u8 },
    IndirectIndexed { a: u8 },
    IndexedIndirect { a: u8 },
    Immediate { o : u8 },
}

impl AddressingMode {
    pub fn has_to_access_memory(&self) -> bool {
        match *self {
            Abs { .. } | AbsX { .. } | AbsY { .. } |
            ZeroPage { .. } | ZeroPageX { .. } |
            ZeroPageY { .. } | Relative { .. } |
            Indirect { .. } | IndirectIndexed { .. } |
            IndexedIndirect { .. } => true,
            _ => false
        }
    }

    pub fn has_to_access_registers(&self) -> bool {
        match *self {
            AccA | AccX | AccY | AccSP => true,
            _ => false
        }
    }
}

// Absoluters
fn abs(chunk: &[u8])     -> AddressingMode { Abs { l: chunk[1], h: chunk[2] } }
fn absx(chunk: &[u8])    -> AddressingMode { AbsX { l: chunk[1], h: chunk[2] } }
fn absx_ec(chunk: &[u8]) -> AddressingMode { AbsX { l: chunk[1], h: chunk[2] } }
fn absy(chunk: &[u8])    -> AddressingMode { AbsY { l: chunk[1], h: chunk[2] } }
fn absy_ec(chunk: &[u8]) -> AddressingMode { AbsY { l: chunk[1], h: chunk[2] } }
// Zero Page
fn zero_page(chunk: &[u8])   -> AddressingMode { ZeroPage { o: chunk[1] } }
fn zero_page_x(chunk: &[u8]) -> AddressingMode { ZeroPageX { o: chunk[1] } }
fn zero_page_y(chunk: &[u8]) -> AddressingMode { ZeroPageY { o: chunk[1] } }
// Immediate
fn imm8(chunk: &[u8]) -> AddressingMode { Immediate { o : chunk[1] } }
// Accumulators
fn reg_a(_chunk: &[u8])  -> AddressingMode { AccA  }
fn reg_x(_chunk: &[u8])  -> AddressingMode { AccX  }
fn reg_y(_chunk: &[u8])  -> AddressingMode { AccY  }
fn reg_sp(_chunk: &[u8]) -> AddressingMode { AccSP }
//Indirect
fn indirect(chunk: &[u8]) -> AddressingMode { Indirect { l: chunk[1], h: chunk[2] } }
// Indexed-Indirect
fn indexed_indirect(chunk: &[u8])    -> AddressingMode { IndexedIndirect { a: chunk[1] } }
fn indexed_indirect_ec(chunk: &[u8]) -> AddressingMode { IndexedIndirect { a: chunk[1] } }
// Indirect-Indexed
fn indirect_indexed(chunk: &[u8])    -> AddressingMode { IndirectIndexed { a: chunk[1] } }
fn indirect_indexed_ec(chunk: &[u8]) -> AddressingMode { IndirectIndexed { a: chunk[1] } }
// Relative
fn rel(chunk: &[u8])   -> AddressingMode { Relative { o : chunk[1] as i8 } }
// Implicit
fn none(_chunk: &[u8]) -> AddressingMode { Implicit }

impl fmt::Debug for AddressingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Implicit              => write!(f, ""),
            Abs { l, h }          => write!(f, "${:02x}{:02x}", h, l),
            AbsX { l, h }         => write!(f, "${:02x}{:02x}, X", h, l),
            AbsY { l, h }         => write!(f, "${:02x}{:02x}, Y", h, l),
            ZeroPage { o }        => write!(f, "${:02x}", o),
            ZeroPageX { o }       => write!(f, "${:02x}, X", o),
            ZeroPageY { o }       => write!(f, "${:02x}, Y", o),
            Relative { o }        => write!(f, "* ${:02x}", o),
            AccA                  => write!(f, "A"),
            AccX                  => write!(f, "X"),
            AccY                  => write!(f, "Y"),
            AccSP                 => write!(f, "SP"),
            Indirect { l, h }     => write!(f, "(${:02x}{:02x})", h, l),
            IndirectIndexed { a } => write!(f, "(${:02x}), Y", a),
            IndexedIndirect { a } => write!(f, "(${:02x}, X)", a),
            Immediate { o }       => write!(f, "#${:02x}", o)
        }
    }
}

pub struct Instruction {
    bytecode: u8,
    opcode: Opcode,
    mode: AddressingMode,
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {:?}", self.opcode, self.mode)
    }
}

impl Instruction {
    pub fn bytesize(&self) -> u8 {
        match &self.mode {
            Implicit                                               => 1,
            Abs { .. } | AbsX { .. } | AbsY { .. }                 => 3,
            ZeroPage { .. } | ZeroPageX { .. } | ZeroPageY { .. }  => 2,
            Relative { .. }                                        => 2,
            AccA { .. } | AccX { .. } | AccY { .. } | AccSP { .. } => 1,
            Indirect { .. }                                        => 3,
            IndexedIndirect { .. }                                 => 2,
            IndirectIndexed { .. }                                 => 2,
            Immediate { .. }                                       => 2
        }
    }
}

use self::Opcode::*;
use self::AddressingMode::*;

#[macro_export]
macro_rules! instructions_map {
    ( $( $bytecode:expr => $opcode:ident | $addressing_mode_constructor:path | $code:ident ),* ) => {
        impl Instruction {
            pub fn build(chunk: &[u8]) -> Option<Instruction> {
                let bytecode = chunk[0];

                match bytecode {
                    $( $bytecode => Some( Instruction { opcode: $opcode, mode: $addressing_mode_constructor(chunk), bytecode: $bytecode } ), )*
                               _ => None
                }
            }

            pub fn exec(&self, cpu: &mut CPU) {
                match self.bytecode {
                    $( $bytecode => cpu.$code(&self.mode), )*
                               _ => cpu.nop(&self.mode)
                };
            }
        }
    }
}


/* MOS6502 CPU's instruction set.
 *
 * bytecode => opcode | addressing_mode | cpu_method
 *
 */
instructions_map!(
    0x65 => ADC | zero_page | adc,
    0x6D => ADC | abs | adc,
    0x75 => ADC | zero_page_x | adc,
    0x69 => ADC | imm8 | adc,
    0x79 => ADC | absy | adc,
    0x7D => ADC | absx | adc,
    0x71 => ADC | indirect_indexed | adc,
    0x61 => ADC | indexed_indirect | adc,

    0x25 => AND | zero_page | and,
    0x2D => AND | abs | and,
    0x35 => AND | zero_page_x | and,
    0x29 => AND | imm8 | and,
    0x3D => AND | absx | and,
    0x39 => AND | absy | and,
    0x21 => AND | indexed_indirect | and,
    0x31 => AND | indirect_indexed | and,
    0x06 => ASL | zero_page | asl,
    0x0E => ASL | abs | asl,
    0x16 => ASL | zero_page_x | asl,
    0x1E => ASL | absx_ec | asl,
    0x0A => ASL | reg_a | asl,
    0xB0 => BCS | rel | bcs,
    0x90 => BCC | rel | bcc,
    0xF0 => BEQ | rel | beq,
    0xD0 => BNE | rel | bne,
    0x30 => BMI | rel | bmi,
    0x10 => BPL | rel | bpl,
    0x50 => BVC | rel | bvc,
    0x70 => BVS | rel | bvs,
    0x24 => BIT | zero_page | bit,
    0x2C => BIT | abs | bit,

    0x00 => BRK | none | brk,
    0x18 => CLC | none | clc,
    0xD8 => CLD | none | cld,
    0x58 => CLI | none | cli,
    0xB8 => CLV | none | clv,
    0xC5 => CMP | zero_page | cmp,
    0xCD => CMP | abs | cmp,
    0xD5 => CMP | zero_page_x | cmp,
    0xC9 => CMP | imm8 | cmp,
    0xD9 => CMP | absy | cmp,
    0xDD => CMP | absx | cmp,
    0xC1 => CMP | indexed_indirect | cmp,
    0xD1 => CMP | indirect_indexed | cmp,
    0xE4 => CPX | zero_page | cpx,
    0xEC => CPX | abs | cpx,
    0xE0 => CPX | imm8 | cpx,
    0xC4 => CPY | zero_page | cpy,
    0xCC => CPY | abs | cpy,
    0xC0 => CPY | imm8 | cpy,
    0xC6 => DEC | zero_page | dec,
    0xCE => DEC | abs | dec,
    0xD6 => DEC | zero_page_x | dec,
    0xDE => DEC | absx_ec | dec,
    0xCA => DEX | none | dex,
    0x88 => DEY | none | dey,
    0x45 => EOR | zero_page | eor,
    0x4D => EOR | abs | eor,
    0x55 => EOR | zero_page_x | eor,
    0x59 => EOR | absy | eor,
    0x5D => EOR | absx | eor,
    0x49 => EOR | imm8 | eor,
    0x51 => EOR | indirect_indexed | eor,
    0x41 => EOR | indexed_indirect | eor,
    0xE6 => INC | zero_page | inc,
    0xEE => INC | abs | inc,
    0xF6 => INC | zero_page_x | inc,
    0xFE => INC | absx_ec | inc,
    0xE8 => INX | reg_x | inx,
    0xC8 => INY | reg_y | iny,

    0x6C => JMP | indirect | jmp,
    0x4C => JMP | abs | jmp,
    0x20 => JSR | abs | jsr,
    0xA5 => LDA | zero_page | lda,
    0xAD => LDA | abs | lda,
    0xB5 => LDA | zero_page_x | lda,
    0xB9 => LDA | absy | lda,
    0xBD => LDA | absx | lda,
    0xA9 => LDA | imm8 | lda,
    0xB1 => LDA | indirect_indexed | lda,
    0xA1 => LDA | indexed_indirect | lda,
    0xA6 => LDX | zero_page | ldx,
    0xAE => LDX | abs | ldx,
    0xB6 => LDX | zero_page_y | ldx,
    0xA2 => LDX | imm8 | ldx,
    0xBE => LDX | absy | ldx,
    0xA4 => LDY | zero_page | ldy,
    0xAC => LDY | abs | ldy,
    0xB4 => LDY | zero_page_x | ldy,
    0xBC => LDY | absx | ldy,
    0xA0 => LDY | imm8 | ldy,
    0x46 => LSR | zero_page | lsr,
    0x4E => LSR | abs | lsr,
    0x56 => LSR | zero_page_x | lsr,
    0x5E => LSR | absx_ec | lsr,
    0x4A => LSR | reg_a | lsr,
    0xEA => NOP | none | nop,
    0x05 => ORA | zero_page | ora,
    0x0D => ORA | abs | ora,
    0x15 => ORA | zero_page_x | ora,
    0x09 => ORA | imm8 | ora,
    0x1D => ORA | absx | ora,
    0x19 => ORA | absy | ora,
    0x11 => ORA | indirect_indexed | ora,
    0x01 => ORA | indexed_indirect | ora,
    0x48 => PHA | none | pha,
    0x08 => PHP | none | php,
    0x68 => PLA | none | pla,
    0x28 => PLP | none | plp,
    0x26 => ROL | zero_page | rol,
    0x2E => ROL | abs | rol,
    0x36 => ROL | zero_page_x | rol,
    0x3E => ROL | absx_ec | rol,
    0x2A => ROL | reg_a | rol,
    0x66 => ROR | zero_page | ror,
    0x6E => ROR | abs | ror,
    0x76 => ROR | zero_page_x | ror,
    0x7E => ROR | absx_ec | ror,
    0x6A => ROR | reg_a | ror,
    0x40 => RTI | none | rti,
    0x60 => RTS | none | rts,
    0xE5 => SBC | zero_page | sbc,
    0xED => SBC | abs | sbc,
    0xF5 => SBC | zero_page_x | sbc,
    0xE9 => SBC | imm8 | sbc,
    0xF9 => SBC | absy | sbc,
    0xFD => SBC | absx | sbc,
    0xE1 => SBC | indexed_indirect | sbc,
    0xF1 => SBC | indirect_indexed | sbc,
    0x38 => SEC | none | sec,
    0xF8 => SED | none | sed,
    0x78 => SEI | none | sei,
    0x85 => STA | zero_page | sta,
    0x8D => STA | abs | sta,
    0x95 => STA | zero_page_x | sta,
    0x99 => STA | absy_ec | sta,
    0x9D => STA | absx_ec | sta,
    0x91 => STA | indirect_indexed_ec | sta,
    0x81 => STA | indexed_indirect_ec | sta,
    0x86 => STX | zero_page | stx,
    0x8E => STX | abs | stx,
    0x96 => STX | zero_page_y | stx,
    0x84 => STY | zero_page | sty,
    0x8C => STY | abs | sty,
    0x94 => STY | zero_page_x | sty,
    0xAA => TAX | reg_a | tax,
    0xA8 => TAY | reg_a | tay,
    0xBA => TSX | reg_sp | tsx,
    0x8A => TXA | reg_x | txa,
    0x9A => TXS | none | txs,
    0x98 => TYA | reg_y | tya,
    // Unofficial opcodes.
    0xA7 => LAX | zero_page | lax,
    0xB7 => LAX | zero_page_y | lax,
    0xA3 => LAX | indexed_indirect | lax,
    0xB3 => LAX | indirect_indexed | lax,
    0xAF => LAX | abs | lax,
    0xBF => LAX | absy | lax,
    0x87 => SAX | zero_page | sax,
    0x97 => SAX | zero_page_x | sax,
    0x83 => SAX | indexed_indirect | sax,
    0x8F => SAX | abs | sax,
    // Alternative name: SBX, SAX
    0xCB => AXS | imm8 | axs
);
