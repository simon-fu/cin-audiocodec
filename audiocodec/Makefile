LIB_NAME=audiocodecs
OUT_LIB=lib$(LIB_NAME).a
THIRD_DIR=../3rd



SOURCE=audiocodecs.cpp \
		audiotranscoder.cpp \
		g711codec.cpp \
		opuscodec.cpp \
		amrwbcodec.cpp \
		amrnbcodec.cpp \
		aaccodec.cpp \
		g729codec.cpp \
		mp3codec.cpp 
		 
OBJ=${SOURCE:.cpp=.o}

INCLUDE=-I . -I .. \
		 -I $(THIRD_DIR)/spandsp-0.0.6/src \
		 -I $(THIRD_DIR)/tiff-3.7.1/libtiff \
		 -I $(THIRD_DIR)/opus-1.4/out/include/opus \
		 -I $(THIRD_DIR)/vo-amrwbenc-0.1.3 \
		 -I $(THIRD_DIR)/vo-amrwbenc-0.1.3/common/include \
		 -I $(THIRD_DIR)/opencore-amr-0.1.6 \
		 -I $(THIRD_DIR)/fdk-aac-2.0.3/libAACenc/include \
		 -I $(THIRD_DIR)/fdk-aac-2.0.3/libAACdec/include \
		 -I $(THIRD_DIR)/fdk-aac-2.0.3/libSYS/include \
		 -I $(THIRD_DIR)/speexdsp-1.2.1/include/speex \
		 -I $(THIRD_DIR)/bcg729-1.1.1/include \
		 -I $(THIRD_DIR)/lame-3.100/include \
		 -I $(THIRD_DIR)/minimp3 


CPPFLAGS=-g $(DEBUGMEM) -Wall -Wno-literal-suffix -Wno-sign-compare \
			-O2 \
			-std=c++11 \
			-Wno-non-virtual-dtor \
			-D_USC_DTMF_TD -DLINUX32 \
			-D_USE_VALGRIND_TO_CHECK_MEM_LEAK \
			$(INCLUDE)
		
CC=g++

out:${OUT_LIB}
	@echo make completed.

${OUT_LIB}:${OBJ}
	ar cru $@ ${OBJ}
	ranlib $@

# $(OUT): $(SOURCE:.cpp=.o)
# 	$(CC) -o $@ $^ $(LIBS)

include $(SOURCE:.cpp=.d)

%.d: %.cpp
	set -e; rm -f $@; \
	$(CC) $< -MM $(CPPFLAGS) > $@.$$$$; \
	sed 's,\($*\)\.o[ :]*,\1.o $@ : ,g' < $@.$$$$ > $@; \
	rm -f $@.$$$$

.PHONY: test
test: ${OUT_LIB}
	make -C test

clean:
	rm -f $(OUT_LIB)
	rm -f $(SOURCE:.cpp=.o)
	rm -f $(SOURCE:.cpp=.d)
	make -C test clean
	@echo cleaned
	
clean3rd: 
	make -C test clean3rd


