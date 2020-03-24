use std::fmt;
use std::collections::HashSet;

use mos6502::instruction::AddressingMode;
use mos6502::memory_map::MemoryMap;

#[derive(PartialEq, Eq, Hash, Debug)]
enum Flag {
    Carry,
    Zero,
    Interrupt,
    Decimal,
    Break,
    Overflow,
    Negative,
}

struct CPUFlags {
    flags: HashSet<Flag>
}

impl CPUFlags {
    fn new() -> CPUFlags {
        CPUFlags { flags: HashSet::new() }
    }

    fn bit(&self, f: Flag) -> u8 {
        if self.flags.contains(&f) { 1 } else { 0 }
    }

    fn has_set(&self, f: Flag) -> bool {
        self.flags.contains(&f)
    }

    fn set(&mut self, f: Flag, val: bool) {
        if val {
            self.flags.insert(f);
        } else {
            self.flags.remove(&f);
        }
    }
}

impl fmt::Debug for CPUFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Flag::*;

        write!(f, "{}", if self.has_set(Carry)     { "C" } else { "-" })?;
        write!(f, "{}", if self.has_set(Zero)      { "Z" } else { "-" })?;
        write!(f, "{}", if self.has_set(Interrupt) { "I" } else { "-" })?;
        write!(f, "{}", if self.has_set(Decimal)   { "D" } else { "-" })?;
        write!(f, "{}", if self.has_set(Break)     { "B" } else { "-" })?;
        write!(f, "{}", if self.has_set(Overflow)  { "V" } else { "-" })?;
        write!(f, "{}", if self.has_set(Negative)  { "N" } else { "-" })
    }
}

pub struct CPU {
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,

    flags: CPUFlags,

    mem: MemoryMap
}

impl CPU {
    pub fn new() -> Self {
        let flags = CPUFlags::new();
        let mem = MemoryMap::new();

        CPU {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFF,
            pc: 0,
            flags,
            mem
        }
    }

    pub fn load_program(&mut self, from: u16, program: Vec<u8>) {
        self.mem.copy(from, program);
    }

    pub fn exec(&mut self, from_address: u16) -> Result<(), &str> {
        use mos6502::instruction::Instruction;

        self.pc = from_address;
        loop {
            println!("{:?}", self);
            match Instruction::build(self.mem.from(self.pc)) {
                Some(instruction) => {
                    println!("Executing {:?}", instruction);
                    self.pc += instruction.bytesize() as u16;
                    instruction.exec(self);
                },
                None => {
                    self.pc += 1 as u16; // Acts as NOP
                }
            };
            if self.mem.is_overflowing(self.pc) {
                break
            }
        }

        Ok(())
    }

    pub fn adc(&mut self, am: &AddressingMode) {
        let c = self.flags.bit(Flag::Carry);
        let m = self.read_op(am);
        let a = self.a;
        if self.flags.has_set(Flag::Decimal) {
            let mut low_digit = (a as u16 & 0x0f) + (m as u16 & 0xf) + c as u16;
            let mut has_first_carry = false;
            if low_digit > 9 {
                low_digit = low_digit + 6;
                has_first_carry = true
            }

            let mut high_digit = (a as u16 >> 4) + (m as u16 >> 4);
            let mut has_last_carry = false;
            if has_first_carry { high_digit += 1; }
            if high_digit > 9 {
                high_digit = high_digit + 6;
                has_last_carry = true;
            }

            let r = (high_digit << 4) | (low_digit & 0x0f);

            self.flags.set(Flag::Carry, has_last_carry);
            self.flags.set(Flag::Overflow, (a ^ m) & 0x80 == 0 && (a ^ r as u8) & 0x80 == 0x80);
            self.flags.set(Flag::Zero, a == 0);
            self.flags.set(Flag::Negative, a < 0);

            self.a = r as u8;
        } else {
          let r = a as u16 + m as u16 + c as u16;

          self.flags.set(Flag::Carry, (r & 0x100) != 0);
          self.flags.set(Flag::Overflow, (a ^ m) & 0x80 == 0 && (a ^ r as u8) & 0x80 == 0x80);
          self.flags.set(Flag::Zero, a == 0);
          self.flags.set(Flag::Negative, a < 0);

          self.a = r as u8;
        }
    }

    pub fn and(&mut self, am: &AddressingMode) {
        let m = self.read_op(am);
        self.a = self.a & m;
        self.check_zf_with(self.a);
        self.check_nf_with(self.a);
    }

    pub fn asl(&mut self, am: &AddressingMode) {
        let mut o = self.read_op(am);
        self.check_nf_with(o);
        o = o << 0;
        self.write_op(am, o);
    }

    pub fn bcc(&mut self, am: &AddressingMode) {
        self.branch_on_flag(am, Flag::Carry, false);
    }

    pub fn bcs(&mut self, am: &AddressingMode) {
        self.branch_on_flag(am, Flag::Carry, true);
    }

    pub fn beq(&mut self, am: &AddressingMode) {
        self.branch_on_flag(am, Flag::Zero, true);
    }

    pub fn bit(&mut self, am: &AddressingMode) {
        let t = self.a & self.read_op(am);
        self.flags.set(Flag::Zero, t == 0);
        self.flags.set(Flag::Negative, (t & 0x8) == 1);
        self.flags.set(Flag::Overflow, (t & 0x4) == 1);
    }

    pub fn bmi(&mut self, am: &AddressingMode) {
        self.branch_on_flag(am, Flag::Negative, true);
    }

    pub fn bne(&mut self, am: &AddressingMode) {
        self.branch_on_flag(am, Flag::Zero, false);
    }

    pub fn bpl(&mut self, am: &AddressingMode) {
        self.branch_on_flag(am, Flag::Negative, false);
    }

    pub fn brk(&mut self, _am: &AddressingMode) {
        // TODO
        println!("BRK opcode not implemented!");
    }

    pub fn bvc(&mut self, am: &AddressingMode) {
        self.branch_on_flag(am, Flag::Overflow, false);
    }

    pub fn bvs(&mut self, am: &AddressingMode) {
        self.branch_on_flag(am, Flag::Overflow, true);
    }

    pub fn clc(&mut self, _am: &AddressingMode) {
        self.flags.set(Flag::Carry, false);
    }

    pub fn cld(&mut self, _am: &AddressingMode) {
        self.flags.set(Flag::Decimal, false);
    }

    pub fn cli(&mut self, _am: &AddressingMode) {
        self.flags.set(Flag::Interrupt, false);
    }

    pub fn clv(&mut self, _am: &AddressingMode) {
        self.flags.set(Flag::Overflow, false);
    }

    pub fn cmp(&mut self, am: &AddressingMode) {
        self.compare(am, self.a);
    }

    pub fn cpx(&mut self, am: &AddressingMode) {
        self.compare(am, self.x);
    }

    pub fn cpy(&mut self, am: &AddressingMode) {
        self.compare(am, self.y);
    }

    pub fn dec(&mut self, am: &AddressingMode) {
        let r = self.read_op(am) - 1;
        self.write_op(am, r);
        self.check_zf_with(r);
        self.check_zf_with(r);
    }

    pub fn dex(&mut self, am: &AddressingMode) {
        let r = self.read_op(am) - 1;
        self.x = r;
        self.check_zf_with(r);
        self.check_zf_with(r);
    }

    pub fn dey(&mut self, am: &AddressingMode) {
        let r = self.read_op(am) - 1;
        self.y = r;
        self.check_zf_with(r);
        self.check_zf_with(r);
    }

    pub fn eor(&mut self, am: &AddressingMode) {
        let r = self.a ^ self.read_op(am);
        self.a = r;
        self.check_zf_with(r);
        self.check_nf_with(r);
    }

    pub fn inc(&mut self, am: &AddressingMode) {
        let r = self.read_op(am) + 1;
        self.write_op(am, r);
        self.check_zf_with(r);
        self.check_zf_with(r);
    }

    pub fn inx(&mut self, am: &AddressingMode) {
        let r = self.read_op(am) + 1;
        self.x = r;
        self.check_zf_with(r);
        self.check_zf_with(r);
    }

    pub fn iny(&mut self, am: &AddressingMode) {
        let r = self.read_op(am) + 1;
        self.y = r;
        self.check_zf_with(r);
        self.check_zf_with(r);
    }

    pub fn jmp(&mut self, am: &AddressingMode) {
        self.pc = self.resolve_mem_address(am);
    }

    pub fn jsr(&mut self, am: &AddressingMode) {
        self.pc = self.resolve_mem_address(am);
        println!("JSR: TODO push onto stack return address!");
    }

    pub fn lda(&mut self, am: &AddressingMode) {
        let r = self.read_op(am);
        self.a = r;
        self.check_nf_with(r);
        self.check_zf_with(r);
    }

    pub fn ldx(&mut self, am: &AddressingMode) {
        let r = self.read_op(am);
        self.x = r;
        self.check_nf_with(r);
        self.check_zf_with(r);
    }

    pub fn ldy(&mut self, am: &AddressingMode) {
        let r = self.read_op(am);
        self.y = r;
        self.check_nf_with(r);
        self.check_zf_with(r);
    }

    pub fn lsr(&mut self, am: &AddressingMode) {
        let op = self.read_op(am);
        let r =  op >> 1;
        self.write_op(am, r);
        self.flags.set(Flag::Carry, (op & 0x01) == 1);
        self.check_nf_with(r);
        self.check_zf_with(r);
    }

    pub fn nop(&mut self, _am: &AddressingMode) {
        // Do nothing
    }

    pub fn ora(&mut self, am: &AddressingMode) {
        let r = self.a | self.read_op(am);
        self.a = r;
        self.check_nf_with(r);
        self.check_zf_with(r);
    }

    pub fn pha(&mut self, am: &AddressingMode) {
        // TODO
        println!("PHA opcode not implemented!");
    }

    pub fn php(&mut self, am: &AddressingMode) {
        // TODO
        println!("PHP opcode not implemented!");
    }

    pub fn pla(&mut self, am: &AddressingMode) {
        // TODO
        println!("PLA opcode not implemented!");
    }

    pub fn plp(&mut self, am: &AddressingMode) {
        // TODO
        println!("PLP opcode not implemented!");
    }

    pub fn rol(&mut self, am: &AddressingMode) {
        let op = self.read_op(am);
        let has_new_carry = (op & 0xf0) == 1;
        let mut r =  op << 1;
        r = r | self.flags.bit(Flag::Carry);
        self.write_op(am, r);
        self.flags.set(Flag::Carry, has_new_carry);
        self.check_nf_with(r);
        self.check_zf_with(r);
    }

    pub fn ror(&mut self, am: &AddressingMode) {
        let op = self.read_op(am);
        let has_new_carry = (op & 0x01) == 1;
        let mut r =  op >> 1;
        r = r | (self.flags.bit(Flag::Carry) << 7);
        self.write_op(am, r);
        self.flags.set(Flag::Carry, has_new_carry);
        self.check_nf_with(r);
        self.check_zf_with(r);
    }

    pub fn rti(&mut self, am: &AddressingMode) {
        // TODO
        println!("RTI opcode not implemented!");
    }

    pub fn rts(&mut self, am: &AddressingMode) {
        // TODO
        println!("RTS opcode not implemented!");
    }

    pub fn sbc(&mut self, am: &AddressingMode) {
        // TODO
        println!("SBC opcode not implemented!");
    }

    pub fn sec(&mut self, _am: &AddressingMode) {
        self.flags.set(Flag::Carry, true);
    }

    pub fn sed(&mut self, _am: &AddressingMode) {
        self.flags.set(Flag::Decimal, true);
    }

    pub fn sei(&mut self, _am: &AddressingMode) {
        self.flags.set(Flag::Interrupt, true);
    }

    pub fn sta(&mut self, am: &AddressingMode) {
        self.write_op(am, self.a);
    }

    pub fn stx(&mut self, am: &AddressingMode) {
        self.write_op(am, self.x);
    }

    pub fn sty(&mut self, am: &AddressingMode) {
        self.write_op(am, self.y);
    }

    pub fn tax(&mut self, am: &AddressingMode) {
        self.x = self.a;
        self.check_nf_with(self.x);
        self.check_zf_with(self.x);
    }

    pub fn tay(&mut self, am: &AddressingMode) {
        self.y = self.a;
        self.check_nf_with(self.y);
        self.check_zf_with(self.y);
    }

    pub fn tsx(&mut self, am: &AddressingMode) {
        self.x = self.sp;
        self.check_nf_with(self.x);
        self.check_zf_with(self.x);
    }

    pub fn txa(&mut self, am: &AddressingMode) {
        self.a = self.x;
        self.check_nf_with(self.a);
        self.check_zf_with(self.a);
    }

    pub fn txs(&mut self, am: &AddressingMode) {
        self.sp = self.x;
        self.check_nf_with(self.sp);
        self.check_zf_with(self.sp);
    }

    pub fn tya(&mut self, am: &AddressingMode) {
        self.a = self.y;
        self.check_nf_with(self.a);
        self.check_zf_with(self.a);
    }

    // Unofficial instructions

    pub fn lax(&mut self, am: &AddressingMode) {
        println!("LAX opcode not implemented!");
    }

    pub fn sax(&mut self, am: &AddressingMode) {
        println!("SAX opcode not implemented!");
    }

    pub fn axs(&mut self, am: &AddressingMode) {
        println!("AXS opcode not implemented!");
    }

    // Private methods

    fn compare(&mut self, am: &AddressingMode, reg: u8) {
        let m = self.read_op(am);
        let c = reg.wrapping_sub(m);
        self.flags.set(Flag::Carry, reg >= m);
        self.check_zf_with(c);
        self.check_zf_with(c);
    }

    fn branch_on_flag(&mut self, am: &AddressingMode, f: Flag, value: bool) {
        if self.flags.has_set(f) == value {
            self.pc = self.resolve_mem_address(am);
        }
    }

    fn check_zf_with(&mut self, val: u8) {
        self.flags.set(Flag::Zero, val == 0);
    }

    fn check_nf_with(&mut self, val: u8) {
        self.flags.set(Flag::Negative, (val & 0xf0) == 1);
    }

    fn read_op(&self, am: &AddressingMode) -> u8 {
        use self::AddressingMode::*;

        if am.has_to_access_memory() {
            let address = self.resolve_mem_address(am);
            self.mem.read(address)
        } else {
            match *am {
                AccA  => self.a,
                AccX  => self.x,
                AccY  => self.y,
                AccSP => self.sp,
                Immediate { o } => o,
                _ => panic!("Exhausted addressing modes in #read_op")
            }
        }
    }

    fn write_op(&mut self, am: &AddressingMode, value: u8) -> u8 {
        use self::AddressingMode::*;

        if am.has_to_access_memory() {
            let address = self.resolve_mem_address(am);
            self.mem.write(address, value)
        } else {
            match *am {
                AccA  => {
                    self.a = value;
                    value
                },
                AccX  => {
                    self.x = value;
                    value
                },
                AccY  => {
                    self.y = value;
                    value
                },
                AccSP => {
                    self.sp = value;
                    value
                },
                _ => panic!("Exhausted addressing modes in #write_op")
            }
        }
    }

    fn resolve_mem_address(&self, am: &AddressingMode) -> u16 {
        use self::AddressingMode::*;

        let address8to16 = |l: u8, h: u8| ((h as u16) << 8) | l as u16;

        match *am {
            Abs { l, h } => {
                let address = address8to16(l, h);
                address
            },
            AbsX { l, h } => {
                let address = address8to16(l, h) + address8to16(self.x, 0);
                address
            },
            AbsY { l, h } => {
                let address = address8to16(l, h) + address8to16(self.y, 0);
                address
            },
            ZeroPage { o } => {
                let address = address8to16(o, 0);
                address
            },
            ZeroPageX { o } => {
                let address = address8to16(o.wrapping_add(self.x), 0);
                address
            },
            ZeroPageY { o } => {
                let address = address8to16(o.wrapping_add(self.y), 0);
                address
            },
            Relative { o } => {
                let address = self.pc.wrapping_add(o as u16);
                address
            },
            Indirect { l, h } => {
                let ind_address = address8to16(l, h);
                let li = self.mem.read(ind_address);
                let hi = self.mem.read(ind_address + 1);
                let address = address8to16(li, hi);
                address
            },
            IndirectIndexed { a } => {
                let ind_address = address8to16(a, 0);
                let l_address = self.mem.read(ind_address);
                let h_address = self.mem.read(ind_address + 1);
                let address = address8to16(l_address, h_address) + address8to16(self.y, 0);
                address
            },
            IndexedIndirect { a } => {
                let ind_address = address8to16(a.wrapping_add(self.x), 0);
                println!("{:x}", ind_address);
                let l_address = self.mem.read(ind_address);
                let h_address = self.mem.read(ind_address + 1);
                let address = address8to16(l_address, h_address);
                address
            },
            _ => panic!("{} is not a memory type address mode")
        }
    }
}

impl fmt::Debug for CPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CPU State: <a: {:02x}, x: {:02x}, y: {:02x}, flags: {:?}, sp: {:02x}, pc: {:04x}>\n",
               self.a, self.x, self.y, self.flags, self.sp, self.pc)
    }
}
