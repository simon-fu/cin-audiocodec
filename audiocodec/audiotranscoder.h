#ifndef _AUDIOTRANSCODER_H
#define _AUDIOTRANSCODER_H

#include <memory>
#include "audiocodecs.h"



#define DEF_VALUE_CLASS(_xname, _xtype, _xdefault) \
class _xname \
{ \
   public: \
      _xname(): m_value(_xdefault) {} \
      _xname(int value): m_value(value) {} \
      inline _xtype value() const { return m_value; } \
   private: \
      _xtype m_value; \
}

DEF_VALUE_CLASS(Channels, int, 0);
DEF_VALUE_CLASS(Samplerate, int, 0);
DEF_VALUE_CLASS(Framesize, int, 0);

class FrameMillis
{
   public: 
      FrameMillis(int millis): m_millis(millis) {}
      inline Framesize framesize(const Samplerate& hz) { return Framesize(m_millis * hz.value() / 1000); }
   private:
      int m_millis;
};

class FrameSamples
{
   public: 
      FrameSamples(int samples): m_samples(samples) {}
      inline Framesize framesize(const Samplerate& hz) { return Framesize(m_samples); }
   private:
      int m_samples;
};

struct AudioCodecArgs
{
   // int channels;
   // int samplerate; 
   // int framesize;

   Channels channels;
   Samplerate samplerate; 
   Framesize framesize;

   int calcMillis(int samples) const
   {
      return 1000 * samples / this->channels.value() / this->samplerate.value();
   }

   int calcSamples(int millis) const
   {
      return this->channels.value() * this->samplerate.value() * millis / 1000 ;
   }

   int convertSampleCount(const AudioCodecArgs& other, int samples) const
   {
      auto millis = this->calcMillis(samples);
      return other.calcSamples(millis);
   }
};

class CAudioTranscoder
{
   public:
   // CAudioTranscoder();
   virtual ~CAudioTranscoder() {}

   virtual int open
   (
      std::shared_ptr<CAudioDecoder> srcDec, 
      AudioCodecArgs src,
      std::shared_ptr<CAudioEncoder> dstEnc,
      AudioCodecArgs dst
   ) = 0;

   virtual void close( ) = 0;

   virtual int push(const uint8_t data[], int len) = 0;
   
   virtual int pull(uint8_t buf[], const int bufSize) = 0;

   virtual int flush(uint8_t buf[], const int bufSize) = 0;
};

std::shared_ptr<CAudioTranscoder> new_audio_transcoder();

int testSpeexResampler(const std::string& srcFile, const AudioCodecArgs& srcArgs, const std::string& dstFile, const AudioCodecArgs& dstArgs);

#endif
