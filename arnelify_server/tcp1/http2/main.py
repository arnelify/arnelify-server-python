# MIT LICENSE

# COPYRIGHT (R) 2025 ARNELIFY. AUTHOR: TARON SARKISYAN

# PERMISSION IS HEREBY GRANTED, FREE OF CHARGE, TO ANY PERSON OBTAINING A COPY
# OF THIS SOFTWARE AND ASSOCIATED DOCUMENTATION FILES (THE "SOFTWARE"), TO DEAL
# IN THE SOFTWARE WITHOUT RESTRICTION, INCLUDING WITHOUT LIMITATION THE RIGHTS
# TO USE, COPY, MODIFY, MERGE, PUBLISH, DISTRIBUTE, SUBLICENSE, AND/OR SELL
# COPIES OF THE SOFTWARE, AND TO PERMIT PERSONS TO WHOM THE SOFTWARE IS
# FURNISHED TO DO SO, SUBJECT TO THE FOLLOWING CONDITIONS:

# THE ABOVE COPYRIGHT NOTICE AND THIS PERMISSION NOTICE SHALL BE INCLUDED IN ALL
# COPIES OR SUBSTANTIAL PORTIONS OF THE SOFTWARE.

# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.

from ... import arnelify_server as native

from ...ipc.uds import UnixDomainSocket
from ...ipc.uds import UnixDomainSocketBytes
from ...ipc.uds import UnixDomainSocketCtx
from ...ipc.uds import UnixDomainSocketOpts

import asyncio
from typing import Any, Awaitable, Callable, Dict, Optional, TypedDict, List
import json

class Http2Stream:
  def __init__(self, id: int):
    self.id: int = id
    self.topic: str = ""
    self.cb_send: Callable[[str, List[Any], bytes | bytearray], Awaitable[None]] = \
      lambda _topic, _args, bytes_: print(bytes_)

  async def add_header(self, key: str, value: str) -> Awaitable[None]:
    args: List[Any] = [self.id, key, value]
    await self.cb_send("http2_add_header", args, b"")

  async def end(self) -> Awaitable[None]:
    args: List[Any] = [self.id]
    await self.cb_send("http2_end", args, b"")
  
  def on_send(self, cb: Callable[[str, List[Any], bytes | bytearray], Awaitable[None]]) -> None:
    self.cb_send = cb

  async def push_bytes(self, bytes_: bytes | bytearray, is_attachment: bool = False) -> Awaitable[None]:
    args: List[Any] = [self.id, int(is_attachment)]
    await self.cb_send("http2_push_bytes", args, bytes_)

  async def push_file(self, file_path: str, is_attachment: bool = False) -> Awaitable[None]:
    args: List[Any] = [self.id, file_path, int(is_attachment)]
    await self.cb_send("http2_push_file", args, b"")

  async def push_json(self, payload: Any, is_attachment: bool = False) -> Awaitable[None]:
    args: List[Any] = [self.id, payload, int(is_attachment)]
    await self.cb_send("http2_push_json", args, b"")

  async def set_code(self, code: int) -> Awaitable[None]:
    args: List[Any] = [self.id, code]
    await self.cb_send("http2_set_code", args, b"")

  async def set_compression(self, compression: Optional[str]) -> Awaitable[None]:
    args: List[Any] = [self.id, compression if compression else ""]
    await self.cb_send("http2_set_compression", args, b"")

  async def set_headers(self, headers: List[Dict[str, str]]) -> Awaitable[None]:
    args = [self.id, headers]
    await self.cb_send("http2_set_headers", args, b"")

type Http2Ctx = Dict[str, Any]
type Http2Handler = Callable[[Http2Ctx, Http2Stream], Awaitable[None]]
type Http2Logger = Callable[[str, str], Awaitable[None]]

class Http2Opts(TypedDict, total=True):
    allow_empty_files: bool
    block_size_kb: int
    charset: str
    compression: bool
    keep_alive: int
    keep_extensions: bool
    max_fields: int
    max_fields_size_total_mb: int
    max_files: int
    max_files_size_total_mb: int
    max_file_size_mb: int
    port: int
    storage_path: str
    thread_limit: int

class Http2:
  def __init__(self, opts: Dict[str, Any]):
    self.id: int = 0
    self.socket_path: str = "/var/run/arnelify_server.sock"
    self.handlers: Dict[str, Http2Handler] = {}
    self.uds: UnixDomainSocket
    self.opts = opts

    uds_opts: UnixDomainSocketOpts = {
      'block_size_kb': opts.get('block_size_kb'),
      'socket_path': self.socket_path,
      'thread_limit': opts.get('thread_limit')
    }

    self.uds = UnixDomainSocket(uds_opts)
    self.id = native.http2_create(json.dumps({
      "socket_path": self.socket_path,
      **self.opts,
    }, separators=(',', ':')))

  def logger(self, cb: Http2Logger) -> None:
    async def logger_adapter(ctx: UnixDomainSocketCtx, bytes_: UnixDomainSocketBytes) -> Awaitable[None]:
      level, message = ctx
      await cb(level, message)

    self.uds.on("http2_logger", logger_adapter)
    native.http2_logger(self.id)

  def on(self, path: str, cb: Http2Handler) -> None:
    self.handlers[path] = cb

    async def handler_adapter(ctx: List[Any], bytes_: bytes | bytearray) -> Awaitable[None]:
      stream_id, handler_path, handler_ctx = ctx

      stream: Http2Stream = Http2Stream(stream_id)
      async def stream_handler(topic: str, args: List[Any], bytes_: bytes | bytearray) -> Awaitable[None]:
        await self.uds.push(topic, args, bytes_)
      stream.on_send(stream_handler)

      handler = self.handlers.get(handler_path)
      if handler:
        await handler(handler_ctx, stream)

    self.uds.on("http2_on", handler_adapter)
    native.http2_on(self.id, path)

  async def start(self) -> Awaitable[None]:
    native.http2_start_ipc(self.id)
    await self.uds.start()
    native.http2_start(self.id)

    try:
      await asyncio.Event().wait()
    except (KeyboardInterrupt, asyncio.CancelledError):
      pass
  
  async def stop(self) -> Awaitable[None]:
    native.http2_stop(self.id)
    await self.uds.stop()
  
  def __del__(self):
    native.http2_destroy(self.id)