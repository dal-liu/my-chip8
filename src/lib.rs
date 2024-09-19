use std::fs;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
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
            draw_flag: false,
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        let rom = fs::read(path).expect("Failed to load ROM");
        let start_addr = START_ADDR as usize;
        self.memory[start_addr..start_addr + rom.len()].copy_from_slice(&rom);
    }

    pub fn run(&mut self) {
        self.draw_flag = false;
        let opcode = self.fetch_instruction();
        self.execute_instruction(opcode);
    }

    pub fn display(&self) -> &[u8] {
        &self.display
    }

    pub fn draw_flag(&self) -> bool {
        self.draw_flag
    }

    fn fetch_instruction(&mut self) -> u16 {
        let pc = self.pc as usize;
        let byte1 = self.memory[pc] as u16;
        let byte2 = self.memory[pc + 1] as u16;

        self.pc += 2;

        (byte1 << 8) | byte2
    }

    fn execute_instruction(&mut self, opcode: u16) {
        let x = ((opcode & 0x0f00) >> 8) as usize;
        let y = ((opcode & 0x00f0) >> 4) as usize;
        let n = (opcode & 0x000f) as usize;
        let nn = (opcode & 0x00ff) as u8;
        let nnn = opcode & 0x0fff;

        match (opcode & 0xf000) >> 12 {
            0x0 => match nn {
                0xe0 => self.clear_display(),
                _ => panic!("Unknown opcode: {opcode}"),
            },
            0x1 => self.jump(nnn),
            0x6 => self.set_reg_to_imm(x, nn),
            0x7 => self.add_imm_to_reg(x, nn),
            0xa => self.set_i_to_addr(nnn),
            0xd => self.draw(x, y, n),
            _ => panic!("Unknown opcode: {opcode}"),
        };
    }

    fn clear_display(&mut self) {
        self.display.fill(0);
        self.draw_flag = true;
    }

    fn jump(&mut self, addr: u16) {
        self.pc = addr;
    }

    fn set_reg_to_imm(&mut self, x: usize, nn: u8) {
        self.v[x] = nn;
    }

    fn add_imm_to_reg(&mut self, x: usize, nn: u8) {
        self.v[x] += nn;
    }

    fn set_i_to_addr(&mut self, addr: u16) {
        self.i = addr;
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
