#!/bin/bash
cat output.json | jq -M '"\(.title), by \(.author) (\(.date))"' | cut -d '"' -f 2 | sed -e 's/ (Unknown)//'
cat output.json | jq -M '"[\(.world)] - %\(.board)%"' | cut -d '"' -f 2 | tr '%' '"'
cat output.json | jq -M '"\(.url)"' | cut -d '"' -f 2
