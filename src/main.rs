// Reference: <https://tobiasvl.github.io/blog/write-a-chip-8-emulator/>

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
enum Px {
    Black = 0xff000000,
    White = 0xffffffff,
}

#[derive(Debug)]
struct Chip8 {
    memory: Box<[u8; 4096]>,
    display: Box<[Px; WIDTH * HEIGHT]>,
    pc: u16,
    ri: u16,
    delay_timer: u8,
    sound_timer: u8,
    rv: [u8; 16],
    stack: Vec<u16>,
}

impl Chip8 {
    fn new() -> Self {
        const FONT_DATA: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];
        let memory: Box<[u8; 4096]> = [0; 0x4f]
            .into_iter()
            .chain(FONT_DATA)
            .chain([0; 4096 - 0x4f - FONT_DATA.len()])
            .collect::<Vec<_>>()
            .into_boxed_slice()
            .try_into()
            .unwrap_or_else(|e: Box<[_]>| panic!("expected 4096 bytes but got {}", e.len()));
        Self {
            memory,
            display: Box::new([Px::Black; WIDTH * HEIGHT]),
            pc: 0x200,
            ri: 0x0,
            delay_timer: 0,
            sound_timer: 0,
            rv: [0; 16],
            stack: Vec::new(),
        }
    }
}

fn main() {
    let _ = Chip8::new();
}
