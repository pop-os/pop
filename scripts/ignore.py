#!/usr/bin/env python3

import argparse
import json
import subprocess
import sys
import urllib.request

parser = argparse.ArgumentParser(description="Generate gitignore file")
args = parser.parse_args(sys.argv[1:])

response = urllib.request.urlopen("https://api.github.com/orgs/pop-os/repos")
repos = json.loads(response.read())

ignore = ""

repos.sort(key=lambda repo: repo["name"])
for repo in repos:
    ignore += "/" + repo["name"] + "/\n"

f = open(".gitignore", "w")
f.write(ignore)
f.close()
