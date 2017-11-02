import json
import os.path
import urllib.request

# Documentation can be found here: https://launchpad.net/+apidoc/devel.html
from launchpadlib.launchpad import Launchpad

def launchpad():
    return Launchpad.login_with("pop-os/pop", "production", "scripts/__lpcache__", version="devel")

def launchpad_anon():
    return Launchpad.login_anonymously("pop-os/pop", "production", "scripts/__lpcache__", version="devel")

def github(url):
    # Put a token in scripts/.github_token to increase rate limit
    if os.path.exists("scripts/.github_token"):
        f = open("scripts/.github_token")
        url += "?access_token=" + f.read().strip()
        f.close()

    response = urllib.request.urlopen(url)
    return json.loads(response.read().decode())

def foreach_repo(fn, selected=[]):
    selected = [item.rstrip('/') for item in selected]

    repos = github("https://api.github.com/orgs/pop-os/repos")

    repos.sort(key=lambda repo: repo["name"])

    ret = {}
    for repo in repos:
        if len(selected) == 0 or repo["name"] in selected:
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
