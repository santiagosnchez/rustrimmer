#!/usr/bin/env bash
set -euo pipefail
# Creates tests/sample.fastq.gz and paired gz files from tests/*.fastq
cd "$(dirname "$0")"
for f in *.fastq; do
	gzip -c "$f" > "${f%.fastq}.fastq.gz"
	echo "Created ${f%.fastq}.fastq.gz"
    rm "$f"
done
