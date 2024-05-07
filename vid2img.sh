#!/usr/bin/env nix-shell
#!nix-shell -i bash -p bash ffmpeg imagemagick yt-dlp

rm -rf images_png images_gif
mkdir images_png images_gif

[ -f bad-apple.mp4 ] || yt-dlp -f bestvideo -o bad-apple.mp4 "https://www.youtube.com/watch?v=FtutLA63Cp8"

ffmpeg -i bad-apple.mp4 -vf "fps=8,scale=85:64,format=gray" images_png/output%d.png
mogrify -path images_gif -remap pattern:gray50 -dither FloydSteinberg -monochrome -format gif images_png/*.png
# mogrify -path images_gif -colorspace gray -ordered-dither o4x4 -monochrome images_png/*.png

convert -delay 12.5 -loop 0 $(ls images_gif/*.gif | sort -V) bad-apple.gif
