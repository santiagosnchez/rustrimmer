#!/usr/bin/env bash
set -euo pipefail
# Creates tests/sample.fastq.gz from tests/sample.fastq
cd "$(dirname "$0")"
gzip -c sample.fastq > sample.fastq.gz
echo "Created sample.fastq.gz"
