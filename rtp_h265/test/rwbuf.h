#ifndef _RWBUF_H
#define _RWBUF_H

#include <cstring>
#include <vector>

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

#endif
