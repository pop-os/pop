import json
import multiprocessing
import os.path
import urllib.request

# Documentation can be found here: https://launchpad.net/+apidoc/devel.html
from launchpadlib.launchpad import Launchpad

# Packages to release in system76-dev
DEV_REPOS = (
    "distinst",
    "hidpi-daemon",
    "system76-coreboot-dkms",
    "system76-dkms",
    "system76-driver",
    "system76-firmware",
    "system76-io-dkms",
    "system76-power",
    "gnome-shell-extension-system76-power",
    "system76-wallpapers"
)

def launchpad():
    return Launchpad.login_with("pop-os/pop", "production", "scripts/__lpcache__", version="devel")

def launchpad_anon():
    return Launchpad.login_anonymously("pop-os/pop", "production", "scripts/__lpcache__", version="devel")

def github_inner(url, data=None):
    headers = {"Accept": "application/vnd.github.v3+json"}

    if data:
        request_data = json.dumps(data).encode()
        headers["Content-Type"] = "application/json"
    else:
        request_data = None

    request = urllib.request.Request(
        url,
        request_data,
        headers
    )

    response = urllib.request.urlopen(request)
    return json.loads(response.read().decode())

def github(url):
    data = []
    page = 0
    per_page = 100
    while(1):
        page += 1
        page_url = url + "?page=" + str(page) + "&per_page=" + str(per_page)

        # Put a token in scripts/.github_token to increase rate limit
        if os.path.exists("scripts/.github_token"):
            f = open("scripts/.github_token")
            page_url += "&access_token=" + f.read().strip()
            f.close()

        page_data = github_inner(page_url)
        data.extend(page_data)
        if len(page_data) < per_page:
            return data

def github_no_pages(url):
    if os.path.exists("scripts/.github_token"):
        f = open("scripts/.github_token")
        url += "?access_token=" + f.read().strip()
        f.close()

    return github_inner(url)

def github_post(url, data):
    if os.path.exists("scripts/.github_token"):
        f = open("scripts/.github_token")
        url += "?access_token=" + f.read().strip()
        f.close()

    return github_inner(url, data)

def foreach_repo(fn, selected=[], dev=False):
    selected = [item.rstrip('/') for item in selected]

    repos = github("https://api.github.com/orgs/pop-os/repos")

    repos.sort(key=lambda repo: repo["name"])

    ret = {}
    for repo in repos:
        if (len(selected) == 0 or repo["name"] in selected) and (not dev or repo["name"] in DEV_REPOS):
            ret[repo["name"]] = fn(repo)

    return ret

def foreach_repo_parallel(fn, selected=[], dev=False):
    selected = [item.rstrip('/') for item in selected]

    repos = github("https://api.github.com/orgs/pop-os/repos")

    repos.sort(key=lambda repo: repo["name"])

    args = []
    keys = []
    for repo in repos:
        if (len(selected) == 0 or repo["name"] in selected) and (not dev or repo["name"] in DEV_REPOS):
            args.append(repo)
            keys.append(repo["name"])

    pool = multiprocessing.Pool()
    values = pool.map(fn, args)
    pool.close()
    pool.join()

    return dict(zip(keys, values))


# Escaping is done according to https://enterprise.github.com/downloads/en/markdown-cheatsheet.pdf
# This may need to be improved. Always check the output of generated files
def markdown_escape(string):
    escape = [
        "\\",
        "`",
        "*",
        "_",
        "{", "}",
        "[", "]",
        "(", ")",
        "#",
        "+",
        "-",
        ".",
        "!"
    ]

    for c in escape:
        string = string.replace(c, "\\" + c)

    return string
