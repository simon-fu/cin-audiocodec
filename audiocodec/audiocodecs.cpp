#include "audiocodecs.h"
#include <cstring>

class CAudioCopyEncoder : public CAudioEncoder
{
   public:
      CAudioCopyEncoder() : m_state(NULL)
      {
      }

      virtual ~CAudioCopyEncoder() 
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
         m_state = &channels;

         return 0;
      }

      virtual int encode(const int16_t samples[], int inSize, uint8_t*out, int outSize) override
      {
         if (!m_state)
         {
            return -__LINE__;
         }

         int inbytes = inSize * sizeof(samples[0]);
         int outbytes = outSize * sizeof(out[0]);

         if (inbytes > outbytes)
         {
            // printf("acopy encode: NOT enough, inbytes %d, outbytes %d\n", inbytes, outbytes);
            return -__LINE__;
         }

         memcpy(out, samples, inbytes);

         return inbytes;
      }

      virtual void close( ) override
      {
         if (m_state)
         {
            m_state = NULL;
         }
      }
   private:
      void * m_state;
};


class CAudioCopyDecoder : public CAudioDecoder
{
   public:
      CAudioCopyDecoder() : m_state(NULL)
      {
      }

      virtual ~CAudioCopyDecoder() 
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
         m_state = &channels;

         return 0;
      }

      virtual int decode(const uint8_t in[], int *pinSize, int16_t*outSamples, int outSize) override
      {
         if (!m_state)
         {
            return -__LINE__;
         }

         int inSize = *pinSize;
         auto inbytes = inSize * sizeof(in[0]);
         auto outbytes = outSize * sizeof(outSamples[0]);

         if (inbytes > outbytes)
         {
            return -__LINE__;
         }

         memcpy(outSamples, in, inbytes);

         return inbytes/sizeof(outSamples[0]);
      }

      virtual void close( ) override
      {
         if (m_state)
         {
            m_state = NULL;
         }
      }
   private:
      void * m_state;
};


CAudioEncoderPtr new_acopy_encoder()
{
   return std::make_shared<CAudioCopyEncoder>();
}

CAudioDecoderPtr new_acopy_decoder()
{
   return std::make_shared<CAudioCopyDecoder>();
}

const CAudioCodec& get_acopy_codec()
{
   static CAudioCodec CODEC = {"acopy", new_acopy_encoder, new_acopy_decoder};
   return CODEC;
}
