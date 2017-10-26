import json
import urllib.request

def foreach_repo(fn):
    response = urllib.request.urlopen("https://api.github.com/orgs/pop-os/repos")
    repos = json.loads(response.read())

    repos.sort(key=lambda repo: repo["name"])
    for repo in repos:
        if repo["name"] != "pop":
            fn(repo)
