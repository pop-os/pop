# Pop!\_OS

Pop!\_OS is designed for people who use their computer to create; whether it’s complicated, professional grade software and products, sophisticated 3D models, computer science in academia, or makers working on their latest invention. The Pop! user interface stays out of the way while offering extensive customization to perfect your work flow. Built on Ubuntu, you have access to vast repositories of open source software and development tools.

Pop!\_OS’s first release was on October 19th, 2017. For more information, [visit the Pop!\_OS website](https://system76.com/pop) and [view the Pop!\_OS documentation](https://support.system76.com/).

## Purpose

The purpose of this repository is to allow easy management of all Pop!\_OS related source code and assets. To view a list of all included repositories, see [REPOS.md](./REPOS.md).

Binary packages are hosted [on Launchpad](https://launchpad.net/~system76/+archive/ubuntu/pop/+packages). Many packages have source on Github, under the [Pop!\_OS organization](https://github.com/pop-os).

## Developer Resources

For instructions on how to build the shell:

* [COSMIC (GNOME-based)](https://github.com/pop-os/cosmic)
* [COSMIC-Epoch (Rust-based)](https://github.com/pop-os/cosmic-epoch)
 
Developer chat: https://chat.pop-os.org/pop-os/channels/development

## Contributing to Pop!_OS

For instructions and guidelines to make changes in Pop!_OS, see [CONTRIBUTING.md](./CONTRIBUTING.md).

## Dependencies

You must install the following packages to use this repository:

- `python3-launchpadlib`

## Scripts

This repository contains the following commands:

- `scripts/clone` - clone source code
- `scripts/debversion` - show version of debian package
- `scripts/ignore` - generate `.gitignore`
- `scripts/issues` - show issues
- `scripts/launchpad` - show PPA packages
- `scripts/prs` - show pull requests
- `scripts/pull` - update source code
- `scripts/readme` - generate `REPOS.md`
- `scripts/validate` - validate Pop!\_OS source code for presence of `LICENSE`, `README`, and `TESTING` documentation
