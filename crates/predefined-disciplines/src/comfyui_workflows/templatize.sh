#!/usr/bin/env bash
set -euo pipefail

dir="$(cd "$(dirname "$0")" && pwd)"

for f in "$dir"/*.json; do
  echo "Processing: $(basename "$f")"

  python3 -c "
import json, sys

with open(sys.argv[1]) as f:
    data = json.load(f)

for node in data.values():
    ct = node.get('class_type', '')
    inputs = node.get('inputs', {})

    if ct == 'KSampler' and 'steps' in inputs:
        inputs['steps'] = '__STEPS__'

    if 'width' in inputs and 'height' in inputs and 'Latent' in ct:
        inputs['width'] = '__WIDTH__'
        inputs['height'] = '__HEIGHT__'

with open(sys.argv[1], 'w') as f:
    json.dump(data, f, indent=2)
    f.write('\n')
" "$f"

done

echo "Done."
