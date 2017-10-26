import json
import urllib.request

def foreach_repo(fn):
    response = urllib.request.urlopen("https://api.github.com/orgs/pop-os/repos")
    repos = json.loads(response.read())

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
