#include "amrwbcodec.h"
#include "enc_if.h"
#include "amrwb/dec_if.h"
#include "voAMRWB.h"

static const int SAMPLESRATE = 16000;
static const int FRAME_SAMPLES = 320; // SAMPLESRATE * 20 / 1000

class CAmrWbEncoder : public CAudioEncoder
{
   public:
      CAmrWbEncoder():
         m_state(NULL),
         m_channels(1)
      {

      }

      virtual ~CAmrWbEncoder() 
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

         auto enc = E_IF_init();

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

         int frameAll = FRAME_SAMPLES * m_channels;
         int inPos = 0;
         int outPos = 0;

         while (inPos < inSize && outPos < outSize) 
         {
            int remains = inSize - inPos;
            if (remains < frameAll) 
            {
               return -1;
            }

            int ret = E_IF_encode(m_state, VOAMRWB_MD2385, samples+inPos, out+outPos, 0);
            if (ret < 0) 
            {
               return ret;
            }

            outPos += ret;
            inPos += frameAll;
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
            E_IF_exit(m_state);
            m_state = NULL;
         }
      }
   private:
      void *m_state;
      int m_channels;
};

class CAmrWbDecoder : public CAudioDecoder
{
   public:
      CAmrWbDecoder():
         m_state(NULL),
         m_channels(1)
      {

      }

      virtual ~CAmrWbDecoder()
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

         auto dec = D_IF_init();

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
         
         D_IF_decode(m_state, in, outSamples, 0);
         return FRAME_SAMPLES * m_channels;
      }

      virtual void close( ) override
      {
         if (m_state)
         {
            D_IF_exit(m_state);
            m_state = NULL;
         }
      }
   private:
      void *m_state;
      int m_channels;
};


CAudioEncoderPtr new_amrwb_encoder()
{
   return std::make_shared<CAmrWbEncoder>();
}

CAudioDecoderPtr new_amrwb_decoder()
{
   return std::make_shared<CAmrWbDecoder>();
}

const CAudioCodec& get_amrwb_codec()
{
   static CAudioCodec CODEC = {"amrwb", new_amrwb_encoder, new_amrwb_decoder};
   return CODEC;
}
