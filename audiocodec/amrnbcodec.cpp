#include "amrnbcodec.h"
#include "amrnb/interf_enc.h"
#include "amrnb/interf_dec.h"
// #include "voAMRWB.h"

static const int SAMPLESRATE = 8000;
static const int FRAME_SAMPLES = 160; // SAMPLESRATE * 20 / 1000

class CAmrNbEncoder : public CAudioEncoder
{
   public:
      CAmrNbEncoder():
         m_state(NULL),
         m_channels(1)
      {

      }

      virtual ~CAmrNbEncoder() 
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

         if (channels != 1)
         {
            return -__LINE__;
         }
         
         if (samplingRate != SAMPLESRATE) 
         {
            return -__LINE__;
         }

         int dtx = 0;
         auto enc = Encoder_Interface_init(dtx);

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

         int frame_size = FRAME_SAMPLES * m_channels;
         int inPos = 0;
         int outPos = 0;

         while (inPos < inSize && outPos < outSize) 
         {
            int remains = inSize - inPos;
            if (remains < FRAME_SAMPLES) 
            {
               return -1;
            }

            const int forceSpeech = 0;
            int ret = Encoder_Interface_Encode(m_state, MR122, samples+inPos, out+outPos, forceSpeech);
            if (ret < 0) 
            {
               return ret;
            }

            outPos += ret;
            inPos += frame_size;
         }

         if (inPos < inSize) 
         {
            // Not enough out buffer
            return -1;
         }

         return outPos;
      }

      virtual void close( ) override
      {
         if (m_state)
         {
            Encoder_Interface_exit(m_state);
            m_state = NULL;
         }
      }
   private:
      void *m_state;
      int m_channels;
};

class CAmrNbDecoder : public CAudioDecoder
{
   public:
      CAmrNbDecoder():
         m_state(NULL),
         m_channels(1)
      {

      }

      virtual ~CAmrNbDecoder()
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

         if (channels != 1)
         {
            return -__LINE__;
         }

         if (samplingRate != SAMPLESRATE) 
         {
            return -__LINE__;
         }

         auto dec = Decoder_Interface_init();

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
         
         const int forceSpeech = 0;
         Decoder_Interface_Decode(m_state, in, outSamples, forceSpeech);
         return FRAME_SAMPLES * m_channels;
      }

      virtual void close( ) override
      {
         if (m_state)
         {
            Decoder_Interface_exit(m_state);
            m_state = NULL;
         }
      }
   private:
      void *m_state;
      int m_channels;
};

CAudioEncoderPtr new_amrnb_encoder()
{
   return std::make_shared<CAmrNbEncoder>();
}

CAudioDecoderPtr new_amrnb_decoder()
{
   return std::make_shared<CAmrNbDecoder>();
}

const CAudioCodec& get_amrnb_codec()
{
   static CAudioCodec CODEC = {"amrnb", new_amrnb_encoder, new_amrnb_decoder};
   return CODEC;
}
