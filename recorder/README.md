- 如何让ffmpeg 输出的rtp流里包含 spspps？  
答： 未测试方案：https://stackoverflow.com/questions/65800733/send-sprop-parameter-sets-inband-rather-than-in-sdp
        -bsf:v extract_extradata,dump_extra


- 为什么要先probe？  
答：原因如下  
    a）生成mp4时，需要知道有哪些track；  
    b）对于 h264 track，还需要得到 spspps 数据，
    c）如果sdp里没有spsppps，是否还需要继续向前探测得到 spspps？
