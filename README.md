# Pop!\_OS

Pop!\_OS is designed for people who use their computer to create; whether it’s complicated, professional grade software and products, sophisticated 3D models, computer science in academia, or makers working on their latest invention. The Pop! user interface stays out of the way while offering extensive customization to perfect your work flow. Built on Ubuntu, you have access to vast repositories of open source software and development tools.

Pop!\_OS’s first release was on October 19th, 2017. For more information, [visit the Pop!\_OS website](https://system76.com/pop) and [view the Pop!\_OS documentation](http://pop.system76.com/docs/).

## Purpose

The purpose of this repository is to allow easy management of all Pop!\_OS related source code and assets.

Binary packages are hosted [on Launchpad](https://launchpad.net/~system76/+archive/ubuntu/pop/+packages). Many packages have source on Github, under the [Pop!\_OS organization](https://github.com/pop-os).

To view a list of all included repositores, see [REPOS.md](./REPOS.md)

### Dependencies

You must install the following packages to use this repository:

- `python3-launchpadlib`

### Scripts

This repository contains the following commands:

- `scripts/clone` - clone Pop!\_OS source code
- `scripts/ignore` - generate `.gitignore`
- `scripts/launchpad` - show PPA packages
- `scripts/pull` - update Pop!\_OS source code
- `scripts/readme` - generate `REPOS.md`
- `scripts/validate` - validate Pop!\_OS source code for presence of `LICENSE`, `README`, and `TESTING` documentation
