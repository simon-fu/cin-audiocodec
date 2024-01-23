## Download 3rd and PCM files
3rd 和 测试PCM文件 打包放在github，下载时可能需要代理，在脚本里搜 get_3rd get_pcm 可找到下载链接。
```shell
./setup-3rd.sh auto 
``` 

## Build Library
```shell
make -C audiocodec 3rd && make -C audiocodec 
``` 
## Build and Test
```shell
make -C audiocodec 3rd && make -C audiocodec test && ./audiocodec/test/audiocodectest
``` 

## 3rd
### ITU-T_pesq
从 [ITU-T_pesq](https://github.com/simon-fu/ITU-T_pesq/tree/simon-dev) 下载 [zip包](https://codeload.github.com/simon-fu/ITU-T_pesq/zip/refs/heads/simon-dev)，解压缩后创建符号链接
```shell
ln -s ITU-T_pesq-simon-dev ITU-T_pesq
```  

### bcg729
下载并解压缩 [bcg729-1.1.1.tar.gz](https://gitlab.linphone.org/BC/public/bcg729/-/archive/1.1.1/bcg729-1.1.1.tar.gz) 

### fdk-aac
下载并解压缩 [fdk-aac-2.0.3.tar.gz](https://jaist.dl.sourceforge.net/project/opencore-amr/fdk-aac/fdk-aac-2.0.3.tar.gz)  

### lame
使用原版 svn MS/3rd/lame-3.100

### opencore-amr
下载并解压缩 [opencore-amr-0.1.6.tar.gz](https://jaist.dl.sourceforge.net/project/opencore-amr/opencore-amr/opencore-amr-0.1.6.tar.gz)  

### tiff
使用原版 svn MS/3rd/tiff-3.7.1

### speexdsp
下载并解压缩 [speexdsp-1.2.1.tar.gz](https://ftp.osuosl.org/pub/xiph/releases/speex/speexdsp-1.2.1.tar.gz)

### vo-amrwbenc
下载并解压缩 [vo-amrwbenc-0.1.3.tar.gz](https://jaist.dl.sourceforge.net/project/opencore-amr/vo-amrwbenc/vo-amrwbenc-0.1.3.tar.gz)  


### opus
编译svn 原版，M1 成功， 但kylin10 失败。
```
/home/funing/opus-1.4/missing:行81: automake-1.13：未找到命令
```
[原版](https://downloads.xiph.org/releases/opus/opus-1.4.tar.gz) 都能编译成功。
```shell
./configure --enable-shared=no --enable-static=yes && make
```

### spandsp  

- svn 原版 spandsp-0.0.6   
  在 kylin10 (192.168.2.52) 上编译失败，苹果M1也编译失败。  

- 最新版 [spandsp 7b0b8cf](https://github.com/freeswitch/spandsp/commit/7b0b8cf3d42b725405bcc63145de5e280265ce4e)   
  kylin10 编译成功，M1编译失败
    ```
    error: initializer element is not a compile-time constant
    ```
- 原版 [spandsp-0.0.6.tar.gz](https://src.fedoraproject.org/lookaside/pkgs/spandsp/spandsp-0.0.6.tar.gz/897d839516a6d4edb20397d4757a7ca3/spandsp-0.0.6.tar.gz)   
  kylin10 和 M1 都编译成功，但configure命令不一样。  
  M1： 
    ```shell  
    ./configure --enable-shared=no --enable-static=yes 
    ```

  
  kylin10：    
    ```shell
    ./configure --enable-shared=no --enable-static=yes --build=aarch64-unknown-linux-gnu
    ```
  kylin10 如果没有 --build=aarch64-unknown-linux-gnu 会提示
    ```shell
    configure: error: cannot guess build type; you must specify one
    ```
  参考[这个](https://stackoverflow.com/questions/4810996/how-to-resolve-configure-guessing-build-type-failure)，可以解决
    ```shell
    $ cp /usr/share/automake-1.16/config.guess ./config/config.guess

    $ ./configure --enable-shared=no --enable-static=yes
    ```
  用 kylin10 上的 config.guess 在 M1 上也能编译成功。

### minimp3
下载 [minimp3](https://github.com/lieff/minimp3) 两个头文件 


## TODO
- CAudioTranscoder::push 要缓存输入数据（如果decoder没有消费完）
