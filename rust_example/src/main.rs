//! Minimal Rust example for AE350 on TangMega-138K FPGA
//!
//! This demonstrates running Rust on the Andes AE350 RISC-V core.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

// Memory-mapped GPIO base address (from ae350 datasheet)
const GPIO_BASE: usize = 0xF010_0000;
const GPIO_DATA_OUT: *mut u32 = (GPIO_BASE + 0x00) as *mut u32;
const GPIO_DIR: *mut u32 = (GPIO_BASE + 0x08) as *mut u32;

/// Entry point - called by the runtime after initialization
#[no_mangle]
pub extern "C" fn main() -> ! {
    // Set GPIO pins as outputs
    unsafe {
        GPIO_DIR.write_volatile(0xFF);
    }

    let mut counter: u32 = 0;

    loop {
        // Simple LED pattern
        unsafe {
            GPIO_DATA_OUT.write_volatile(counter & 0xFF);
        }

        // Crude delay
        for _ in 0..100_000 {
            core::hint::spin_loop();
        }

        counter = counter.wrapping_add(1);
    }
}

/// Panic handler - required for no_std
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // In a real application, you might:
    // - Write to UART
    // - Blink an LED pattern
    // - Trigger a breakpoint
    loop {
        core::hint::spin_loop();
    }
}

/// Required by riscv-rt: called before main
#[export_name = "_setup_interrupts"]
fn setup_interrupts() {
    // Configure PLIC or other interrupt controller here
}

/// Required by riscv-rt: handle exceptions
#[export_name = "ExceptionHandler"]
fn exception_handler(_trap_frame: &riscv_rt::TrapFrame) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
