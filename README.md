# Pop!\_OS

**Pop!\_OS** is a Linux distribution designed for creators—whether you're building professional software, crafting 3D models, diving into computer science, or working on innovative projects. The interface is sleek and unobtrusive, offering extensive customization options to optimize your workflow.

Built on Ubuntu, Pop!\_OS opens the door to a vast repository of open-source software and development tools.

- **First Release**: October 19, 2017  
- [Official Website](https://system76.com/pop)  
- [Pop!\_OS Documentation](https://support.system76.com/)  

---

## Purpose

This repository is designed to facilitate the management of Pop!\_OS-related source code and assets.

To explore the list of all included repositories, refer to [REPOS.md](./REPOS.md).

- **Binary Packages**: Hosted in the [Pop!\_OS APT repositories](https://apt.pop-os.org/).  
- **Source Code**: Available on GitHub under the [Pop!\_OS organization](https://github.com/pop-os).  
  - Additional components and documentation are found under the [System76 organization](https://github.com/system76).

---

## Developer Resources

### Building the Shell

To build the Pop!\_OS shell, check out the following guides:

- [COSMIC (GNOME-based)](https://github.com/pop-os/cosmic)  
- [COSMIC Epoch (Rust-based)](https://github.com/pop-os/cosmic-epoch)  

### Developer Communication

Join the conversation on the [Pop!\_OS Development Chat](https://chat.pop-os.org/pop-os/channels/development).

---

## Contributing

To contribute to Pop!\_OS, follow the guidelines in [CONTRIBUTING.md](./CONTRIBUTING.md).

---

## Dependencies

Before using this repository, install the following package:

- `python3-launchpadlib`

---

## Available Scripts

The repository includes several helpful scripts for managing Pop!\_OS:

- **`scripts/clone`** - Clone source code repositories.  
- **`scripts/debversion`** - Display the version of a Debian package.  
- **`scripts/ignore`** - Generate a `.gitignore` file.  
- **`scripts/issues`** - List open issues.  
- **`scripts/launchpad`** - Display available PPA packages.  
- **`scripts/prs`** - View pull requests.  
- **`scripts/pull`** - Update source code repositories.  
- **`scripts/readme`** - Generate the `REPOS.md` file.  
- **`scripts/validate`** - Validate the presence of key documentation files (e.g., `LICENSE`, `README`, and `TESTING`).

---

