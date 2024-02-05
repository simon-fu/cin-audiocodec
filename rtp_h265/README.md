

cd librtp && make RELEASE=1

ffmpeg -re  -i ../sample-data/sampleh265.mp4 -vcodec copy -an -f rtp -sdp_file output.sdp rtp://127.0.0.1:1234
ffplay -protocol_whitelist file,rtp,udp -i output.sdp

https://github.com/ireader/media-server
https://github.com/ultravideo/uvgRTP
