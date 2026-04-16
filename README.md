<img src="https://static.wikia.nocookie.net/arnelify/images/c/c8/Arnelify-logo-2024.png/revision/latest?cb=20240701012515" style="width:336px;" alt="Arnelify Logo" />

![Arnelify Server for Python](https://img.shields.io/badge/Arnelify%20Server%20for%20Python-1.0.8-yellow) ![Python](https://img.shields.io/badge/Python-3.13.5-blue) ![Nuitka](https://img.shields.io/badge/Nuitka-2.6.4-blue)

## 🚀 About

**Arnelify® Server for Rust** — a multi-language server with HTTP 3.0 and WebTransport support.

All supported protocols:
| **#** | **Protocol** | **Transport** |
| - | - | - |
| 1 | TCP2 | WebTransport |
| 2 | TCP2 | HTTP 3.0 |
| 3 | TCP1 | WebSocket |
| 4 | TCP1 | HTTP 2.0 |
| 5 | TCP1 | HTTP 1.1 |

## 📋 Minimal Requirements
> Important: It's strongly recommended to use in a container that has been built from the gcc v15.2.0 image.
* CPU: Apple M1 / Intel Core i7 / AMD Ryzen 7
* OS: Debian 11 / MacOS 15 / Windows 10 with <a href="https://learn.microsoft.com/en-us/windows/wsl/install">WSL2</a>.
* RAM: 4 GB

## 📦 Installation
Run in terminal:
```bash
pip install arnelify_server
```
## 🎉 TCP2 / WebTransport

### 📚 Configuration

| **Option** | **Description** |
| - | - |
| **BLOCK_SIZE_KB**| The size of the allocated memory used for processing large packets. |
| **CERT_PEM**| Path to the TLS cert-file in PEM format. |
| **COMPRESSION**| If this option is enabled, the server will use BROTLI compression if the client application supports it. This setting increases CPU resource consumption. The server will not use compression if the data size exceeds the value of **BLOCK_SIZE_KB**. |
| **HANDSHAKE_TIMEOUT**| Maximum time in seconds to complete the TLS handshake. |
| **KEY_PEM**| Path to the TLS private key-file in PEM format. |
| **MAX_MESSAGE_SIZE_MB**| Maximum size of a single message the server will accept from a client. |
| **PING_TIMEOUT**| Maximum time the server will wait for a ping from the client. |
| **PORT**| Defines which port the server will listen on. |
| **SEND_TIMEOUT**| Maximum time for the client to receive a response from the server. |
| **THREAD_LIMIT**| Defines the maximum number of threads that will handle requests.|

### 📚 Examples

```python
from arnelify_server import WebTransport
from arnelify_server import WebTransportBytes
from arnelify_server import WebTransportCtx
from arnelify_server import WebTransportOpts
from arnelify_server import WebTransportStream

import asyncio
from typing import Awaitable

async def main() -> Awaitable[None]:

  wt_opts: WebTransportOpts = {
    "block_size_kb": 64,
    "cert_pem": "certs/cert.pem",
    "compression": True,
    "handshake_timeout": 30,
    "key_pem": "certs/key.pem",
    "max_message_size_kb": 64,
    "ping_timeout": 15,
    "port": 4433,
    "send_timeout": 30,
    "thread_limit": 4
  }

  wt: WebTransport = WebTransport(wt_opts)
  async def wt_logger(_level: str, message: str) -> Awaitable[None]:
    print("[Arnelify Server]: " + message)

  wt.logger(wt_logger)
  async def wt_handler(ctx: WebTransportCtx, bytes: WebTransportBytes, stream: WebTransportStream) -> Awaitable[None]:
    await stream.push(ctx, bytes)
    await stream.close()

  wt.on("connect", wt_handler)
  await wt.start()

if __name__ == "__main__":
    asyncio.run(main())
```
## 🎉 TCP2 / HTTP 3.0

### 📚 Configuration

| **Option** | **Description** |
| - | - |
| **ALLOW_EMPTY_FILES**| If this option is enabled, the server will not reject empty files. |
| **BLOCK_SIZE_KB**| The size of the allocated memory used for processing large packets. |
| **CERT_PEM**| Path to the TLS cert-file in PEM format. |
| **CHARSET**| Defines the encoding that the server will recommend to all client applications. |
| **COMPRESSION**| If this option is enabled, the server will use BROTLI compression if the client application supports it. This setting increases CPU resource consumption. The server will not use compression if the data size exceeds the value of **BLOCK_SIZE_KB**. |
| **KEEP_ALIVE**| defines how long the HTTP server keeps a connection. |
| **KEEP_EXTENSIONS**| If this option is enabled, file extensions will be preserved. |
| **KEY_PEM**| Path to the TLS private key-file in PEM format. |
| **MAX_FIELDS**| Defines the maximum number of fields in the received form. |
| **MAX_FIELDS_SIZE_TOTAL_MB**| Defines the maximum total size of all fields in the form. This option does not include file sizes. |
| **MAX_FILES**| Defines the maximum number of files in the form. |
| **MAX_FILES_SIZE_TOTAL_MB** | Defines the maximum total size of all files in the form. |
| **MAX_FILE_SIZE_MB**| Defines the maximum size of a single file in the form. |
| **PORT**| Defines which port the server will listen on. |
| **STORAGE_PATH**| Specifies the upload directory for storage. |
| **THREAD_LIMIT**| Defines the maximum number of threads that will handle requests. |

### 📚 Examples

```python
from arnelify_server import Http3
from arnelify_server import Http3Ctx
from arnelify_server import Http3Opts
from arnelify_server import Http3Stream

import asyncio
from typing import Awaitable

async def main() -> Awaitable[None]:

  http3_opts: Http3Opts = {
      "allow_empty_files": True,
      "block_size_kb": 64,
      "cert_pem": "certs/cert.pem",
      "charset": "utf-8",
      "compression": True,
      "keep_alive": 30,
      "keep_extensions": True,
      "key_pem": "certs/key.pem",
      "max_fields": 60,
      "max_fields_size_total_mb": 1,
      "max_files": 3,
      "max_files_size_total_mb": 60,
      "max_file_size_mb": 60,
      "port": 4433,
      "storage_path": "/var/www/cpp/storage",
      "thread_limit": 4,
  }

  http3: Http3 = Http3(http3_opts)
  async def http3_logger(_level: str, message: str) -> Awaitable[None]:
    print("[Arnelify Server]: " + message)

  http3.logger(http3_logger)
  async def http3_handler(ctx: Http3Ctx, stream: Http3Stream) -> Awaitable[None]:
    await stream.set_code(200)
    await stream.push_json(ctx)
    await stream.end()

  http3.on("/", http3_handler)
  await http3.start()

if __name__ == "__main__":
    asyncio.run(main())
```

## 🎉 TCP1 / WebSocket

### 📚 Configuration

| **Option** | **Description** |
| - | - |
| **BLOCK_SIZE_KB**| The size of the allocated memory used for processing large packets. |
| **COMPRESSION**| If this option is enabled, the server will use BROTLI compression if the client application supports it. This setting increases CPU resource consumption. The server will not use compression if the data size exceeds the value of **BLOCK_SIZE_KB**. |
| **MAX_MESSAGE_SIZE_MB**| Maximum size of a single message the server will accept from a client. |
| **PING_TIMEOUT**| Maximum time the server will wait for a ping from the client. |
| **PORT**| Defines which port the server will listen on. |
| **RATE_LIMIT**| Defines the maximum number of connections allowed from a single IP address. |
| **READ_TIMEOUT**| Maximum time allowed for a client to send a request to the server. |
| **SEND_TIMEOUT**| Maximum time allowed for the client to receive a response from the server. |
| **THREAD_LIMIT**| Defines the maximum number of threads that will handle requests.|

### 📚 Examples

```python
from arnelify_server import WebSocket
from arnelify_server import WebSocketBytes
from arnelify_server import WebSocketCtx
from arnelify_server import WebSocketOpts
from arnelify_server import WebSocketStream

import asyncio
from typing import Awaitable

async def main() -> Awaitable[None]:

  ws_opts: WebSocketOpts = {
    "block_size_kb": 64,
    "compression": True,
    "max_message_size_kb": 64,
    "ping_timeout": 15,
    "port": 4433,
    "rate_limit": 5,
    "read_timeout": 30,
    "send_timeout": 30,
    "thread_limit": 4
  }

  ws: WebSocket = WebSocket(ws_opts)
  async def ws_logger(_level: str, message: str) -> Awaitable[None]:
    print("[Arnelify Server]: " + message)

  ws.logger(ws_logger)
  async def ws_handler(ctx: WebSocketCtx, bytes: WebSocketBytes, stream: WebSocketStream) -> Awaitable[None]:
    await stream.push(ctx, bytes)
    await stream.close()

  ws.on("connect", ws_handler)
  await ws.start()

if __name__ == "__main__":
    asyncio.run(main())
```

## 🎉 TCP1 / HTTP 2.0

### 📚 Configuration

| **Option** | **Description** |
| - | - |
| **ALLOW_EMPTY_FILES**| If this option is enabled, the server will not reject empty files. |
| **BLOCK_SIZE_KB**| The size of the allocated memory used for processing large packets. |
| **CERT_PEM**| Path to the TLS cert-file in PEM format. |
| **CHARSET**| Defines the encoding that the server will recommend to all client applications. |
| **COMPRESSION**| If this option is enabled, the server will use BROTLI compression if the client application supports it. This setting increases CPU resource consumption. The server will not use compression if the data size exceeds the value of **BLOCK_SIZE_KB**. |
| **KEEP_ALIVE**| defines how long the HTTP server keeps a connection. |
| **KEEP_EXTENSIONS**| If this option is enabled, file extensions will be preserved. |
| **KEY_PEM**| Path to the TLS private key-file in PEM format. |
| **MAX_FIELDS**| Defines the maximum number of fields in the received form. |
| **MAX_FIELDS_SIZE_TOTAL_MB**| Defines the maximum total size of all fields in the form. This option does not include file sizes. |
| **MAX_FILES**| Defines the maximum number of files in the form. |
| **MAX_FILES_SIZE_TOTAL_MB** | Defines the maximum total size of all files in the form. |
| **MAX_FILE_SIZE_MB**| Defines the maximum size of a single file in the form. |
| **PORT**| Defines which port the server will listen on. |
| **STORAGE_PATH**| Specifies the upload directory for storage. |
| **THREAD_LIMIT**| Defines the maximum number of threads that will handle requests. |

### 📚 Examples

```python
from arnelify_server import Http2
from arnelify_server import Http2Ctx
from arnelify_server import Http2Opts
from arnelify_server import Http2Stream

import asyncio
from typing import Awaitable

async def main() -> Awaitable[None]:
   
  http2_opts: Http2Opts = {
      "allow_empty_files": True,
      "block_size_kb": 64,
      "cert_pem": "certs/cert.pem",
      "charset": "utf-8",
      "compression": True,
      "keep_alive": 30,
      "keep_extensions": True,
      "key_pem": "certs/key.pem",
      "max_fields": 60,
      "max_fields_size_total_mb": 1,
      "max_files": 3,
      "max_files_size_total_mb": 60,
      "max_file_size_mb": 60,
      "port": 4433,
      "storage_path": "/var/www/cpp/storage",
      "thread_limit": 4,
  }

  http2: Http2 = Http2(http2_opts)
  async def http2_logger(_level: str, message: str) -> Awaitable[None]:
    print("[Arnelify Server]: " + message)

  http2.logger(http2_logger)
  async def http2_handler(ctx: Http2Ctx, stream: Http2Stream) -> Awaitable[None]:
    await stream.set_code(200)
    await stream.push_json(ctx)
    await stream.end()

  http2.on("/", http2_handler)
  await http2.start()

if __name__ == "__main__":
    asyncio.run(main())
```

## 🎉 TCP1 / HTTP 1.1

### 📚 Configuration

| **Option** | **Description** |
| - | - |
| **ALLOW_EMPTY_FILES**| If this option is enabled, the server will not reject empty files. |
| **BLOCK_SIZE_KB**| The size of the allocated memory used for processing large packets. |
| **CHARSET**| Defines the encoding that the server will recommend to all client applications. |
| **COMPRESSION**| If this option is enabled, the server will use BROTLI compression if the client application supports it. This setting increases CPU resource consumption. The server will not use compression if the data size exceeds the value of **BLOCK_SIZE_KB**. |
| **KEEP_ALIVE**| defines how long the HTTP server keeps a connection. |
| **KEEP_EXTENSIONS**| If this option is enabled, file extensions will be preserved. |
| **MAX_FIELDS**| Defines the maximum number of fields in the received form. |
| **MAX_FIELDS_SIZE_TOTAL_MB**| Defines the maximum total size of all fields in the form. This option does not include file sizes. |
| **MAX_FILES**| Defines the maximum number of files in the form. |
| **MAX_FILES_SIZE_TOTAL_MB** | Defines the maximum total size of all files in the form. |
| **MAX_FILE_SIZE_MB**| Defines the maximum size of a single file in the form. |
| **PORT**| Defines which port the server will listen on. |
| **STORAGE_PATH**| Specifies the upload directory for storage. |
| **THREAD_LIMIT**| Defines the maximum number of threads that will handle requests. |

### 📚 Examples

```python
from arnelify_server import Http1
from arnelify_server import Http1Ctx
from arnelify_server import Http1Opts
from arnelify_server import Http1Stream

import asyncio
from typing import Awaitable

async def main() -> Awaitable[None]:

  http1_opts: Http1Opts = {
    "allow_empty_files": True,
    "block_size_kb": 64,
    "charset": "utf-8",
    "compression": True,
    "keep_alive": 30,
    "keep_extensions": True,
    "max_fields": 60,
    "max_fields_size_total_mb": 1,
    "max_files": 3,
    "max_files_size_total_mb": 60,
    "max_file_size_mb": 60,
    "port": 4433,
    "storage_path": "/var/www/cpp/storage",
    "thread_limit": 4,
  }

  http1: Http1 = Http1(http1_opts)
  async def http1_logger(_level: str, message: str) -> Awaitable[None]:
    print("[Arnelify Server]: " + message)

  http1.logger(http1_logger)
  async def http1_handler(ctx: Http1Ctx, stream: Http1Stream) -> Awaitable[None]:
    await stream.set_code(200)
    await stream.push_json(ctx)
    await stream.end()

  http1.on("/", http1_handler)
  await http1.start()
  
if __name__ == "__main__":
    asyncio.run(main())
```

## ⚖️ MIT License
This software is licensed under the <a href="https://github.com/arnelify/arnelify-server-python/blob/main/LICENSE">MIT License</a>. The original author's name, logo, and the original name of the software must be included in all copies or substantial portions of the software.

## 🛠️ Contributing
Join us to help improve this software, fix bugs or implement new functionality. Active participation will help keep the software up-to-date, reliable, and aligned with the needs of its users.

Run in terminal:
```bash
docker compose up -d --build
docker ps
docker exec -it <CONTAINER ID> bash
make install
source venv/bin/activate
make build
```
For TCP2 / WebTransport:
```bash
make test_wt
```
For TCP2 / HTTP 3.0:
```bash
make test_http3
```
For TCP1 / WebSocket:
```bash
make test_ws
```
For TCP1 / HTTP 2.0:
```bash
make test_http2
```
For TCP1 / HTTP 1.1:
```bash
make test_http1
```

## ⭐ Release Notes
Version 1.0.8 — a multi-language server with HTTP 3.0 and WebTransport support.

We are excited to introduce the Arnelify Server for Python! Please note that this version is raw and still in active development.

Change log:

* HTTP 3.0 + WebTransport.
* Security-aware logging with attack detection.
* Async Runtime & Multi-Threading.
* Large file upload and download support.
* BROTLI compression (still in development).
* FFI, PYO3 and NEON support.
* Significant refactoring and optimizations.

Please use this version with caution, as it may contain bugs and unfinished features. We are actively working on improving and expanding the server's capabilities, and we welcome your feedback and suggestions.

## 🔗 Links

* <a href="https://github.com/arnelify/arnelify-pod-cpp">Arnelify POD for C++</a>
* <a href="https://github.com/arnelify/arnelify-pod-node">Arnelify POD for NodeJS</a>
* <a href="https://github.com/arnelify/arnelify-pod-python">Arnelify POD for Python</a>
* <a href="https://github.com/arnelify/arnelify-pod-rust">Arnelify POD for Rust</a>
* <a href="https://github.com/arnelify/arnelify-react-native">Arnelify React Native</a>