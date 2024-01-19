#ifndef _AMRNBCODEC_H
#define _AMRNBCODEC_H

#include <memory>
#include "audiocodecs.h"

CAudioEncoderPtr new_amrnb_encoder();

CAudioDecoderPtr new_amrnb_decoder();

const CAudioCodec& get_amrnb_codec();

#endif
