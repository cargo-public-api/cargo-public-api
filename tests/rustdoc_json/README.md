To update *-expected.txt that must not have a trailing newline, do this:
```
printf "%s" "$(cargo run tests/rustdoc_json/syntect-v4.6.0_FORMAT_VERSION_10.json)" >! tests/rustdoc_json/syntect-v4.6.0-expected.txt
```
