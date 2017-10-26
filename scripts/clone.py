#!/usr/bin/env python3

import argparse
import json
import subprocess
import sys
import urllib.request

parser = argparse.ArgumentParser(description="Clone all Pop!_OS repositories")
parser.add_argument('--ssh', action='store_true')

args = parser.parse_args(sys.argv[1:])

response = urllib.request.urlopen("https://api.github.com/orgs/pop-os/repos")
repos = json.loads(response.read())

repos.sort(key=lambda repo: repo["name"])
for repo in repos:
    if args.ssh:
        url = repo["ssh_url"]
    else:
        url = repo["clone_url"]

    subprocess.run(["git", "clone", "--recursive", url, repo["name"]], check=True)
