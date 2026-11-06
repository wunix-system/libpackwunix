use libpackwunix::kernel::{generate_limine_conf, KernelEntry};
use libpackwunix::version::Version;

#[test]
fn test_generate_limine_conf() {
    let current = KernelEntry {
        version: Version::new(0, 4, 2),
        kernel_path: "boot():/boot/wunix-kernel-0.4.2".into(),
        cmdline: "root=/dev/vda1 lang=ru mode=disk".into(),
    };

    let fallback = KernelEntry {
        version: Version::new(0, 4, 1),
        kernel_path: "boot():/boot/wunix-kernel-0.4.1".into(),
        cmdline: "root=/dev/vda1 lang=ru mode=disk".into(),
    };

    let conf = generate_limine_conf(&current, Some(&fallback), 3);

    assert!(conf.contains("timeout: 3"));
    assert!(conf.contains("default_entry: 1"));
    assert!(conf.contains("/Wunix OS (Current: v0.4.2)"));
    assert!(conf.contains("kernel_path: boot():/boot/wunix-kernel-0.4.2"));
    assert!(conf.contains("/Wunix OS (Fallback: v0.4.1)"));
    assert!(conf.contains("kernel_path: boot():/boot/wunix-kernel-0.4.1"));
}
