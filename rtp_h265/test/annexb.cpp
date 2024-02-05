
#include "annexb.h"
#include <vector>

// static 
// int leading_zero_bytes(const uint8_t* data, int bytes)
// {
// 	int i;
// 	for (i = 0; i < bytes; i++)
// 	{
//       if (data[i] != 0)
//       {
//          return i;
//       }
// 	}

// 	return bytes;
// }

// static 
// int tailing_zero_bytes(const uint8_t* data, int bytes)
// {
// 	int reamins = bytes;
// 	for (; reamins > 0; reamins--)
// 	{
//       data--;
//       if (*data != 0)
//       {
//          break;
//       }
// 	}

// 	return bytes - reamins;
// }



// static 
// const uint8_t* h264_startcode(const uint8_t* data, int bytes)
// {
// 	int i;
// 	for (i = 2; i + 1 < bytes; i++)
// 	{
// 		if (0x01 == data[i] && 0x00 == data[i - 1] && 0x00 == data[i - 2])
// 			return data + i + 1;
// 	}

// 	return NULL;
// }

// static 
// const uint8_t* h264_startcode_at(const uint8_t* data, int bytes, int * pnzeros)
// {
//    auto start = h264_startcode(data, bytes);
// 	if (start)
//    {
//       --start;
//       int pos = start - data;
//       int nzeros = tailing_zero_bytes(start, pos);
//       start = start-nzeros;
//       if (pnzeros)
//       {
//          *pnzeros = nzeros;
//       }
//    }

// 	return start;
// }




// AnnexBCursor AnnexBCursor::parseFrom(const uint8_t * data, int len)
// {
//    auto cursor = AnnexBCursor
//    {
//       .zeros = 0,
//       .start = h264_startcode(data, len)
//    };

// 	if (cursor.start)
//    {
//       --cursor.start;
//       int offset = cursor.start - data;
//       cursor.zeros = tailing_zero_bytes(cursor.start, offset);
//       cursor.start = cursor.start-cursor.zeros;
//    }

// 	return cursor;
// }



int AnnexBCursor::continueParse(const uint8_t * data, int len)
{
   for(int i = 0; i < len; ++i)
   {
      auto value = data[i];

      if (value == 0)
      {
         ++this->zeros;
      }
      else if (value == 1 && this->zeros >= 2)
      {
         this->start = data + i - this->zeros;
         return i+1;
      }
      else
      {
         this->zeros = 0;
      }
   }
   return len;
}

