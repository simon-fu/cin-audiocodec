#ifndef _AACCODEC_H
#define _AACCODEC_H

#include <memory>
#include "audiocodecs.h"

CAudioEncoderPtr new_aac_encoder();

CAudioDecoderPtr new_aac_decoder();

const CAudioCodec& get_aac_codec();

#endif
