# Pop!\_OS

Pop!\_OS is designed for people who use their computer to create; whether it’s complicated, professional grade software and products, sophisticated 3D models, computer science in academia, or makers working on their latest invention. The Pop! user interface stays out of the way while offering extensive customization to perfect your work flow. Built on Ubuntu, you have access to vast repositories of open source software and development tools.

Pop!\_OS’s first release was on October 19th, 2017. For more information, [visit the Pop!\_OS website](https://system76.com/pop) and [view the Pop!\_OS documentation](https://pop.system76.com/docs/).

## Purpose

The purpose of this repository is to allow easy management of all Pop!\_OS related source code and assets.

Binary packages are hosted [on Launchpad](https://launchpad.net/~system76/+archive/ubuntu/pop/+packages). Many packages have source on Github, under the [Pop!\_OS organization](https://github.com/pop-os).

## Developer Resources
For instructions on how to build the shell:
	* [Cosmic, our current gnome-based shell](https://github.com/pop-os/cosmic). 
	* [Cosmic-Epoch, the new Rust based shell](https://github.com/pop-os/cosmic-epoch).

To view a list of all included repositories, see [REPOS.md](./REPOS.md)
Developer chat: https://chat.pop-os.org/

## How to Make a Change
First of all, thank you for taking interest in contributing to our community! Every bit helps, even if it's just contributing to our documentation. 

### Find the Correct Repo
Before you make a change, you need to find the relevant repository to make a contribution in. See the 'Developer Resources' section for help finding the correct one.  When in doubt, you can make an issue in this catch all repo. 

### Make An Issue
All changes should start with an issue so the change can be properly discussed. The more complex the issue, the more discussion would be required. If an issue doesn't already exist, here would be the place to start.

### Make a PR.
Every PR to Pop!_OS components requires approval from the engineering team (for code quality and architectural fit) and quality assurance team (for stability and UX sanity.) Anything that significantly impacts the user experience (e.g. new GUI features) may also require approval from the user experience team. 

### Enjoy Your Contribution!
The Pop!_OS CI server automatically builds the master/main branch of every git repository (every 15 minutes), and all packages from those git branches are published in the master staging apt repository. Packages are then released from master staging as regular updates via PRs to the repo-release repository, which contains a list of the current version of every package in the release repository: https://github.com/pop-os/repo-release (There is another CI job that checks this list and copies the listed versions of each package from the master staging repo to the release repo.)

Changes can be withheld until the next major release by using a release-specific master branch (e.g. master_jammy only builds packages for 22.04, so it would have been the "next release" branch when 21.10 was current), or by building into new packages that are not installed by default (as is the case with the current COSMIC Epoch packages). But this is more of the exception than the norm.

### Wait, I Thought Pop Didn't Use Rolling Release Schedules?
While the six-month release cycle applies to packages inherited from Ubuntu, as well as very large UX changes (such as the introduction of COSMIC with 21.10.) Pop!_OS component updates such as security patches, bug fixes, and even some new features are released regularly (in a rolling-release fashion.)

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
