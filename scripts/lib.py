import json
import os.path
import urllib.request

# Packages to release in system76-dev
DEV_REPOS = (
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
)

def github_inner(url, data=None):
    headers = {"Accept": "application/vnd.github.v3+json"}

    # Put a token in scripts/.github_token to increase rate limit
    for location in ["scripts/.github_token","./.github_token","~/.github_token"]:
    	if os.path.exists(location):
        	f = open(location)
        	headers["Authorization"] = "token " + f.read().strip()
        	f.close()
        	break

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
        page_data = github_inner(page_url)
        data.extend(page_data)
        if len(page_data) < per_page:
            return data

def github_no_pages(url):
    return github_inner(url)

def github_post(url, data):
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
