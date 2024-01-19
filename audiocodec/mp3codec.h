#ifndef _MP3CODEC_H
#define _MP3CODEC_H

#include "audiocodecs.h"

CAudioEncoderPtr new_mp3_encoder();

CAudioDecoderPtr new_mp3_decoder();

const CAudioCodec& get_mp3_codec();

#endif
