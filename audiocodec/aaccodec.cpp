#include "aaccodec.h"
#include <aacenc_lib.h>
#include <aacdecoder_lib.h>

class CAacEncoder : public CAudioEncoder
{
   public:
      CAacEncoder():
         m_state(NULL),
         m_channels(1),
         m_frameSize(0)
      {

      }

      virtual ~CAacEncoder() 
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

         HANDLE_AACENCODER enc = NULL;
         int ret= 0;

         // printf("init encoder begin\n");
         do
         {
            ret = aacEncOpen(&enc, 0, 0);
            if (ret != AACENC_OK) {
               ret = -__LINE__;
               break; 
            }
            // printf("opened encoder\n");

            // ret = aacEncEncode(enc, NULL, NULL, NULL, NULL);
            // if (ret != AACENC_OK) {
            //    break;
            // }
            // printf("encoded first null frame\n");

            ret = aacEncoder_SetParam(enc, AACENC_AOT, AOT_AAC_LC);
            if (ret != AACENC_OK) {
               ret = -__LINE__;
               break;
            }
            // printf("set audio object type\n");

            ret = aacEncoder_SetParam(enc, AACENC_SAMPLERATE, samplingRate);
            if (ret != AACENC_OK) {
               ret = -__LINE__;
               break;
            }
            // printf("set sampling rate %d\n", samplingRate);

            ret = aacEncoder_SetParam(enc, AACENC_CHANNELMODE, channels);
            if (ret != AACENC_OK) {
               ret = -__LINE__;
               break;
            }
            // printf("set channel mode %d\n", channels);

            // ret = aacEncoder_SetParam(enc, AACENC_BITRATE, 24000);
            // if (ret != AACENC_OK) {
            //    break;
            // }

            ret = aacEncoder_SetParam(enc, AACENC_TRANSMUX, TT_MP4_ADTS);
            if (ret != AACENC_OK) {
               ret = -__LINE__;
               break;
            }

            ret = 0;
         } while (0);

         // printf("init encoder ret %d\n", ret);

         if (ret == 0)
         {
            m_channels = channels;
            // m_frameSize= samplingRate * 20 / 1000;
            m_frameSize = 1024;
            m_state = enc;
         } 
         else 
         {
            if (enc) 
            {
               aacEncClose(&enc);
               enc = NULL;
            }
         }

         return ret;
      }

      virtual int encode(const int16_t samples[], int inSize, uint8_t*out, int outSize) override
      {
         if (!m_state)
         {
            return -__LINE__;
         }

         AACENC_BufDesc inBufDesc, outBufDesc;
         AACENC_InArgs inArgs;
         AACENC_OutArgs outArgs;

         int frame_size = m_frameSize;
         int inPos = 0;
         int outPos = 0;

         while (inPos < inSize && outPos < outSize) 
         {
            // printf("aac encode: inSize %d, inPos %d, frame_size %d\n", inSize, inPos, frame_size);

            int remains = inSize - inPos;
            if (remains < frame_size) 
            {
               return -__LINE__;
            }

            void *inPtr = (void *) (samples + inPos);
            int inIdentifier = IN_AUDIO_DATA;
            int inNumBytes = frame_size * sizeof(int16_t);
            int inElemSize = sizeof(int16_t);

            inBufDesc.numBufs = 1;
            inBufDesc.bufs = &inPtr;
            inBufDesc.bufferIdentifiers = &inIdentifier;
            inBufDesc.bufSizes = &inNumBytes;
            inBufDesc.bufElSizes = &inElemSize;

            void *outPtr = out+outPos;
            int outIdentifier = OUT_BITSTREAM_DATA;
            int outNumBytes = outSize;
            int outElemSize = 1;

            outBufDesc.numBufs = 1;
            outBufDesc.bufs = (void **) &outPtr;
            outBufDesc.bufferIdentifiers = &outIdentifier;
            outBufDesc.bufSizes = &outNumBytes;
            outBufDesc.bufElSizes = &outElemSize;

            inArgs.numInSamples = frame_size;  


            AACENC_ERROR err = aacEncEncode(m_state, &inBufDesc, &outBufDesc, &inArgs, &outArgs);
            if (err != AACENC_OK) {
               return -__LINE__;
            }

            // printf("aac encode: numOutBytes %d\n", outArgs.numOutBytes);

            outPos += outArgs.numOutBytes;
            inPos += frame_size;
         }

         if (inPos < inSize) 
         {
            // Not enough out buffer
            return -__LINE__;
         }

         return outPos;
      }

      virtual void close( ) override
      {
         if (m_state)
         {
            aacEncClose(&m_state);
            m_state = NULL;
         }
      }
   private:
      HANDLE_AACENCODER m_state;
      int m_channels;
      int m_frameSize;
};

class CAacDecoder : public CAudioDecoder
{
   public:
      CAacDecoder():
         m_state(NULL),
         m_channels(1)
      {

      }

      virtual ~CAacDecoder()
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

         HANDLE_AACDECODER dec = aacDecoder_Open(TT_MP4_ADTS, 1);
         

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

         const UINT bufferSize = *pinSize;
         UINT valid = *pinSize;
         AAC_DECODER_ERROR decErr = aacDecoder_Fill(m_state, (uint8_t**)&in, &bufferSize, &valid);
         if (decErr != AAC_DEC_OK) {
            return -1;
         }

         decErr = aacDecoder_DecodeFrame(m_state, outSamples, outSize, 0);
         if (decErr != AAC_DEC_OK) {
            return -1;
         }

         CStreamInfo *info = aacDecoder_GetStreamInfo(m_state);
         if (!info) {
            return -1;
         }

         return info->numChannels * info->frameSize;
      }

      virtual void close( ) override
      {
         if (m_state)
         {
            aacDecoder_Close(m_state);
            m_state = NULL;
         }
      }
   private:
      HANDLE_AACDECODER m_state;
      int m_channels;
};

CAudioEncoderPtr new_aac_encoder()
{
   return std::make_shared<CAacEncoder>();
}

CAudioDecoderPtr new_aac_decoder()
{
   return std::make_shared<CAacDecoder>();
}

const CAudioCodec& get_aac_codec()
{
   static CAudioCodec CODEC = {"aac", new_aac_encoder, new_aac_decoder};
   return CODEC;
}


