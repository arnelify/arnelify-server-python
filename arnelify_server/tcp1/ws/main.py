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

class WebSocketStream:
  def __init__(self, id: int):
    self.id: int = id
    self.topic: str = ""
    self.cb_send: Callable[[str, List[Any], bytes | bytearray], Awaitable[None]] = \
      lambda _topic, _args, bytes_: print(bytes_)

  async def close(self) -> Awaitable[None]:
    args: List[Any] = [self.id]
    await self.cb_send("ws_close", args, b"")

  def on_send(self, cb: Callable[[str, List[Any], bytes], Awaitable[None]]) -> None:
    self.cb_send = cb

  async def push(self, payload: Any, bytes_: bytes | bytearray) -> Awaitable[None]:
    args = [self.id, payload]
    await self.cb_send("ws_push", args, bytes_)

  async def push_bytes(self, bytes_: bytes) -> Awaitable[None]:
    args = [self.id]
    await self.cb_send("ws_push_bytes", args, bytes_)

  async def push_json(self, payload: Dict[str, Any]) -> Awaitable[None]:
    args = [self.id, payload]
    await self.cb_send("ws_push_json", args, b"")

  async def set_compression(self, compression: Optional[str]) -> Awaitable[None]:
    args = [self.id, compression if compression else ""]
    await self.cb_send("ws_set_compression", args, b"")

type WebSocketBytes = bytes | bytearray
type WebSocketCtx = Dict[str, Any]
type WebSocketHandler = Callable[[WebSocketCtx, WebSocketBytes, WebSocketStream], Awaitable[None]]
type WebSocketLogger = Callable[[str, str], Awaitable[None]]

class WebSocketOpts(TypedDict, total=True):
  block_size_kb: int
  compression: bool
  max_message_size_kb: int
  ping_timeout: int
  port: int
  rate_limit: int
  read_timeout: int
  send_timeout: int
  thread_limit: int

class WebSocket:
  def __init__(self, opts: Dict[str, Any]):
    self.id: int = 0
    self.socket_path: str = "/var/run/arnelify_server.sock"
    self.handlers: Dict[str, WebSocketHandler] = {}
    self.uds: UnixDomainSocket
    self.opts = opts

    uds_opts: UnixDomainSocketOpts = {
      'block_size_kb': opts.get('block_size_kb'),
      'socket_path': self.socket_path,
      'thread_limit': opts.get('thread_limit')
    }

    self.uds = UnixDomainSocket(uds_opts)
    self.id = native.ws_create(json.dumps({
      "socket_path": self.socket_path,
      **self.opts,
    }, separators=(',', ':')))

  def logger(self, cb: WebSocketLogger) -> None:
    async def logger_adapter(ctx: UnixDomainSocketCtx, bytes_: UnixDomainSocketBytes) -> Awaitable[None]:
      level, message = ctx
      await cb(level, message)

    self.uds.on("ws_logger", logger_adapter)
    native.ws_logger(self.id)

  def on(self, path: str, cb: WebSocketHandler) -> None:
    self.handlers[path] = cb

    async def handler_adapter(ctx: List[Any], bytes_: bytes | bytearray) -> Awaitable[None]:
      stream_id, handler_path, handler_ctx = ctx

      stream: WebSocketStream = WebSocketStream(stream_id)
      async def stream_handler(topic: str, args: List[Any], bytes_: bytes | bytearray) -> Awaitable[None]:
        await self.uds.push(topic, args, bytes_)
      stream.on_send(stream_handler)

      handler = self.handlers.get(handler_path)
      if handler:
        await handler(handler_ctx, bytes_, stream)

    self.uds.on("ws_on", handler_adapter)
    native.ws_on(self.id, path)

  async def start(self) -> Awaitable[None]:
    native.ws_start_ipc(self.id)
    await self.uds.start()
    native.ws_start(self.id)
    
    try:
      await asyncio.Event().wait()
    except (KeyboardInterrupt, asyncio.CancelledError):
      pass
  
  async def stop(self) -> Awaitable[None]:
    native.ws_stop(self.id)
    await self.uds.stop()
  
  def __del__(self):
    native.ws_destroy(self.id)