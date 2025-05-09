import cffi
import json
import os
import threading

from .contracts.res import Http1Res

class Http1:
  def __init__(self, opts: dict):
    srcDir: str = os.walk(os.path.abspath('venv/lib64'))
    libPaths: list[str] = []
    for root, dirs, files in srcDir:
      for file in files:
        if file.startswith('arnelify-server') and file.endswith('.so'):
          libPaths.append(os.path.join(root, file))

    self.ffi = cffi.FFI()
    self.lib = self.ffi.dlopen(libPaths[0])

    required: list[str] = [
      "SERVER_ALLOW_EMPTY_FILES",
      "SERVER_BLOCK_SIZE_KB",
      "SERVER_CHARSET",
      "SERVER_GZIP",
      "SERVER_KEEP_EXTENSIONS",
      "SERVER_MAX_FIELDS",
      "SERVER_MAX_FIELDS_SIZE_TOTAL_MB",
      "SERVER_MAX_FILES",
      "SERVER_MAX_FILES_SIZE_TOTAL_MB",
      "SERVER_MAX_FILE_SIZE_MB",
      "SERVER_NET_CHECK_FREQ_MS",
      "SERVER_PORT",
      "SERVER_QUEUE_LIMIT"
    ]

    for key in required:
      if key not in opts:
        print(f"[Arnelify Server FFI]: Python error: '{key}' is missing")
        exit(1)

    self.opts: str = json.dumps(opts, separators=(',', ':'))
    
    self.ffi.cdef("""
      typedef const char* opts;
      typedef const char* (*cHandler)(const char*);
      typedef const void (*cCallback)(const char*, const int);

      void server_http1_create(opts);
      void server_http1_destroy();
      void server_http1_handler(cHandler cHandler);
      void server_http1_start(cCallback cCallback);
      void server_http1_stop();
    """)

  def handler(self, cb: callable) -> None:
    self.callback = cb
    
  def start(self, cb: callable) -> None:
    cOpts = self.ffi.new("char[]", self.opts.encode('utf-8'))
    self.lib.server_http1_create(cOpts)

    if hasattr(self, 'callback'):
      def handlerWrapper(cSerialized):
        serialized: str = self.ffi.string(cSerialized).decode('utf-8')
        try: 
          json.loads(serialized)
        except json.JSONDecodeError as err:
          print("[Arnelify Server FFI]: Python error: The Request must be a valid JSON.")
          exit(1)

        req: dict = json.loads(serialized)
        res: Http1Res = Http1Res()
        self.callback(req, res)
        serialized: str = res.serialize()
        return self.ffi.new("char[]", serialized.encode('utf-8'))

      self.cHandler = self.ffi.callback("const char* (const char*)", handlerWrapper)
      self.lib.server_http1_handler(self.cHandler)

    def callbackWrapper(cMessage, isError):
      message: str = self.ffi.string(cMessage).decode('utf-8')
      if isError:
        cb(message, True)
        return
      cb(message, False)
    
    cCallback = self.ffi.callback("const void (const char*, const int)", callbackWrapper)
    def server_thread():
        self.lib.server_http1_start(cCallback)

    thread = threading.Thread(target=server_thread)
    thread.daemon = True
    thread.start()
    
    try:
      thread.join()
    except KeyboardInterrupt:
      exit(1)
  
  def stop(self) -> None:
    self.lib.server_http1_stop()