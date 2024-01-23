#!/bin/bash

CMD0="$0"
THIZ_DIR="$( cd "$( dirname $0)" && pwd )"
THIZ_3RD_DIR="$THIZ_DIR/3rd"
THIZ_3RD_GZFILE="$THIZ_DIR/3rd.tar.gz"

THIZ_PCM_DIR="$THIZ_DIR/sample_pcm"
THIZ_PCM_GZFILE="$THIZ_DIR/sample_pcm.tar.gz"

cd $THIZ_DIR

function fail2die {
    "$@"
    local status=$?
    if [ $status -ne 0 ]; then
        echo "failed: $@" >&2
        exit 1
    fi
}


function clean() {
    echo cleaning ...
    rm -rf $THIZ_3RD_DIR
    # rm -f ITU-T_pesq  
    # rm -rf ITU-T_pesq-simon-dev/
    # rm -rf bcg729-1.1.1/
    # rm -rf fdk-aac-2.0.3/
    # rm -rf lame-3.100/
    # rm -rf opencore-amr-0.1.6/
    # rm -rf opus-1.4/
    # rm -rf spandsp-0.0.6/
    # rm -rf speexdsp-1.2.1/
    # rm -rf tiff-3.7.1/
    # rm -rf vo-amrwbenc-0.1.3/
    echo clean done
}

function unpack_3rd() {
    echo unpacking 3rd ... \
    && cd $THIZ_DIR \
    && tar xf 3rd.tar.gz \
    && cd 3rd \
    && ln -s ITU-T_pesq-simon-dev ITU-T_pesq \
    && cp spandsp-fix/config.guess spandsp-0.0.6/config/config.guess \
    && echo unpack 3rd done

    # echo unpacking ... \
    # && unzip -q ITU-T_pesq-simon-dev.zip \
    # && ln -s ITU-T_pesq-simon-dev ITU-T_pesq \
    # && tar xf bcg729-1.1.1.tar.gz \
    # && tar xf fdk-aac-2.0.3.tar.gz \
    # && tar xf lame-3.100.tar.gz \
    # && tar xf opencore-amr-0.1.6.tar.gz \
    # && tar xf opus-1.4.tar.gz \
    # && tar xf spandsp-0.0.6.tar.gz \
    # && tar xf speexdsp-1.2.1.tar.gz \
    # && tar xf tiff-3.7.1.tar.gz \
    # && tar xf vo-amrwbenc-0.1.3.tar.gz \
    # && cp spandsp-fix/config.guess spandsp-0.0.6/config/config.guess \
    # && echo unpack done
}

function unpack_pcm() {
    echo unpacking pcm ... \
    && cd $THIZ_DIR \
    && tar xf sample_pcm.tar.gz \
    && echo unpack pcm done
}

function get_3rd() {
    local url="https://github.com/simon-fu/cin-audiocodec/releases/download/3rd/3rd.tar.gz"
    echo "downloading [$url]"
    fail2die curl -L $url -o $THIZ_3RD_GZFILE
    echo "downloaded [$THIZ_3RD_GZFILE]"
}

function get_pcm() {
    local url="https://github.com/simon-fu/cin-audiocodec/releases/download/3rd/sample_pcm.tar.gz"
    echo "downloading [$url]"
    fail2die curl -L $url -o $THIZ_PCM_GZFILE
    echo "downloaded [$THIZ_PCM_GZFILE]"
}

function unpack() {
    fail2die unpack_3rd
    fail2die unpack_pcm
}

function reset() {
    fail2die clean
    fail2die unpack
}

function download() {
    fail2die get_3rd
    fail2die get_pcm
}

function auto_get () {
    local get_func=$1
    local dir=$2
    local file=$3
    local unpack_pcm=$4

    if [ -d "$dir" ]; then 
        echo "exist dir [$dir], skip download"
        return
    fi

    if [ -f "$file" ]; then 
        echo "exist file [$file], unpack it"
        fail2die $unpack_pcm
        return
    fi

    fail2die $get_func
    fail2die $unpack_pcm
}

function auto () {
    auto_get get_3rd $THIZ_3RD_DIR $THIZ_3RD_GZFILE unpack_3rd
    auto_get get_pcm $THIZ_PCM_DIR $THIZ_PCM_GZFILE unpack_pcm

    # if [ ! -f "$THIZ_3RD_GZFILE" ]; then 
    #     # echo "NOT exit [$THIZ_3RD_GZFILE]"
    #     fail2die get_3rd
    # fi

    # if [ ! -d "$THIZ_3RD_DIR" ]; then 
    #     # echo "NOT exit [$THIZ_3RD_DIR]"
    #     fail2die unpack
    # fi
}

function help() {
    echo "usage: "
    echo "  $0 <cmd>"
    echo "  "
    echo "  commands: "
    echo "    auto      auto download and unpack"
    echo "    download  download 3rd package file"
    echo "    clean     delete directories"
    echo "    unpack    unpack/unzip files"
    echo "    reset     clean and unpack"
}




if  [ ! "$1" ] ;then
    help
else
    $@
fi


