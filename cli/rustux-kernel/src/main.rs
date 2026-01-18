use std::env;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::fs;

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
        "qemu" => run_qemu(&args[2..]),
        "image" => create_image(&args[2..]),
        "features" => show_features(),
        "arch" => show_arch_status(),
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
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--arch" => {
                if i + 1 < args.len() {
                    arch = &args[i + 1];
                    i += 1;
                }
            }
            "--debug" => release = false,
            _ => {}
        }
        i += 1;
    }

    // Map architecture to target triple
    let target = match arch {
        "amd64" | "x86_64" => "x86_64-unknown-uefi",
        "arm64" | "aarch64" => {
            eprintln!("ERROR: ARM64 target not yet supported");
            process::exit(1);
        }
        "riscv64" => {
            eprintln!("ERROR: RISC-V target not yet supported");
            process::exit(1);
        }
        _ => {
            eprintln!("ERROR: Unknown architecture: {}", arch);
            eprintln!("Supported: amd64, arm64 (WIP), riscv64 (WIP)");
            process::exit(1);
        }
    };

    // Build cargo command
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

    println!("Configuration:");
    println!("  Architecture: {}", arch);
    println!("  Target: {}", target);
    println!("  Profile: {}", if release { "release" } else { "debug" });
    println!();

    println!("Building...");
    let status = cmd.status();

    match status {
        Ok(status) => {
            if status.success() {
                println!();
                println!("✓ Build successful!");
                let elf_path = PathBuf::from(KERNEL_DIR)
                    .join("target")
                    .join(target)
                    .join(if release { "release" } else { "debug" })
                    .join("rustux.efi");
                println!("  Output: {}", elf_path.display());

                // Show file size
                if let Ok(metadata) = elf_path.metadata() {
                    println!("  Size: {} bytes", metadata.len());
                }
            } else {
                println!();
                eprintln!("✗ Build failed with exit code: {:?}", status.code());
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to execute cargo build: {}", e);
            process::exit(1);
        }
    }
}

fn test_kernel(args: &[String]) {
    println!("Testing Rustux kernel in QEMU...");
    println!("Kernel directory: {}", KERNEL_DIR);
    println!();

    // Parse QEMU options
    let mut memory = "512M";
    let mut machine = "q35";
    let mut timeout = 15;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--memory" | "-m" => {
                if i + 1 < args.len() {
                    memory = &args[i + 1];
                    i += 1;
                }
            }
            "--machine" => {
                if i + 1 < args.len() {
                    machine = &args[i + 1];
                    i += 1;
                }
            }
            "--timeout" => {
                if i + 1 < args.len() {
                    timeout = args[i + 1].parse().unwrap_or(15);
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    println!("QEMU options:");
    println!("  Memory: {}", memory);
    println!("  Machine: {}", machine);
    println!("  Timeout: {} seconds", timeout);
    println!();

    // Find OVMF firmware
    let ovmf_paths = [
        "/usr/share/ovmf/OVMF.fd",
        "/usr/share/edk2-ovmf/x64/OVMF_CODE.fd",
    ];

    let ovmf_fd = ovmf_paths.iter()
        .find(|p| Path::new(p).exists())
        .map(|s| s.to_string());

    let ovmf_fd = match ovmf_fd {
        Some(path) => path,
        None => {
            eprintln!("ERROR: OVMF firmware not found!");
            eprintln!("Searched paths:");
            for path in &ovmf_paths {
                eprintln!("  {}", path);
            }
            eprintln!();
            eprintln!("Install with: sudo apt install ovmf");
            process::exit(1);
        }
    };

    // Check if kernel image exists
    let kernel_img = PathBuf::from(KERNEL_DIR).join("rustux.img");
    if !kernel_img.exists() {
        eprintln!("ERROR: Kernel image not found: {}", kernel_img.display());
        eprintln!();
        eprintln!("Build the kernel first:");
        eprintln!("  rustux-kernel build");
        process::exit(1);
    }

    println!("Starting QEMU test...");
    println!();

    // Run QEMU with timeout
    let result = Command::new("timeout")
        .arg(format!("{}", timeout))
        .arg("qemu-system-x86_64")
        .arg("-bios")
        .arg(&ovmf_fd)
        .arg("-drive")
        .arg(format!("file={},format=raw", kernel_img.display()))
        .arg("-nographic")
        .arg("-device")
        .arg("isa-debugcon,iobase=0xE9,chardev=debug")
        .arg("-chardev")
        .arg("file,id=debug,path=/tmp/rustux-kernel-test.log")
        .arg("-m")
        .arg(memory)
        .arg("-machine")
        .arg(machine)
        .arg("-smp")
        .arg("1")
        .arg("-no-reboot")
        .arg("-no-shutdown")
        .current_dir(KERNEL_DIR)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .status();

    println!();
    println!("Test Results:");
    println!();

    match result {
        Ok(status) => {
            // Check the debug log
            let log_path = "/tmp/rustux-kernel-test.log";
            if Path::new(log_path).exists() {
                if let Ok(content) = fs::read_to_string(log_path) {
                    if content.contains("[TICK]") {
                        println!("✓ Timer interrupts detected!");
                        let tick_count = content.matches("[TICK]").count();
                        println!("  Tick count: {}", tick_count);
                    } else {
                        println!("✗ No timer ticks detected");
                    }

                    if content.contains("RSDP found") {
                        println!("✓ ACPI RSDP discovered");
                    }

                    if content.contains("GDT configured") {
                        println!("✓ GDT configured");
                    }

                    if content.contains("IDT configured") {
                        println!("✓ IDT configured");
                    }

                    if content.contains("APIC initialized") {
                        println!("✓ APIC initialized");
                    }
                }
            }

            if status.success() {
                println!();
                println!("✓ Test completed successfully!");
            } else {
                println!();
                println!("Test exited with code: {:?}", status.code());
                println!("Debug log: {}", log_path);
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to run QEMU: {}", e);
            process::exit(1);
        }
    }
}

fn run_qemu(args: &[String]) {
    println!("Running Rustux kernel in QEMU (interactive)...");
    println!();

    // Parse QEMU options
    let mut memory = "512M";
    let mut machine = "q35";

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--memory" | "-m" => {
                if i + 1 < args.len() {
                    memory = &args[i + 1];
                    i += 1;
                }
            }
            "--machine" => {
                if i + 1 < args.len() {
                    machine = &args[i + 1];
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    // Find OVMF firmware
    let ovmf_paths = [
        "/usr/share/ovmf/OVMF.fd",
        "/usr/share/edk2-ovmf/x64/OVMF_CODE.fd",
    ];

    let ovmf_fd = ovmf_paths.iter()
        .find(|p| Path::new(p).exists())
        .map(|s| s.to_string());

    let ovmf_fd = match ovmf_fd {
        Some(path) => path,
        None => {
            eprintln!("ERROR: OVMF firmware not found!");
            eprintln!("Install with: sudo apt install ovmf");
            process::exit(1);
        }
    };

    // Check if kernel image exists
    let kernel_img = PathBuf::from(KERNEL_DIR).join("rustux.img");
    if !kernel_img.exists() {
        eprintln!("ERROR: Kernel image not found");
        eprintln!("Build first: rustux-kernel build");
        process::exit(1);
    }

    // Run QEMU interactively
    let status = Command::new("qemu-system-x86_64")
        .arg("-bios")
        .arg(&ovmf_fd)
        .arg("-drive")
        .arg(format!("file={},format=raw", kernel_img.display()))
        .arg("-nographic")
        .arg("-m")
        .arg(memory)
        .arg("-machine")
        .arg(machine)
        .arg("-smp")
        .arg("1")
        .current_dir(KERNEL_DIR)
        .status();

    match status {
        Ok(_) => println!("QEMU exited"),
        Err(e) => {
            eprintln!("Failed to run QEMU: {}", e);
            process::exit(1);
        }
    }
}

fn create_image(args: &[String]) {
    println!("Creating bootable disk image...");
    println!();

    let mut size = "64M";
    let mut output = "rustux.img";

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--size" => {
                if i + 1 < args.len() {
                    size = &args[i + 1];
                    i += 1;
                }
            }
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output = &args[i + 1];
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    println!("Options:");
    println!("  Size: {}", size);
    println!("  Output: {}", output);
    println!();

    // Check for required tools
    if !command_exists("mkfs.fat") {
        eprintln!("ERROR: mkfs.fat not found");
        eprintln!("Install: sudo apt install dosfstools");
        process::exit(1);
    }

    if !command_exists("mcopy") {
        eprintln!("ERROR: mcopy not found");
        eprintln!("Install: sudo apt install mtools");
        process::exit(1);
    }

    // Find kernel binary
    let kernel_bin = PathBuf::from(KERNEL_DIR)
        .join("target/x86_64-unknown-uefi/release/rustux.efi");

    if !kernel_bin.exists() {
        eprintln!("ERROR: Kernel binary not found");
        eprintln!("Build first: rustux-kernel build");
        process::exit(1);
    }

    let output_path = PathBuf::from(KERNEL_DIR).join(output);

    // Create image
    println!("Creating disk image...");
    let status = Command::new("dd")
        .arg("if=/dev/zero")
        .arg(format!("of={}", output_path.display()))
        .arg("bs=1M")
        .arg(format!("count={}", size.trim_end_matches('M')))
        .current_dir(KERNEL_DIR)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match status {
        Ok(status) if !status.success() => {
            eprintln!("ERROR: dd failed");
            process::exit(1);
        }
        Err(_) => {
            eprintln!("ERROR: Failed to run dd");
            process::exit(1);
        }
        _ => {}
    }

    // Format as FAT32
    println!("Formatting as FAT32...");
    let status = Command::new("mkfs.fat")
        .arg("-F")
        .arg("32")
        .arg(&output_path)
        .stdout(Stdio::null())
        .status();

    match status {
        Ok(status) if !status.success() => {
            eprintln!("ERROR: mkfs.fat failed");
            process::exit(1);
        }
        Err(e) => {
            eprintln!("ERROR: Failed to run mkfs.fat: {}", e);
            process::exit(1);
        }
        _ => {}
    }

    // Create EFI directory structure
    let efi_dir = PathBuf::from("/tmp/rustux-efi/EFI/BOOT");
    let rustux_dir = PathBuf::from("/tmp/rustux-efi/EFI/Rustux");

    fs::create_dir_all(&efi_dir).unwrap();
    fs::create_dir_all(&rustux_dir).unwrap();

    // Copy kernel
    fs::copy(&kernel_bin, efi_dir.join("BOOTX64.EFI")).unwrap();
    fs::copy(&kernel_bin, rustux_dir.join("kernel.efi")).unwrap();

    // Install to image
    println!("Installing kernel to image...");
    let status = Command::new("mcopy")
        .arg("-i")
        .arg(&output_path)
        .arg("-s")
        .arg("/tmp/rustux-efi/EFI")
        .arg("::")
        .status();

    // Cleanup
    fs::remove_dir_all("/tmp/rustux-efi").unwrap();

    match status {
        Ok(status) if status.success() => {
            println!();
            println!("✓ Image created successfully!");
            println!("  Output: {}", output_path.display());
        }
        _ => {
            eprintln!("ERROR: Failed to copy files to image");
            process::exit(1);
        }
    }
}

fn show_features() {
    println!("Rustux Kernel Features v{}", VERSION);
    println!();
    println!("Implemented Features:");
    println!("  ✓ UEFI boot support");
    println!("  ✓ ACPI discovery (RSDP)");
    println!("  ✓ GDT/IDT setup");
    println!("  ✓ APIC initialization");
    println!("  ✓ Timer interrupts");
    println!("  ✓ Keyboard interrupt routing");
    println!("  ✓ Capability-based object system");
    println!("  ✓ Handle & rights management");
    println!("  ✓ VMO (Virtual Memory Objects)");
    println!("  ✓ IPC channels");
    println!("  ✓ Event objects");
    println!("  ✓ Timer objects");
    println!("  ✓ Job objects");
    println!();
    println!("Partially Implemented:");
    println!("  ⚠ System calls (1 working, 28 stubs)");
    println!("  ⚠ Process management (structure in place)");
    println!("  ⚠ Thread management (structure in place)");
    println!("  ⚠ Memory manager (PMM stub, allocator stub)");
    println!();
    println!("Not Yet Implemented:");
    println!("  ✗ Userspace process execution");
    println!("  ✗ Filesystem");
    println!("  ✗ Network stack");
    println!("  ✗ ARM64 support (placeholder only)");
    println!("  ✗ RISC-V support (placeholder only)");
}

fn show_arch_status() {
    println!("Rustux Kernel - Architecture Support");
    println!();
    println!("AMD64 (x86_64):");
    println!("  Status: ✓ Fully Implemented");
    println!("  Boot: UEFI");
    println!("  Interrupts: APIC (Local + I/O)");
    println!("  Memory: Page tables, MMU");
    println!("  Syscall: syscall instruction");
    println!();
    println!("ARM64 (AArch64):");
    println!("  Status: ⚠ Placeholder Only");
    println!("  Boot: UEFI (planned)");
    println!("  Interrupts: GIC (stub only)");
    println!("  Memory: MMU stub");
    println!("  Syscall: svc #0 (ABI defined)");
    println!();
    println!("RISC-V:");
    println!("  Status: ⚠ Placeholder Only");
    println!("  Boot: UEFI (planned)");
    println!("  Interrupts: PLIC (stub only)");
    println!("  Memory: MMU stub");
    println!("  Syscall: ecall (ABI defined)");
}

fn print_version() {
    println!("rustux-kernel v{}", VERSION);
    println!();
    println!("Kernel directory: {}", KERNEL_DIR);
    println!();
    println!("Supported architectures:");
    println!("  amd64, x86_64    ✓ Fully implemented");
    println!("  arm64, aarch64    ⚠ Placeholder");
    println!("  riscv64          ⚠ Placeholder");
}

fn print_help() {
    println!("Rustux Kernel CLI Tool v{}", VERSION);
    println!();
    println!("USAGE:");
    println!("    rustux-kernel <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    build       Build the kernel");
    println!("    test        Test kernel in QEMU (with validation)");
    println!("    qemu        Run kernel in QEMU (interactive)");
    println!("    image       Create bootable disk image");
    println!("    features    Show implemented features");
    println!("    arch        Show architecture support status");
    println!("    version     Show version information");
    println!("    help        Show this help message");
    println!();
    println!("BUILD OPTIONS:");
    println!("    --arch <ARCH>       Target architecture (amd64, arm64, riscv64)");
    println!("    --debug             Build debug profile (default: release)");
    println!();
    println!("TEST OPTIONS:");
    println!("    --memory, -m <SIZE> Set QEMU memory (default: 512M)");
    println!("    --machine <TYPE>    Set QEMU machine type (default: q35)");
    println!("    --timeout <SEC>     Test timeout in seconds (default: 15)");
    println!();
    println!("IMAGE OPTIONS:");
    println!("    --size <SIZE>       Image size (default: 64M)");
    println!("    --output, -o <PATH> Output file (default: rustux.img)");
    println!();
    println!("EXAMPLES:");
    println!("    rustux-kernel build");
    println!("    rustux-kernel test --memory 1G");
    println!("    rustux-kernel qemu -m 2G");
    println!("    rustux-kernel image --size 128M");
    println!("    rustux-kernel features");
    println!();
    println!("KERNEL LOCATION:");
    println!("    {}", KERNEL_DIR);
}

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
