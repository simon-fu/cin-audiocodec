#include "opuscodec.h"
extern "C" {
#include "opus.h"
};


class COpusEncoder : public CAudioEncoder
{
   public:
      COpusEncoder():
         m_state(NULL),
         m_channels(1)
      {

      }

      virtual ~COpusEncoder() 
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

         int error = 0;
         auto enc = opus_encoder_create(samplingRate, channels, OPUS_APPLICATION_VOIP, &error);

         if(error != OPUS_OK)
         {
            return -__LINE__;
         }

         m_channels = channels;
         m_state = enc;

         return 0;
      }

      virtual int encode(const int16_t samples[], int inSize, uint8_t*out, int outSize) override
      {
         if (!m_state)
         {
            return -__LINE__;
         }
         
         int frame_size = inSize / m_channels;
         auto ret = opus_encode(m_state, samples, frame_size, out, outSize);
         return ret;
      }

      virtual void close( ) override
      {
         if (m_state)
         {
            opus_encoder_destroy(m_state);
            m_state = NULL;
         }
      }
   private:
      OpusEncoder *m_state;
      int m_channels;
};

class COpusDecoder : public CAudioDecoder
{
   public:
      COpusDecoder():
         m_state(NULL),
         m_channels(1)
      {

      }

      virtual ~COpusDecoder()
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

         int error = 0;
         auto dec = opus_decoder_create(samplingRate, channels, &error);

         if(error != OPUS_OK)
         {
            return -__LINE__;
         }

         m_channels = channels;
         m_state = dec;

         return 0;
      }

      virtual int decode(const uint8_t in[], int *pinSize, int16_t*outSamples, int outSize) override
      {
         if (!m_state)
         {
            return -__LINE__;
         }

         int frame_size = outSize / m_channels;
         auto ret = opus_decode(m_state, in, *pinSize, outSamples, frame_size, 0);
         if (ret < 0)
         {
            return ret;
         }
         return ret*m_channels;
      }

      virtual void close( ) override
      {
         if (m_state)
         {
            opus_decoder_destroy(m_state);
            m_state = NULL;
         }
      }
   private:
      OpusDecoder *m_state;
      int m_channels;
};

CAudioEncoderPtr new_opus_encoder()
{
   return std::make_shared<COpusEncoder>();
}

CAudioDecoderPtr new_opus_decoder()
{
   return std::make_shared<COpusDecoder>();
}


const CAudioCodec& get_opus_codec()
{
   static CAudioCodec CODEC = {"opus", new_opus_encoder, new_opus_decoder};
   return CODEC;
}
