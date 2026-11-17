#!/usr/bin/env bash
set -euo pipefail
if [[ $# != 2 ]]; then
  echo 'Usage: bash tools/encode-campaign.sh FRAME_DIRECTORY OUTPUT.gif' >&2
  exit 1
fi
# 50 fps maps exactly to GIF's 20 ms frame delay. A flat background and a global
# palette keep the image stable; supersampling preserves clean subpixel edges.
ffmpeg -nostdin -y -loglevel warning -framerate 50 -i "$1/frame-%04d.png" \
  -filter_complex '[0:v]scale=1280:640:flags=lanczos,split[a][b];[a]palettegen=stats_mode=full:max_colors=128:reserve_transparent=1[p];[b][p]paletteuse=dither=none:diff_mode=rectangle' \
  -loop 0 "$2"
if command -v gifsicle >/dev/null 2>&1; then
  floops_gif_tmp="$(mktemp "${2}.opt.XXXXXX")"
  trap 'rm -f "$floops_gif_tmp"' EXIT
  gifsicle -O3 "$2" -o "$floops_gif_tmp"
  mv "$floops_gif_tmp" "$2"
fi
