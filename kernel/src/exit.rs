#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

fn exit_qemu(exit_code: QemuExitCode) -> ! {
    use x86_64::instructions::{nop, port::Port};

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }

    loop {
        nop();
    }
}

#[allow(unused)]
pub(crate) fn success() -> ! {
    exit_qemu(QemuExitCode::Success)
}

#[allow(unused)]
pub(crate) fn failure() -> ! {
    exit_qemu(QemuExitCode::Failed)
}
