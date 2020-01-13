#!/bin/bash
grep -e 'Oktober, 2017' akademisches_jahrbuch.txt | grep -o -e '^\w\+[,][ ]\w\+'
grep -e 'Wilhelm Schwick' akademisches_jahrbuch.txt | awk 'BEGIN {FS=" - "} { print $2 }'

