cargo lambda invoke --data-file payload.json | jq -r '.body' > dist/index.html
