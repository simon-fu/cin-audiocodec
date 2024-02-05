
### x265  
只有编码，没有解码。  
官网[地址](https://www.videolan.org/developers/x265.html)， [github](https://github.com/videolan/x265)， 下载[地址](http://ftp.videolan.org/pub/videolan/x265/) （http链接，不安全）  
最新版是 x265_3.2.tar.gz                                    25-Sep-2019 13:42             1425689  
  
编译 （cmake）  
```
cd build && cmake ../source/ -DENABLE_SHARED=OFF && make
``` 
  
查看cmake编译选项
```
cd build && cmake .. -LH
```


### libde265
解码测试没问题。  
编码还是实验性，测试是crash，不成熟。  
[github](https://github.com/strukturag/libde265)  
编译  
```
mkdir -p build && cd build && cmake .. -DENABLE_SHARED=OFF && make
```
  
查看cmake编译选项
```
cd build && cmake .. -LH
```
