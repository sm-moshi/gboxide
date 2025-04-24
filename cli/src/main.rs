/// gboxide CLI
///
/// Provides subcommands for running and testing Game Boy ROMs.
use anyhow::Context;
use clap::{Parser, Subcommand};
use core_lib::{cpu::CPU, mmu::MMU};
use std::process;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Enable debug output globally (for troubleshooting and development)
    #[arg(
        long,
        global = true,
        help = "Enable debug output globally (for troubleshooting and development)"
    )]
    debug: bool,
    /// Enable verbose output globally (for more detailed logs and diagnostics)
    #[arg(
        long,
        global = true,
        help = "Enable verbose output globally (for more detailed logs and diagnostics)"
    )]
    verbose: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a Game Boy ROM
    Run {
        /// Path to the ROM file
        #[arg(value_name = "ROM_PATH")]
        rom_path: std::path::PathBuf,
        /// Run in headless mode (no window)
        #[arg(long, default_value_t = false)]
        headless: bool,
    },
    /// Run a test ROM in headless mode and print serial output (for blargg/mooneye tests)
    Test {
        /// Path to the test ROM file
        #[arg(value_name = "ROM_PATH")]
        rom_path: std::path::PathBuf,
    },
}

struct EmulatorApp {
    window: Option<Window>,
    mmu: MMU,
    cpu: CPU,
    debug: bool,
    _verbose: bool,
    should_exit: bool,
}

impl EmulatorApp {
    fn map_key(key: &Key) -> Option<core_lib::mmu::GameBoyButton> {
        match key {
            Key::Named(NamedKey::ArrowUp) => Some(core_lib::mmu::GameBoyButton::Up),
            Key::Named(NamedKey::ArrowDown) => Some(core_lib::mmu::GameBoyButton::Down),
            Key::Named(NamedKey::ArrowLeft) => Some(core_lib::mmu::GameBoyButton::Left),
            Key::Named(NamedKey::ArrowRight) => Some(core_lib::mmu::GameBoyButton::Right),
            Key::Character(s) if s.eq_ignore_ascii_case("z") => {
                Some(core_lib::mmu::GameBoyButton::A)
            }
            Key::Character(s) if s.eq_ignore_ascii_case("x") => {
                Some(core_lib::mmu::GameBoyButton::B)
            }
            Key::Named(NamedKey::Enter) => Some(core_lib::mmu::GameBoyButton::Start),
            Key::Named(NamedKey::Shift) => Some(core_lib::mmu::GameBoyButton::Select),
            _ => None,
        }
    }
}

impl ApplicationHandler for EmulatorApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window = match event_loop.create_window(WindowAttributes::default()) {
                Ok(w) => w,
                Err(e) => panic!("Failed to create window: {e}"),
            };
            self.window = Some(window);
        }
    }
    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.should_exit = true;
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(button) = Self::map_key(&event.logical_key) {
                    let pressed = event.state == ElementState::Pressed;
                    self.mmu.update_joypad(button, pressed);
                }
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        for _ in 0..1000 {
            if self.should_exit {
                return;
            }
            let opcode = self.mmu.read(self.cpu.regs.pc);
            if opcode == 0x76 {
                self.should_exit = true;
                return;
            }
            match self.cpu.step(&mut self.mmu) {
                Ok(cycles) => self.mmu.step(cycles),
                Err(e) => {
                    eprintln!("[ERROR] CPU step failed: {e}");
                    self.should_exit = true;
                    return;
                }
            }
            if self.debug {
                // Print debug info (placeholder)
                eprintln!("PC: {:04X}", self.cpu.regs.pc);
            }
        }
    }
}

fn run_rom_headless(mut mmu: MMU, mut cpu: CPU) {
    for _ in 0..10_000 {
        let opcode = mmu.read(cpu.regs.pc);
        if opcode == 0x76 {
            // HALT instruction: exit
            return;
        }
        match cpu.step(&mut mmu) {
            Ok(cycles) => mmu.step(cycles),
            Err(e) => {
                eprintln!("[ERROR] CPU step failed: {e}");
                return;
            }
        }
    }
}

fn run_rom(
    rom_path: &std::path::Path,
    headless: bool,
    debug: bool,
    verbose: bool,
) -> anyhow::Result<()> {
    // Validate ROM path
    if !rom_path.exists() {
        anyhow::bail!("ROM file not found: {}", rom_path.display());
    }
    let rom_data = std::fs::read(rom_path)
        .with_context(|| format!("Failed to read ROM from {}", rom_path.display()))?;

    let mmu = MMU::new(rom_data)?;
    let cpu = CPU::new();

    if headless {
        run_rom_headless(mmu, cpu);
        return Ok(());
    }

    let event_loop = EventLoop::new()?;
    let mut app = EmulatorApp {
        window: None,
        mmu,
        cpu,
        debug,
        _verbose: verbose,
        should_exit: false,
    };
    let _ = event_loop.run_app(&mut app);
    Ok(())
}

/// Run a test ROM in headless mode, capturing serial output and printing it.
/// Returns exit code: 0 for pass, 1 for fail/timeout.
fn run_test_rom(rom_path: &std::path::Path, debug: bool, _verbose: bool) -> anyhow::Result<i32> {
    use core_lib::{cpu::CPU, mmu::MMU};
    use std::io::Write;
    const MAX_CYCLES: u64 = 10_000_000;
    const SERIAL_DATA: u16 = 0xFF01;
    const SERIAL_CTRL: u16 = 0xFF02;

    if !rom_path.exists() {
        anyhow::bail!("ROM file not found: {}", rom_path.display());
    }
    let rom_data = std::fs::read(rom_path)
        .with_context(|| format!("Failed to read ROM from {}", rom_path.display()))?;

    let mut mmu = MMU::new(rom_data).map_err(anyhow::Error::from)?;
    let mut cpu = CPU::new();
    cpu.regs.pc = 0x0100;
    let mut serial_output = Vec::new();
    let mut cycles: u64 = 0;
    let mut pass = false;
    let mut step_count = 0;

    // Run until pass/fail or timeout
    while cycles < MAX_CYCLES {
        let opcode = mmu.read(cpu.regs.pc);
        if debug && step_count < 1000 {
            eprintln!(
                "[DEBUG] Step {}: PC={:04X} OPCODE={:02X}",
                step_count, cpu.regs.pc, opcode
            );
        }
        if opcode == 0x76 {
            // HALT
            if debug {
                eprintln!("[DEBUG] HALT at PC={:04X}", cpu.regs.pc);
            }
            break;
        }
        let step_cycles = cpu.step(&mut mmu);
        match step_cycles {
            Ok(cycles_val) => {
                mmu.step(cycles_val);
                cycles += u64::from(cycles_val);
            }
            Err(e) => {
                eprintln!("[ERROR] CPU step failed: {e}");
                break;
            }
        }
        step_count += 1;

        // Serial transfer: if 0xFF02 == 0x81, output 0xFF01
        if mmu.read(SERIAL_CTRL) == 0x81 {
            let byte = mmu.read(SERIAL_DATA);
            serial_output.push(byte);
            if debug {
                eprintln!(
                    "[DEBUG] Serial transfer: PC={:04X} Byte={:02X} Char={} (step {})",
                    cpu.regs.pc, byte, byte as char, step_count
                );
            }
            // Print as soon as received
            print!("{}", byte as char);
            std::io::stdout().flush().ok();
            // Clear transfer flag
            let _ = mmu.write(SERIAL_CTRL, 0x00);
        }

        // Check for 'Passed' or 'Failed' in output
        if serial_output.ends_with(b"Passed\n") {
            pass = true;
            break;
        }
        if serial_output.ends_with(b"Failed\n") {
            break;
        }
    }
    eprintln!("[DEBUG] Cycles executed: {cycles}");
    Ok(i32::from(!pass))
}

fn main() {
    let exit_code = match real_main() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    };
    std::process::exit(exit_code);
}

fn real_main() -> anyhow::Result<i32> {
    let cli = Cli::parse();
    // Set up logging or debugging as needed
    if cli.debug {
        // In a real app, set up debug logging here
        eprintln!("[DEBUG] Debug mode enabled");
    }
    if cli.verbose {
        // In a real app, set up verbose logging here
        eprintln!("[VERBOSE] Verbose mode enabled");
    }
    match &cli.command {
        Commands::Run { rom_path, headless } => {
            if *headless {
                let exit_code = run_test_rom(rom_path, cli.debug, cli.verbose)?;
                Ok(exit_code)
            } else {
                run_rom(rom_path, false, cli.debug, cli.verbose)?;
                Ok(0)
            }
        }
        Commands::Test { rom_path } => {
            let exit_code = run_test_rom(rom_path, cli.debug, cli.verbose)?;
            Ok(exit_code)
        }
    }
}

#[cfg(test)]
mod tests;
