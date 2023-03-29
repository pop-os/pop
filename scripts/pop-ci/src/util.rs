use std::collections::BTreeMap;
use std::fs::{self, ReadDir};
use std::path::PathBuf;
use std::{io, process};

pub static DEV_REPOS: &[&str] = &[
    "accountsservice",
    "amd-ppt-bin",
    "alsa-ucm-conf",
    "alsa-utils",
    "bcmwl",
    "bluez",
    "distinst",
    "dwarves",
    "firmware-manager",
    "fwupd",
    "fwupd-efi",
    "gdm3",
    "gnome-desktop3",
    "gnome-settings-daemon",
    "gnome-shell",
    "gnome-shell-extension-system76-power",
    "hidpi-daemon",
    "libabigail",
    "libasound2",
    "libbpf",
    "libdrm",
    "libxmlb",
    "linux",
    "linux-firmware",
    "mesa",
    "ninja-build",
    "nvidia-graphics-drivers",
    "nvidia-graphics-drivers-470",
    "system76-acpi-dkms",
    "system76-dkms",
    "system76-driver",
    "system76-firmware",
    "system76-io-dkms",
    "system76-keyboard-configurator",
    "system76-oled",
    "system76-power",
    "system76-wallpapers",
    "systemd",
    "ubuntu-drivers-common",
    "virtualbox",
    "zfs-linux",
];

pub fn check_output(output: process::Output) -> io::Result<process::Output> {
    check_status(output.status)?;
    Ok(output)
}

pub fn check_status(status: process::ExitStatus) -> io::Result<()> {
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, format!("{}", status)))
    }
}

/// Find all buildable repos that live under a path, recusively.
pub fn get_repos(repos: &mut BTreeMap<String, PathBuf>, maybe_dir: io::Result<ReadDir>, dev: bool) {
    if let Ok(dir) = maybe_dir {
        for file_res in dir {
            let file = file_res.expect("failed to read directory entry");
            let path = file.path();

            // check recusively
            if path.is_dir() {
                get_repos(repos, fs::read_dir(&path), dev);
            }

            // Is debian buildable
            if path.join("debian").is_dir() {
                let file_name = file
                    .file_name()
                    .into_string()
                    .expect("filename is not utf-8");
                let is_in_build_dir = path.as_path().to_string_lossy().contains("_build");
                if !dev && !DEV_REPOS.contains(&file_name.as_str()) && !is_in_build_dir {
                    println!("{path:?}:{file_name}");
                    // Skip if building dev repos and this is not one of them
                    assert_eq!(repos.insert(file_name, path), None);
                }
            }
        }
    }
}
