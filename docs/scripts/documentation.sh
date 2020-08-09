#!/bin/bash

echo "# Documentation" > ./src/documentation.md && \
	exa documentation/*.pdf | sed -n "s/documentation\/\(.*\).pdf/- \1/p" >> ./src/documentation.md
