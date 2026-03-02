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

class Http3Stream:
  id: int = 0

  def __init__(self, id):
    self.id = id

  def add_header(self, key: str, value: str):
    native.http3_add_header(self.id, key, value)

  def end(self):
    native.http3_end(self.id)

  def push_bytes(self, bytes: bytes | bytearray, is_attachment: bool = False):
    native.http3_push_bytes(self.id, bytes, int(is_attachment))

  def push_file(self, file_path: str, is_attachment: bool = False):
    native.http3_push_file(self.id, file_path, int(is_attachment))

  def push_json(self, data: Dict[str, Any], is_attachment: bool = False):
    native.http3_push_json(self.id, json.dumps(data, separators=(',', ':')), int(is_attachment))

  def set_code(self, code: int):
    native.http3_set_code(self.id, code)

  def set_compression(self, compression: str | None):
    native.http3_set_compression(self.id, "" if not compression else compression)

  def set_headers(self, headers: list[Dict[str, str]]):
    native.http3_set_headers(self.id, json.dumps(headers, separators=(',', ':')))

type Http3Ctx = Dict[str, Any]
type Http3Handler = Callable[[Http3Ctx, Http3Stream], None]
type Http3Logger = Callable[[str, str], None]

class Http3Opts(TypedDict, total=True):
    allow_empty_files: bool
    block_size_kb: int
    cert_pem: str
    charset: str
    compression: bool
    keep_alive: int
    keep_extensions: bool
    key_pem: str
    max_fields: int
    max_fields_size_total_mb: int
    max_files: int
    max_files_size_total_mb: int
    max_file_size_mb: int
    port: int
    storage_path: str
    thread_limit: int

class Http3:
  id: int = 0

  def __init__(self, opts):
    self.opts = opts
    self.id = native.http3_create(json.dumps(opts, separators=(',', ':')))

  def logger(self, cb: Http3Logger):
    native.http3_logger(self.id, cb)

  def on(self, path: str, cb: Http3Handler):
    def handler_adapter(stream_id: int, ctx: str):
      stream = Http3Stream(stream_id)
      cb(json.loads(ctx), stream)

    native.http3_on(self.id, path, handler_adapter)

  def start(self):
    try:
      native.http3_start(self.id)
      signal.pause()
    except KeyboardInterrupt:
      sys.exit(0)
  
  def stop(self):
    native.http3_stop(self.id)
  
  def __del__(self):
    native.http3_destroy(self.id)