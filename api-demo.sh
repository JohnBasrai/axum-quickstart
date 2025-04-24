#!/bin/bash
set -e

verbose="$1"

base_url="http://localhost:8080"

echo -n "Adding a movie..."
curl $verbose -X POST "$base_url/add" \
     --json '{
           "id": "t2001Qw22",
           "title": "The Princess Bride",
           "year": 1987,
           "stars": 5
         }'

echo -e -n "\nLookup movie  ..."
curl $verbose "$base_url/get/t2001Qw22" -H "Content-Type: application/json"

echo -e -n "\nBad movie     ..."
curl $verbose "$base_url/get/t2008Rxyz" -H "Content-Type: application/json"
echo
