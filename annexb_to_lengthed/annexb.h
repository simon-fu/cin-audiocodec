#ifndef _ANNEXB_H
#define _ANNEXB_H

#include "rwbuf.h"
#include <stdlib.h>

struct AnnexBCursor {
   const uint8_t * start;
   int zeros;

   AnnexBCursor(): start(NULL), zeros(0){}

   bool isValid()
   {
      return start != NULL;
   }

   const uint8_t * data()
   {
      return start + zeros + 1;
   }

   int continueParse(const uint8_t * data, int len);



   void clear()
   {
      this->start = NULL;
      this->zeros = 0;
   }



};

struct AnnexBUnit {
   AnnexBCursor cursor;
   int length;

   AnnexBUnit(): cursor(AnnexBCursor()), length(0) {}

   bool isValid()
   {
      return cursor.isValid();
   }

   const uint8_t *  annexbPtr()
   {
      return this->cursor.start;
   }

   int annexbLen()
   {
      return this->length;
   }

   const uint8_t *  unitPtr()
   {
      return this->cursor.data();
   }

   int unitLen()
   {
      return this->length - this->cursor.zeros - 1;
   }

   // static AnnexBUnit empty()
   // {
   //    return AnnexBUnit{.cursor=AnnexBCursor::empty(), .length=0};
   // }
};

class AnnexBBuf
{
   public:
      AnnexBBuf()
      : m_cursor1(AnnexBCursor())
      , m_cursor2(AnnexBCursor())
      , m_parsed(0)
      {}

      uint8_t * wBuf()
      {
         return m_buf.wBuf();
      }

      int wSize()
      {
         return m_buf.wSize();
      }

      void wAdvance(int cnt)
      {
         m_buf.wAdvance(cnt);
      }

      void reserve(int extra)
      {
         // printf("reserve: 111 m_cursor1.start %p, m_buf.rData %p\n", m_cursor1.start, m_buf.rData());
         if (m_cursor1.isValid())
         {
            int offset = m_cursor1.start - m_buf.rData();
            m_buf.reserve(extra);
            m_cursor1.start = m_buf.rData() + offset;
         }
         else
         {
            m_buf.reserve(extra);
         }

         // printf("reserve: 222 m_cursor1.start %p, m_buf.rData %p\n", m_cursor1.start, m_buf.rData());
      }

      int unparsed()
      {
         return m_buf.rLen() - m_parsed;
      }

      int dataLen()
      {
         return m_buf.rLen();
      }

      void trim()
      {
         if (m_cursor1.isValid())
         {
            int offset = m_cursor1.start - m_buf.rData();
            // int unparsed = this->unparsed();  
            // printf("trim: 111 offset %d, m_parsed %d, rlen %d, m_cursor1.start %p, m_buf.rData %p\n", offset, m_parsed, m_buf.rLen(), m_cursor1.start, m_buf.rData());
            
            m_buf.rAdvance(offset);
            m_parsed -= offset;

            m_buf.trim();

            m_cursor1.start = m_buf.rData();

            // printf("trim: 222 offset %d, m_parsed %d, rlen %d, m_cursor1.start %p, m_buf.rData %p\n", offset, m_parsed, m_buf.rLen(), m_cursor1.start, m_buf.rData());
         }
         else
         {
            int offset = m_buf.rLen() - m_cursor1.zeros;
            m_buf.rAdvance(offset);
            m_parsed -= offset;

            m_buf.trim();
         }
      }

      bool next(AnnexBUnit& unit)
      {
         // printf("next, 111 m_cursor1.start %p, m_buf.rData %p, m_parsed %d\n", m_cursor1.start, m_buf.rData(), m_parsed);
         if (!m_cursor1.isValid()) 
         {
            auto nparsed = m_cursor1.continueParse(m_buf.rData() + m_parsed, m_buf.rLen() - m_parsed);
            m_parsed += nparsed;
            // printf("cursor1, m_parsed %d\n", m_parsed);

            if (!m_cursor1.isValid())
            {
               return false;
            } 
         }

         auto nparsed = m_cursor2.continueParse(m_buf.rData() + m_parsed, m_buf.rLen() - m_parsed);
         m_parsed += nparsed;

         if (!m_cursor2.isValid())
         {
            // printf("next, m_cursor2 invalid, m_cursor1.start %p, m_buf.rData %p\n", m_cursor1.start, m_buf.rData());
            return false;
         }

         // printf("cursor2, m_parsed %d, start1 %p, start2 %p\n", m_parsed, m_cursor1.start, m_cursor2.start);

         unit.cursor = m_cursor1;
         unit.length = m_cursor2.start - m_cursor1.start;

         m_cursor1 = m_cursor2;
         m_cursor2.clear();

         return true;
      }

   private:
      RwBuf<uint8_t> m_buf;
      AnnexBCursor m_cursor1;
      AnnexBCursor m_cursor2;
      int m_parsed;
};




#endif
