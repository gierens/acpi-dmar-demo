use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};
use std::env;
use std::process::Command;

fn main() {
    // read env variables that were set in build script
    let uefi_path = env!("UEFI_PATH");

    let mut cmd = Command::new("qemu-system-x86_64");
    // enable KVM and pass through host CPU
    cmd.arg("-machine")
        .arg("q35,accel=kvm,kernel_irqchip=split");
    cmd.arg("-device")
        .arg("intel-iommu,intremap=on,caching-mode=on");
    cmd.arg("-cpu").arg("host");
    // set number of processors and amount of memory
    cmd.arg("-smp").arg("1").arg("-m").arg("1G");
    // print serial output to the shell
    cmd.arg("-serial").arg("mon:stdio");
    // don't display video output
    cmd.arg("-display").arg("none");
    // enable the guest to exit qemu
    cmd.arg("-device")
        .arg("isa-debug-exit,iobase=0xf4,iosize=0x04");

    let prebuilt =
        Prebuilt::fetch(Source::LATEST, "target/ovmf").expect("failed to update prebuilt");

    let code = prebuilt.get_file(Arch::X64, FileType::Code);
    let vars = prebuilt.get_file(Arch::X64, FileType::Vars);

    cmd.arg("-drive")
        .arg(format!("format=raw,file={uefi_path}"));
    cmd.arg("-drive").arg(format!(
        "if=pflash,format=raw,unit=0,file={},readonly=on",
        code.display()
    ));
    // copy vars and enable rw instead of snapshot if you want to store data (e.g. enroll secure boot keys)
    cmd.arg("-drive").arg(format!(
        "if=pflash,format=raw,unit=1,file={},snapshot=on",
        vars.display()
    ));

    let mut child = cmd.spawn().expect("failed to start qemu-system-x86_64");
    let status = child.wait().expect("failed to wait on qemu");
    match status.code().unwrap_or(1) {
        0x10 => 0, // success
        0x11 => 1, // failure
        _ => 2,    // unknown fault
    };
}
