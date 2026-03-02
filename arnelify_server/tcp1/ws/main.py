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

from typing import Any, Callable, Dict, TypedDict
import json
import signal
import sys

class WebSocketStream:
  id: int = 0

  def __init__(self, id):
    self.id = id

  def close(self):
    native.ws_close(self.id)

  def push(self, data: Dict[str, Any], bytes: bytes | bytearray):
    native.ws_push(self.id, json.dumps(data, separators=(',', ':')), bytes)

  def push_bytes(self, bytes: bytes | bytearray):
    native.ws_push_bytes(self.id, bytes)

  def push_json(self, data: Dict[str, Any]):
    native.ws_push_json(self.id, json.dumps(data, separators=(',', ':')))

  def set_compression(self, compression: str | None):
    native.ws_set_compression(self.id, "" if not compression else compression)

type WebSocketBytes = bytes | bytearray
type WebSocketCtx = Dict[str, Any]
type WebSocketHandler = Callable[[WebSocketCtx, WebSocketBytes, WebSocketStream], None]
type WebSocketLogger = Callable[[str, str], None]

class WebSocketOpts(TypedDict, total=True):
  block_size_kb: int
  compression: bool
  handshake_timeout: int
  max_message_size_kb: int
  ping_timeout: int
  port: int
  send_timeout: int
  thread_limit: int

class WebSocket:
  id: int = 0

  def __init__(self, opts):
    self.opts = opts
    self.id = native.ws_create(json.dumps(opts, separators=(',', ':')))

  def logger(self, cb: WebSocketLogger):
    native.ws_logger(self.id, cb)

  def on(self, path: str, cb: WebSocketHandler):
    def handler_adapter(stream_id: int, ctx: str, bytes: bytes | bytearray):
      stream = WebSocketStream(stream_id)
      cb(json.loads(ctx), bytes, stream)

    native.ws_on(self.id, path, handler_adapter)

  def start(self):
    try:
      native.ws_start(self.id)
      signal.pause()
    except KeyboardInterrupt:
      sys.exit(0)
  
  def stop(self):
    native.ws_stop(self.id)
  
  def __del__(self):
    native.ws_destroy(self.id)