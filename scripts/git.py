from subprocess import check_call, check_output

def git_ids_and_branches(cwd):
    """
    Returns a `dict` mapping each commit ID to a list of branches.

    `git ls-remote` will return something like::

        f73e8fc2c5bb5756032c35b6be1fcd84106413b1	refs/heads/overhaul
        ffb788cccfe0cd7feedcfe8f0b8e9154097a46ca	refs/heads/master
        f73e8fc2c5bb5756032c35b6be1fcd84106413b1	refs/heads/test_bionic

    For which this function would return::

        {
            'f73e8fc2c5bb5756032c35b6be1fcd84106413b1': ['overhaul', 'test_bionic'],
            'ffb788cccfe0cd7feedcfe8f0b8e9154097a46ca':	['master'],
        }
    """
    check_call(['git', 'fetch', 'origin'], cwd=cwd)
    o = check_output(['git', 'ls-remote', '--heads', 'origin'], cwd=cwd)
    prefix = 'refs/heads/'
    result = {}
    for line in o.decode().splitlines():
        (_id, rawbranch) = line.split('\t')
        assert rawbranch.startswith(prefix)
        branch = rawbranch[len(prefix):]
        if _id not in result:
            result[_id] = []
        result[_id].append(branch)
    return result


def git_clean(cwd):
    check_call(['git', 'clean', '-xfd'], cwd=cwd)


def git_checkout_id(cwd, _id):
    check_call(['git', 'checkout', '--force', '--detach', _id], cwd=cwd)
    check_call(['git', 'submodule', 'sync', '--recursive'], cwd=cwd)
    check_call(['git', 'submodule', 'update', '--init', '--recursive'], cwd=cwd)
    git_clean(cwd)


def git_timestamp_id(cwd, _id):
    o = check_output(["git", "log", "-1", "--pretty=format:%ct", _id], cwd=cwd)
    return o.decode().strip()


def git_datetime_id(cwd, _id):
    o = check_output(["git", "log", "-1", "--pretty=format:%cD", _id], cwd=cwd)
    return o.decode().strip()


def git_archive_id(cwd, _id, archive):
    return check_output(["git", "archive", "--format", "tar", "-o", archive, _id], cwd=cwd)
