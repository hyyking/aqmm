#!/bin/sh

echo "====> formating documents"
fish -c "fmtmd **.md"

echo "====> deploying to github"

git worktree add "/tmp/book" "gh-pages"

rm -rf /tmp/book/*
cp -rp book/* /tmp/book/

cd /tmp/book && \
	git add -A && \
	git commit -m "deployed by ${USER}" && \
	git push origin gh-pages

git worktree remove "/tmp/book"
