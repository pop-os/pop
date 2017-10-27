import json
import os.path
import urllib.request

def github(url):
    # Put a token in scripts/.github_token to increase rate limit
    if os.path.exists("scripts/.github_token"):
        f = open("scripts/.github_token")
        url += "?access_token=" + f.read().strip()
        f.close()

    response = urllib.request.urlopen(url)
    return json.loads(response.read())

def foreach_repo(fn):
    repos = github("https://api.github.com/orgs/pop-os/repos")

    repos.sort(key=lambda repo: repo["name"])

    for repo in repos:
        if repo["name"] != "pop":
            fn(repo)

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
