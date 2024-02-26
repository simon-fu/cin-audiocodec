
#include "annexb.h"
#include <stdio.h>
#include <stdlib.h>
#include <vector>
#include <algorithm>

// #define ENABLE_DEBUG 1

#ifdef ENABLE_DEBUG
#define dbgd(...) printf(__VA_ARGS__)
#else 
#define dbgd(...)
#endif

#define dbgi(...) printf(__VA_ARGS__)

int annexb_to_lengthed(const char * input, const char * output, int max_bytes)
{
   AnnexBBuf annexbuf;
   FILE* ifile = NULL;
   FILE* ofile = NULL;
   int num_units = 0;
   int total_read_nbytes = 0;

   do
   {
      ifile = fopen(input, "rb");
      if (!ifile) {
         dbgi("Error opening file [%s]\n", input);
         break;
      }
      dbgi("opened input [%s]\n", input);

      ofile = fopen(output, "wb");
      if (!ofile) {
         dbgi("Error opening file [%s]\n", output);
         break;
      }
      dbgi("opened output [%s]\n", output);


      auto unit = AnnexBUnit();

      while(feof(ifile) == 0 && total_read_nbytes < max_bytes) 
      {
         dbgd("====================\n");
         annexbuf.trim();

         if (annexbuf.wSize() == 0)
         {
            annexbuf.reserve(4*1024);
         }

         {
            auto wsize = annexbuf.wSize();
            int nbytes = fread(annexbuf.wBuf(), 1, wsize, ifile);
            total_read_nbytes += nbytes;
            annexbuf.wAdvance(nbytes);
            dbgd("read: wsize %d, bytes %d, total_read_nbytes %d\n", wsize, nbytes, total_read_nbytes);
         }

         
         while (annexbuf.next(unit))
         {
            auto nalulen = unit.unitLen();
            auto naluptr = unit.unitPtr();
            dbgd(
               "nalu bytes %d, [0x%02X]\n", 
               nalulen, 
               naluptr[0]
            );
            
            uint32_t nal_size_big_endian = htonl(nalulen);
            fwrite(&nal_size_big_endian, 4, 1, ofile);
            fwrite(naluptr, 1, nalulen, ofile);

            ++num_units;
         }
         dbgd("buf data len %d, unparsed %d, read_bytes %d\n", annexbuf.dataLen(), annexbuf.unparsed(), total_read_nbytes);
      }
   } while (0);
   
   dbgi("read: bytes %d, unit %d\n", total_read_nbytes, num_units);

   if (ifile)
   {
      fclose(ifile);
      ifile = NULL;
   }

   if (ofile)
   {
      fclose(ofile);
      ofile = NULL;
   }

   return 0;
}

int lengthed_to_annexb(const char * input, const char * output, int max_bytes)
{
   FILE* ifile = NULL;
   FILE* ofile = NULL;
   int num_units = 0;
   int total_read_nbytes = 0;
   const uint8_t ANNEXB[] = { 0, 0, 1 };

   do
   {
      ifile = fopen(input, "rb");
      if (!ifile) {
         dbgi("Error opening file [%s]\n", input);
         break;
      }
      dbgi("opened input [%s]\n", input);

      ofile = fopen(output, "wb");
      if (!ofile) {
         dbgi("Error opening file [%s]\n", output);
         break;
      }
      dbgi("opened output [%s]\n", output);


      auto buf = std::vector<uint8_t>(4*1024);
      int unit_len = 0;

      while(feof(ifile) == 0 && total_read_nbytes < max_bytes) 
      {
         dbgd("====================\n");


         {
            int nbytes = fread(&unit_len, 1, 4, ifile);
            if (nbytes < 4) 
            {
               break;
            }
            unit_len = htonl(unit_len);

            total_read_nbytes += nbytes;
            dbgd("read: wsize %d, bytes %d, total_read_nbytes %d\n", wsize, nbytes, total_read_nbytes);

            fwrite(ANNEXB, 1, sizeof(ANNEXB)/sizeof(ANNEXB[0]), ofile);
         }

         if (unit_len > 0)
         {
            int num = 0;
            while (num < unit_len) 
            {
               auto remains = (unit_len - num);
               auto chunk_len = std::min((int)buf.size(), remains);
               int nbytes = fread(buf.data(), 1, chunk_len, ifile);
               if (nbytes > 0) 
               {
                  fwrite(buf.data(), 1, chunk_len, ofile);
               }
               num += chunk_len;
            }

            total_read_nbytes += unit_len;
            ++num_units;
         }
      }
   } while (0);
   
   dbgi("read: bytes %d, unit %d\n", total_read_nbytes, num_units);

   if (ifile)
   {
      fclose(ifile);
      ifile = NULL;
   }

   if (ofile)
   {
      fclose(ofile);
      ofile = NULL;
   }

   return 0;
}

// static int test()
// {
//    const char * output = "/tmp/test_lengthed.data";
//    const int max_bytes = INT_MAX;
//    int ret = annexb_to_lengthed
//    (
//       "/tmp/sample-data/test.h264", 
//       output, 
//       max_bytes
//    );

//    if (ret == 0)
//    {
//       ret = lengthed_to_annexb
//       (
//          output,
//          "/tmp/test_lengthed.h264", 
//          max_bytes
//       );
//    }
   
//    return ret;
// }

int main(int argc, char *argv[]) 
{   
   // return test();

   if (argc != 3) {
      dbgi("Usage: %s <input.h264> <output.data>\n", argv[0]);
      return -1;
   }

   auto input = argv[1];
   auto output = argv[2];

   return annexb_to_lengthed(argv[1], argv[2], INT_MAX);
}

