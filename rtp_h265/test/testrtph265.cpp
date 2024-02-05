
#include "annexb.h"
#include "rtp-payload.h"
#include <stdio.h>
#include <stdlib.h>
#include <vector>



struct Tester
{
   uint8_t buf[1700*1024];
};

static 
void* rtp_alloc(void* param, int bytes)
{
   printf("rtp alloc: bytes %d\n", bytes);
   Tester * tester = (Tester *) param;
   return tester->buf;
}

static 
void rtp_free(void* /*param*/, void * /*packet*/)
{
   printf("rtp free: \n");
}

static 
int rtp_encoded_packet(void* param, const void *packet, int bytes, uint32_t timestamp, int flags)
{
   printf("rtp encoded: bytes %d, ts %u, flags 0x%08X\n", bytes, timestamp, flags);
   int r = 0;
   return r >= 0 ? 0 : r;
}



int main(int argc, char *argv[]) 
{
   printf("helloworld!\n");
   const char * filepath = "/tmp/h265-rtp/sample.h265";
   int payload = 96;
   const char * encoding = "H265";
   uint16_t seq = 0;
   uint32_t ssrc = 9999;

   Tester * tester = (Tester *) calloc(1, sizeof(Tester));

   struct rtp_payload_t handler2;
   handler2.alloc = rtp_alloc;
   handler2.free = rtp_free;
   handler2.packet = rtp_encoded_packet;
   auto packer = rtp_payload_encode_create(payload, encoding, seq, ssrc, &handler2, tester);

   AnnexBBuf annexbuf;
   FILE* file = NULL;
   do
   {
      file = fopen(filepath, "rb");
      if (!file) {
         printf("Error opening file [%s]\n", filepath);
         break;
      }
      printf("opened file [%s]\n", filepath);


      int total_read_nbytes = 0;
      auto unit = AnnexBUnit();

      while(feof(file) == 0 && total_read_nbytes < 10240) 
      {
         printf("====================\n");
         annexbuf.trim();

         if (annexbuf.wSize() == 0)
         {
            annexbuf.reserve(4*1024);
         }

         {
            auto wsize = annexbuf.wSize();
            int nbytes = fread(annexbuf.wBuf(), 1, wsize, file);
            total_read_nbytes += nbytes;
            annexbuf.wAdvance(nbytes);
            printf("read: wsize %d, bytes %d, total_read_nbytes %d\n", wsize, nbytes, total_read_nbytes);
         }

         
         while (annexbuf.next(unit))
         {
            auto nalulen = unit.unitLen();
            auto naluptr = unit.unitPtr();
            printf(
               "nalu bytes %d, [0x%02X]\n", 
               nalulen, 
               naluptr[0]
            );

            rtp_payload_encode_input(packer, unit.annexbPtr(), unit.annexbLen(), 0);
         }
         printf("buf data len %d, unparsed %d\n", annexbuf.dataLen(), annexbuf.unparsed());
      }
   } while (0);
   

   if (file)
   {
      fclose(file);
      file = NULL;
   }

   free(tester);
   return 0;
}

