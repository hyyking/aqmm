#!/bin/sh

echo "====> formating documents"
fish -c "fmtmd **.md" > /dev/null

echo "====> running documentation script"
sh ./scripts/documentation.sh

echo "====> deploying to github"

git worktree remove "/tmp/book" > /dev/null
git worktree add "/tmp/book" "gh-pages" > /dev/null

rm -rf /tmp/book/*
cp -rp book/* /tmp/book/

cd /tmp/book && \
	git add -A > /dev/null && \
	git commit -m "deployed by ${USER}" && \
	git push origin gh-pages
