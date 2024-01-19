#ifndef _AMRWBCODEC_H
#define _AMRWBCODEC_H

#include <memory>
#include "audiocodecs.h"

CAudioEncoderPtr new_amrwb_encoder();

CAudioDecoderPtr new_amrwb_decoder();

const CAudioCodec& get_amrwb_codec();

#endif
