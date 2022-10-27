# Contributing to Pop!_OS

## Find the Correct Repo

Before you make a change, you need to find the relevant repository to make a contribution in. See the 'Developer Resources' section for help finding the correct one. 

## Make An Issue

For large features, it's recommended to start with an issue for discussion if it doesn't already exist. Your work will have the highest chance of being merged if the discussion reaches a consensus in advance on how (or if) the feature should be implemented.

## Make a Pull Request

Fork the repository, make your changes, and then make a pull request! It helps to use detail and explain what your change does. 

Every PR to Pop!_OS components requires approval from the engineering team (for code quality and architectural fit) and quality assurance team (for stability and UX sanity.) Request a review from each of these teams in order to make sure your PR is seen. Any change that significantly impacts the user experience (e.g. new GUI features) may also require approval from the user experience team. 

## Post-Merge Release Process

The Pop!_OS CI server automatically builds the master (or main) branch of every git repository every 15 minutes. All packages from those git branches are published in the [master staging apt repository](http://apt-origin.pop-os.org/staging/master/).

Packages are then released from master staging as regular updates via PRs to the [repo-release repository](https://github.com/pop-os/repo-release/), which contains a [list](https://github.com/pop-os/repo-release/blob/master/sync) of the name and version of every currently released package. After the list is updated, another CI job automatically releases the package versions contained in the list.

### Pop!_OS Release Frequency

Pop!_OS component updates such as security patches, bug fixes, and even some new features are released regularly (in a rolling-release fashion.)

Feature updates for packages inherited from Ubuntu, as well as very large UX changes (such as the introduction of the COSMIC interface), are released as major version upgrades.
