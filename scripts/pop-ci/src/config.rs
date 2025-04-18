/// Repos to build for both Pop and Ubuntu
pub static DEV_REPOS: &'static [&'static str] = &[
    "accountsservice",
    "alsa-ucm-conf",
    "alsa-utils",
    "amd-ppt-bin",
    "amd64-microcode",
    "bcmwl",
    "bluez",
    "directx-headers",
    "distinst",
    "dwarves",
    "egl-wayland",
    "firmware-manager",
    "fwupd",
    "fwupd-efi",
    "gdm3",
    "gnome-desktop3",
    "gnome-settings-daemon",
    "gnome-shell",
    "gnome-shell-extension-system76-power",
    "hidpi-daemon",
    "kbuild",
    "libabigail",
    "libasound2",
    "libbpf",
    "libdrm",
    "libtraceevent",
    "libtracefs",
    "libvdpau",
    "libxmlb",
    "linux",
    "linux-firmware",
    "mesa",
    "ninja-build",
    "nvidia-graphics-drivers",
    "nvidia-graphics-drivers-470",
    "nvidia-graphics-drivers-565",
    "spirv-headers",
    "spirv-llvm-translator-15",
    "spirv-tools",
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
    "virtualbox-ext-pack",
    "wayland",
    "wayland-protocols",
    "zfs-linux",
];

/// Repos to build for Pop 20.04, in addition to DEV_REPOS
pub static POP_FOCAL_REPOS: &'static [&'static str] = &[
    "alacritty",
    "appstream-data",
    "apt",
    "atom-editor",
    "buildchain",
    "bustd",
    "connectivity",
    "cosmic-design-demo",
    "cosmic-screenshot",
    "debconf",
    "default-settings",
    "desktop",
    "desktop-icons-ng",
    "distinst-v2",
    "eddy",
    "flatpak",
    "fonts",
    "gamehub",
    "gnome-control-center",
    "gnome-initial-setup",
    "gnome-online-accounts",
    "gnome-shell-extension-alt-tab-raise-first-window",
    "gnome-shell-extension-always-show-workspaces",
    "gnome-shell-extension-do-not-disturb",
    "gnome-shell-extension-pop-battery-icon-fix",
    "gnome-shell-extension-pop-shop-details",
    "gnome-shell-extension-pop-suspend-button",
    "gnome-terminal",
    "granite",
    "grub-theme",
    "gtk-theme",
    "happiness",
    "hidpi-widget",
    "icon-theme",
    "installer",
    "just",
    "kernelstub",
    "keyboard-configurator",
    "keyring",
    "launcher",
    "libhandy",
    "libnvidia-container",
    "lutris",
    "meta-python",
    "nvidia-container-runtime",
    "nvidia-container-toolkit",
    "nvidia-vaapi-driver",
    "packaging-natron",
    "packaging-rust",
    "plymouth",
    "plymouth-theme",
    "popsicle",
    "protonvpn-nm-lib",
    "python-apt",
    "repolib",
    "repoman",
    "rtl8821ce-dkms",
    "session",
    "sessioninstaller",
    "shell",
    "shell-shortcuts",
    "shop",
    "steam",
    "support-panel",
    "system-updater",
    "system76-scheduler",
    "tensorman",
    "theme",
    "theme-switcher",
    "transition",
    "upgrade",
    "v4l2loopback",
    "wallpapers",
];
