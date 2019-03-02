# panic at 'kernel mapping failed: pageAlredyMapped'

解决方案：将 main.rs 中的 **pub extern "C" fn main()** 改为 **pub extern "C" fn _start()**。

> 适用于MacOS。