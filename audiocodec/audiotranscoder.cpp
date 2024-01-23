
#include "audiotranscoder.h"
// #include "resampler.h"
#include "speex_resampler.h"
#include <vector>
#include <cstring>

template<typename T>
class RwBuf
{
   public:
      void reinit(int size)
      {
         this->buf.resize(size);
         this->pos = 0;
         this->end = 0;
      }

      void clear()
      {
         this->pos = 0;
         this->end = 0;
      }

      void trim()
      {
         if (this->pos > 0)
         {
            int remains = this->remains();
            if (remains > 0)
            {
               std::memmove(this->buf.data(), this->rData(), remains*sizeof(T));
            }
            this->pos = 0;
            this->end = remains;
         }
      }

      int remains()
      {
         return this->end - this->pos;
      }

      T * wBuf()
      {
         return this->buf.data() + this->end;
      }

      int wSize()
      {
         return this->buf.size() - this->end;
      }

      void wAdvance(int cnt)
      {
         this->end += cnt;
      }

      void reserve(int extra)
      {
         this->buf.resize(this->buf.size() + extra);
      }

      void append(const T * in, int inlen)
      {
         if (this->wSize() < inlen)
         {
            this->reserve(inlen - this->wSize());
         }

         std::memcpy(this->wBuf(), in, inlen*sizeof(T));
      }
      
      const T * rData()
      {
         return this->buf.data() + this->pos;
      }

      int rLen()
      {
         return this->remains();
      }

      void rAdvance(int cnt)
      {
         this->pos += cnt;
      }

   private:
      std::vector<T> buf;
      int pos;
      int end;
};

typedef RwBuf<int16_t> PcmBuf;

// class PcmBuf
// {
//    public:
//       void reinit(int size)
//       {
//          this->buf.resize(size, 0);
//          this->pos = 0;
//          this->end = 0;
//       }

//       void clear()
//       {
//          this->pos = 0;
//          this->end = 0;
//       }

//       void trim()
//       {
//          if (this->pos > 0)
//          {
//             int remains = this->remains();
//             if (remains > 0)
//             {
//                std::memmove(this->buf.data(), this->rData(), remains*sizeof(int16_t));
//             }
//             this->pos = 0;
//             this->end = remains;
//          }
//       }

//       int remains()
//       {
//          return this->end - this->pos;
//       }

//       int16_t * wBuf()
//       {
//          return this->buf.data() + this->end;
//       }

//       int wSize()
//       {
//          return this->buf.size() - this->end;
//       }

//       void wAdvance(int cnt)
//       {
//          this->end += cnt;
//       }

      
//       const int16_t * rData()
//       {
//          return this->buf.data() + this->pos;
//       }

//       int rLen()
//       {
//          return this->remains();
//       }

//       void rAdvance(int cnt)
//       {
//          this->pos += cnt;
//       }

//    private:
//       std::vector<int16_t> buf;
//       int pos;
//       int end;
// };




class CAudioTranscoderImpl: public CAudioTranscoder
{
   public:
   CAudioTranscoderImpl()
   : m_speexResampler(NULL)
   {

   }

   virtual ~CAudioTranscoderImpl() {
      this->close();
   }

   virtual int open
   (
      std::shared_ptr<CAudioDecoder> srcDec, 
      AudioCodecArgs src,
      std::shared_ptr<CAudioEncoder> dstEnc,
      AudioCodecArgs dst
   ) override
   {
      if (m_srcDec || m_dstEnc)
      {
         return -__LINE__;
      }
      if (!srcDec || !dstEnc)
      {

         return -__LINE__;
      }

      if (src.samplerate.value() <= 0 || src.channels.value() <= 0 
         || dst.samplerate.value() <= 0 || dst.channels.value() <= 0) 
      {
         return -__LINE__;
      }

      if (src.channels.value() != dst.channels.value()) 
      {
         return -__LINE__;
      }

      if (dst.framesize.value() <= 0)
      {
         return -__LINE__;
      }

      {
         auto codec = &src;
         int default_frame_size = codec->samplerate.value() * 500 / 1000; // 500 milliseconds
         int frameSize = std::max(codec->framesize.value(), default_frame_size);

         m_srcPcm.reinit(frameSize * codec->channels.value());
      }

      if (src.samplerate.value() != dst.samplerate.value())
      {
         // m_resampler = std::make_shared<Resampler>();
         // int ret = m_resampler->Reset(src.samplerate, dst.samplerate, src.channels);
         // printf("make resampler: src.samplerate %d, dst.samplerate %d, src.channels %d, ret %d\n", src.samplerate, dst.samplerate, src.channels, ret);

         if (m_speexResampler)
         {
            speex_resampler_destroy(m_speexResampler);
            m_speexResampler = NULL;
         }

         int err = 0;
         m_speexResampler = speex_resampler_init(src.channels.value(),
                                          src.samplerate.value(),
                                          dst.samplerate.value(),
                                          SPEEX_RESAMPLER_QUALITY_DEFAULT,
                                          &err);
         if (!m_speexResampler)
         {
            return -__LINE__;
         }
         

         auto codec = &dst;
         int default_frame_size = codec->samplerate.value() * 500 / 1000; // 500 milliseconds
         int frameSize = std::max(codec->framesize.value(), default_frame_size);

         m_dstPcm.reinit(frameSize * codec->channels.value());
      }


      m_srcDec = srcDec;
      m_dstEnc = dstEnc;

      m_srcCodec = src;
      m_dstCodec = dst;
      m_srcBuf.clear();

      return 0;
   }

   virtual void close( ) override
   {
      if (m_srcDec)
      {
         m_srcDec->close();
         m_srcDec.reset();
      }
      
      if (m_dstEnc)
      {
         m_dstEnc->close();
         m_dstEnc.reset();
      }

      // if (m_resampler)
      // {
      //    m_resampler.reset();
      // }

      if (m_speexResampler)
      {
         speex_resampler_destroy(m_speexResampler);
         m_speexResampler = NULL;
      }
   }

   virtual int push(const uint8_t data[], int len) override
   {
      int ret = 0;

      m_srcPcm.trim();

      if (m_srcBuf.rLen() > 0)
      {
         m_srcBuf.trim();
         m_srcBuf.append(data, len);

         ret = this->tryDecodeInternal();
         if (ret < 0)
         {
            return ret;
         }
      }
      else 
      {
         int consumed = len;
         ret = m_srcDec->decode(data, &consumed, m_srcPcm.wBuf(), m_srcPcm.wSize());
         // printf("trans src decode ret %d\n", ret);
         if (ret < 0)
         {
            return ret;
         }
         m_srcPcm.wAdvance(ret);

         if (consumed < len)
         {
            m_srcBuf.append(data+consumed, len-consumed);
         }
      }

      

      // if (this->m_resampler)
      // {
      //    m_dstBuf.trim();

      //    size_t outLen = 0;
      //    ret = this->m_resampler->Push(m_srcBuf.rData(), m_srcBuf.rLen(), m_dstBuf.wBuf(), m_dstBuf.wSize(), outLen);
      //    if (ret != 0)
      //    {
      //       printf("trans resample failed %d, rLen %d, wSize %d\n", ret, m_srcBuf.rLen(), m_dstBuf.wSize());
      //       return -__LINE__;
      //    }
      //    m_dstBuf.wAdvance(outLen);
      //    m_srcBuf.clear();
      // }

      if (m_speexResampler)
      {
         m_dstPcm.trim();

         // const spx_int16_t * in = m_srcBuf.rData();
         spx_uint32_t in_len = m_srcPcm.rLen()/m_srcCodec.channels.value();
         // spx_int16_t *out = m_dstBuf.wBuf();
         spx_uint32_t out_len = m_dstPcm.wSize()/m_dstCodec.channels.value();
         ret = speex_resampler_process_interleaved_int(m_speexResampler,
                                                      m_srcPcm.rData(),
                                                      &in_len,
                                                      m_dstPcm.wBuf(),
                                                      &out_len);
         // printf("trans resample ret %d, rLen %d, wSize %d, in_len %d, out_len %d\n", ret, m_srcPcm.rLen(), m_dstPcm.wSize(), in_len, out_len);
         if (ret < 0)
         {
            return -__LINE__;
         }
         m_dstPcm.wAdvance(out_len * m_dstCodec.channels.value());
         m_srcPcm.rAdvance(in_len * m_srcCodec.channels.value());
      }

      return len;
   }
   
   virtual int pull(uint8_t buf[], const int bufSize) override
   {
      PcmBuf* pcmbuf = this->getPcmBuf();

      int remains = pcmbuf->remains();
      int expect = (m_dstCodec.framesize.value() * m_dstCodec.channels.value());

      if (remains <  expect)
      {
         int ret = this->tryDecodeInternal();
         if (ret <= 0)
         {
            return ret;
         }
      }

      if (remains >=  expect)
      {
         int ret = m_dstEnc->encode(pcmbuf->rData(), expect, buf, bufSize);
         // printf("trans dst encode ret %d\n", ret);
         if (ret >= 0)
         {
            pcmbuf->rAdvance(expect);
         }
         // printf("trans dst encode remains1 %d\n", pcmbuf->remains());
         return ret;
      } 
      else 
      {
         // printf("trans dst encode remains2 %d, resample %d\n", remains, m_speexResampler != nullptr);
         return 0;
      }
   }

   virtual int flush(uint8_t buf[], const int bufSize) override
   {
      int ret = m_dstEnc->flush(buf, bufSize);
      return ret;
   }

   private:
      int tryDecodeInternal()
      {
         int inlen = m_srcBuf.rLen();
         if (inlen > 0)
         {
            int ret = m_srcDec->decode(m_srcBuf.rData(), &inlen, m_srcPcm.wBuf(), m_srcPcm.wSize());
            if (ret < 0)
            {
               return ret;
            }
            m_srcPcm.wAdvance(ret);

            m_srcBuf.rAdvance(inlen);
         }
         return inlen;
      }

      PcmBuf* getPcmBuf()
      {
         if (m_speexResampler)
         {
            return &m_dstPcm;
         }
         else{
            return &m_srcPcm;
         }
      }

   private:
      std::shared_ptr<CAudioDecoder> m_srcDec;
      std::shared_ptr<CAudioEncoder> m_dstEnc;
      // std::shared_ptr<Resampler> m_resampler;
      SpeexResamplerState * m_speexResampler;
      
      RwBuf<uint8_t> m_srcBuf;

      PcmBuf m_srcPcm;
      PcmBuf m_dstPcm;

      AudioCodecArgs m_srcCodec;
      AudioCodecArgs m_dstCodec;
};

std::shared_ptr<CAudioTranscoder> new_audio_transcoder()
{
   return std::make_shared<CAudioTranscoderImpl>();
}




