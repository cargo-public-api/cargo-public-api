#/usr/bin/env bash

python3 -c "import json, sys; print(json.load(sys.stdin)['${1}'])"
