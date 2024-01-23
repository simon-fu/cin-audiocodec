/*
   - ffmpeg -i /tmp/sample-data/sample.mp4 -ss 00:00:10 -t 00:00:05 -acodec pcm_s16le -f s16le -ar 48000 -ac 1 /tmp/sample-48000Hz-1ch.pcm

   - ffmpeg -i /tmp/sample-data/sample.mp4 -ss 00:00:10 -t 00:00:05 -acodec pcm_s16le -f s16le -ar 16000 -ac 1 /tmp/sample-16000Hz-1ch.pcm

   - ffmpeg -i /tmp/sample-data/sample.mp4 -ss 00:00:10 -t 00:00:05 -acodec pcm_s16le -f s16le -ar 8000 -ac 1 /tmp/sample-8000Hz-1ch.pcm

   - ffmpeg -f s16le -ac 1 -ar 16000 -i dst_001_mp3_16000hz1ch.pcm dst_001_mp3_16000hz1ch.wav
*/

#include <stdio.h>
#include <cmath>
#include <memory>
#include <iostream>
#include <iomanip>
#include <fstream>
#include <sstream>
#include <cstring>
#include <string>
#include <vector>
#include <cstdarg>
#include "g711codec.h"
#include "opuscodec.h"
#include "amrwbcodec.h"
#include "amrnbcodec.h"
#include "aaccodec.h"
#include "g729codec.h"
#include "mp3codec.h"
#include "audiotranscoder.h"
// #include <iostream>
#include "speex_resampler.h"
extern "C"
{
   #include "pesqraw.h"
}



const double A4_FREQ = 440.0; // A4
const double C5_FREQ = 523.25; // C5

class ToneGenerator {
private:
   int m_sampleRate;
   int m_channels;
   double m_leftFreq;
   double m_rightFreq;
   double m_beepDuration;
   int m_beepLength;
   int m_beepToggle; // For channel switching
   int m_position; // To maintain the position in the buffer across calls

public:
   ToneGenerator(int sampleRate, int channels, double leftFreq, double rightFreq, double beepDuration)
      : m_sampleRate(sampleRate), m_channels(channels), m_leftFreq(leftFreq), m_rightFreq(rightFreq), m_beepDuration(beepDuration), m_beepToggle(0), m_position(0) {
      m_beepLength = static_cast<int>(m_sampleRate * m_beepDuration);
   }

   ToneGenerator(int sampleRate, int channels)
      : ToneGenerator(sampleRate, channels, A4_FREQ, C5_FREQ, 0.5) {
      
   }

   void generate(int16_t* buffer, int bufferSize) {
      this->generateTone(buffer, bufferSize);
      // this->generateSpeechFreq(buffer, bufferSize);
   }

   void generateTone(int16_t* buffer, int bufferSize) {
      int channels = m_channels;
      for (int i = 0; i < bufferSize; i += channels) {
         double frequency = (m_beepToggle ? m_rightFreq : m_leftFreq);
         int16_t sample = static_cast<int16_t>(32767.0 * sin((2.0 * M_PI * frequency * m_position) / m_sampleRate));
         
         for (int j = 0; j < channels; j++) {
               buffer[i + j] = (channels == 1) ? sample : (j == m_beepToggle ? sample : 0);
         }

         m_position++;
         if (m_position % m_beepLength == 0) {
               m_beepToggle = !m_beepToggle; // Switch the channel
         }
      }
   }

   // void generateSpeechFreq(int16_t* buffer, int bufferSize) {
   //    const double TWO_PI = 6.283185307179586476925286766559;
   //    const double MAX_AMP = 32760;  // "volume"

   //    double hz = 300.0;
   //    for (int n = 0; n < bufferSize; n++) {
   //       // Gradually increase the frequency from 300Hz to 3400Hz
   //       if (n % 100 == 0 && hz < 3400.0) {
   //          hz += 10.0;
   //       }

   //       // double amplitude = (double)n / bufferSize * MAX_AMP;
   //       // double value     = sin((TWO_PI * hz / m_sampleRate) * n) * amplitude;
   //       double value     = sin((TWO_PI * hz / m_sampleRate) * n) * MAX_AMP;
   //       buffer[n] = (short)value;
   //    }
   // }
};

static int write_to_file(const std::string& filename, const char * buffer, std::streamsize length)
{
   std::ofstream file(filename, std::ios::binary);
   if (!file.is_open()) {
      std::cerr << "Unable to open file\n";
      return -1;
   }
   file.write(buffer, length);
   file.close();
   return 0;
}

static inline std::string strFormat(const char* fmt, ...)
{
   int size = 512;
   char* buffer = 0;
   buffer = new char[size];
   va_list vl;
   va_start(vl, fmt);
   int nsize = vsnprintf(buffer, size, fmt, vl);
   if(size<=nsize){ //fail delete buffer and try again
      delete[] buffer;
      buffer = 0;
      buffer = new char[nsize+1]; //+1 for /0
      nsize = vsnprintf(buffer, size, fmt, vl);
   }
   std::string ret(buffer);
   va_end(vl);
   delete[] buffer;
   return ret;
}

class ErrMsg
{
   public:
      static inline ErrMsg fmt(const char* fmt, ...) 
      {
         int size = 512;
         char* buffer = 0;
         buffer = new char[size];
         va_list vl;
         va_start(vl, fmt);
         int nsize = vsnprintf(buffer, size, fmt, vl);
         if(size<=nsize){ //fail delete buffer and try again
            delete[] buffer;
            buffer = 0;
            buffer = new char[nsize+1]; //+1 for /0
            nsize = vsnprintf(buffer, size, fmt, vl);
         }
         std::string ret(buffer);
         va_end(vl);
         delete[] buffer;

         return ErrMsg(ret);
      }

      ErrMsg(const std::string& str) : m_msg(str) {}
      ErrMsg() : m_msg("") {}

      bool isOk()
      {
         return m_msg.empty();
      }

      const std::string& msg()
      {
         return m_msg;
      }

   private:
      std::string m_msg;
};

class ErrOStr : public std::ostringstream {
   public:
      ErrMsg msg()
      {
         auto content = this->str();
         return ErrMsg(content);
      }
};

#define ret_if_err(ret) if (!(ret).isOk()) { return (ret); }


static int64_t calculateMSE(int16_t data1[], int16_t data2[], int length) {
   int64_t mse = 0;

   for (int i = 0; i < length; i++) {
      auto diff1 = data1[i] - data2[i];
      auto diff2 = data2[i] - data1[i];
      auto diff1_s = diff1 * diff1;
      auto diff2_s = diff2 * diff2;
      if (diff1_s <= diff2_s) 
      {
         mse += diff1_s;
      }
      else
      {
         mse += diff2_s;
      }
   }

   mse /= length;
   return mse;
}

// typedef std::shared_ptr<CAudioEncoder> (*MakeEncoder)() ;
// typedef std::shared_ptr<CAudioDecoder> (*MakeDecoder)() ;
// typedef const CAudioCodec& (*GetAudioCodec)() ;



struct CCodecDesc {
   std::string name;
   MakeAudioEncoder encMaker;
   MakeAudioDecoder decMaker;
   // int frameMS; // milliseconds
   int frame_size; // sample per seconds
   std::string encoded_file_ext; 
};



#define FRAME_SIZE(rate, ms) ((ms) * (rate) / 1000)

static int readPCMData(const std::string &filename, std::vector<int16_t> &buffer) 
{
   std::ifstream file(filename, std::ios::binary);
   if (!file) {
      return -1;
   }

   // 获取文件大小
   file.seekg(0, std::ios::end);
   size_t file_size = file.tellg();
   file.seekg(0, std::ios::beg);

   // 调整vector的大小以容纳所有数据
   buffer.resize(file_size / sizeof(int16_t));

   // 从文件中读取数据到buffer
   file.read(reinterpret_cast<char*>(buffer.data()), file_size);
   if (!file) {
      return -2;
   }

   file.close();
   return 0;
}

struct PcmBuf
{
   int m_samplingRate;
   int m_channels;
   std::vector<int16_t> m_pcmBuf;

   int genPcm(int samplingRate, int channels, int seconds)
   {
      m_samplingRate = samplingRate;
      m_channels = channels;

      int length = samplingRate * channels * seconds;

      m_pcmBuf = std::vector<int16_t>(length, 0);

      ToneGenerator gen = ToneGenerator(samplingRate, channels);
      gen.generate(m_pcmBuf.data(), m_pcmBuf.size());

      // m_encBuf = std::vector<uint8_t>(m_pcmBuf.size(), 0);
      // m_decBuf = std::vector<int16_t>(m_pcmBuf.size(), 0);

      // if (!m_dumpDir.empty())
      // {
      //    std::ostringstream filenameStream;
      //    filenameStream << m_dumpDir << "/src_" << m_samplingRate << "hz_" << m_channels << "ch.pcm";

      //    std::string filename = filenameStream.str();
      //    auto nbytes = m_pcmBuf.size() * sizeof(int16_t);
      //    write_to_file(filename, reinterpret_cast<const char*>(m_pcmBuf.data()), nbytes);

      //    std::cout << "write to file: " << filename << ", bytes " << nbytes << std::endl;
      // }

      return 0;
   }

   int loadPCMFile(const Samplerate& samplingRate, const Channels& channels, const std::string &filename) 
   {
      {
         auto ret = readPCMData(filename, m_pcmBuf);
         if (ret != 0)
         {
            return ret;
         }
      }

      m_samplingRate = samplingRate.value();
      m_channels = channels.value();

      // m_encBuf = std::vector<uint8_t>(m_pcmBuf.size(), 0);
      // m_decBuf = std::vector<int16_t>(m_pcmBuf.size(), 0);
      
      return 0;
   }
};


struct TestContext
{
   bool forceGen;
   std::string dumpDir;
   int maxMillis;
   PcmBuf pcmBuf;

   ErrMsg makePcmData(const bool& forceGen, const std::string& pcmFile, const AudioCodecArgs& args, int millis)
   {
      if (pcmFile.empty() || forceGen)
      {
         auto ret = pcmBuf.genPcm(args.samplerate.value(), args.channels.value(), (millis + 999)/1000);
         if (ret != 0)
         {
            ErrOStr os;
            os 
               << "gen pcm failed: ret=[" << ret << "]" 
               <<  ", samplingRate=[" << args.samplerate.value() << "]"
               <<  ", channels=[" << args.channels.value() << "]"
               // << std::endl
            ;
            return os.msg();
         }
      }
      else
      {
         auto ret = pcmBuf.loadPCMFile(args.samplerate, args.channels, pcmFile);
         if (ret != 0)
         {
            ErrOStr os;
            os << "load pcm file failed: ret=[" << ret << "]" 
               <<  ", file=[" << pcmFile << "]"
               // << std::endl
            ;
            return os.msg();
         }
         std::cout << "loaded pcm file: [" << pcmFile << "]" << std::endl;

         int maxLen = args.samplerate.value() * args.channels.value() * millis / 1000;
         if (pcmBuf.m_pcmBuf.size() > maxLen)
         {
            pcmBuf.m_pcmBuf.resize(maxLen, 0);
         }
      }

      return ErrMsg();
   }
};



class Tester
{
   public: 

      Tester(const std::string& dumpDir):
         m_samplingRate(0),
         m_channels(0),
         m_dumpDir(dumpDir),
         m_pcmBuf(),
         m_encBuf(),
         m_decBuf()
      {
      }

      int genPcm(int samplingRate, int channels, int seconds)
      {
         m_samplingRate = samplingRate;
         m_channels = channels;

         int length = samplingRate * channels * seconds;

         m_pcmBuf = std::vector<int16_t>(length, 0);

         ToneGenerator gen = ToneGenerator(samplingRate, channels);
         gen.generate(m_pcmBuf.data(), m_pcmBuf.size());

         m_encBuf = std::vector<uint8_t>(m_pcmBuf.size(), 0);
         m_decBuf = std::vector<int16_t>(m_pcmBuf.size(), 0);

         if (!m_dumpDir.empty())
         {
            std::ostringstream filenameStream;
            filenameStream << m_dumpDir << "/src_" << m_samplingRate << "hz_" << m_channels << "ch.pcm";

            std::string filename = filenameStream.str();
            auto nbytes = m_pcmBuf.size() * sizeof(int16_t);
            write_to_file(filename, reinterpret_cast<const char*>(m_pcmBuf.data()), nbytes);

            std::cout << "write to file: " << filename << ", bytes " << nbytes << std::endl;
         }

         return 0;
      }

      int loadPCMFile(int samplingRate, int channels, const std::string &filename) 
      {
         {
            auto ret = readPCMData(filename, m_pcmBuf);
            if (ret != 0)
            {
               return ret;
            }
         }

         m_samplingRate = samplingRate;
         m_channels = channels;

         m_encBuf = std::vector<uint8_t>(m_pcmBuf.size(), 0);
         m_decBuf = std::vector<int16_t>(m_pcmBuf.size(), 0);
         
         return 0;
      }

      std::string testCodec(const CCodecDesc& codec)
      {
         // int unitSamples = m_channels * codec.frameMS * m_samplingRate / 1000;
         int frameSampAll = m_channels * codec.frame_size;

         auto buffer = m_pcmBuf.data();
         int bufferLength = m_pcmBuf.size() - (m_pcmBuf.size() % frameSampAll);

         auto encoder = codec.encMaker();
         auto decoder = codec.decMaker();
         
         {
            int ret = encoder->open(m_channels, m_samplingRate);
            if (ret != 0) 
            {
               return strFormat("encoder open failed %d", ret);
            }
         }

         {
            int ret = decoder->open(m_channels, m_samplingRate);
            if (ret != 0) 
            {
               return strFormat("decoder open failed %d", ret);
            }
         }

         // std::shared_ptr<std::ofstream> encoded_file;
         // if (!codec.encoded_file_ext.empty()) 
         // {
         //    std::ostringstream filenameStream;
         //    filenameStream << m_dumpDir << "/dst_" << codec.name << "_" << m_samplingRate << "hz_" << m_channels << "ch" << codec.encoded_file_ext;

         //    std::string filename = filenameStream.str();

         //    encoded_file = std::make_shared<std::ofstream>(filename, std::ios::binary); // new std::ofstream(filename, std::ios::binary);
         //    if (!encoded_file->is_open()) {
         //       return strFormat("Unable to open file [%s]", filename.c_str());
         //    }
         // }

         auto encBuf = m_encBuf.data();
         auto decBuf = m_decBuf.data();
         int encBufSize = m_encBuf.size();
         int decBufSize = m_decBuf.size();

         int srcPos = 0;
         int encPos = 0;
         int decPos = 0;
         int encConsumed = 0;

         while ((srcPos + frameSampAll) <= bufferLength)
         {
            int encLen = encoder->encode(buffer+srcPos, frameSampAll, encBuf+encPos, encBufSize-encPos);
            // std::cout << "encoded length: " << encLen << std::endl;
            if (encLen <= 0) 
            {
               return strFormat("encode failed %d", encLen);
            }
            srcPos += frameSampAll;
            encPos += encLen;

            int decPosOld = decPos;
            while (encConsumed < encPos)
            {
               int inlen = encPos - encConsumed;
               int ret = decoder->decode(encBuf+encPos, &inlen, decBuf+decPos, decBufSize-decPos);
               // std::cout << "decoded length: " << decLen << std::endl;
               if (ret < 0) 
               {
                  return strFormat("decode failed %d", ret);
               }
               encConsumed += inlen;

               if (ret == 0)
               {
                  break;
               }

               decPos += ret;
            }
            
            int decLen = (decPos- decPosOld) ;
            if (decLen != frameSampAll) 
            {
               return strFormat("decoded frame samples expect %d but %d", frameSampAll, decLen);
            }
         }

         if (decPos != bufferLength)
         {
            return strFormat("decode all samples expect %d but %d", bufferLength, decPos);
         }

         {
            auto mse = calculateMSE(buffer, decBuf, bufferLength);
            std::cout << "MSE: " << mse << std::endl;
            // if (mse > 100000) 
            // {
            //    return strFormat("MSE exceed: %ld > 10000", mse);
            // }
         }

         if (!m_dumpDir.empty())
         {
            {
               std::ostringstream filenameStream;
               filenameStream << m_dumpDir << "/dst_" << codec.name << "_" << m_samplingRate << "hz_" << m_channels << "ch.pcm";

               std::string filename = filenameStream.str();
               auto nbytes = bufferLength * sizeof(int16_t);
               write_to_file(filename, reinterpret_cast<const char*>(decBuf), nbytes);

               std::cout << "write to file: " << filename << ", bytes " << nbytes << std::endl;
            }


            if (!codec.encoded_file_ext.empty()) 
            {
               std::ostringstream filenameStream;
               filenameStream << m_dumpDir << "/dst_" << codec.name << "_" << m_samplingRate << "hz_" << m_channels << "ch" << codec.encoded_file_ext;

               std::string filename = filenameStream.str();

               // encoded_file = std::make_shared<std::ofstream>(filename, std::ios::binary); // new std::ofstream(filename, std::ios::binary);
               // if (!encoded_file->is_open()) {
               //    return strFormat("Unable to open file [%s]", filename.c_str());
               // }

               write_to_file(filename, reinterpret_cast<const char*>(encBuf), encPos);

               std::cout << "write to file: " << filename << ", bytes " << encPos << std::endl;
            }
         }
         return "";
      }
   private:
      int m_samplingRate;
      int m_channels;
      std::string m_dumpDir;
      std::vector<int16_t> m_pcmBuf;
      std::vector<uint8_t> m_encBuf;
      std::vector<int16_t> m_decBuf;
};




int testCodecs(int channels, int samplingRate, const std::string& pcmFile, const std::vector<CCodecDesc>& codecs)
{
   int seconds = 5;
      
   Tester tester("/tmp");

   if (pcmFile.empty())
   {
      auto ret = tester.genPcm(samplingRate, channels, seconds);
      if (ret != 0)
      {
         std::cerr 
            << "gen pcm failed: ret=[" << ret << "]" 
            <<  ", samplingRate=[" << samplingRate << "]"
            <<  ", channels=[" << channels << "]"
            << std::endl;
         return -1;
      }
   }
   else
   {
      auto ret = tester.loadPCMFile(samplingRate, channels, pcmFile);
      if (ret != 0)
      {
         std::cerr << "load pcm file failed: ret=[" << ret << "]" 
            <<  ", file=[" << pcmFile << "]"
            << std::endl;
         return -1;
      }
      std::cout << "loaded pcm file: [" << pcmFile << "]" << std::endl;
   }

   for (const auto& codec : codecs) 
   {
      std::cout 
         << "------ " 
         << codec.name << " " << samplingRate << "/" << channels 
         <<" ------" 
         << std::endl;

      auto ret = tester.testCodec(codec); // (codec.name, codec.encMaker, codec.decMaker);
      if (!ret.empty()) {
         std::cerr << "test codec [" << codec.name << "] failed: [" << ret << "]" << std::endl;
         return -1;
      }
      std::cout << "test codec [" << codec.name << "] success" << std::endl;
   }

   return 0;
}

struct CodecLegDesc
{
   GetAudioCodec getter;
   AudioCodecArgs args;
   std::string encoded_file_ext; 
};

struct CTransCase {
   std::string pcmFile;
   CodecLegDesc src; 
   CodecLegDesc dst; 
   float mos;
};

struct CCodecHub
{
   const CAudioCodec& codec;
   CAudioEncoderPtr encoder;
   CAudioDecoderPtr decoder;
};

static CCodecHub buildCodec(const CAudioCodec& codec)
{
   return CCodecHub
   {
      codec,
      codec.encMaker(),
      codec.decMaker(),
   };
}



static int resampleAll
(
   unsigned int in_rate,
   unsigned int out_rate,
   unsigned int channels,
   const int16_t* inPtr, int inLen,
   std::vector<int16_t>& outBuf
)
{
   AudioCodecArgs inArgs = {.channels=Channels(channels), .samplerate=Samplerate(in_rate), .framesize=FrameMillis(20).framesize(in_rate)};

   AudioCodecArgs outArgs = {.channels=Channels(channels), .samplerate=Samplerate(out_rate), .framesize=FrameMillis(20).framesize(out_rate)};

   int dstPcmSize = inArgs.convertSampleCount(outArgs, inLen*2);

   outBuf.resize(dstPcmSize, 0);

   // unsigned int in_rate = args.samplerate.value();
   // unsigned int out_rate = ARGS_16K.samplerate.value();
   // unsigned int channels = ARGS_16K.channels.value();

   int err = 0;

   SpeexResamplerState *resampler = speex_resampler_init(channels, in_rate, out_rate, SPEEX_RESAMPLER_QUALITY_DEFAULT, &err);
   if (err != RESAMPLER_ERR_SUCCESS) {
      return -__LINE__;
   }

   int inPos = 0;
   int outPos = 0;

   while (inPos < inLen && outPos < outBuf.size())
   {
      spx_uint32_t remains = (inLen - inPos)/channels;
      spx_uint32_t out_len = outBuf.size() - outPos;

      err = speex_resampler_process_interleaved_int(resampler, inPtr, &remains, outBuf.data() + outPos, &out_len);
      if (err != RESAMPLER_ERR_SUCCESS) {
         err = -__LINE__;
         break;
      }

      inPos += remains * channels;
      outPos += out_len * channels;

      if (out_len == 0)
      {
         break;
      }
   }

   outBuf.resize(outPos, 0);

   speex_resampler_destroy(resampler);
   return err;
}

struct PesqPcm
{
   std::vector<int16_t> allocVec;

   AudioCodecArgs args;
   const int16_t* ptr;
   int len;
};

static int buildPesqPcm(const AudioCodecArgs& srcArgs, const int16_t* pcmBuf, int pcmLen, const AudioCodecArgs dstArgs, PesqPcm& pesqPcm)
{
   if (srcArgs.channels.value() != dstArgs.channels.value())
   {
      return -__LINE__;
   }

   if (srcArgs.samplerate.value() != dstArgs.samplerate.value())
   {
      int ret = resampleAll(srcArgs.samplerate.value(), dstArgs.samplerate.value(), dstArgs.channels.value(), pcmBuf, pcmLen, pesqPcm.allocVec);
      if (ret < 0)
      {
         return ret;
      }
         
      pesqPcm.args = dstArgs;
      pesqPcm.ptr = pesqPcm.allocVec.data();
      pesqPcm.len = pesqPcm.allocVec.size();
      return 0;
   }
   else
   {
      pesqPcm.args = srcArgs;
      pesqPcm.ptr = pcmBuf;
      pesqPcm.len = pcmLen;
      return 0;
   }

}

// static int makePesqPcm(const AudioCodecArgs& args, const int16_t* pcmBuf, int pcmLen, PesqPcm& pesqPcm)
// {

//    if (args.samplerate.value() != 8000 && args.samplerate.value() != 16000)
//    {
//       const AudioCodecArgs dst_16k = 
//       {
//          .channels=args.channels, 
//          .samplerate = Samplerate(16000), 
//          .framesize = args.framesize
//       };

//       return resampleToPesqPcm(args, pcmBuf, pcmLen, dst_16k, pesqPcm);
//    }
//    else
//    {
//       pesqPcm.args = args;
//       pesqPcm.ptr = pcmBuf;
//       pesqPcm.len = pcmLen;
//       return 0;
//    }
// }

static int calcPESQ
(
   const AudioCodecArgs& refArgs,
   const int16_t* refPcmData, int refPcmLen, 

   const AudioCodecArgs& degArgs,
   const int16_t* degPcmData, int degPcmLen,

   float * p_mos
)
{
   int ret = 0;

   if (refArgs.channels.value() != degArgs.channels.value())
   {
      return -__LINE__;
   }

   PesqPcm refPcm;
   {
      AudioCodecArgs tmpArgs = refArgs;
      
      if (tmpArgs.samplerate.value() != 8000 && tmpArgs.samplerate.value() != 16000)
      {
         tmpArgs.samplerate = Samplerate(16000);
      }

      ret = buildPesqPcm(refArgs, refPcmData, refPcmLen, tmpArgs, refPcm);
      if (ret)
      {
         return -__LINE__;
      }
   }

   PesqPcm degPcm;
   {
      AudioCodecArgs tmpArgs = degArgs;
      
      if (tmpArgs.samplerate.value() != refPcm.args.samplerate.value())
      {
         tmpArgs.samplerate = refPcm.args.samplerate;
      }

      ret = buildPesqPcm(degArgs, degPcmData, degPcmLen, tmpArgs, degPcm);
      if (ret)
      {
         return -__LINE__;
      }
   }

   float pesq_mos = 0.0f;
   float mapped_mos = 0.0f;

   long error_flag = 0; 
   char * error_type = NULL;

   if (refArgs.channels.value() == 1)
   {
      pesq_measure_pcm
      (
         refPcm.ptr, refPcm.len,
         degPcm.ptr, degPcm.len,
         degPcm.args.samplerate.value(),
         NB_MODE,
         &pesq_mos, &mapped_mos,
         &error_flag, &error_type
      );
      if (error_flag)
      {
         return -__LINE__;
      }
      *p_mos = mapped_mos;
   }
   else
   {
      int channels = refPcm.args.channels.value();
      int samplerate = refPcm.args.samplerate.value();

      auto refBuf1 = std::vector<int16_t>(refPcm.len/channels, 0);
      auto degBuf1 = std::vector<int16_t>(degPcm.len/channels, 0);

      *p_mos = 5.0f;
      for (int ch = 0; ch < channels; ch++)
      {
         for (int i = 0; i < refBuf1.size(); i++)
         {
            refBuf1[i] = refPcm.ptr[i*channels + ch];
         }

         for (int i = 0; i < degBuf1.size(); i++)
         {
            degBuf1[i] = degPcm.ptr[i*channels + ch];
         }

         pesq_measure_pcm
         (
            refBuf1.data(), refBuf1.size(),
            degBuf1.data(), degBuf1.size(),
            samplerate,
            NB_MODE,
            &pesq_mos, &mapped_mos,
            &error_flag, &error_type
         );
         if (error_flag)
         {
            return -__LINE__;
         }

         // printf("check ch: mos=%.3f, Hz=%d, ch=%d, ref.len=%ld, deg.len=%ld \n", mapped_mos, samplerate, ch, refBuf1.size(), degBuf1.size());

         if (mapped_mos < *p_mos)
         {
            *p_mos = mapped_mos;
         }
      }
   }

   return ret;
}

static ErrMsg testTranscode(TestContext& ctx, const CTransCase& desc, int caseIndex )
{
   
   auto src = buildCodec(desc.src.getter());
   auto dst = buildCodec(desc.dst.getter());

   std::string caseName ;
   if (src.codec.name == get_acopy_codec().name)
   {
      caseName = strFormat("codec [%s]-[%dHz-%dch-%d]", dst.codec.name.c_str(), desc.dst.args.samplerate.value(), desc.dst.args.channels.value(), desc.dst.args.framesize.value());
   }
   else
   {
      caseName = strFormat(
         "trascode [%s]-[%dHz-%dch-%d] => [%s]-[%dHz-%dch-%d]", 
         src.codec.name.c_str(), desc.src.args.samplerate.value(), desc.src.args.channels.value(), desc.src.args.framesize.value(),
         dst.codec.name.c_str(), desc.dst.args.samplerate.value(), desc.dst.args.channels.value(), desc.dst.args.framesize.value()
      );
   }
   

   {
      auto ret = ctx.makePcmData(ctx.forceGen, desc.pcmFile, desc.src.args, ctx.maxMillis);
      ret_if_err(ret);
   }

   

   {
      auto& pair = src;
      auto args = &desc.src.args;

      {
         int ret = pair.encoder->open(args->channels.value(), args->samplerate.value());
         if (ret != 0) 
         {
            return ErrMsg::fmt("encoder init failed %d, channels %d, samplerate %d", ret, args->channels, args->samplerate);
         }
      }

      {
         int ret = pair.decoder->open(args->channels.value(), args->samplerate.value());
         if (ret != 0) 
         {
            return ErrMsg::fmt("decoder init failed %d, channels %d, samplerate %d", ret, args->channels, args->samplerate);
         }
      }
   }

   {
      auto pair = dst;
      auto args = &desc.dst.args;

      {
         int ret = pair.encoder->open(args->channels.value(), args->samplerate.value());
         if (ret != 0) 
         {
            return ErrMsg::fmt("encoder open failed %d, channels %d, samplerate %d", ret, args->channels, args->samplerate);
         }
      }

      {
         int ret = pair.decoder->open(args->channels.value(), args->samplerate.value());
         if (ret != 0) 
         {
            return ErrMsg::fmt("decoder open failed %d, channels %d, samplerate %d", ret, args->channels, args->samplerate);
         }
      }
   }
   
   auto transcoder = new_audio_transcoder();
   {
      auto ret = transcoder->open(src.decoder, desc.src.args, dst.encoder, desc.dst.args);
      if (ret != 0) 
      {
         return ErrMsg::fmt("transcoder open failed %d", ret);
      }
   }
   

   const std::vector<int16_t>& srcPcm = ctx.pcmBuf.m_pcmBuf;
   

   int frameSampAll = desc.src.args.channels.value() * desc.src.args.framesize.value();
   
   int srcPcmTotal = srcPcm.size() - (srcPcm.size() % frameSampAll);
   int dstPcmSize = desc.src.args.convertSampleCount(desc.src.args, srcPcmTotal*2);

   auto srcEncVec = std::vector<uint8_t>(dstPcmSize, 0);
   int srcEncPos = 0;

   auto dstPcm = std::vector<int16_t>(dstPcmSize, 0);

   auto dstEncVec = std::vector<uint8_t>(srcPcmTotal*2, 0);
   int dstEncPos = 0;
   int dstConsumed = 0;

   int srcPcmPos = 0;
   int dstPcmPos = 0;
   
   
   while ((srcPcmPos + frameSampAll) <= srcPcmTotal)
   {
      int encLen = 0;
      // printf("------\n");
      // printf("before trans encode: srcEncPos %d, encLen %d\n", srcEncPos, encLen);

      {
         int ret = src.encoder->encode(srcPcm.data()+srcPcmPos, frameSampAll, srcEncVec.data() + srcEncPos, srcEncVec.size()-srcEncPos);
         if (ret <= 0) 
         {
            return ErrMsg::fmt("src encode failed %d", ret);
         }
         encLen = srcEncPos + ret;
         srcPcmPos += frameSampAll;
      }
      
      // printf("after trans encode: srcEncPos %d, encLen %d\n", srcEncPos, encLen);

      srcEncPos = 0;
      while (srcEncPos < encLen)
      {
         int remains = encLen-srcEncPos;
         // printf("before trans push: srcEncPos %d, encLen %d\n", srcEncPos, encLen);
         int ret = transcoder->push(srcEncVec.data()+srcEncPos, remains);
         // printf("trans push ret %d\n", ret);
         if (ret < 0) 
         {
            return ErrMsg::fmt("trans push failed %d", ret);
         } 
         else if (ret == 0 )
         {
            break;
         }
         else 
         {
            srcEncPos += remains;
            // srcEncPos += ret;
            
            // printf("after trans push: srcEncPos %d, encLen %d\n", srcEncPos, encLen);
         }
      }

      if (srcEncPos < encLen)
      {
         std::memmove(srcEncVec.data(), srcEncVec.data()+srcEncPos, (encLen - srcEncPos)*sizeof(srcEncVec[0]));
         srcEncPos = encLen - srcEncPos;
      }
      else
      {
         srcEncPos = 0;
      }
      // printf("final trans push: srcEncPos %d, encLen %d\n", srcEncPos, encLen);

      
      while(1)
      {
         int dstEncLen = 0;
         auto dstEncPtr = dstEncVec.data() + dstEncPos;
         auto dstEncSize = dstEncVec.size() - dstEncPos;
         {
            int ret = transcoder->pull(dstEncPtr, dstEncSize);
            // printf("trans pull ret %d\n", ret);
            if (ret < 0) 
            {
               return ErrMsg::fmt("trans pull failed %d", ret);
            } 
            else if (ret == 0)
            {
               break;
            }

            dstEncLen = ret;
         }
         dstEncPos += dstEncLen;
         // printf("after trans pull: dstConsumed %d, dstEncPos %d\n", dstConsumed, dstEncPos);
         
         while(dstConsumed < dstEncPos)
         {
            auto dstConsumedPtr = dstEncVec.data() + dstConsumed;
            int inlen = dstEncPos - dstConsumed;
            // printf("before trans decode: dstConsumed %d, dstEncPos %d, inlen %d\n", dstConsumed, dstEncPos, inlen);
            int ret = dst.decoder->decode(dstConsumedPtr, &inlen, dstPcm.data() + dstPcmPos, dstPcm.size() - dstPcmPos);
            if (ret < 0) 
            {
               return ErrMsg::fmt("dst decode failed %d", ret);
            }
            dstPcmPos += ret;
            dstConsumed += inlen;
            // printf("after trans decode: dstConsumed %d, dstEncPos %d, ret %d, inlen %d\n", dstConsumed, dstEncPos, ret, inlen);
            if (ret == 0 && inlen == 0) 
            {
               break;
            }
         }
      }

   }

   {
      // int dstEncLen = 0;
      auto dstEncPtr = dstEncVec.data() + dstEncPos;
      auto dstEncSize = dstEncVec.size() - dstEncPos;
      int ret = transcoder->flush(dstEncPtr, dstEncSize);
      if (ret < 0) 
      {
         return ErrMsg::fmt("trans flush failed %d", ret);
      } 
      dstEncPos += ret;

      do
      {
         auto consumedPtr = dstEncVec.data() + dstConsumed;
         int inlen = dstEncPos - dstConsumed;
         int ret = dst.decoder->flush(consumedPtr, &inlen, dstPcm.data() + dstPcmPos, dstPcm.size() - dstPcmPos);
         if (ret < 0) 
         {
            return ErrMsg::fmt("dst decode flush failed %d", ret);
         }
         else if (ret == 0)
         {
            break;
         }

         dstPcmPos += ret;
         dstConsumed += inlen;
      } while(ret > 0);
   }

   {
      float mos = 0.0f;
      int ret = calcPESQ
      (
         desc.src.args, 
         srcPcm.data(), srcPcmTotal, 
         desc.dst.args, 
         dstPcm.data(), dstPcmPos,
         &mos
      );

      if (ret)
      {
         return ErrMsg::fmt("calc pesq error [%d]", ret);
      }

      float expect_mos = desc.mos;
      // float expect_mos = 2.0f; 
      if (mos < expect_mos)
      {
         return ErrMsg::fmt("[%s]: low pesq mos [%.3f] < [%.3f]", caseName.c_str(), mos, expect_mos);
      }

      std::cout << "Case " << caseName << ": pesq score " 
         << mos 
         << " >= "  
         // << std::setw(3) << std::setfill('0') 
         << expect_mos 
         << ", PASS "<< std::endl;

   }


   if (!ctx.dumpDir.empty())
   {
      auto& codec = dst.codec;
      auto& leg = desc.dst;
      auto& args = desc.dst.args;

      {
         std::ostringstream filenameStream;
         filenameStream 
            << ctx.dumpDir 
            << "/dst" 
            << "_" << std::setw(3) << std::setfill('0') << caseIndex
            << "_" << codec.name 
            << "_" << args.samplerate.value() << "hz" << args.channels.value() << "ch"
            << ".pcm";

         std::string filename = filenameStream.str();
         auto nbytes = dstPcmPos * sizeof(int16_t);
         write_to_file(filename, reinterpret_cast<const char*>(dstPcm.data()), nbytes);

         std::cout << "write to file: " << filename << ", bytes " << nbytes << std::endl;
      }


      if (!leg.encoded_file_ext.empty()) 
      {
         std::ostringstream filenameStream;
         filenameStream 
            << ctx.dumpDir 
            << "/dst" 
            << "_" << std::setw(3) << std::setfill('0') << caseIndex
            << "_" << codec.name 
            << "_" << args.samplerate.value() << "hz" << args.channels.value() << "ch"
            << leg.encoded_file_ext;

         std::string filename = filenameStream.str();

         write_to_file(filename, reinterpret_cast<const char*>(dstEncVec.data()), dstEncPos);

         std::cout << "write to file: " << filename << ", bytes " << dstEncPos << std::endl;
      }
   }
   return ErrMsg();
}

inline static void printDivLine()
{
   std::cout << "------ " << std::endl;
}

static ErrMsg testTranscodes(TestContext& ctx, int& caseIndex, const std::vector<CTransCase>& cases)
{
   for (const auto& caze : cases) 
   {
      printDivLine();
      caseIndex += 1;
      auto ret = testTranscode(ctx, caze, caseIndex);
      if (!ret.isOk())
      {
         std::cerr 
            << "testTranscode failed: ret=[" << ret.msg() << "]" 
            << ", index=" << caseIndex
            << ", src codec [" << caze.src.getter().name << "]"
            << ", dst codec [" << caze.dst.getter().name << "]"
            << std::endl;
         return ret;
      }
   }
   return ErrMsg();
}



struct AudioFile
{
   std::string filepath;
   Samplerate samplerate; 
   Channels channels;
};

// #define AARG_MILLIS(ch, hz, millis) {.channels=ch, .samplerate = hz, .framesize = FRAME_SIZE(hz, millis)}
#define AARG_SAMPLES(ch, hz, samples) {.channels=ch, .samplerate = hz, .framesize = samples}

#define TRANS_CASE_FULL(file, srcmaker, srcframe, dstmaker, dstframe, dsthz, dstext, moscode) \
{ \
   .pcmFile=(file).filepath, \
   .src={srcmaker, AARG_SAMPLES((file).channels, (file).samplerate, srcframe.framesize((file).samplerate)), ""}, \
   .dst={dstmaker, AARG_SAMPLES((file).channels, dsthz, dstframe.framesize(dsthz)), dstext}, \
   .mos=moscode \
}

#define TRANS_CASE(file, srcmaker, srcframe, dstmaker, dstframe, dsthz, moscode) \
TRANS_CASE_FULL(file, srcmaker, srcframe, dstmaker, dstframe, dsthz, "", moscode)

#define TRANS_CASE2(file, srcmaker, srcframe, dstmaker, dstframe, dsthz, dstext, moscode) \
TRANS_CASE_FULL(file, srcmaker, srcframe, dstmaker, dstframe, dsthz, dstext, moscode)

#define CODEC_CASE(file, dstmaker, dstframe, dsthz, dstext, moscode) \
TRANS_CASE_FULL(file, get_acopy_codec, dstframe, dstmaker, dstframe, dsthz, dstext, moscode)


int testResampler(const std::string& srcFile, const AudioCodecArgs& srcArgs, const std::string& dstFile, const AudioCodecArgs& dstArgs)
{
   int ret = 0;
   
   std::vector<int16_t> srcBuf;
   ret = readPCMData(srcFile, srcBuf);
   if (ret) {
      return -__LINE__;
   }

   std::vector<int16_t> dstBuf;
   ret = resampleAll(srcArgs.samplerate.value(), dstArgs.samplerate.value(), dstArgs.channels.value(), srcBuf.data(), srcBuf.size(), dstBuf);
   if (ret) {
      return -__LINE__;
   }

   FILE *output_file = fopen(dstFile.c_str(), "wb");
   if (output_file == NULL) {
      return -__LINE__;
   }

   fwrite(dstBuf.data(), sizeof(spx_int16_t), dstBuf.size(), output_file);
   fclose(output_file);

   return 0;
}

int testSpeexResampler(const std::string& srcFile, const AudioCodecArgs& srcArgs, const std::string& dstFile, const AudioCodecArgs& dstArgs)
{
   unsigned int in_rate = srcArgs.samplerate.value();
   unsigned int out_rate = dstArgs.samplerate.value();
   unsigned int channels = dstArgs.channels.value();
   int err;

   SpeexResamplerState *resampler = speex_resampler_init(channels, in_rate, out_rate, SPEEX_RESAMPLER_QUALITY_DEFAULT, &err);
   if (err != RESAMPLER_ERR_SUCCESS) {
      return -__LINE__;
   }

   // Open the input and output files
   FILE *input_file = fopen(srcFile.c_str(), "rb");
   if (input_file == NULL) {
      return -__LINE__;
   }

   FILE *output_file = fopen(dstFile.c_str(), "wb");
   if (output_file == NULL) {
      return -__LINE__;
   }

   auto srcframesampls = srcArgs.framesize.value() * srcArgs.channels.value();
   auto dstframesampls = dstArgs.framesize.value() * dstArgs.channels.value();

   auto in_buf = std::vector<spx_int16_t>(srcframesampls, 0);
   auto out_buf = std::vector<spx_int16_t>(dstframesampls, 0);

   spx_uint32_t in_len;
   spx_uint32_t out_len;

   while (!feof(input_file)) {
      in_len = fread(in_buf.data(), sizeof(spx_int16_t), srcframesampls, input_file);
      in_len = in_len/srcArgs.channels.value();
      out_len = out_buf.size()/dstArgs.channels.value();

      err = speex_resampler_process_interleaved_int(resampler, in_buf.data(), &in_len, out_buf.data(), &out_len);
      if (err != RESAMPLER_ERR_SUCCESS) {
         return -__LINE__;
      }

      std::cout 
         << "resample" 
         << ", in_len " << in_len
         << ", out_len " << out_len
         << std::endl;

      fwrite(out_buf.data(), sizeof(spx_int16_t), out_len * dstArgs.channels.value(), output_file);
   }

   // Close the files
   fclose(input_file);
   fclose(output_file);

   // Now, output contains the resampled audio

   speex_resampler_destroy(resampler);


   return 0;
}



// #include <stdio.h>

// #define MINIMP3_IMPLEMENTATION
// // #define MINIMP3_ONLY_MP3
// #include <minimp3.h>
// // #include <minimp3_ex.h>

// int test_mp3_decode(const std::string& inputPath, const std::string& outputPath) 
// {
//    FILE *mp3File = fopen(inputPath.c_str(), "rb");
//    FILE *pcmFile = fopen(outputPath.c_str(), "wb");

//    mp3dec_t dec;
//    mp3dec_init(&dec);

//    const int MP3_BUF_SIZE = 16*1024; // 960000;
//    uint8_t * mp3buf = (uint8_t *) malloc(MP3_BUF_SIZE);
//    size_t buf_size = 0;
//    short pcm[2*4096];
//    mp3dec_frame_info_t frame_info;

//    int readBytes = 0;
//    int wroteBytes = 0;

//    while (!feof(mp3File)) 
//    {
//       int offset = buf_size;
//       int space = MP3_BUF_SIZE-buf_size;
//       if (space > 0) 
//       {
//          int rb = fread(mp3buf + offset, 1, MP3_BUF_SIZE-buf_size, mp3File);
//          buf_size += rb;
//          readBytes += rb;
//          printf("readBytes offset %d, delta %d, total %d\n", offset, rb, readBytes);
//       }

//       int samples = 0;
//       do 
//       {
//          samples = mp3dec_decode_frame(&dec, mp3buf, buf_size, pcm, &frame_info);
//          printf("mp3 decode: samples %d, frame_bytes %d, frame_offset %d, buf_size %ld\n", samples, frame_info.frame_bytes, frame_info.frame_offset, buf_size);
//          if (samples) 
//          {
//             fwrite(pcm, 2, samples * frame_info.channels, pcmFile);
//             wroteBytes += samples * frame_info.channels * 2;

//             memmove(mp3buf, mp3buf + frame_info.frame_bytes, buf_size - frame_info.frame_bytes);
//             buf_size -= frame_info.frame_bytes;
//          }
//       } while (0);
//    }
   
//    {
//       int samples = 0;
//       do 
//       {
//          samples = mp3dec_decode_frame(&dec, mp3buf, buf_size, pcm, &frame_info);
//          printf("mp3 decode: samples %d, frame_bytes %d, frame_offset %d, buf_size %ld\n", samples, frame_info.frame_bytes, frame_info.frame_offset, buf_size);
//          if (samples) 
//          {
//             fwrite(pcm, 2, samples * frame_info.channels, pcmFile);
//             wroteBytes += samples * frame_info.channels * 2;

//             memmove(mp3buf, mp3buf + frame_info.frame_bytes, buf_size - frame_info.frame_bytes);
//             buf_size -= frame_info.frame_bytes;
//          }
//       } while (samples);
//    }

//    fclose(pcmFile);
//    fclose(mp3File);

//    printf("final readBytes %d\n", readBytes);
//    printf("final wroteBytes %d\n", wroteBytes);

//    return 0;
// }




// #include <stdio.h>
// #include <stdlib.h>
// #include "minimp3.h"

// #define MP3_BUFFER_SIZE 4096
// #define PCM_BUFFER_SIZE (MP3_BUFFER_SIZE * MINIMP3_MAX_SAMPLES_PER_FRAME)

// int testMp3V2(int argc, char *argv[]) {
//     if (argc < 3) {
//         printf("Usage: %s <input.mp3> <output.pcm>\n", argv[0]);
//         return 1;
//     }

//     FILE *mp3_file = fopen(argv[1], "rb");
//     if (!mp3_file) {
//         perror("Error opening MP3 file");
//         return 1;
//     }

//     FILE *pcm_file = fopen(argv[2], "wb");
//     if (!pcm_file) {
//         perror("Error opening PCM file");
//         fclose(mp3_file);
//         return 1;
//     }

//     unsigned char mp3_buffer[MP3_BUFFER_SIZE];
//     short pcm_buffer[PCM_BUFFER_SIZE];
//     mp3dec_t mp3;
//     mp3dec_frame_info_t info;
//     mp3dec_init(&mp3);

//     int bytes_read;
//     while ((bytes_read = fread(mp3_buffer, 1, MP3_BUFFER_SIZE, mp3_file)) > 0) {
//         int samples = mp3dec_decode_frame(&mp3, mp3_buffer, bytes_read, pcm_buffer, &info);
//         if (samples > 0) {
//             fwrite(pcm_buffer, samples * sizeof(short), info.channels, pcm_file);
//         }
//     }

//     fclose(mp3_file);
//     fclose(pcm_file);
//     printf("Decoding complete.\n");

//     return 0;
// }


#include <string>
// #include <filesystem>

// static std::string get_directory(const std::string& path) {
//     return std::filesystem::path(path).parent_path().string();
// }

static std::string get_directory(const std::string& path) {
    size_t pos = path.find_last_of("\\/");
    return (std::string::npos == pos) ? "" : path.substr(0, pos);
}

// static std::string get_thiz_directory() {
//     return get_directory(__FILE__);
// }



int main(int argc, char *argv[]) 
{
   // {
   //    int ret = test_mp3_decode("/tmp/dst_002_mp3_48000hz2ch.mp3", "/tmp/dst_002_mp3_48000hz2ch.pcm");
   //    return ret;
   // }

   // {
   //    int ret = testSpeexResampler
   //    (
   //       "/tmp/sample-48000Hz-2ch.pcm",
   //       {.channels=2, .samplerate = 48000, .framesize = 1024},
   //       "/tmp/dst_resample_speex_16000Hz_2ch.pcm",
   //       {.channels=2, .samplerate = 16000, .framesize = FRAME_SIZE(16000, 20)}
   //    );
   //    std::cerr 
   //          << "testSpeexResampler ret=[" << ret << "]" 
   //          << std::endl;
   // }

   // {
   //    int ret = testResampler
   //    (
   //       "/tmp/sample-48000Hz-2ch.pcm",
   //       {.channels=2, .samplerate = 48000, .framesize = 1024},
   //       "/tmp/dst_resample_16000Hz_2ch.pcm",
   //       {.channels=2, .samplerate = 16000, .framesize = FRAME_SIZE(16000, 20)}
   //    );
   //    std::cerr 
   //          << "testResampler ret=[" << ret << "]" 
   //          << std::endl;
   // }

   {
      // auto thiz_dir = get_thiz_directory();
      const auto thiz_dir = get_directory(argv[0]);
      std::cout << "thiz_dir " << thiz_dir << std::endl;

      const auto pcm_dir = thiz_dir + "/../../sample_pcm/";
      std::cout << "pcm_dir " << pcm_dir << std::endl;
      
      auto dumpDir = ""; // "/tmp"; // empty string for disable dump output file
      auto forceGen = false;

      TestContext ctx = {.forceGen=forceGen, .dumpDir=dumpDir, .maxMillis = 5000};

      AudioFile file48000Hz1ch = 
      {
         .filepath = pcm_dir + "sample-48000Hz-1ch.pcm", 
         .samplerate = Samplerate(48000),
         .channels = Channels(1),  
      };

      AudioFile file48000Hz2ch = 
      {
         .filepath = pcm_dir + "sample-48000Hz-2ch.pcm", 
         .samplerate = Samplerate(48000),
         .channels = Channels(2), 
      };

      AudioFile file16000Hz1ch = 
      {
         .filepath = pcm_dir + "sample-16000Hz-1ch.pcm", 
         .samplerate = Samplerate(16000),
         .channels = Channels(1),  
      };

      AudioFile file16000Hz2ch = 
      {
         .filepath = pcm_dir + "sample-16000Hz-2ch.pcm", 
         .samplerate = Samplerate(16000),
         .channels = Channels(2),  
      };

      AudioFile file8000Hz1ch = 
      {
         .filepath = pcm_dir + "sample-8000Hz-1ch.pcm", 
         .samplerate = Samplerate(8000),
         .channels = Channels(1),  
      };

      AudioFile file8000Hz2ch = 
      {
         .filepath = pcm_dir + "sample-8000Hz-2ch.pcm", 
         .samplerate = Samplerate(8000),
         .channels = Channels(2),  
      };

      int ptime = 20;

      const std::vector<CTransCase> cases = {
         // opus 1 channel
         CODEC_CASE
         (
            file48000Hz1ch,
            get_opus_codec, FrameMillis(ptime), Samplerate(48000), "", 3.8f
         ),
         CODEC_CASE
         (
            file48000Hz1ch,
            get_opus_codec, FrameMillis(ptime), Samplerate(24000), "", 3.8f
         ),
         CODEC_CASE
         (
            file16000Hz1ch,
            get_opus_codec, FrameMillis(ptime), Samplerate(16000), "", 3.8f
         ),
         CODEC_CASE
         (
            file16000Hz1ch,
            get_opus_codec, FrameMillis(10), Samplerate(12000), "", 3.7f
         ),
         CODEC_CASE
         (
            file8000Hz1ch,
            get_opus_codec, FrameMillis(ptime), Samplerate(8000), "", 3.3f
         ),

         // opus 2 channels
         CODEC_CASE
         (
            file48000Hz2ch,
            get_opus_codec, FrameMillis(ptime), Samplerate(48000), "", 3.8f
         ),
         CODEC_CASE
         (
            file48000Hz2ch,
            get_opus_codec, FrameMillis(ptime), Samplerate(24000), "", 3.8f
         ),
         CODEC_CASE
         (
            file16000Hz2ch,
            get_opus_codec, FrameMillis(ptime), Samplerate(16000), "", 3.8f
         ),
         CODEC_CASE
         (
            file16000Hz2ch,
            get_opus_codec, FrameMillis(ptime), Samplerate(12000), "", 3.3f
         ),
         CODEC_CASE
         (
            file8000Hz2ch,
            get_opus_codec, FrameMillis(ptime), Samplerate(8000), "", 3.0f
         ),



         // alaw
         CODEC_CASE
         (
            file48000Hz1ch,
            get_alaw_codec, FrameMillis(ptime), Samplerate(48000), "", 4.2f
         ),
         CODEC_CASE
         (
            file48000Hz2ch,
            get_alaw_codec, FrameMillis(ptime), Samplerate(48000), "", 4.2f
         ),

         // ulaw
         CODEC_CASE
         (
            file48000Hz1ch,
            get_ulaw_codec, FrameMillis(ptime), Samplerate(48000), "", 4.2f
         ),
         CODEC_CASE
         (
            file48000Hz2ch,
            get_ulaw_codec, FrameMillis(ptime), Samplerate(48000), "", 4.2f
         ),

         // amrwb
         CODEC_CASE
         (
            file16000Hz1ch,
            get_amrwb_codec, FrameMillis(ptime), Samplerate(16000), "", 4.0f 
         ),

         // amrnb
         CODEC_CASE
         (
            file8000Hz1ch,
            get_amrnb_codec, FrameMillis(ptime), Samplerate(8000), "", 3.7f
         ),

         // aac
         CODEC_CASE
         (
            file48000Hz1ch,
            get_aac_codec, FrameSamples(1024), Samplerate(48000), "", 3.2f
         ),
         CODEC_CASE
         (
            file48000Hz1ch,
            get_aac_codec, FrameSamples(1024), Samplerate(44100), "", 3.2f 
         ),
         CODEC_CASE
         (
            file48000Hz2ch,
            get_aac_codec, FrameSamples(1024), Samplerate(48000), "", 3.2f
         ),
         CODEC_CASE
         (
            file48000Hz2ch,
            get_aac_codec, FrameSamples(1024), Samplerate(44100), "", 2.8f
         ),

         // g729
         CODEC_CASE
         (
            file8000Hz1ch,
            get_g729_codec, FrameMillis(10), Samplerate(8000), ".g729", 2.8f 
         ),
         CODEC_CASE
         (
            file8000Hz1ch,
            get_g729_codec, FrameMillis(20), Samplerate(8000), ".g729", 2.8f 
         ),

         // mp3
         CODEC_CASE
         (
            file48000Hz1ch,
            get_mp3_codec, FrameMillis(20), Samplerate(48000), ".mp3", 4.2f
         ),
         CODEC_CASE
         (
            file48000Hz2ch,
            get_mp3_codec, FrameMillis(20), Samplerate(48000), ".mp3", 4.2f
         ),
         CODEC_CASE
         (
            file16000Hz1ch,
            get_mp3_codec, FrameMillis(20), Samplerate(16000), ".mp3", 4.2f
         ),



         TRANS_CASE
         (
            file48000Hz1ch, 
            get_aac_codec, FrameSamples(1024),
            get_opus_codec, FrameMillis(20), Samplerate(48000), 2.6f 
         ),
         TRANS_CASE
         (
            file48000Hz1ch, 
            get_aac_codec, FrameSamples(1024),
            get_opus_codec, FrameMillis(20), Samplerate(16000), 2.6f
         ),
         TRANS_CASE
         (
            file48000Hz2ch, 
            get_aac_codec, FrameSamples(1024),
            get_opus_codec, FrameMillis(20), Samplerate(16000), 2.5f
         ),
         TRANS_CASE2
         (
            file48000Hz2ch, 
            get_opus_codec, FrameMillis(20),
            get_aac_codec, FrameSamples(1024), Samplerate(44100),
            ".aac", 2.5f // 3.0f
         ),
      };

      int index = 0;
      auto ret = testTranscodes(ctx, index, cases);
      if (!ret.isOk())
      {
         return -1;
      }
      printDivLine();

      // for (const auto& caze : cases) 
      // {
      //    index += 1;
      //    std::cout << "------ " << std::endl;
      //    auto ret = testTranscode(ctx, caze, index);
      //    if (!ret.isOk())
      //    {
      //       std::cerr 
      //          << "testTranscode failed: ret=[" << ret.msg() << "]" 
      //          << std::endl;
      //       return -1;
      //    }
      // }
      // std::cout << "------ " << std::endl;
   }

   // {
   //    int samplingRate = 48000;
   //    const std::vector<CCodecDesc> codecs = {
   //       {"aac", new_aac_encoder, new_aac_decoder, 1024, ".aac"},
   //    };

   //    int ret = testCodecs(1, samplingRate, "/tmp/sample-48000Hz-1ch.pcm", codecs);
   //    if (ret != 0)
   //    {
   //       return ret;
   //    }
   // }

   // {
   //    int samplingRate = 48000;
   //    const std::vector<CCodecDesc> codecs = {
   //       {"pcmu", new_ulaw_encoder, new_ulaw_decoder, FRAME_SIZE(samplingRate,20)},
   //       {"pcma", new_alaw_encoder, new_alaw_decoder, FRAME_SIZE(samplingRate,20)},
   //       {"opus", new_opus_encoder, new_opus_decoder, FRAME_SIZE(samplingRate,20)}
   //    };

   //    int ret = testCodecs(1, samplingRate, "/tmp/sample-48000Hz-1ch.pcm", codecs);
   //    if (ret != 0)
   //    {
   //       return ret;
   //    }
   // }

   // {
   //    int samplingRate = 16000;
   //    const std::vector<CCodecDesc> codecs = {
   //       {"pcmu", new_ulaw_encoder, new_ulaw_decoder, FRAME_SIZE(samplingRate,20)},
   //       {"pcma", new_alaw_encoder, new_alaw_decoder, FRAME_SIZE(samplingRate,20)},
   //       {"opus", new_opus_encoder, new_opus_decoder, FRAME_SIZE(samplingRate,20)},
   //       {"amrwb", new_amrwb_encoder, new_amrwb_decoder, FRAME_SIZE(samplingRate,20)}
   //    };

   //    int ret = testCodecs(1, samplingRate, "/tmp/sample-16000Hz-1ch.pcm", codecs);
   //    if (ret != 0)
   //    {
   //       return ret;
   //    }
   // }

   // {
   //    int samplingRate = 8000;
   //    const std::vector<CCodecDesc> codecs = {
   //       {"pcmu", new_ulaw_encoder, new_ulaw_decoder, FRAME_SIZE(samplingRate,20)},
   //       {"pcma", new_alaw_encoder, new_alaw_decoder, FRAME_SIZE(samplingRate,20)},
   //       {"opus", new_opus_encoder, new_opus_decoder, FRAME_SIZE(samplingRate,20)},
   //       {"amrnb", new_amrnb_encoder, new_amrnb_decoder, FRAME_SIZE(samplingRate,20)}
   //    };

   //    int ret = testCodecs(1, samplingRate, "/tmp/sample-8000Hz-1ch.pcm", codecs);
   //    if (ret != 0)
   //    {
   //       return ret;
   //    }
   // }

   std::cout << "test done" << std::endl;

   return 0;
}
