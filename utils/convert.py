#!/usr/bin/env python3
import sys

for line in sys.stdin.readlines():
  theline = line.replace("\n", "")
  if theline != "":
    if theline != 'ZOP::PrintOps{text: "".to_string()},':
      print ("file.write_zop(&" + theline + " false);")
  else:
    print
