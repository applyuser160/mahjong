import urllib.request
import json

req = urllib.request.Request("https://api.github.com/repos/PyO3/maturin/releases/latest")
try:
    with urllib.request.urlopen(req) as response:
        print(response.read().decode('utf-8')[:100])
except Exception as e:
    print(e)
