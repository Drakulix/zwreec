#!/usr/bin/env python3
import sys

for line in sys.stdin.readlines():
  if line.replace("\n", "") != "":
    print ("file.write_zop(&" + line.replace("\n", "") + " false);")
  else:
    print
