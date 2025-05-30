<img src="https://static.wikia.nocookie.net/arnelify/images/c/c8/Arnelify-logo-2024.png/revision/latest?cb=20240701012515" style="width:336px;" alt="Arnelify Logo" />

![Arnelify Server for Python](https://img.shields.io/badge/Arnelify%20Server%20for%20Python-0.7.8-yellow) ![C++](https://img.shields.io/badge/C++-2b-red) ![G++](https://img.shields.io/badge/G++-14.2.0-blue) ![Python](https://img.shields.io/badge/Python-3.11.2-blue) ![Nuitka](https://img.shields.io/badge/Nuitka-2.6.4-blue)

## 🚀 About
**Arnelify® Server for Python** - is a minimalistic dynamic library which is a powerful server written in C and C++.

## 📋 Minimal Requirements
> Important: It's strongly recommended to use in a container that has been built from the gcc v14.2.0 image.
* CPU: Apple M1 / Intel Core i7 / AMD Ryzen 7
* OS: Debian 11 / MacOS 15 / Windows 10 with <a href="https://learn.microsoft.com/en-us/windows/wsl/install">WSL2</a>.
* RAM: 4 GB

## 📦 Installation
Installing via pip:
```
pip install arnelify-server
```
## 🎉 Usage
Install dependencies:
```
make install
```
Compile library:
```
make build
```
Compile & Run test:
```
make test_nuitka
```
Run test:
```
make test
```
## 📚 Code Examples
Configure the C/C++ IntelliSense plugin for VSCode (optional).
```
Clang_format_fallback = Google
```

IncludePath for VSCode (optional):
```
"includePath": [
  "/opt/homebrew/Cellar/jsoncpp/1.9.6/include/json",
  "${workspaceFolder}/src/cpp",
  "${workspaceFolder}/src"
],
```
You can find code examples <a href="https://github.com/arnelify/arnelify-server-python/blob/main/tests/index.py">here</a>.

| **Option**|**Description**|
|-|-|
| **SERVER_ALLOW_EMPTY_FILES**| If this option is enabled, the server will not reject empty files.|
| **SERVER_BLOCK_SIZE_KB**| The size of the allocated memory used for processing large packets.|
| **SERVER_CHARSET**| Defines the encoding that the server will recommend to all client applications.|
| **SERVER_GZIP**| If this option is enabled, the server will use GZIP compression if the client application supports it. This setting increases CPU resource consumption. The server will not use compression if the data size exceeds the value of **SERVER_BLOCK_SIZE_KB**.|
| **SERVER_KEEP_EXTENSIONS**| If this option is enabled, file extensions will be preserved.|
| **SERVER_MAX_FIELDS**| Defines the maximum number of fields in the received form.|
| **SERVER_MAX_FIELDS_SIZE_TOTAL_MB**| Defines the maximum total size of all fields in the form. This option does not include file sizes.|
| **SERVER_MAX_FILES**| Defines the maximum number of files in the form.|
| **SERVER_MAX_FILES_SIZE_TOTAL_MB** | Defines the maximum total size of all files in the form.|
| **SERVER_MAX_FILE_SIZE_MB**| Defines the maximum size of a single file in the form.|
| **SERVER_NET_CHECK_FREQ_MS**| Network interface check frequency in milliseconds. The lower the value, the higher the CPU 
| **SERVER_PORT**| Defines which port the server will listen on.|
| **SERVER_THREAD_LIMIT**| Defines the maximum number of threads that will handle requests.|
| **SERVER_QUEUE_LIMIT**| Defines the maximum size of the queue on the client socket.|
| **SERVER_UPLOAD_DIR**| Specifies the upload directory for storage.|

## ⚖️ MIT License
This software is licensed under the <a href="https://github.com/arnelify/arnelify-server-python/blob/main/LICENSE">MIT License</a>. The original author's name, logo, and the original name of the software must be included in all copies or substantial portions of the software.

## 🛠️ Contributing
Join us to help improve this software, fix bugs or implement new functionality. Active participation will help keep the software up-to-date, reliable, and aligned with the needs of its users.

## ⭐ Release Notes
Version 0.7.8 - Minimalistic dynamic library

We are excited to introduce the Arnelify Server dynamic library for Python! Please note that this version is raw and still in active development.

Change log:

* Minimalistic dynamic library for Python
* Block processing in "on-the-fly" mode
* Multi-Threading
* GZIP support
* Significant refactoring and optimizations

Please use this version with caution, as it may contain bugs and unfinished features. We are actively working on improving and expanding the server's capabilities, and we welcome your feedback and suggestions.

## 🔗 Mentioned

* <a href="https://github.com/arnelify/arnelify-pod-cpp">Arnelify POD for C++</a>
* <a href="https://github.com/arnelify/arnelify-pod-python">Arnelify POD for Python</a>
* <a href="https://github.com/arnelify/arnelify-pod-node">Arnelify POD for NodeJS</a>
* <a href="https://github.com/arnelify/arnelify-react-native">Arnelify React Native</a>