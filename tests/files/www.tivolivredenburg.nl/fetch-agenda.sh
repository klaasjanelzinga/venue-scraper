#!/usr/bin/env bash
set -e

mkdir -p default-test-case/agenda/page

curl https://www.tivolivredenburg.nl/agenda/ -o default-test-case/agenda/index
for ((i = 2; i < 20; ++i)); do
  curl https://www.tivolivredenburg.nl/agenda/page/${i}/ -o default-test-case/agenda/page/${i}
done

