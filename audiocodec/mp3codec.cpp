#include "mp3codec.h"

extern "C"
{
#include <lame/lame.h>
}

//#define MINIMP3_ONLY_MP3
//#define MINIMP3_ONLY_SIMD
//#define MINIMP3_NO_SIMD
//#define MINIMP3_NONSTANDARD_BUT_LOGICAL
//#define MINIMP3_FLOAT_OUTPUT
#define MINIMP3_IMPLEMENTATION
// #include "minimp3.h"
#include "minimp3_ex.h"




#define BREAK_IF_NON_ZERO(ret) \
if (ret) \
{ \
   ret = -__LINE__; \
   break; \
}
            
class CMp3Encoder : public CAudioEncoder
{
   public:
      CMp3Encoder():
         m_state(NULL)
      {

      }

      virtual ~CMp3Encoder() 
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

         int ret = 0;
         lame_t lame = lame_init();
         do
         {
            ret = lame_set_in_samplerate(lame, samplingRate);
            BREAK_IF_NON_ZERO(ret)

            ret = lame_set_num_channels(lame, channels);
            BREAK_IF_NON_ZERO(ret)

            ret = lame_set_VBR(lame, vbr_default);
            BREAK_IF_NON_ZERO(ret)

            ret = lame_set_brate(lame, 16);
            BREAK_IF_NON_ZERO(ret)

            if(channels == 1)
            {
               ret = lame_set_mode(lame, MONO);
               BREAK_IF_NON_ZERO(ret)
            }
            else
            {
               ret = lame_set_mode(lame, STEREO);
               BREAK_IF_NON_ZERO(ret)
            }
               
            ret = lame_set_quality(lame, 2);
            BREAK_IF_NON_ZERO(ret)

            ret = lame_init_params(lame);
            BREAK_IF_NON_ZERO(ret)

            ret = 0;
         } while (0);
         
         if (ret == 0)
         {
            m_state = lame;
            m_channels = channels;
         }
         else
         {
            if (lame)
            {
               lame_close(lame);
               lame = NULL;
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
         // printf("mp3 encode channels %d\n", m_channels);
         if (m_channels == 1)
         {
            int ret = lame_encode_buffer(m_state, samples, samples, inSize, out, outSize);
            return ret;
         } 
         else
         {
            int ret = lame_encode_buffer_interleaved(m_state, (int16_t*)samples, inSize/m_channels, out, outSize);   
            return ret;
         }
      }

      virtual int flush(uint8_t*out, int outSize)
      {
         if (!m_state)
         {
            return -__LINE__;
         }

         int ret = lame_encode_flush(m_state, out, outSize);
         
         return ret;
      }

      virtual void close( ) override
      {
         if (m_state)
         {
            lame_close(m_state);
            m_state = NULL;
         }
      }
   private:
      lame_t m_state;
      int m_channels;
};

static int mp3d_iter_cb(void *user_data, const uint8_t *frame, int frame_size, int free_format_bytes, size_t buf_size, uint64_t offset, mp3dec_frame_info_t *info)
{
   auto pisExist = (bool *) user_data;
   *pisExist = true;
   return 0;
}

class CMp3Decoder : public CAudioDecoder
{
   public:
      CMp3Decoder():
         m_state(NULL)
      {

      }

      virtual ~CMp3Decoder()
      {
         this->close();
      }

      virtual int open(int channels, int samplingRate) override
      {
         if (m_state)
         {
            return -__LINE__;
         }

         m_state = (mp3dec_t *) malloc(sizeof(mp3dec_t));
         mp3dec_init(m_state);

         return 0;
      }

      virtual int decode(const uint8_t in[], int *pinSize, int16_t*outSamples, int outSize) override
      {
         if (!m_state)
         {
            return -__LINE__;
         }

         if (*pinSize == 0)
         {
            return 0;
         }

         // bool isExist = (*pinSize >= 16*1024);         
         bool isExist = false;
         int ret = mp3dec_iterate_buf(in, *pinSize, mp3d_iter_cb, &isExist);
         if (ret)
         {
            return -__LINE__;
         }

         if (isExist)
         {
            return this->decode_one(in, pinSize, outSamples, outSize);
         }
         else
         {
            *pinSize = 0;
            return 0;
         }


         


         // int inSize = *pinSize;
         // int inPos= 0;
         // int decSamples = 0;
         
         // do
         // {
         //    int inlen = inSize-inPos;
         //    // More than 0 samples and frame_bytes > 0: Succesful decode
         //    // 0 samples and frame_bytes >  0: The decoder skipped ID3 or invalid data
         //    // 0 samples and frame_bytes == 0: Insufficient data
         //    int samples = mp3dec_decode_frame(m_state, in+inPos, inlen, outSamples, &m_info);
         //    printf("mp3 decode: inlen %d, sample %d, frame_bytes %d\n", inlen, samples, m_info.frame_bytes);
         //    if (samples < 0)
         //    {
         //       return -__LINE__;
         //    }
            
         //    inPos += m_info.frame_bytes;
         //    if (samples > 0)
         //    {
         //       decSamples += samples * m_info.channels;
         //       break;
         //    }

         // }while(inPos < inSize && m_info.frame_bytes > 0);

         // *pinSize = inPos;

         // return decSamples;
      }

      virtual int flush(const uint8_t in[], int *pinSize, int16_t*outSamples, int outSize) override
      {
         if (!m_state)
         {
            return -__LINE__;
         }
         // printf("mp3 decode flush (%d) ---\n", *pinSize);
         return this->decode_one(in, pinSize, outSamples, outSize);

         // int inSize = *pinSize;
         // *pinSize = 0;

         // // More than 0 samples and frame_bytes > 0: Succesful decode
         // // 0 samples and frame_bytes >  0: The decoder skipped ID3 or invalid data
         // // 0 samples and frame_bytes == 0: Insufficient data
         // int samples = mp3dec_decode_frame(m_state, in, inSize, outSamples, &m_info);
         // printf("mp3 decode: inSize %d, sample %d, frame_bytes %d\n", inSize, samples, m_info.frame_bytes);
         // if (samples < 0)
         // {
         //    return -__LINE__;
         // }
         
         // if (samples > 0)
         // {
         //    *pinSize = m_info.frame_bytes;
         //    samples *= m_info.channels;
         // }

         // return samples;
      }

      virtual void close( ) override
      {
         if (m_state)
         {
            free(m_state);
            m_state = NULL;
         }
      }

   private:
      int decode_one(const uint8_t in[], int *pinSize, int16_t*outSamples, int outSize)
      {
         if (!m_state)
         {
            return -__LINE__;
         }

         int inSize = *pinSize;
         *pinSize = 0;

         // More than 0 samples and frame_bytes > 0: Succesful decode
         // 0 samples and frame_bytes >  0: The decoder skipped ID3 or invalid data
         // 0 samples and frame_bytes == 0: Insufficient data
         int samples = mp3dec_decode_frame(m_state, in, inSize, outSamples, &m_info);
         // printf("mp3 decode: inSize %d, sample %d, frame_bytes %d\n", inSize, samples, m_info.frame_bytes);
         if (samples < 0)
         {
            return -__LINE__;
         }
         
         if (samples > 0)
         {
            *pinSize = m_info.frame_bytes;
            samples *= m_info.channels;
         }

         return samples;
      }

   private:
      mp3dec_t * m_state;
      mp3dec_frame_info_t m_info;
};

// class CMp3Decoder : public CAudioDecoder
// {
//    public:
//       CMp3Decoder():
//          m_state(NULL)
//       {

//       }

//       virtual ~CMp3Decoder()
//       {
//          this->close();
//       }

//       virtual int open(int channels, int samplingRate) override
//       {
//          if (m_state)
//          {
//             return -__LINE__;
//          }

//          m_state = hip_decode_init();

//          return 0;
//       }

//       virtual int decode(const uint8_t in[], int *pinSize, int16_t*outSamples, int outSize) override
//       {
//          if (!m_state)
//          {
//             return -__LINE__;
//          }

//          int inSize = *pinSize;
//          int inPos= 0;
//          int decSamples = 0;
         
//          int pcmBufferSize = 8192;
//          short pcm_l[pcmBufferSize];
//          short pcm_r[pcmBufferSize];
//          mp3data_struct mp3data;

//          do
//          {
//             int inlen = inSize-inPos;

//             int decoded = hip_decode1_headers(m_state, (uint8_t*)in+inPos, inlen, pcm_l, pcm_r, &mp3data);
//             int decoded = hip_decode1(m_state, (uint8_t*)in+inPos, inlen, pcm_l, pcm_r);
//             int decoded = hip_decode(m_state, (uint8_t*)in+inPos, inlen, pcm_l, pcm_r);

//             // More than 0 samples and frame_bytes > 0: Succesful decode
//             // 0 samples and frame_bytes >  0: The decoder skipped ID3 or invalid data
//             // 0 samples and frame_bytes == 0: Insufficient data
//             int samples = mp3dec_decode_frame(m_state, in+inPos, inlen, outSamples, &m_info);
//             printf("mp3 decode: inlen %d, sample %d, frame_bytes %d\n", inlen, samples, m_info.frame_bytes);
//             if (samples < 0)
//             {
//                return -__LINE__;
//             }
            
//             inPos += m_info.frame_bytes;
//             if (samples > 0)
//             {
//                decSamples += samples * m_info.channels;
//                break;
//             }

//          }while(inPos < inSize && m_info.frame_bytes > 0);

//          *pinSize = inPos;

//          return decSamples;
//       }

//       virtual void close( ) override
//       {
//          if (m_state)
//          {
//             free(m_state);
//             m_state = NULL;
//          }
//       }
//    private:
//       hip_t m_state;
// };


CAudioEncoderPtr new_mp3_encoder()
{
   return std::make_shared<CMp3Encoder>();
}

CAudioDecoderPtr new_mp3_decoder()
{
   return std::make_shared<CMp3Decoder>();
}

const CAudioCodec& get_mp3_codec()
{
   static CAudioCodec CODEC = {"mp3", new_mp3_encoder, new_mp3_decoder};
   return CODEC;
}


