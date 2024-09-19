use rand::Rng;
use std::fs;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
const MEM_SIZE: usize = 4096;
const NUM_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const START_ADDR: u16 = 0x200;

#[derive(Debug)]
pub struct Chip8 {
    memory: [u8; MEM_SIZE],
    display: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    pc: u16,
    i: u16,
    stack: [u16; STACK_SIZE],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    v: [u8; NUM_REGISTERS],
    draw_flag: bool,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut memory = [0; MEM_SIZE];

        let font = [
            0xf0, 0x90, 0x90, 0x90, 0xf0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xf0, 0x10, 0xf0, 0x80, 0xf0, // 2
            0xf0, 0x10, 0xf0, 0x10, 0xf0, // 3
            0x90, 0x90, 0xf0, 0x10, 0x10, // 4
            0xf0, 0x80, 0xf0, 0x10, 0xf0, // 5
            0xf0, 0x80, 0xf0, 0x90, 0xf0, // 6
            0xf0, 0x10, 0x20, 0x40, 0x40, // 7
            0xf0, 0x90, 0xf0, 0x90, 0xf0, // 8
            0xf0, 0x90, 0xf0, 0x10, 0xf0, // 9
            0xf0, 0x90, 0xf0, 0x90, 0x90, // A
            0xe0, 0x90, 0xe0, 0x90, 0xe0, // B
            0xf0, 0x80, 0x80, 0x80, 0xf0, // C
            0xe0, 0x90, 0x90, 0x90, 0xe0, // D
            0xf0, 0x80, 0xf0, 0x80, 0xf0, // E
            0xf0, 0x80, 0xf0, 0x80, 0x80, // F
        ];

        (0x50..=0x09f)
            .into_iter()
            .zip(font.iter())
            .for_each(|(i, &d)| memory[i] = d);

        Chip8 {
            memory,
            display: [0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            pc: START_ADDR,
            i: 0,
            stack: [0; STACK_SIZE],
            delay_timer: 0,
            sound_timer: 0,
            v: [0; NUM_REGISTERS],
            sp: 0,
            draw_flag: false,
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        let rom = fs::read(path).expect("Failed to load ROM");
        let start_addr = START_ADDR as usize;
        self.memory[start_addr..start_addr + rom.len()].copy_from_slice(&rom);
    }

    pub fn run_cycle(&mut self) {
        self.draw_flag = false;
        let opcode = self.fetch_inst();
        self.execute_inst(opcode);
    }

    pub fn display(&self) -> &[u8] {
        &self.display
    }

    pub fn draw_flag(&self) -> bool {
        self.draw_flag
    }

    fn fetch_inst(&mut self) -> u16 {
        let pc = self.pc as usize;
        let byte1 = self.memory[pc] as u16;
        let byte2 = self.memory[pc + 1] as u16;

        self.pc += 2;

        (byte1 << 8) | byte2
    }

    fn execute_inst(&mut self, opcode: u16) {
        let x = ((opcode & 0x0f00) >> 8) as usize;
        let y = ((opcode & 0x00f0) >> 4) as usize;
        let n = (opcode & 0x000f) as usize;
        let nn = (opcode & 0x00ff) as u8;
        let nnn = opcode & 0x0fff;

        match (opcode & 0xf000) >> 12 {
            0x0 => match nn {
                0xe0 => self.clear_display(),
                0xee => self.ret(),
                _ => panic!("Unknown opcode: {opcode}"),
            },
            0x1 => self.jump(nnn),
            0x2 => self.call(nnn),
            0x3 => self.skip_if_reg_eq_imm(x, nn),
            0x4 => self.skip_if_reg_neq_imm(x, nn),
            0x5 => self.skip_if_reg_eq_reg(x, y),
            0x6 => self.set_reg_to_imm(x, nn),
            0x7 => self.add_imm_to_reg(x, nn),
            0x8 => match n {
                0x0 => self.set_reg_to_reg(x, y),
                0x1 => self.bitwise_or(x, y),
                0x2 => self.bitwise_and(x, y),
                0x3 => self.bitwise_xor(x, y),
                0x4 => self.add_reg_to_reg(x, y),
                0x5 => self.sub_reg_from_reg(x, y),
                0x6 => self.right_shift(x),
                0x7 => self.rsb_reg_from_reg(x, y),
                0xe => self.left_shift(x),
                _ => panic!("Unknown opcode: {opcode}"),
            },
            0x9 => self.skip_if_reg_neq_reg(x, y),
            0xa => self.set_i_to_addr(nnn),
            0xb => self.jump_with_offset(nnn),
            0xc => self.set_reg_to_rand(x, nn),
            0xd => self.draw(x, y, n),
            _ => panic!("Unknown opcode: {opcode}"),
        };
    }

    fn clear_display(&mut self) {
        self.display.fill(0);
        self.draw_flag = true;
    }

    fn ret(&mut self) {
        let sp = self.sp as usize;
        self.pc = self.stack[sp];
        self.stack[sp] = 0;
        self.sp -= 1;
    }

    fn jump(&mut self, addr: u16) {
        self.pc = addr;
    }

    fn call(&mut self, addr: u16) {
        self.stack[self.sp as usize] = self.pc;
        self.pc = addr;
        self.sp += 1;
    }

    fn skip_if_reg_eq_imm(&mut self, x: usize, nn: u8) {
        if self.v[x] == nn {
            self.pc += 2;
        }
    }

    fn skip_if_reg_neq_imm(&mut self, x: usize, nn: u8) {
        if self.v[x] != nn {
            self.pc += 2;
        }
    }

    fn skip_if_reg_eq_reg(&mut self, x: usize, y: usize) {
        if self.v[x] == self.v[y] {
            self.pc += 2;
        }
    }

    fn set_reg_to_imm(&mut self, x: usize, nn: u8) {
        self.v[x] = nn;
    }

    fn add_imm_to_reg(&mut self, x: usize, nn: u8) {
        self.v[x] = self.v[x].wrapping_add(nn);
    }

    fn set_reg_to_reg(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y];
    }

    fn bitwise_or(&mut self, x: usize, y: usize) {
        self.v[x] |= self.v[y];
    }

    fn bitwise_and(&mut self, x: usize, y: usize) {
        self.v[x] &= self.v[y];
    }

    fn bitwise_xor(&mut self, x: usize, y: usize) {
        self.v[x] ^= self.v[y];
    }

    fn add_reg_to_reg(&mut self, x: usize, y: usize) {
        let (res, overflow) = self.v[x].overflowing_add(self.v[y]);
        self.v[x] = res;
        self.v[0xf] = overflow as u8;
    }

    fn sub_reg_from_reg(&mut self, x: usize, y: usize) {
        let (res, overflow) = self.v[x].overflowing_sub(self.v[y]);
        self.v[x] = res;
        self.v[0xf] = !overflow as u8;
    }

    fn right_shift(&mut self, x: usize) {
        self.v[0xf] = self.v[x] & 0x1;
        self.v[x] >>= 1;
    }

    fn rsb_reg_from_reg(&mut self, x: usize, y: usize) {
        let (res, overflow) = self.v[y].overflowing_sub(self.v[y]);
        self.v[x] = res;
        self.v[0xf] = !overflow as u8;
    }

    fn left_shift(&mut self, x: usize) {
        self.v[0xf] = self.v[x] >> 7;
        self.v[x] <<= 1;
    }

    fn skip_if_reg_neq_reg(&mut self, x: usize, y: usize) {
        if self.v[x] != self.v[y] {
            self.pc += 2;
        }
    }

    fn set_i_to_addr(&mut self, addr: u16) {
        self.i = addr;
    }

    fn jump_with_offset(&mut self, addr: u16) {
        self.pc = addr + self.v[0] as u16;
    }

    fn set_reg_to_rand(&mut self, x: usize, nn: u8) {
        let num: u8 = rand::thread_rng().gen_range(0..=255);
        self.v[x] = num & nn;
    }

    fn draw(&mut self, x: usize, y: usize, n: usize) {
        let x = (self.v[x] as usize) % DISPLAY_WIDTH;
        let y = (self.v[y] as usize) % DISPLAY_HEIGHT;

        self.v[0xf] = 0;

        for row in 0..n {
            let sprite = self.memory[self.i as usize + row];
            for col in 0..8 {
                if (sprite & (0x80 >> col)) != 0 {
                    let index = x + col + ((y + row as usize) * DISPLAY_WIDTH);
                    if self.display[index] == 1 {
                        self.v[0xf] = 1;
                    }
                    self.display[index] ^= 1;
                }
                if x + col >= DISPLAY_WIDTH {
                    break;
                }
            }
            if y + row >= DISPLAY_HEIGHT {
                break;
            }
        }

        self.draw_flag = true;
    }
}
