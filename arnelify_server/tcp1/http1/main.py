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

class Http1Stream:
  id: int = 0

  def __init__(self, id):
    self.id = id

  def add_header(self, key: str, value: str):
    native.http1_add_header(self.id, key, value)

  def end(self):
    native.http1_end(self.id)

  def push_bytes(self, bytes: bytes | bytearray, is_attachment: bool = False):
    native.http1_push_bytes(self.id, bytes, int(is_attachment))

  def push_file(self, file_path: str, is_attachment: bool = False):
    native.http1_push_file(self.id, file_path, int(is_attachment))

  def push_json(self, data: Dict[str, Any], is_attachment: bool = False):
    native.http1_push_json(self.id, json.dumps(data, separators=(',', ':')), int(is_attachment))

  def set_code(self, code: int):
    native.http1_set_code(self.id, code)

  def set_compression(self, compression: str | None):
    native.http1_set_compression(self.id, "" if not compression else compression)

  def set_headers(self, headers: list[Dict[str, str]]):
    native.http1_set_headers(self.id, json.dumps(headers, separators=(',', ':')))

type Http1Ctx = Dict[str, Any]
type Http1Handler = Callable[[Http1Ctx, Http1Stream], None]
type Http1Logger = Callable[[str, str], None]

class Http1Opts(TypedDict, total=True):
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

class Http1:
  id: int = 0

  def __init__(self, opts):
    self.opts = opts
    self.id = native.http1_create(json.dumps(opts, separators=(',', ':')))

  def logger(self, cb: Http1Logger):
    native.http1_logger(self.id, cb)

  def on(self, path: str, cb: Http1Handler):
    def handler_adapter(stream_id: int, ctx: str):
      stream = Http1Stream(stream_id)
      cb(json.loads(ctx), stream)

    native.http1_on(self.id, path, handler_adapter)

  def start(self):
    try:
      native.http1_start(self.id)
      signal.pause()
    except KeyboardInterrupt:
      sys.exit(0)
  
  def stop(self):
    native.http1_stop(self.id)
  
  def __del__(self):
    native.http1_destroy(self.id)