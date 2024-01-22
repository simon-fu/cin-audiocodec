#!/bin/bash

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
    rm -f ITU-T_pesq  
    rm -rf ITU-T_pesq-simon-dev/
    rm -rf bcg729-1.1.1/
    rm -rf fdk-aac-2.0.3/
    rm -rf lame-3.100/
    rm -rf opencore-amr-0.1.6/
    rm -rf opus-1.4/
    rm -rf spandsp-0.0.6/
    rm -rf speexdsp-1.2.1/
    rm -rf tiff-3.7.1/
    rm -rf vo-amrwbenc-0.1.3/
    echo clean done
}

function unpack() {
    echo unpacking ... \
    && unzip -q ITU-T_pesq-simon-dev.zip \
    && ln -s ITU-T_pesq-simon-dev ITU-T_pesq \
    && tar xf bcg729-1.1.1.tar.gz \
    && tar xf fdk-aac-2.0.3.tar.gz \
    && tar xf lame-3.100.tar.gz \
    && tar xf opencore-amr-0.1.6.tar.gz \
    && tar xf opus-1.4.tar.gz \
    && tar xf spandsp-0.0.6.tar.gz \
    && tar xf speexdsp-1.2.1.tar.gz \
    && tar xf tiff-3.7.1.tar.gz \
    && tar xf vo-amrwbenc-0.1.3.tar.gz \
    && cp spandsp-fix/config.guess spandsp-0.0.6/config/config.guess \
    && echo unpack done
}

function reset() {
    fail2die clean
    fail2die unpack
}

function help() {
    echo "usage: "
    echo "  $0 <cmd>"
    echo "  "
    echo "  commands: "
    echo "    clean     delete directories"
    echo "    unpack    unpack/unzip files"
    echo "    reset     clean and unpack"
}

CMD0="$0"
THIZ_DIR="$( cd "$( dirname $0)" && pwd )"
cd $THIZ_DIR


if  [ ! "$1" ] ;then
    help
else
    $@
fi


