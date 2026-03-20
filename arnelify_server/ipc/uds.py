import asyncio
import json
from typing import Any, Awaitable, Callable, Dict, List, Optional, TypedDict

type UnixDomainSocketBytes = bytes
type UnixDomainSocketCtx = List[Any]

class UnixDomainSocketOpts(TypedDict, total=True):
 block_size_kb: int
 socket_path: str
 thread_limit: int

class UnixDomainSocketReq:
  def __init__(self, opts: UnixDomainSocketOpts):
    self.opts: UnixDomainSocketOpts = opts
    self.has_meta: bool = False
    self.has_body: bool = False
    self.json_length: int = 0
    self.binary_length: int = 0
    self.topic: str = ''
    self.buff: bytes = b''
    self.binary: bytes = b''
    self.ctx: UnixDomainSocketCtx = []

  def add(self, buff: bytes) -> None:
    self.buff += buff

  def get_ctx(self) -> UnixDomainSocketCtx:
    return self.ctx

  def get_bytes(self) -> bytes | bytearray:
    return self.binary

  def get_topic(self) -> str:
    return self.topic

  def is_empty(self) -> bool:
    return len(self.buff) == 0

  def read_meta(self, meta_end: int) -> str | int:
    meta_bytes: bytes | bytearray = self.buff[:meta_end]
    pos: int = meta_bytes.find(b'+')
    if pos == -1:
      return "Missing '+' in meta"

    try:
      json_length: int = int(meta_bytes[:pos])
      binary_length: int = int(meta_bytes[pos + 1:])
    except ValueError:
      return "Invalid meta."

    self.json_length = json_length
    self.binary_length = binary_length
    return 1

  def read_body(self) -> Optional[str | int]:
    if not self.has_meta:
      meta_end: int = self.buff.find(b':')
      if meta_end == -1:
        if len(self.buff) > 8192:
          self.buff = b''
          return "The maximum size of the meta has been exceeded."
        return None

      res: str | int = self.read_meta(meta_end)
      if isinstance(res, str):
        return res

      self.has_meta = True
      self.buff = self.buff[meta_end + 1:]

    if self.json_length != 0 and len(self.buff) >= self.json_length:
      raw: str = self.buff[:self.json_length].decode('utf-8')
      
      try:
        json_: dict = json.loads(raw)
      except json.JSONDecodeError:
        self.buff = b''
        return "Invalid JSON."

      if 'topic' not in json_ or 'payload' not in json_:
        self.buff = b''
        return "Invalid message."

      self.topic = json_['topic']
      self.ctx = json_['payload']
      self.buff = self.buff[self.json_length:]
      if self.binary_length == 0:
        self.has_body = True
        return 1

    if self.binary_length != 0 and len(self.buff) >= self.binary_length:
      self.binary = self.buff[:self.binary_length]
      self.buff = self.buff[self.binary_length:]
      self.has_body = True
      return 1

    return None

  def read_block(self) -> Optional[str | int]:
    if not self.has_body:
      res: Optional[str | int] = self.read_body()
      if isinstance(res, str):
        return res
      if res is None:
        return None
    return 1

  def reset(self) -> None:
    self.has_meta = False
    self.has_body = False
    self.topic = ""
    self.binary = b''
    self.json_length = 0
    self.binary_length = 0
    self.ctx = []

class UnixDomainSocketStream:
  def __init__(self, opts: UnixDomainSocketOpts):
    self.opts = opts
    self.topic: Optional[str] = None
    self.cb_send: Callable[[bytes], Awaitable[None]] = lambda b: print(b)

  def on_send(self, cb: Callable[[bytes], Awaitable[None]]) -> None:
    self.cb_send = cb

  async def push(self, payload: dict, bytes_: bytes | bytearray = None) -> None:
    json_ = json.dumps({
      'topic': self.topic,
      'payload': payload
    }).encode('utf-8')

    bytes_ = bytes_ or b''
    meta_ = f"{len(json_)}+{len(bytes_)}:".encode('utf-8')
    buff = meta_ + json_ + bytes_
    await self.cb_send(buff)

  def set_topic(self, topic: str) -> None:
    self.topic = topic

type UnixDomainSocketHandler = Callable[[UnixDomainSocketCtx, UnixDomainSocketBytes], Awaitable [None]]
type UnixDomainSocketLogger = Callable[[str, str], Awaitable[None]]

class UnixDomainSocket:
  def __init__(self, opts: UnixDomainSocketOpts):
    self.opts: UnixDomainSocketOpts = opts
    self.reader: Optional[asyncio.StreamReader] = None
    self.writer: Optional[asyncio.StreamWriter] = None
    self.cb_logger: UnixDomainSocketLogger = lambda _level, msg: print(msg)
    self.cb_handlers: Dict[str, UnixDomainSocketHandler] = {}

  async def acceptor(self) -> None:
    req: UnixDomainSocketReq = UnixDomainSocketReq(self.opts)
    stream: UnixDomainSocketStream = UnixDomainSocketStream(self.opts)
        
    async def send_handler(buff: bytes) -> None:
      self.writer.write(buff)
      await self.writer.drain()
    stream.on_send(send_handler)

    block_size: int = self.opts["block_size_kb"] * 1024

    while True:
      try:
        bytes_ = await self.reader.read(block_size)
        if not bytes_:
          await self.cb_logger('success', f"Connection closed")
          break

        req.add(bytes_)

        while True:
          res = req.read_block()
          if res == 1:
            topic = req.get_topic()
            bytes_ = req.get_bytes()
            ctx = req.get_ctx()
            stream.set_topic(topic)
            req.reset()

            handler = self.cb_handlers.get(topic)
            if handler:
              asyncio.create_task(handler(ctx, bytes_))
          elif res is None:
            break
          elif isinstance(res, str):
            await self.cb_logger('error', res)
            break
          if req.is_empty():
            break
        
      except Exception as e:
        await self.cb_logger('error', f"Connection error: {e}")
        break
            
  def logger(self, cb: UnixDomainSocketLogger) -> None:
    self.cb_logger = cb

  def on(self, topic: str, cb: UnixDomainSocketHandler) -> None:
    self.cb_handlers[topic] = cb

  async def push(self, topic: str, payload: Any, bytes_: bytes | bytearray = b'') -> Awaitable[None]:
    if not self.writer:
      await self.cb_logger("error", f"No client connected for push: {topic}")
      return
    
    json_: bytes | bytearray = json.dumps({'topic': topic, 'payload': payload}).encode('utf-8')
    meta_: bytes | bytearray = f"{len(json_)}+{len(bytes_)}:".encode('utf-8')
    self.writer.write(meta_ + json_ + bytes_)
    await self.writer.drain()

  async def start(self) -> Awaitable[None]:
    try:
      self.reader, self.writer = await asyncio.open_unix_connection(self.opts["socket_path"])
    except Exception as e:
      await self.cb_logger("error", f"Connection error: {e}")
      return
    
    asyncio.create_task(self.acceptor())

  async def stop(self) -> Awaitable[None]:
    if self.writer:
      self.writer.close()
      await self.writer.wait_closed()