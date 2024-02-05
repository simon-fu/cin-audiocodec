LIB_NAME=audiocodecs
LIB_DIR=..
THIRD_DIR=../../3rd

LIB_RTP_DIR=../librtp
LIB_RTP_OUTPUT_DIR=$(LIB_RTP_DIR)/debug.-linux64
LIB_RTP_STATIC=$(LIB_RTP_OUTPUT_DIR)/librtp.a

LIBS= \
	-L$(LIB_RTP_OUTPUT_DIR) \
	-lrtp \
	-lstdc++ \
	-lm
	
	
Depend3rdLibs=\
		$(LIB_RTP_STATIC) 

DependLibs= 

INCLUDE=-I . -I .. \
	-I $(LIB_RTP_DIR)/include 


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


out:rtp_test
	@echo make test completed.
	./rtp_test

rtp_test: testrtph265.cpp annexb.cpp $(DependLibs)
	$(CC) -o $@ testrtph265.cpp annexb.cpp $(CPPFLAGS)

clean:
	rm -f rtp_test
	rm -rf rtp_test.dSYM/

clean3rd: 
	make -C $(LIB_RTP_DIR) clean


