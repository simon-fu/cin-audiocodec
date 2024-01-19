
#include "g711codec.h"


extern "C" {
#include "spandsp.h"
};


class CG711Encoder : public CAudioEncoder
{
   public:
      CG711Encoder(int codecId) ;
      virtual ~CG711Encoder() ;

      virtual int open(int channels, int samplingRate) override;
      virtual int encode(const int16_t samples[], int inSize, uint8_t*out, int outSize) override;
      virtual void close( ) override;
   private:
      g711_state_t *m_state;
      int m_codecId;
};

CG711Encoder::CG711Encoder(int codecId):
      m_state(NULL),
      m_codecId(codecId)
{

}

CG711Encoder::~CG711Encoder() 
{
   this->close();
}

int CG711Encoder::open(int channels, int samplingRate)
{
   // this->close();
   if (m_state)
   {
      return -__LINE__;
   }
   m_state = g711_init(NULL, m_codecId);
   return 0; 
}

void CG711Encoder::close()
{
   if (m_state) {
      g711_release(m_state);
      g711_free(m_state);
      m_state = NULL;
   }
}

int CG711Encoder::encode(const int16_t samples[], int inSize, uint8_t*out, int outSize) 
{
   if (!m_state)
   {
      return -__LINE__;
   }

   int ret = g711_encode(m_state, out, samples, inSize);
   return ret; 
}


class CG711Decoder : public CAudioDecoder
{
   public:
      CG711Decoder(int codecId) ;
      virtual ~CG711Decoder() ;

      virtual int open(int channels, int samplingRate) override;
      virtual int decode(const uint8_t in[], int *pinSize, int16_t*outSamples, int outSize) override;
      virtual void close( ) override;
   private:
      g711_state_t *m_state;
      int m_codecId;
};

CG711Decoder::CG711Decoder(int codecId):
      m_state(NULL),
      m_codecId(codecId)
{

}

CG711Decoder::~CG711Decoder() 
{
   this->close();
}

int CG711Decoder::open(int channels, int samplingRate)
{
   // this->close();
   if (m_state)
   {
      return -__LINE__;
   }
   m_state = g711_init(NULL, m_codecId);
   return 0; 
}

void CG711Decoder::close()
{
   if (m_state) 
   {
      g711_release(m_state);
      g711_free(m_state);
      m_state = NULL;
   }
}

int CG711Decoder::decode(const uint8_t in[], int *pinSize, int16_t*outSamples, int outSize)
{
   if (!m_state)
   {
      return -__LINE__;
   }
   
   int ret = g711_decode(m_state, outSamples, in, *pinSize);
   return ret; 
}


CAudioEncoderPtr new_ulaw_encoder() 
{
   return std::make_shared<CG711Encoder>((int) G711_ULAW);
}

CAudioDecoderPtr new_ulaw_decoder() 
{
   return std::make_shared<CG711Decoder>((int) G711_ULAW);
}

CAudioEncoderPtr new_alaw_encoder() 
{
   return std::make_shared<CG711Encoder>((int) G711_ALAW);
}

CAudioDecoderPtr new_alaw_decoder() 
{
   return std::make_shared<CG711Decoder>((int) G711_ALAW);
}


const CAudioCodec& get_ulaw_codec()
{
   static CAudioCodec CODEC = {"ulaw", new_ulaw_encoder, new_ulaw_decoder};
   return CODEC;
}

const CAudioCodec& get_alaw_codec()
{
   static CAudioCodec CODEC = {"alaw", new_alaw_encoder, new_alaw_decoder};
   return CODEC;
}
