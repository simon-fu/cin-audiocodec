#ifndef _G711CODEC_H
#define _G711CODEC_H

#include <memory>
#include "audiocodecs.h"

CAudioEncoderPtr new_ulaw_encoder();

CAudioDecoderPtr new_ulaw_decoder();

CAudioEncoderPtr new_alaw_encoder();

CAudioDecoderPtr new_alaw_decoder();

const CAudioCodec& get_ulaw_codec();

const CAudioCodec& get_alaw_codec();


#endif
