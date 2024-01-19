#ifndef _G729CODEC_H
#define _G729CODEC_H

#include "audiocodecs.h"

CAudioEncoderPtr new_g729_encoder();

CAudioDecoderPtr new_g729_decoder();

const CAudioCodec& get_g729_codec();

#endif
