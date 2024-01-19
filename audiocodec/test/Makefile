LIB_NAME=audiocodecs
LIB_DIR=..
THIRD_DIR=../../3rd

LIBS= -L$(LIB_DIR) \
	-l$(LIB_NAME) \
	-L$(THIRD_DIR)/spandsp-0.0.6 \
	-lspandsp \
	-L $(THIRD_DIR)/opus-1.4/out/lib \
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
	-L $(THIRD_DIR)/ITU-T_pesq-simon-dev \
	-lpesqraw \
	-lstdc++ \
	-lm

#	-L $(THIRD_DIR)/webrtc_vad/lib \
#	-lsignal_process \
	
DependLibs=\
		$(LIB_DIR)/lib$(LIB_NAME).a \
		$(THIRD_DIR)/spandsp-0.0.6/libspandsp.a \
		$(THIRD_DIR)/opus-1.4/out/lib/libopus.a \
		$(THIRD_DIR)/vo-amrwbenc-0.1.3/.libs/libvo-amrwbenc.a \
		$(THIRD_DIR)/opencore-amr-0.1.6/amrwb/.libs/libopencore-amrwb.a \
		$(THIRD_DIR)/fdk-aac-2.0.3/.libs/libfdk-aac.a \
		$(THIRD_DIR)/speexdsp-1.2.1/libspeexdsp/.libs/libspeexdsp.a \
		$(THIRD_DIR)/bcg729-1.1.1/target/src/libbcg729.a \
		$(THIRD_DIR)/lame-3.100/out/lib/libmp3lame.a \
		$(THIRD_DIR)/ITU-T_pesq-simon-dev/libpesqraw.a

INCLUDE=-I . -I .. \
	-I $(THIRD_DIR)/speexdsp-1.2.1/include/speex \
	-I $(THIRD_DIR)/ITU-T_pesq-simon-dev/source 

CPPFLAGS=-g $(DEBUGMEM) -Wall -Wno-literal-suffix -Wno-sign-compare \
			-O2 \
			-std=c++11 \
			-Wno-non-virtual-dtor \
			-D_USC_DTMF_TD -DLINUX32 \
			-D_USE_VALGRIND_TO_CHECK_MEM_LEAK \
			$(INCLUDE) $(LIBS)

OUT_LIST=audiocodectest

out:audiocodectest
	@echo make test completed.

audiocodectest: audiocodectest.cpp $(DependLibs)
	$(CC) -o $@ audiocodectest.cpp $(CPPFLAGS)

clean:
	rm -f $(OUT_LIST)

clean3rd: 
	make -C $(THIRD_DIR)/spandsp-0.0.6 clean
	make -C $(THIRD_DIR)/opus-1.4 -f make.opus clean
	make -C $(THIRD_DIR)/vo-amrwbenc-0.1.3 clean
	make -C $(THIRD_DIR)/opencore-amr-0.1.6 clean
	make -C $(THIRD_DIR)/fdk-aac-2.0.3 clean
	make -C $(THIRD_DIR)/speexdsp-1.2.1 clean
	rm -rf $(THIRD_DIR)/bcg729-1.1.1/target
	make -C $(THIRD_DIR)/lame-3.100 -f make.lame clean
	make -C $(THIRD_DIR)/ITU-T_pesq-simon-dev -f MakefileLib.mak clean
	make -C $(THIRD_DIR)/tiff-3.7.1 clean

$(THIRD_DIR)/spandsp-0.0.6/libspandsp.a:
	make -C $(THIRD_DIR)/spandsp-0.0.6

$(THIRD_DIR)/opus-1.4/out/lib/libopus.a:
	make -C ./3rd/opus-1.4 -f make.opus

$(THIRD_DIR)/vo-amrwbenc-0.1.3/.libs/libvo-amrwbenc.a:
	cd $(THIRD_DIR)/vo-amrwbenc-0.1.3 && \
    ./configure && \
    make

$(THIRD_DIR)/opencore-amr-0.1.6/amrwb/.libs/libopencore-amrwb.a:
	cd $(THIRD_DIR)/opencore-amr-0.1.6 && \
    ./configure && \
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

$(THIRD_DIR)/lame-3.100/out/lib/libmp3lame.a:
	make -C $(THIRD_DIR)/lame-3.100 -f make.lame

$(THIRD_DIR)/ITU-T_pesq-simon-dev/libpesqraw.a:
	make -C $(THIRD_DIR)/ITU-T_pesq-simon-dev -f MakefileLib.mak
