#ifndef _AUDIOCODECS_H
#define _AUDIOCODECS_H

#include <stdint.h>
#include <memory>


class CAudioEncoder
{
   public:
      // CAudioEncoder();
      virtual ~CAudioEncoder() {}

      virtual int open(int channels, int samplingRate) = 0;
      virtual void close( ) = 0;

      virtual int encode(const int16_t samples[], int inSize, uint8_t*out, int outSize) = 0;

      virtual int flush(uint8_t*out, int outSize)
      {
         return 0;
      }
      
};

class CAudioDecoder
{
   public:
      // CAudioDecoder();
      virtual ~CAudioDecoder() {}

      virtual int open(int channels, int samplingRate) = 0;
      virtual void close( ) = 0;

      virtual int decode(const uint8_t in[], int *pinSize, int16_t*outSamples, int outSize) = 0;

      virtual int flush(const uint8_t in[], int *pinSize, int16_t*outSamples, int outSize)
      {
         if (*pinSize > 0)
         {
            return this->decode(in, pinSize, outSamples, outSize);
         }
         else
         {
            return 0;
         }
      }
};

typedef std::shared_ptr<CAudioEncoder> CAudioEncoderPtr;
typedef std::shared_ptr<CAudioDecoder> CAudioDecoderPtr;

typedef CAudioEncoderPtr (*MakeAudioEncoder)() ;
typedef CAudioDecoderPtr (*MakeAudioDecoder)() ;

struct CAudioCodec
{
   const std::string& name;
   MakeAudioEncoder encMaker;
   MakeAudioDecoder decMaker;
};

typedef const CAudioCodec& (*GetAudioCodec)() ;

const CAudioCodec& get_acopy_codec();

#endif
