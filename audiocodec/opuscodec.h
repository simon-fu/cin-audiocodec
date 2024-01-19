#ifndef _OPUSCODEC_H
#define _OPUSCODEC_H

#include <memory>
#include "audiocodecs.h"

CAudioEncoderPtr new_opus_encoder();

CAudioDecoderPtr new_opus_decoder();

const CAudioCodec& get_opus_codec();

#endif
