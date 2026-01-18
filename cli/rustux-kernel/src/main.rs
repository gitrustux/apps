use std::env;
use std::process::{self, Command};
use std::path::PathBuf;

const KERNEL_DIR: &str = "/var/www/rustux.com/prod/rustux";
const VERSION: &str = "0.2.0";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        process::exit(1);
    }

    match args[1].as_str() {
        "build" => build_kernel(&args[2..]),
        "test" => test_kernel(&args[2..]),
        "version" => print_version(),
        "help" | "--help" | "-h" => print_help(),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            println!();
            print_help();
            process::exit(1);
        }
    }
}

fn build_kernel(args: &[String]) {
    println!("Building Rustux kernel v{}...", VERSION);
    println!("Kernel directory: {}", KERNEL_DIR);
    println!();

    // Parse arguments
    let mut arch = "amd64";
    let mut release = true;

    for arg in args {
        match arg.as_str() {
            "--arch" => {
                // Next arg would be the arch value
                // For MVP, just use default
            }
            "--debug" => release = false,
            _ => {}
        }
    }

    // Build cargo command
    let target = "x86_64-unknown-uefi";
    let mut cmd = Command::new("cargo");

    cmd.current_dir(KERNEL_DIR)
        .arg("build");

    if release {
        cmd.arg("--release");
    }

    cmd.arg("--bin")
        .arg("rustux")
        .arg("--features")
        .arg("uefi_kernel")
        .arg("--target")
        .arg(target);

    println!("Running: cargo build --release --bin rustux --features uefi_kernel --target {}", target);
    println!();

    let status = cmd.status();

    match status {
        Ok(status) => {
            if status.success() {
                println!();
                println!("Build successful!");
                let elf_path = PathBuf::from(KERNEL_DIR)
                    .join("target")
                    .join(target)
                    .join(if release { "release" } else { "debug" })
                    .join("rustux.efi");
                println!("Output: {}", elf_path.display());
            } else {
                println!();
                eprintln!("Build failed with exit code: {:?}", status.code());
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to execute cargo build: {}", e);
            process::exit(1);
        }
    }
}

fn test_kernel(args: &[String]) {
    println!("Testing Rustux kernel in QEMU...");
    println!("Kernel directory: {}", KERNEL_DIR);
    println!();

    // Parse basic QEMU options
    let mut memory = "512M";
    let mut machine = "q35";

    for i in 0..args.len() {
        match args[i].as_str() {
            "--memory" | "-m" => {
                if i + 1 < args.len() {
                    memory = &args[i + 1];
                }
            }
            "--machine" => {
                if i + 1 < args.len() {
                    machine = &args[i + 1];
                }
            }
            _ => {}
        }
    }

    println!("QEMU options: memory={}, machine={}", memory, machine);
    println!();

    // Run the test script
    let test_script = PathBuf::from(KERNEL_DIR).join("test-qemu.sh");

    if !test_script.exists() {
        eprintln!("ERROR: test-qemu.sh not found at {}", test_script.display());
        eprintln!("Please ensure the kernel is properly set up.");
        process::exit(1);
    }

    let status = Command::new("bash")
        .arg(test_script)
        .current_dir(KERNEL_DIR)
        .status();

    match status {
        Ok(status) => {
            if status.success() {
                println!();
                println!("Test complete!");
            } else {
                println!();
                eprintln!("Test failed with exit code: {:?}", status.code());
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to run test script: {}", e);
            process::exit(1);
        }
    }
}

fn print_version() {
    println!("rustux-kernel v{}", VERSION);
    println!("Rustux Microkernel - Zircon-style kernel objects");
    println!();
    println!("Kernel directory: {}", KERNEL_DIR);
    println!("Supported architectures: amd64, arm64 (placeholder), riscv64 (placeholder)");
}

fn print_help() {
    println!("Rustux Kernel CLI Tool v{}", VERSION);
    println!();
    println!("USAGE:");
    println!("    rustux-kernel <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    build       Build the kernel");
    println!("    test        Test kernel in QEMU");
    println!("    version     Show version information");
    println!("    help        Show this help message");
    println!();
    println!("BUILD OPTIONS:");
    println!("    --arch <ARCH>       Target architecture (default: amd64)");
    println!("    --debug             Build debug profile (default: release)");
    println!();
    println!("TEST OPTIONS:");
    println!("    --memory, -m <SIZE> Set QEMU memory (default: 512M)");
    println!("    --machine <TYPE>    Set QEMU machine type (default: q35)");
    println!();
    println!("EXAMPLES:");
    println!("    rustux-kernel build");
    println!("    rustux-kernel build --arch amd64");
    println!("    rustux-kernel test");
    println!("    rustux-kernel test --memory 1G");
    println!("    rustux-kernel version");
    println!();
    println!("KERNEL LOCATION:");
    println!("    {}", KERNEL_DIR);
}
