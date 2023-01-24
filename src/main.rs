// Reference: <https://tobiasvl.github.io/blog/write-a-chip-8-emulator/>
use std::{sync::mpsc, thread, time::Duration};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

#[derive(Debug)]
struct Chip8 {
    memory: Box<[u8; 4096]>,
    display: Box<[u8; WIDTH * HEIGHT]>,
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
            display: Box::new([0; WIDTH * HEIGHT]),
            pc: 0x200,
            ri: 0x0,
            delay_timer: 0,
            sound_timer: 0,
            rv: [0; 16],
            stack: Vec::new(),
        }
    }

    fn load_rom(&mut self, rom: &[u8]) {
        for (i, b) in (0x200..).zip(rom.iter()) {
            self.memory[i] = *b;
        }
    }
}

fn main() {
    let mut chip8 = Chip8::new();
    chip8.load_rom(&std::fs::read("IBM_Logo.ch8").unwrap());

    // The delay clock pulses at 60Hz to signal when to decrement the `delay_timer` and `sound_timer`.
    let (delay_clock_tx, delay_clock_rx) = mpsc::channel();
    let _delay_clock = thread::spawn(move || {
        let delay = Duration::from_secs_f64(1.0 / 60.0);
        loop {
            thread::sleep(delay);
            delay_clock_tx.send(()).expect("main thread owns receiver");
        }
    });

    // The clock pulses to ensure 700 instructions are FDE'd per second.
    let (clock_tx, clock_rx) = mpsc::channel();
    let _clock = thread::spawn(move || {
        let delay = Duration::from_secs_f64(1.0 / 700.0);
        loop {
            thread::sleep(delay);
            clock_tx.send(()).expect("main thread owns receiver");
        }
    });

    let (draw_tx, draw_rx) = mpsc::channel::<Box<[u8; WIDTH * HEIGHT]>>();
    let _draw = thread::spawn(move || {
        use std::io::Write;
        const CLEAR: &str = "\x1B[2J\x1B[1;1H";
        while let Ok(buf) = draw_rx.recv() {
            print!("{CLEAR}");
            for y in (0..HEIGHT).step_by(2) {
                for x in 0..WIDTH {
                    print!(
                        "{}",
                        match (buf[y * WIDTH + x], buf[(y + 1) * WIDTH + x]) {
                            (0, 0) => " ",
                            (1, 0) => "\u{2580}",
                            (0, 1) => "\u{2584}",
                            (1, 1) => "\u{2588}",
                            _ => unreachable!(),
                        }
                    );
                }
                println!();
            }
            drop(std::io::stdout().flush());
        }
    });

    // Event loop
    loop {
        if delay_clock_rx.try_recv().is_ok() {
            chip8.delay_timer = chip8.delay_timer.saturating_sub(1);
            chip8.sound_timer = chip8.sound_timer.saturating_sub(1);
        }

        if clock_rx.try_recv().is_err() {
            continue;
        }

        // Fetch
        let current_instruction = ((chip8.memory[chip8.pc as usize] as u16) << 8)
            + chip8.memory[chip8.pc as usize + 1] as u16;
        chip8.pc += 2;

        // Decode + Execute
        match current_instruction >> 12 & 0xf {
            0x0 => match current_instruction {
                // Clear screen.
                0x00e0 => *chip8.display = [0; WIDTH * HEIGHT],
                _ => unimplemented!("no implementation for {current_instruction:#X?}"),
            },
            // Jump to NNN immediate.
            0x1 => chip8.pc = current_instruction & 0x0fff,
            // Set register VX to NN.
            0x6 => chip8.rv[(current_instruction >> 8 & 0xf) as usize] = current_instruction as u8,
            // Add to register VX value NN.
            0x7 => chip8.rv[(current_instruction >> 8 & 0xf) as usize] += current_instruction as u8,
            // Set RI to NNN.
            0xA => chip8.ri = current_instruction & 0x0fff,
            // Draw DXYN.
            0xD => {
                let x = chip8.rv[(current_instruction >> 8 & 0xf) as usize] as usize % WIDTH;
                let y = chip8.rv[(current_instruction >> 4 & 0xf) as usize] as usize % HEIGHT;
                let height = current_instruction & 0xf;

                for (j, row) in (y..y + height as usize).zip(chip8.ri..chip8.ri + height) {
                    for (i, x) in (0..8).zip(x..x + 8) {
                        chip8.display[j * WIDTH + x] ^= (row >> (7-i) & 0x1) as u8;
                    }
                }
                draw_tx
                    .send(chip8.display.clone())
                    .expect("rx thread loops forever");
            }
            _ => unimplemented!("no implementation for {current_instruction:#X?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn init_memory() {
        drop(super::Chip8::new());
    }
}
