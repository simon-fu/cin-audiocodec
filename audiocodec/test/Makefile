LIB_NAME=audiocodecs
LIB_DIR=..
THIRD_DIR=../../3rd

LIBS= -L$(LIB_DIR) \
	-l$(LIB_NAME) \
	-L$(THIRD_DIR)/spandsp-0.0.6/src/.libs \
	-lspandsp \
	-L $(THIRD_DIR)/opus-1.4/.libs \
	-lopus \
	-L $(THIRD_DIR)/vo-amrwbenc-0.1.3/.libs \
	-lvo-amrwbenc \
	-L $(THIRD_DIR)/opencore-amr-0.1.6/amrwb/.libs \
	-lopencore-amrwb \
	-L $(THIRD_DIR)/opencore-amr-0.1.6/amrnb/.libs \
	-lopencore-amrnb \
	-L $(THIRD_DIR)/fdk-aac-2.0.3/.libs \
	-lfdk-aac \
	-L $(THIRD_DIR)/speexdsp-1.2.1/libspeexdsp/.libs \
	-lspeexdsp \
	-L $(THIRD_DIR)/bcg729-1.1.1/target/src \
	-lbcg729 \
	-L $(THIRD_DIR)/lame-3.100/lib \
	-lmp3lame \
	-L $(THIRD_DIR)/ITU-T_pesq \
	-lpesqraw \
	-lstdc++ \
	-lm

#	-L $(THIRD_DIR)/webrtc_vad/lib \
#	-lsignal_process \
	
Depend3rdLibs=\
		$(THIRD_DIR)/spandsp-0.0.6/src/.libs/libspandsp.a \
		$(THIRD_DIR)/opus-1.4/.libs/libopus.a \
		$(THIRD_DIR)/vo-amrwbenc-0.1.3/.libs/libvo-amrwbenc.a \
		$(THIRD_DIR)/opencore-amr-0.1.6/amrwb/.libs/libopencore-amrwb.a \
		$(THIRD_DIR)/fdk-aac-2.0.3/.libs/libfdk-aac.a \
		$(THIRD_DIR)/speexdsp-1.2.1/libspeexdsp/.libs/libspeexdsp.a \
		$(THIRD_DIR)/bcg729-1.1.1/target/src/libbcg729.a \
		$(THIRD_DIR)/lame-3.100/lib/libmp3lame.a \
		$(THIRD_DIR)/ITU-T_pesq/libpesqraw.a

DependLibs= $(LIB_DIR)/lib$(LIB_NAME).a $(Depend3rdLibs)

INCLUDE=-I . -I .. \
	-I $(THIRD_DIR)/speexdsp-1.2.1/include/speex \
	-I $(THIRD_DIR)/ITU-T_pesq/source 


CPPFLAGS=-g $(DEBUGMEM) -Wall

ifeq ($(shell uname),Darwin)
    CPPFLAGS += -Wno-reserved-user-defined-literal
else
    CPPFLAGS += -Wno-literal-suffix
endif

CPPFLAGS += -Wno-sign-compare \
			-O2 \
			-std=c++11 \
			-Wno-non-virtual-dtor \
			-D_USC_DTMF_TD -DLINUX32 \
			-D_USE_VALGRIND_TO_CHECK_MEM_LEAK \
			$(INCLUDE)

CPPFLAGS += $(LIBS)

# CPPFLAGS=-g $(DEBUGMEM) -Wall -Wno-literal-suffix -Wno-sign-compare \
# 			-O2 \
# 			-std=c++11 \
# 			-Wno-non-virtual-dtor \
# 			-D_USC_DTMF_TD -DLINUX32 \
# 			-D_USE_VALGRIND_TO_CHECK_MEM_LEAK \
# 			$(INCLUDE) $(LIBS)



out:audiocodectest
	@echo make test completed.
	./audiocodectest

audiocodectest: audiocodectest.cpp $(DependLibs)
	$(CC) -o $@ audiocodectest.cpp $(CPPFLAGS)

clean:
	rm -f audiocodectest
	rm -rf audiocodectest.dSYM/

clean3rd: 
	make -C $(THIRD_DIR)/spandsp-0.0.6 clean
#	make -C $(THIRD_DIR)/opus-1.4 -f make.opus clean
	make -C $(THIRD_DIR)/opus-1.4 clean
	make -C $(THIRD_DIR)/vo-amrwbenc-0.1.3 clean
	make -C $(THIRD_DIR)/opencore-amr-0.1.6 clean
	make -C $(THIRD_DIR)/fdk-aac-2.0.3 clean
	make -C $(THIRD_DIR)/speexdsp-1.2.1 clean
	rm -rf $(THIRD_DIR)/bcg729-1.1.1/target
	make -C $(THIRD_DIR)/lame-3.100 -f make.lame clean
	make -C $(THIRD_DIR)/ITU-T_pesq -f MakefileLib.mak clean
	make -C $(THIRD_DIR)/tiff-3.7.1 clean

3rd: $(Depend3rdLibs)

$(THIRD_DIR)/spandsp-0.0.6/src/.libs/libspandsp.a:
	cd $(THIRD_DIR)/spandsp-0.0.6 && \
	./configure --enable-shared=no --enable-static=yes && \
	make
#	make -C $(THIRD_DIR)/spandsp-0.0.6


$(THIRD_DIR)/opus-1.4/.libs/libopus.a:
	cd $(THIRD_DIR)/opus-1.4 && \
	./configure --enable-shared=no --enable-static=yes && \
	make
#	make -C ./3rd/opus-1.4 -f make.opus

$(THIRD_DIR)/vo-amrwbenc-0.1.3/.libs/libvo-amrwbenc.a:
	cd $(THIRD_DIR)/vo-amrwbenc-0.1.3 && \
    ./configure --enable-shared=no --enable-static=yes && \
    make

$(THIRD_DIR)/opencore-amr-0.1.6/amrwb/.libs/libopencore-amrwb.a:
	cd $(THIRD_DIR)/opencore-amr-0.1.6 && \
    ./configure --enable-shared=no --enable-static=yes && \
    make

$(THIRD_DIR)/fdk-aac-2.0.3/.libs/libfdk-aac.a:
	cd $(THIRD_DIR)/fdk-aac-2.0.3 && \
    ./configure --enable-shared=no --enable-static=yes && \
    make

$(THIRD_DIR)/speexdsp-1.2.1/libspeexdsp/.libs/libspeexdsp.a:
	cd $(THIRD_DIR)/speexdsp-1.2.1 && \
    ./configure --enable-shared=no --enable-static=yes && \
    make

$(THIRD_DIR)/bcg729-1.1.1/target/src/libbcg729.a:
	mkdir -p $(THIRD_DIR)/bcg729-1.1.1/target && \
	cd $(THIRD_DIR)/bcg729-1.1.1/target && \
	cmake3 .. -DENABLE_SHARED=NO -DENABLE_TESTS=NO && \
	make

$(THIRD_DIR)/lame-3.100/lib/libmp3lame.a:
	make -C $(THIRD_DIR)/lame-3.100 -f make.lame

$(THIRD_DIR)/ITU-T_pesq/libpesqraw.a:
	make -C $(THIRD_DIR)/ITU-T_pesq -f MakefileLib.mak
