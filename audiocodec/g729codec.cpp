#include "g729codec.h"

extern "C"
{
#include <bcg729/decoder.h>
#include <bcg729/encoder.h>
}


#define SAMPLESRATE  8000
#define FRAME_SIZE  80 // 10 ms
#define CHANNELS 1
#define ENCODED_SIZE 10

class CG729Encoder : public CAudioEncoder
{
   public:
      CG729Encoder():
         m_state(NULL)
      {

      }

      virtual ~CG729Encoder() 
      {
        this->close();
      }

      virtual int open(int channels, int samplingRate) override
      {
         // this->close();
         if (m_state)
         {
            return -__LINE__;
         }

         if (channels != CHANNELS || samplingRate != SAMPLESRATE)
         {
            return -__LINE__;
         }

         m_state = initBcg729EncoderChannel(0);
         
         return 0;
      }

      virtual int encode(const int16_t samples[], int inSize, uint8_t*out, int outSize) override
      {
         if (!m_state)
         {
            return -__LINE__;
         }

         if (inSize % FRAME_SIZE!= 0)
         {
            return -__LINE__;
         }

         int encLen = 0;

         while (inSize >= FRAME_SIZE && outSize >= ENCODED_SIZE)
         {
            uint8_t bitStreamLength = 0;
            bcg729Encoder(m_state, samples, out, &bitStreamLength);
            encLen += bitStreamLength;

            outSize -= bitStreamLength;
            out += bitStreamLength;

            inSize -= FRAME_SIZE;
            samples += FRAME_SIZE;
         }
         
         return encLen;
      }

      virtual void close( ) override
      {
         if (m_state)
         {
            closeBcg729EncoderChannel(m_state);
            m_state = NULL;
         }
      }
   private:
      bcg729EncoderChannelContextStruct * m_state;
};

class CG729Decoder : public CAudioDecoder
{
   public:
      CG729Decoder():
         m_state(NULL)
      {

      }

      virtual ~CG729Decoder()
      {
         this->close();
      }

      virtual int open(int channels, int samplingRate) override
      {
         // this->close();
         if (m_state)
         {
            return -__LINE__;
         }

         if (channels != CHANNELS || samplingRate != SAMPLESRATE)
         {
            return -__LINE__;
         }

         m_state = initBcg729DecoderChannel();

         return 0;
      }

      virtual int decode(const uint8_t in[], int *pinSize, int16_t*outSamples, int outSize) override
      {
         if (!m_state)
         {
            return -__LINE__;
         }

         int inSize = *pinSize;
         
         if (inSize % ENCODED_SIZE!= 0)
         {
            return -__LINE__;
         }

         int decLen = 0;

         while (inSize >= ENCODED_SIZE && outSize >= FRAME_SIZE)
         {
            bcg729Decoder(m_state, in, inSize, 0, 0, 0, outSamples);
            decLen += FRAME_SIZE;

            outSize -= FRAME_SIZE;
            outSamples += FRAME_SIZE;

            inSize -= ENCODED_SIZE;
            in += ENCODED_SIZE;
         }
         
         return decLen;
      }

      virtual void close( ) override
      {
         if (m_state)
         {
            closeBcg729DecoderChannel(m_state);
            m_state = NULL;
         }
      }
   private:
      bcg729DecoderChannelContextStruct * m_state;
};

CAudioEncoderPtr new_g729_encoder()
{
   return std::make_shared<CG729Encoder>();
}

CAudioDecoderPtr new_g729_decoder()
{
   return std::make_shared<CG729Decoder>();
}

const CAudioCodec& get_g729_codec()
{
   static CAudioCodec CODEC = {"g729", new_g729_encoder, new_g729_decoder};
   return CODEC;
}


