#!/bin/bash

echo "Fixing SQLx query syntax..."

# Fix query_as patterns - this is more complex and needs manual fixing
# For now, let's just identify the files that need fixing

echo "Files with SQLx queries that need manual fixing:"
grep -r "sqlx::query_as(" src/ | cut -d: -f1 | sort | uniq

echo ""
echo "Files with SQLx scalar queries that need manual fixing:"  
grep -r "sqlx::query_scalar(" src/ | cut -d: -f1 | sort | uniq

echo ""
echo "These files need manual review and fixing of SQLx query syntax."