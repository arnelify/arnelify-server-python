// MIT LICENSE
//
// COPYRIGHT (R) 2025 ARNELIFY. AUTHOR: TARON SARKISYAN
//
// PERMISSION IS HEREBY GRANTED, FREE OF CHARGE, TO ANY PERSON OBTAINING A COPY
// OF THIS SOFTWARE AND ASSOCIATED DOCUMENTATION FILES (THE "SOFTWARE"), TO DEAL
// IN THE SOFTWARE WITHOUT RESTRICTION, INCLUDING WITHOUT LIMITATION THE RIGHTS
// TO USE, COPY, MODIFY, MERGE, PUBLISH, DISTRIBUTE, SUBLICENSE, AND/OR SELL
// COPIES OF THE SOFTWARE, AND TO PERMIT PERSONS TO WHOM THE SOFTWARE IS
// FURNISHED TO DO SO, SUBJECT TO THE FOLLOWING CONDITIONS:
//
// THE ABOVE COPYRIGHT NOTICE AND THIS PERMISSION NOTICE SHALL BE INCLUDED IN ALL
// COPIES OR SUBSTANTIAL PORTIONS OF THE SOFTWARE.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use pyo3::prelude::*;

mod ipc;
mod tcp1;
mod tcp2;

#[pymodule]
mod arnelify_server {
  use pyo3::prelude::*;

  use crate::ipc::{
    UnixDomainSocket, UnixDomainSocketBytes, UnixDomainSocketCtx, UnixDomainSocketHandler,
    UnixDomainSocketOpts,
  };

  use crate::tcp1::{Http1, Http1Ctx, Http1Handler, Http1Logger, Http1Opts, Http1Stream};
  use crate::tcp1::{Http2, Http2Ctx, Http2Handler, Http2Logger, Http2Opts, Http2Stream};
  use crate::tcp1::{
    WebSocket, WebSocketBytes, WebSocketCtx, WebSocketHandler, WebSocketLogger, WebSocketOpts,
    WebSocketStream,
  };

  use crate::tcp2::{Http3, Http3Ctx, Http3Handler, Http3Logger, Http3Opts, Http3Stream};
  use crate::tcp2::{
    WebTransport, WebTransportBytes, WebTransportCtx, WebTransportHandler, WebTransportLogger,
    WebTransportOpts, WebTransportStream,
  };

  use std::{
    collections::HashMap,
    convert::TryFrom,
    sync::{
      Arc, Mutex, MutexGuard, OnceLock,
      atomic::{AtomicUsize, Ordering},
      mpsc,
      mpsc::RecvTimeoutError,
    },
    thread,
    time::Duration,
  };

  use serde_json::Value as JSON;

  type Http1Streams = HashMap<usize, (Arc<Mutex<Http1Stream>>, mpsc::Sender<u8>)>;
  static HTTP1_MAP: OnceLock<Mutex<HashMap<usize, Arc<Http1>>>> = OnceLock::new();
  static HTTP1_ID: OnceLock<Mutex<usize>> = OnceLock::new();
  static HTTP1_STREAM_ID: AtomicUsize = AtomicUsize::new(1);
  static HTTP1_STREAMS: OnceLock<Mutex<Http1Streams>> = OnceLock::new();
  static HTTP1_UDS_MAP: OnceLock<Mutex<HashMap<usize, Arc<UnixDomainSocket>>>> = OnceLock::new();

  type Http2Streams = HashMap<usize, (Arc<Mutex<Http2Stream>>, mpsc::Sender<u8>)>;
  static HTTP2_MAP: OnceLock<Mutex<HashMap<usize, Arc<Http2>>>> = OnceLock::new();
  static HTTP2_ID: OnceLock<Mutex<usize>> = OnceLock::new();
  static HTTP2_STREAM_ID: AtomicUsize = AtomicUsize::new(1);
  static HTTP2_STREAMS: OnceLock<Mutex<Http2Streams>> = OnceLock::new();
  static HTTP2_UDS_MAP: OnceLock<Mutex<HashMap<usize, Arc<UnixDomainSocket>>>> = OnceLock::new();

  type WebSocketStreams = HashMap<usize, (Arc<Mutex<WebSocketStream>>, mpsc::Sender<u8>)>;
  static WS_MAP: OnceLock<Mutex<HashMap<usize, Arc<WebSocket>>>> = OnceLock::new();
  static WS_ID: OnceLock<Mutex<usize>> = OnceLock::new();
  static WS_STREAM_ID: AtomicUsize = AtomicUsize::new(1);
  static WS_STREAMS: OnceLock<Mutex<WebSocketStreams>> = OnceLock::new();
  static WS_UDS_MAP: OnceLock<Mutex<HashMap<usize, Arc<UnixDomainSocket>>>> = OnceLock::new();

  type Http3Streams = HashMap<usize, (Arc<Mutex<Http3Stream>>, mpsc::Sender<u8>)>;
  static HTTP3_MAP: OnceLock<Mutex<HashMap<usize, Arc<Http3>>>> = OnceLock::new();
  static HTTP3_ID: OnceLock<Mutex<usize>> = OnceLock::new();
  static HTTP3_STREAM_ID: AtomicUsize = AtomicUsize::new(1);
  static HTTP3_STREAMS: OnceLock<Mutex<Http3Streams>> = OnceLock::new();
  static HTTP3_UDS_MAP: OnceLock<Mutex<HashMap<usize, Arc<UnixDomainSocket>>>> = OnceLock::new();

  type WebTransportStreams = HashMap<usize, (Arc<Mutex<WebTransportStream>>, mpsc::Sender<u8>)>;
  static WT_MAP: OnceLock<Mutex<HashMap<usize, Arc<WebTransport>>>> = OnceLock::new();
  static WT_ID: OnceLock<Mutex<usize>> = OnceLock::new();
  static WT_STREAM_ID: AtomicUsize = AtomicUsize::new(1);
  static WT_STREAMS: OnceLock<Mutex<WebTransportStreams>> = OnceLock::new();
  static WT_UDS_MAP: OnceLock<Mutex<HashMap<usize, Arc<UnixDomainSocket>>>> = OnceLock::new();

  fn get_str(opts: &JSON, key: &str) -> String {
    opts
      .get(key)
      .and_then(JSON::as_str)
      .expect(&format!(
        "[Arnelify Server]: Rust PYO3 error: '{}' missing or not a string.",
        key
      ))
      .to_string()
  }

  fn get_u64(opts: &JSON, key: &str) -> u64 {
    opts.get(key).and_then(JSON::as_u64).expect(&format!(
      "[Arnelify Server]: Rust PYO3 error: '{}' missing or not a u64.",
      key
    ))
  }

  fn get_usize(opts: &JSON, key: &str) -> usize {
    let val: u64 = get_u64(opts, key);
    usize::try_from(val).expect(&format!(
      "[Arnelify Server]: Rust PYO3 error: '{}' out of usize range.",
      key
    ))
  }

  fn get_u32(opts: &JSON, key: &str) -> u32 {
    let val: u64 = get_u64(opts, key);
    u32::try_from(val).expect(&format!(
      "[Arnelify Server]: Rust PYO3 error: '{}' out of u32 range.",
      key
    ))
  }

  fn get_u16(opts: &JSON, key: &str) -> u16 {
    let val: u64 = get_u64(opts, key);
    u16::try_from(val).expect(&format!(
      "[Arnelify Server]: Rust PYO3 error: '{}' out of u16 range.",
      key
    ))
  }

  fn get_u8(opts: &JSON, key: &str) -> u8 {
    let val: u64 = get_u64(opts, key);
    u8::try_from(val).expect(&format!(
      "[Arnelify Server]: Rust PYO3 error: '{}' out of u8 range.",
      key
    ))
  }

  fn get_bool(opts: &JSON, key: &str) -> bool {
    opts.get(key).and_then(JSON::as_bool).expect(&format!(
      "[Arnelify Server]: Rust PYO3 error: '{}' missing or not a bool.",
      key
    ))
  }

  #[pyfunction]
  fn http1_create(py_opts: &str) -> PyResult<usize> {
    let opts: JSON = {
      match serde_json::from_str(py_opts) {
        Ok(json) => json,
        Err(_) => {
          println!(
            "[Arnelify Server]: Rust PYO3 error in http1_create: Invalid JSON in 'py_opts'."
          );
          return Ok(0);
        }
      }
    };

    let id: &Mutex<usize> = HTTP1_ID.get_or_init(|| Mutex::new(0));
    let new_id: usize = {
      let mut py: MutexGuard<'_, usize> = id.lock().unwrap();
      *py += 1;
      *py
    };

    let uds_opts: UnixDomainSocketOpts = UnixDomainSocketOpts {
      block_size_kb: get_usize(&opts, "block_size_kb"),
      socket_path: get_str(&opts, "socket_path"),
      thread_limit: get_u64(&opts, "thread_limit"),
    };

    let uds_http1_add_header: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let key: String = v[1].as_str().unwrap_or("").to_string();
            let value: String = v[2].as_str().unwrap_or("").to_string();

            if let Some(map) = HTTP1_STREAMS.get() {
              let stream: Option<(Arc<Mutex<Http1Stream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, Http1Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http1Stream> = stream_arc.lock().unwrap();
                  stream_lock.add_header(&key, &value);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_http1_add_header: No stream found."
                  ]);

                  if let Some(map) = HTTP1_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http1_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_http1_end: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            if let Some(map) = HTTP1_STREAMS.get() {
              let stream: Option<(Arc<Mutex<Http1Stream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, Http1Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http1Stream> = stream_arc.lock().unwrap();
                  stream_lock.end();
                  let _ = tx.send(1);
                }
                None => {
                  let args: JSON =
                    serde_json::json!(["error", "PYO3 error in uds_http1_end: No stream found."]);

                  if let Some(map) = HTTP1_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http1_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_http1_push_bytes: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>, bytes: Arc<Mutex<UnixDomainSocketBytes>>| -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let is_attachment: bool = v[1].as_u64().unwrap_or(0) != 0;
            let bytes: Vec<u8> = bytes.lock().unwrap().clone();

            if let Some(map) = HTTP1_STREAMS.get() {
              let stream: Option<(Arc<Mutex<Http1Stream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, Http1Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http1Stream> = stream_arc.lock().unwrap();
                  stream_lock.push_bytes(&bytes, is_attachment);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_http1_push_bytes: No stream found."
                  ]);

                  if let Some(map) = HTTP1_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http1_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_http1_push_file: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let file_path: &str = v[1].as_str().unwrap_or("");
            let is_attachment: bool = v[2].as_u64().unwrap_or(0) != 0;

            if let Some(map) = HTTP1_STREAMS.get() {
              let stream: Option<(Arc<Mutex<Http1Stream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, Http1Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http1Stream> = stream_arc.lock().unwrap();
                  stream_lock.push_file(&file_path, is_attachment);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_http1_push_file: No stream found."
                  ]);

                  if let Some(map) = HTTP1_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http1_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_http1_push_json: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let json: JSON = v[1].clone();
            let is_attachment: bool = v[2].as_u64().unwrap_or(0) != 0;

            if let Some(map) = HTTP1_STREAMS.get() {
              let stream: Option<(Arc<Mutex<Http1Stream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, Http1Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http1Stream> = stream_arc.lock().unwrap();
                  stream_lock.push_json(&json, is_attachment);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_http1_push_json: No stream found."
                  ]);

                  if let Some(map) = HTTP1_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http1_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_http1_set_code: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let code: u64 = v[1].as_u64().unwrap_or(0);

            if let Some(map) = HTTP1_STREAMS.get() {
              let stream: Option<(Arc<Mutex<Http1Stream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, Http1Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http1Stream> = stream_arc.lock().unwrap();
                  stream_lock.set_code(code as u16);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_http1_set_code: No stream found."
                  ]);

                  if let Some(map) = HTTP1_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http1_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_http1_set_compression: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let compression: &str = v[1].as_str().unwrap_or("");

            if let Some(map) = HTTP1_STREAMS.get() {
              let stream: Option<(Arc<Mutex<Http1Stream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, Http1Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http1Stream> = stream_arc.lock().unwrap();
                  if compression.len() > 0 {
                    stream_lock.set_compression(Some(String::from(compression)));
                    return;
                  }

                  stream_lock.set_compression(None);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_http1_set_compression: No stream found."
                  ]);

                  if let Some(map) = HTTP1_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http1_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_http1_set_headers: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let mut headers: Vec<(String, String)> = Vec::new();
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            if let Some(JSON::Object(map)) = v.get(1) {
              for (key, value) in map {
                let value = match value {
                  JSON::String(s) => s.clone(),
                  JSON::Number(n) => n.to_string(),
                  JSON::Bool(b) => b.to_string(),
                  _ => continue,
                };

                headers.push((key.clone(), value));
              }
            }

            if let Some(map) = HTTP1_STREAMS.get() {
              let stream: Option<(Arc<Mutex<Http1Stream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, Http1Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http1Stream> = stream_arc.lock().unwrap();
                  stream_lock.set_headers(headers);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_http1_set_headers: No stream found."
                  ]);

                  if let Some(map) = HTTP1_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http1_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let mut uds: UnixDomainSocket = UnixDomainSocket::new(uds_opts);
    uds.on("http1_add_header", uds_http1_add_header);
    uds.on("http1_end", uds_http1_end);
    uds.on("http1_push_bytes", uds_http1_push_bytes);
    uds.on("http1_push_file", uds_http1_push_file);
    uds.on("http1_push_json", uds_http1_push_json);
    uds.on("http1_set_code", uds_http1_set_code);
    uds.on("http1_set_compression", uds_http1_set_compression);
    uds.on("http1_set_headers", uds_http1_set_headers);

    let uds_map: &Mutex<HashMap<usize, Arc<UnixDomainSocket>>> =
      HTTP1_UDS_MAP.get_or_init(|| Mutex::new(HashMap::new()));
    {
      uds_map.lock().unwrap().insert(new_id, Arc::new(uds));
    }

    let http1_opts: Http1Opts = Http1Opts {
      allow_empty_files: get_bool(&opts, "allow_empty_files"),
      block_size_kb: get_usize(&opts, "block_size_kb"),
      charset: get_str(&opts, "charset"),
      compression: get_bool(&opts, "compression"),
      keep_alive: get_u8(&opts, "keep_alive"),
      keep_extensions: get_bool(&opts, "keep_extensions"),
      max_fields: get_u32(&opts, "max_fields"),
      max_fields_size_total_mb: get_usize(&opts, "max_fields_size_total_mb"),
      max_files: get_u32(&opts, "max_files"),
      max_files_size_total_mb: get_usize(&opts, "max_files_size_total_mb"),
      max_file_size_mb: get_usize(&opts, "max_file_size_mb"),
      port: get_u16(&opts, "port"),
      storage_path: get_str(&opts, "storage_path"),
      thread_limit: get_u64(&opts, "thread_limit"),
    };

    let http1: Http1 = Http1::new(http1_opts);
    let http1_map: &Mutex<HashMap<usize, Arc<Http1>>> =
      HTTP1_MAP.get_or_init(|| Mutex::new(HashMap::new()));
    {
      http1_map.lock().unwrap().insert(new_id, Arc::new(http1));
    }

    Ok(new_id)
  }

  #[pyfunction]
  fn http1_destroy(py_id: usize) -> PyResult<()> {
    if let Some(map) = HTTP1_MAP.get() {
      map.lock().unwrap().remove(&py_id);
    }

    if let Some(map) = HTTP1_UDS_MAP.get() {
      map.lock().unwrap().remove(&py_id);
    }

    Ok(())
  }

  #[pyfunction]
  fn http1_logger(py_id: usize) -> PyResult<()> {
    let http1_logger: Arc<Http1Logger> = Arc::new(move |level: &str, message: &str| -> () {
      let args: JSON = serde_json::json!([level, message]);
      let bytes: UnixDomainSocketBytes = Vec::new();

      if let Some(map) = HTTP1_UDS_MAP.get() {
        if let Some(uds) = map.lock().unwrap().get(&py_id) {
          uds.push("http1_logger", &args, bytes, true);
        }
      }
    });

    if let Some(map) = HTTP1_MAP.get() {
      if let Some(http1) = map.lock().unwrap().get(&py_id) {
        http1.logger(http1_logger);
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http1_on(py_id: usize, py_path: &str) -> PyResult<()> {
    let path_safe: String = String::from(py_path);

    let http1_handler: Arc<Http1Handler> = Arc::new(
      move |ctx: Arc<Mutex<Http1Ctx>>, stream: Arc<Mutex<Http1Stream>>| -> () {
        let (tx, rx) = mpsc::channel::<u8>();
        let stream_id: usize = HTTP1_STREAM_ID.fetch_add(1, Ordering::Relaxed);

        HTTP1_STREAMS
          .get_or_init(|| Mutex::new(HashMap::new()))
          .lock()
          .unwrap()
          .insert(stream_id, (stream, tx));

        let ctx: Http1Ctx = ctx.lock().unwrap().clone();
        let args: JSON = serde_json::json!([stream_id, path_safe, ctx]);
        let bytes: UnixDomainSocketBytes = Vec::new();

        if let Some(map) = HTTP1_UDS_MAP.get() {
          if let Some(uds) = map.lock().unwrap().get(&py_id) {
            uds.push("http1_on", &args, bytes, true);
          }
        }

        loop {
          match rx.recv_timeout(Duration::from_secs(90)) {
            Ok(val) => {
              if val == 1 {
                break;
              }
            }
            Err(RecvTimeoutError::Timeout) => {
              let args: JSON =
                serde_json::json!(["error", "PYO3 error in http1_on: Stream response timeout."]);
              if let Some(map) = HTTP1_UDS_MAP.get() {
                if let Some(uds) = map.lock().unwrap().get(&py_id) {
                  uds.push("http1_logger", &args, Vec::new(), true);
                }
              }
              break;
            }
            Err(RecvTimeoutError::Disconnected) => {
              let args: JSON = serde_json::json!([
                "error",
                "PYO3 error in http1_on: Stream channel disconnected."
              ]);
              if let Some(map) = HTTP1_UDS_MAP.get() {
                if let Some(uds) = map.lock().unwrap().get(&py_id) {
                  uds.push("http1_logger", &args, Vec::new(), true);
                }
              }

              break;
            }
          }
        }

        if let Some(map) = HTTP1_STREAMS.get() {
          map.lock().unwrap().remove(&stream_id);
        }
      },
    );

    if let Some(map) = HTTP1_MAP.get() {
      if let Some(http1) = map.lock().unwrap().get(&py_id) {
        http1.on(py_path, http1_handler);
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http1_start_ipc(py_id: usize) -> PyResult<()> {
    let (tx, rx) = mpsc::channel::<u8>();

    if let Some(map) = HTTP1_UDS_MAP.get() {
      if let Some(uds) = map.lock().unwrap().get(&py_id) {
        let uds_safe: Arc<UnixDomainSocket> = Arc::clone(uds);
        thread::spawn(move || {
          uds_safe.start(Arc::new(move || {
            let _ = tx.send(1);
          }));
        });
      }
    }

    loop {
      match rx.recv() {
        Ok(1) => break,
        Ok(_) => continue,
        Err(_) => break,
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http1_start(py_id: usize) -> PyResult<()> {
    if let Some(map) = HTTP1_MAP.get() {
      if let Some(http1) = map.lock().unwrap().get(&py_id) {
        let http1_safe: Arc<Http1> = Arc::clone(http1);
        thread::spawn(move || {
          http1_safe.start();
        });
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http1_stop(py_id: usize) -> PyResult<()> {
    if let Some(map) = HTTP1_UDS_MAP.get() {
      if let Some(uds) = map.lock().unwrap().get(&py_id) {
        uds.stop();
      }
    }

    if let Some(map) = HTTP1_MAP.get() {
      if let Some(http1) = map.lock().unwrap().get(&py_id) {
        http1.stop();
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http2_create(py_opts: &str) -> PyResult<usize> {
    let opts: JSON = {
      match serde_json::from_str(py_opts) {
        Ok(json) => json,
        Err(_) => {
          println!(
            "[Arnelify Server]: Rust PYO3 error in http2_create: Invalid JSON in 'py_opts'."
          );
          return Ok(0);
        }
      }
    };

    let id: &Mutex<usize> = HTTP2_ID.get_or_init(|| Mutex::new(0));
    let new_id: usize = {
      let mut py: MutexGuard<'_, usize> = id.lock().unwrap();
      *py += 1;
      *py
    };

    let uds_opts: UnixDomainSocketOpts = UnixDomainSocketOpts {
      block_size_kb: get_usize(&opts, "block_size_kb"),
      socket_path: get_str(&opts, "socket_path"),
      thread_limit: get_u64(&opts, "thread_limit"),
    };

    let uds_http2_add_header: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let key: String = v[1].as_str().unwrap_or("").to_string();
            let value: String = v[2].as_str().unwrap_or("").to_string();

            if let Some(map) = HTTP2_STREAMS.get() {
              let stream: Option<(Arc<Mutex<Http2Stream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, Http2Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http2Stream> = stream_arc.lock().unwrap();
                  stream_lock.add_header(&key, &value);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_http2_add_header: No stream found."
                  ]);

                  if let Some(map) = HTTP2_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http2_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_http2_end: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            if let Some(map) = HTTP2_STREAMS.get() {
              let stream: Option<(Arc<Mutex<Http2Stream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, Http2Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http2Stream> = stream_arc.lock().unwrap();
                  stream_lock.end();
                  let _ = tx.send(1);
                }
                None => {
                  let args: JSON =
                    serde_json::json!(["error", "PYO3 error in uds_http2_end: No stream found."]);

                  if let Some(map) = HTTP2_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http2_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_http2_push_bytes: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>, bytes: Arc<Mutex<UnixDomainSocketBytes>>| -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let is_attachment: bool = v[1].as_u64().unwrap_or(0) != 0;
            let bytes: Vec<u8> = bytes.lock().unwrap().clone();

            if let Some(map) = HTTP2_STREAMS.get() {
              let stream = {
                let streams: MutexGuard<'_, Http2Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http2Stream> = stream_arc.lock().unwrap();
                  stream_lock.push_bytes(&bytes, is_attachment);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_http2_push_bytes: No stream found."
                  ]);

                  if let Some(map) = HTTP2_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http2_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_http2_push_file: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let file_path: &str = v[1].as_str().unwrap_or("");
            let is_attachment: bool = v[2].as_u64().unwrap_or(0) != 0;

            if let Some(map) = HTTP2_STREAMS.get() {
              let stream: Option<(Arc<Mutex<Http2Stream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, Http2Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http2Stream> = stream_arc.lock().unwrap();
                  stream_lock.push_file(&file_path, is_attachment);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_http2_push_file: No stream found."
                  ]);

                  if let Some(map) = HTTP2_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http2_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_http2_push_json: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let json: JSON = v[1].clone();
            let is_attachment: bool = v[2].as_u64().unwrap_or(0) != 0;

            if let Some(map) = HTTP2_STREAMS.get() {
              let stream: Option<(Arc<Mutex<Http2Stream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, Http2Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http2Stream> = stream_arc.lock().unwrap();
                  stream_lock.push_json(&json, is_attachment);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_http2_push_json: No stream found."
                  ]);

                  if let Some(map) = HTTP2_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http2_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_http2_set_code: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let code: u64 = v[1].as_u64().unwrap_or(0);

            if let Some(map) = HTTP2_STREAMS.get() {
              let stream: Option<(Arc<Mutex<Http2Stream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, Http2Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http2Stream> = stream_arc.lock().unwrap();
                  stream_lock.set_code(code as u16);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_http2_set_code: No stream found."
                  ]);

                  if let Some(map) = HTTP2_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http2_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_http2_set_compression: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let compression: &str = v[1].as_str().unwrap_or("");

            if let Some(map) = HTTP2_STREAMS.get() {
              let stream = {
                let streams: MutexGuard<'_, Http2Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http2Stream> = stream_arc.lock().unwrap();
                  if compression.len() > 0 {
                    stream_lock.set_compression(Some(String::from(compression)));
                    return;
                  }

                  stream_lock.set_compression(None);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_http2_set_compression: No stream found."
                  ]);

                  if let Some(map) = HTTP2_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http2_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_http2_set_headers: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let mut headers: Vec<(String, String)> = Vec::new();
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;

            if let Some(JSON::Object(map)) = v.get(1) {
              for (key, value) in map {
                let value = match value {
                  JSON::String(s) => s.clone(),
                  JSON::Number(n) => n.to_string(),
                  JSON::Bool(b) => b.to_string(),
                  _ => continue,
                };

                headers.push((key.clone(), value));
              }
            }

            if let Some(map) = HTTP2_STREAMS.get() {
              let stream = {
                let streams: MutexGuard<'_, Http2Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http2Stream> = stream_arc.lock().unwrap();
                  stream_lock.set_headers(headers);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_http2_set_headers: No stream found."
                  ]);

                  if let Some(map) = HTTP2_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http2_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let mut uds: UnixDomainSocket = UnixDomainSocket::new(uds_opts);
    uds.on("http2_add_header", uds_http2_add_header);
    uds.on("http2_end", uds_http2_end);
    uds.on("http2_push_bytes", uds_http2_push_bytes);
    uds.on("http2_push_file", uds_http2_push_file);
    uds.on("http2_push_json", uds_http2_push_json);
    uds.on("http2_set_code", uds_http2_set_code);
    uds.on("http2_set_compression", uds_http2_set_compression);
    uds.on("http2_set_headers", uds_http2_set_headers);

    let uds_map: &Mutex<HashMap<usize, Arc<UnixDomainSocket>>> =
      HTTP2_UDS_MAP.get_or_init(|| Mutex::new(HashMap::new()));
    {
      uds_map.lock().unwrap().insert(new_id, Arc::new(uds));
    }

    let http2_opts: Http2Opts = Http2Opts {
      allow_empty_files: get_bool(&opts, "allow_empty_files"),
      block_size_kb: get_usize(&opts, "block_size_kb"),
      cert_pem: get_str(&opts, "cert_pem"),
      charset: get_str(&opts, "charset"),
      compression: get_bool(&opts, "compression"),
      keep_alive: get_u8(&opts, "keep_alive"),
      keep_extensions: get_bool(&opts, "keep_extensions"),
      key_pem: get_str(&opts, "key_pem"),
      max_fields: get_u32(&opts, "max_fields"),
      max_fields_size_total_mb: get_usize(&opts, "max_fields_size_total_mb"),
      max_files: get_u32(&opts, "max_files"),
      max_files_size_total_mb: get_usize(&opts, "max_files_size_total_mb"),
      max_file_size_mb: get_usize(&opts, "max_file_size_mb"),
      port: get_u16(&opts, "port"),
      storage_path: get_str(&opts, "storage_path"),
      thread_limit: get_u64(&opts, "thread_limit"),
    };

    let http2: Http2 = Http2::new(http2_opts);
    let http2_map: &Mutex<HashMap<usize, Arc<Http2>>> =
      HTTP2_MAP.get_or_init(|| Mutex::new(HashMap::new()));
    {
      http2_map.lock().unwrap().insert(new_id, Arc::new(http2));
    }

    Ok(new_id)
  }

  #[pyfunction]
  fn http2_destroy(py_id: usize) -> PyResult<()> {
    if let Some(map) = HTTP2_MAP.get() {
      map.lock().unwrap().remove(&py_id);
    }

    if let Some(map) = HTTP2_UDS_MAP.get() {
      map.lock().unwrap().remove(&py_id);
    }

    Ok(())
  }

  #[pyfunction]
  fn http2_logger(py_id: usize) -> PyResult<()> {
    let http2_logger: Arc<Http2Logger> = Arc::new(move |level: &str, message: &str| -> () {
      let args: JSON = serde_json::json!([level, message]);
      let bytes: UnixDomainSocketBytes = Vec::new();

      if let Some(map) = HTTP2_UDS_MAP.get() {
        if let Some(uds) = map.lock().unwrap().get(&py_id) {
          uds.push("http2_logger", &args, bytes, true);
        }
      }
    });

    if let Some(map) = HTTP2_MAP.get() {
      if let Some(http2) = map.lock().unwrap().get(&py_id) {
        http2.logger(http2_logger);
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http2_on(py_id: usize, py_path: &str) -> PyResult<()> {
    let path_safe: String = String::from(py_path);
    let http2_handler: Arc<Http2Handler> = Arc::new(
      move |ctx: Arc<Mutex<Http2Ctx>>, stream: Arc<Mutex<Http2Stream>>| -> () {
        let (tx, rx) = mpsc::channel::<u8>();
        let stream_id: usize = HTTP2_STREAM_ID.fetch_add(1, Ordering::Relaxed);

        HTTP2_STREAMS
          .get_or_init(|| Mutex::new(HashMap::new()))
          .lock()
          .unwrap()
          .insert(stream_id, (stream, tx));

        let ctx: Http2Ctx = ctx.lock().unwrap().clone();
        let args: JSON = serde_json::json!([stream_id, path_safe, ctx]);
        let bytes: UnixDomainSocketBytes = Vec::new();

        if let Some(map) = HTTP2_UDS_MAP.get() {
          if let Some(uds) = map.lock().unwrap().get(&py_id) {
            uds.push("http2_on", &args, bytes, true);
          }
        }

        loop {
          match rx.recv_timeout(Duration::from_secs(90)) {
            Ok(val) => {
              if val == 1 {
                break;
              }
            }
            Err(RecvTimeoutError::Timeout) => {
              let args: JSON =
                serde_json::json!(["error", "PYO3 error in http2_on: Stream response timeout."]);
              if let Some(map) = HTTP2_UDS_MAP.get() {
                if let Some(uds) = map.lock().unwrap().get(&py_id) {
                  uds.push("http2_logger", &args, Vec::new(), true);
                }
              }
              break;
            }
            Err(RecvTimeoutError::Disconnected) => {
              let args: JSON = serde_json::json!([
                "error",
                "PYO3 error in http2_on: Stream channel disconnected."
              ]);
              if let Some(map) = HTTP2_UDS_MAP.get() {
                if let Some(uds) = map.lock().unwrap().get(&py_id) {
                  uds.push("http2_logger", &args, Vec::new(), true);
                }
              }

              break;
            }
          }
        }

        if let Some(map) = HTTP2_STREAMS.get() {
          map.lock().unwrap().remove(&stream_id);
        }
      },
    );

    if let Some(map) = HTTP2_MAP.get() {
      if let Some(http2) = map.lock().unwrap().get(&py_id) {
        http2.on(py_path, http2_handler);
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http2_start_ipc(py_id: usize) -> PyResult<()> {
    let (tx, rx) = mpsc::channel::<u8>();

    if let Some(map) = HTTP2_UDS_MAP.get() {
      if let Some(uds) = map.lock().unwrap().get(&py_id) {
        let uds_safe: Arc<UnixDomainSocket> = Arc::clone(uds);
        thread::spawn(move || {
          uds_safe.start(Arc::new(move || {
            let _ = tx.send(1);
          }));
        });
      }
    }

    loop {
      match rx.recv() {
        Ok(1) => break,
        Ok(_) => continue,
        Err(_) => break,
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http2_start(py_id: usize) -> PyResult<()> {
    if let Some(map) = HTTP2_MAP.get() {
      if let Some(http2) = map.lock().unwrap().get(&py_id) {
        let http2_safe: Arc<Http2> = Arc::clone(http2);
        thread::spawn(move || {
          http2_safe.start();
        });
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http2_stop(py_id: usize) -> PyResult<()> {
    if let Some(map) = HTTP2_UDS_MAP.get() {
      if let Some(uds) = map.lock().unwrap().get(&py_id) {
        uds.stop();
      }
    }

    if let Some(map) = HTTP2_MAP.get() {
      if let Some(http2) = map.lock().unwrap().get(&py_id) {
        http2.stop();
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn ws_create(py_opts: &str) -> PyResult<usize> {
    let opts: JSON = {
      match serde_json::from_str(py_opts) {
        Ok(json) => json,
        Err(_) => {
          println!("[Arnelify Server]: Rust PYO3 error in ws_create: Invalid JSON in 'py_opts'.");
          return Ok(0);
        }
      }
    };

    let id: &Mutex<usize> = WS_ID.get_or_init(|| Mutex::new(0));
    let new_id: usize = {
      let mut py: MutexGuard<'_, usize> = id.lock().unwrap();
      *py += 1;
      *py
    };

    let uds_opts: UnixDomainSocketOpts = UnixDomainSocketOpts {
      block_size_kb: get_usize(&opts, "block_size_kb"),
      socket_path: get_str(&opts, "socket_path"),
      thread_limit: get_u64(&opts, "thread_limit"),
    };

    let uds_ws_close: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            if let Some(map) = WS_STREAMS.get() {
              let stream: Option<(Arc<Mutex<WebSocketStream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, WebSocketStreams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, tx)) => {
                  let mut stream_lock: MutexGuard<'_, WebSocketStream> = stream_arc.lock().unwrap();
                  stream_lock.close();
                  let _ = tx.send(1);
                }
                None => {
                  let args: JSON =
                    serde_json::json!(["error", "PYO3 error in uds_ws_close: No stream found."]);

                  if let Some(map) = WS_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("ws_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_ws_push: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>, bytes: Arc<Mutex<UnixDomainSocketBytes>>| -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let json: JSON = v[1].clone();
            let bytes: Vec<u8> = bytes.lock().unwrap().clone();

            if let Some(map) = WS_STREAMS.get() {
              let stream: Option<(Arc<Mutex<WebSocketStream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, WebSocketStreams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, WebSocketStream> = stream_arc.lock().unwrap();
                  stream_lock.push(&json, &bytes);
                }
                None => {
                  let args: JSON =
                    serde_json::json!(["error", "PYO3 error in uds_ws_push: No stream found."]);

                  if let Some(map) = WS_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("ws_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_ws_push_bytes: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>, bytes: Arc<Mutex<UnixDomainSocketBytes>>| -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let bytes: Vec<u8> = bytes.lock().unwrap().clone();

            if let Some(map) = WS_STREAMS.get() {
              let stream: Option<(Arc<Mutex<WebSocketStream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, WebSocketStreams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, WebSocketStream> = stream_arc.lock().unwrap();
                  stream_lock.push_bytes(&bytes);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_ws_push_bytes: No stream found."
                  ]);

                  if let Some(map) = WS_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("ws_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_ws_push_json: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let json: JSON = v[1].clone();

            if let Some(map) = WS_STREAMS.get() {
              let stream: Option<(Arc<Mutex<WebSocketStream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, WebSocketStreams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, WebSocketStream> = stream_arc.lock().unwrap();
                  stream_lock.push_json(&json);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_ws_push_json: No stream found."
                  ]);

                  if let Some(map) = WS_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("ws_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_ws_set_compression: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let compression: &str = v[1].as_str().unwrap_or("");

            if let Some(map) = WS_STREAMS.get() {
              let stream: Option<(Arc<Mutex<WebSocketStream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, WebSocketStreams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, WebSocketStream> = stream_arc.lock().unwrap();
                  if compression.len() > 0 {
                    stream_lock.set_compression(Some(String::from(compression)));
                    return;
                  }

                  stream_lock.set_compression(None);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_ws_set_compression: No stream found."
                  ]);

                  if let Some(map) = WS_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("ws_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let mut uds: UnixDomainSocket = UnixDomainSocket::new(uds_opts);
    uds.on("ws_close", uds_ws_close);
    uds.on("ws_push", uds_ws_push);
    uds.on("ws_push_bytes", uds_ws_push_bytes);
    uds.on("ws_push_json", uds_ws_push_json);
    uds.on("ws_set_compression", uds_ws_set_compression);

    let uds_map: &Mutex<HashMap<usize, Arc<UnixDomainSocket>>> =
      WS_UDS_MAP.get_or_init(|| Mutex::new(HashMap::new()));
    {
      uds_map.lock().unwrap().insert(new_id, Arc::new(uds));
    }

    let ws_opts: WebSocketOpts = WebSocketOpts {
      block_size_kb: get_usize(&opts, "block_size_kb"),
      compression: get_bool(&opts, "compression"),
      handshake_timeout: get_u64(&opts, "handshake_timeout"),
      max_message_size_kb: get_u64(&opts, "max_message_size_kb"),
      ping_timeout: get_u64(&opts, "ping_timeout"),
      port: get_u16(&opts, "port"),
      send_timeout: get_u64(&opts, "send_timeout"),
      thread_limit: get_u64(&opts, "thread_limit"),
    };

    let ws: WebSocket = WebSocket::new(ws_opts);
    let ws_map: &Mutex<HashMap<usize, Arc<WebSocket>>> =
      WS_MAP.get_or_init(|| Mutex::new(HashMap::new()));
    {
      ws_map.lock().unwrap().insert(new_id, Arc::new(ws));
    }

    Ok(new_id)
  }

  #[pyfunction]
  fn ws_destroy(py_id: usize) -> PyResult<()> {
    if let Some(map) = WS_MAP.get() {
      map.lock().unwrap().remove(&py_id);
    }

    if let Some(map) = WS_UDS_MAP.get() {
      map.lock().unwrap().remove(&py_id);
    }

    Ok(())
  }

  #[pyfunction]
  fn ws_logger(py_id: usize) -> PyResult<()> {
    let ws_logger: Arc<WebSocketLogger> = Arc::new(move |level: &str, message: &str| -> () {
      let args: JSON = serde_json::json!([level, message]);
      let bytes: UnixDomainSocketBytes = Vec::new();

      if let Some(map) = WS_UDS_MAP.get() {
        if let Some(uds) = map.lock().unwrap().get(&py_id) {
          uds.push("ws_logger", &args, bytes, true);
        }
      }
    });

    if let Some(map) = WS_MAP.get() {
      if let Some(ws) = map.lock().unwrap().get(&py_id) {
        ws.logger(ws_logger);
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn ws_on(py_id: usize, py_topic: &str) -> PyResult<()> {
    let topic_safe: String = String::from(py_topic);
    let ws_handler: Arc<WebSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<WebSocketCtx>>,
            bytes: Arc<Mutex<WebSocketBytes>>,
            stream: Arc<Mutex<WebSocketStream>>|
            -> () {
        let (tx, rx) = mpsc::channel::<u8>();
        let stream_id: usize = WS_STREAM_ID.fetch_add(1, Ordering::Relaxed);

        WS_STREAMS
          .get_or_init(|| Mutex::new(HashMap::new()))
          .lock()
          .unwrap()
          .insert(stream_id, (stream, tx));

        let ctx: WebSocketCtx = ctx.lock().unwrap().clone();
        let args: JSON = serde_json::json!([stream_id, topic_safe, ctx]);
        let bytes: WebSocketBytes = bytes.lock().unwrap().clone();

        if let Some(map) = WS_UDS_MAP.get() {
          if let Some(uds) = map.lock().unwrap().get(&py_id) {
            uds.push("ws_on", &args, bytes, true);
          }
        }

        loop {
          match rx.recv_timeout(Duration::from_secs(90)) {
            Ok(val) => {
              if val == 1 {
                break;
              }
            }
            Err(RecvTimeoutError::Timeout) => {
              let args: JSON =
                serde_json::json!(["error", "PYO3 error in ws_on: Stream response timeout."]);
              if let Some(map) = WS_UDS_MAP.get() {
                if let Some(uds) = map.lock().unwrap().get(&py_id) {
                  uds.push("ws_logger", &args, Vec::new(), true);
                }
              }
              break;
            }
            Err(RecvTimeoutError::Disconnected) => {
              let args: JSON =
                serde_json::json!(["error", "PYO3 error in ws_on: Stream channel disconnected."]);
              if let Some(map) = WS_UDS_MAP.get() {
                if let Some(uds) = map.lock().unwrap().get(&py_id) {
                  uds.push("ws_logger", &args, Vec::new(), true);
                }
              }

              break;
            }
          }
        }

        if let Some(map) = WS_STREAMS.get() {
          map.lock().unwrap().remove(&stream_id);
        }
      },
    );

    if let Some(map) = WS_MAP.get() {
      if let Some(ws) = map.lock().unwrap().get(&py_id) {
        ws.on(py_topic, ws_handler);
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn ws_start_ipc(py_id: usize) -> PyResult<()> {
    let (tx, rx) = mpsc::channel::<u8>();

    if let Some(map) = WS_UDS_MAP.get() {
      if let Some(uds) = map.lock().unwrap().get(&py_id) {
        let uds_safe: Arc<UnixDomainSocket> = Arc::clone(uds);
        thread::spawn(move || {
          uds_safe.start(Arc::new(move || {
            let _ = tx.send(1);
          }));
        });
      }
    }

    loop {
      match rx.recv() {
        Ok(1) => break,
        Ok(_) => continue,
        Err(_) => break,
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn ws_start(py_id: usize) -> PyResult<()> {
    if let Some(map) = WS_MAP.get() {
      if let Some(ws) = map.lock().unwrap().get(&py_id) {
        let ws_safe: Arc<WebSocket> = Arc::clone(ws);
        thread::spawn(move || {
          ws_safe.start();
        });
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn ws_stop(py_id: usize) -> PyResult<()> {
    if let Some(map) = WS_UDS_MAP.get() {
      if let Some(uds) = map.lock().unwrap().get(&py_id) {
        uds.stop();
      }
    }

    if let Some(map) = WS_MAP.get() {
      if let Some(ws) = map.lock().unwrap().get(&py_id) {
        ws.stop();
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http3_create(py_opts: &str) -> PyResult<usize> {
    let opts: JSON = {
      match serde_json::from_str(py_opts) {
        Ok(json) => json,
        Err(_) => {
          println!(
            "[Arnelify Server]: Rust PYO3 error in http3_create: Invalid JSON in 'py_opts'."
          );
          return Ok(0);
        }
      }
    };

    let id: &Mutex<usize> = HTTP3_ID.get_or_init(|| Mutex::new(0));
    let new_id: usize = {
      let mut py: MutexGuard<'_, usize> = id.lock().unwrap();
      *py += 1;
      *py
    };

    let uds_opts: UnixDomainSocketOpts = UnixDomainSocketOpts {
      block_size_kb: get_usize(&opts, "block_size_kb"),
      socket_path: get_str(&opts, "socket_path"),
      thread_limit: get_u64(&opts, "thread_limit"),
    };

    let uds_http3_add_header: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let key: String = v[1].as_str().unwrap_or("").to_string();
            let value: String = v[2].as_str().unwrap_or("").to_string();

            if let Some(map) = HTTP3_STREAMS.get() {
              let stream: Option<(Arc<Mutex<Http3Stream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, Http3Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http3Stream> = stream_arc.lock().unwrap();
                  stream_lock.add_header(&key, &value);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_http3_add_header: No stream found."
                  ]);

                  if let Some(map) = HTTP3_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http3_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_http3_end: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            if let Some(map) = HTTP3_STREAMS.get() {
              let stream: Option<(Arc<Mutex<Http3Stream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, Http3Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http3Stream> = stream_arc.lock().unwrap();
                  stream_lock.end();
                  let _ = tx.send(1);
                }
                None => {
                  let args: JSON =
                    serde_json::json!(["error", "PYO3 error in uds_http3_end: No stream found."]);

                  if let Some(map) = HTTP3_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http3_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_http3_push_bytes: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>, bytes: Arc<Mutex<UnixDomainSocketBytes>>| -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let is_attachment: bool = v[1].as_u64().unwrap_or(0) != 0;
            let bytes: Vec<u8> = bytes.lock().unwrap().clone();

            if let Some(map) = HTTP3_STREAMS.get() {
              let stream: Option<(Arc<Mutex<Http3Stream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, Http3Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http3Stream> = stream_arc.lock().unwrap();
                  stream_lock.push_bytes(&bytes, is_attachment);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_http3_push_bytes: No stream found."
                  ]);

                  if let Some(map) = HTTP3_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http3_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_http3_push_file: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let file_path: &str = v[1].as_str().unwrap_or("");
            let is_attachment: bool = v[2].as_u64().unwrap_or(0) != 0;

            if let Some(map) = HTTP3_STREAMS.get() {
              let stream: Option<(Arc<Mutex<Http3Stream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, Http3Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http3Stream> = stream_arc.lock().unwrap();
                  stream_lock.push_file(&file_path, is_attachment);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_http3_push_file: No stream found."
                  ]);

                  if let Some(map) = HTTP3_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http3_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_http3_push_json: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let json: JSON = v[1].clone();
            let is_attachment: bool = v[2].as_u64().unwrap_or(0) != 0;

            if let Some(map) = HTTP3_STREAMS.get() {
              let stream: Option<(Arc<Mutex<Http3Stream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, Http3Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http3Stream> = stream_arc.lock().unwrap();
                  stream_lock.push_json(&json, is_attachment);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_http3_push_json: No stream found."
                  ]);

                  if let Some(map) = HTTP3_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http3_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_http3_set_code: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let code: u64 = v[1].as_u64().unwrap_or(0);

            if let Some(map) = HTTP3_STREAMS.get() {
              let stream: Option<(Arc<Mutex<Http3Stream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, Http3Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http3Stream> = stream_arc.lock().unwrap();
                  stream_lock.set_code(code as u16);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_http3_set_code: No stream found."
                  ]);

                  if let Some(map) = HTTP3_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http3_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_http3_set_compression: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let compression: &str = v[1].as_str().unwrap_or("");

            if let Some(map) = HTTP3_STREAMS.get() {
              let stream: Option<(Arc<Mutex<Http3Stream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, Http3Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http3Stream> = stream_arc.lock().unwrap();
                  if compression.len() > 0 {
                    stream_lock.set_compression(Some(String::from(compression)));
                    return;
                  }

                  stream_lock.set_compression(None);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_http3_set_compression: No stream found."
                  ]);

                  if let Some(map) = HTTP3_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http3_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_http3_set_headers: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let mut headers: Vec<(String, String)> = Vec::new();
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;

            if let Some(JSON::Object(map)) = v.get(1) {
              for (key, value) in map {
                let value = match value {
                  JSON::String(s) => s.clone(),
                  JSON::Number(n) => n.to_string(),
                  JSON::Bool(b) => b.to_string(),
                  _ => continue,
                };

                headers.push((key.clone(), value));
              }
            }

            if let Some(map) = HTTP3_STREAMS.get() {
              let stream: Option<(Arc<Mutex<Http3Stream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, Http3Streams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, Http3Stream> = stream_arc.lock().unwrap();
                  stream_lock.set_headers(headers);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_http3_set_headers: No stream found."
                  ]);

                  if let Some(map) = HTTP3_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("http3_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let mut uds: UnixDomainSocket = UnixDomainSocket::new(uds_opts);
    uds.on("http3_add_header", uds_http3_add_header);
    uds.on("http3_end", uds_http3_end);
    uds.on("http3_push_bytes", uds_http3_push_bytes);
    uds.on("http3_push_file", uds_http3_push_file);
    uds.on("http3_push_json", uds_http3_push_json);
    uds.on("http3_set_code", uds_http3_set_code);
    uds.on("http3_set_compression", uds_http3_set_compression);
    uds.on("http3_set_headers", uds_http3_set_headers);

    let uds_map: &Mutex<HashMap<usize, Arc<UnixDomainSocket>>> =
      HTTP3_UDS_MAP.get_or_init(|| Mutex::new(HashMap::new()));
    {
      uds_map.lock().unwrap().insert(new_id, Arc::new(uds));
    }

    let http3_opts: Http3Opts = Http3Opts {
      allow_empty_files: get_bool(&opts, "allow_empty_files"),
      block_size_kb: get_usize(&opts, "block_size_kb"),
      cert_pem: get_str(&opts, "cert_pem"),
      charset: get_str(&opts, "charset"),
      compression: get_bool(&opts, "compression"),
      keep_alive: get_u8(&opts, "keep_alive"),
      keep_extensions: get_bool(&opts, "keep_extensions"),
      key_pem: get_str(&opts, "key_pem"),
      max_fields: get_u32(&opts, "max_fields"),
      max_fields_size_total_mb: get_usize(&opts, "max_fields_size_total_mb"),
      max_files: get_u32(&opts, "max_files"),
      max_files_size_total_mb: get_usize(&opts, "max_files_size_total_mb"),
      max_file_size_mb: get_usize(&opts, "max_file_size_mb"),
      port: get_u16(&opts, "port"),
      storage_path: get_str(&opts, "storage_path"),
      thread_limit: get_u64(&opts, "thread_limit"),
    };

    let http3: Http3 = Http3::new(http3_opts);
    let http3_map: &Mutex<HashMap<usize, Arc<Http3>>> =
      HTTP3_MAP.get_or_init(|| Mutex::new(HashMap::new()));
    {
      http3_map.lock().unwrap().insert(new_id, Arc::new(http3));
    }

    Ok(new_id)
  }

  #[pyfunction]
  fn http3_destroy(py_id: usize) -> PyResult<()> {
    if let Some(map) = HTTP3_MAP.get() {
      map.lock().unwrap().remove(&py_id);
    }

    if let Some(map) = HTTP3_UDS_MAP.get() {
      map.lock().unwrap().remove(&py_id);
    }

    Ok(())
  }

  #[pyfunction]
  fn http3_logger(py_id: usize) -> PyResult<()> {
    let http3_logger: Arc<Http3Logger> = Arc::new(move |level: &str, message: &str| -> () {
      let args: JSON = serde_json::json!([level, message]);
      let bytes: UnixDomainSocketBytes = Vec::new();

      if let Some(map) = HTTP3_UDS_MAP.get() {
        if let Some(uds) = map.lock().unwrap().get(&py_id) {
          uds.push("http3_logger", &args, bytes, true);
        }
      }
    });

    if let Some(map) = HTTP3_MAP.get() {
      if let Some(http3) = map.lock().unwrap().get(&py_id) {
        http3.logger(http3_logger);
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http3_on(py_id: usize, py_path: &str) -> PyResult<()> {
    let path_safe: String = String::from(py_path);
    let http3_handler: Arc<Http3Handler> = Arc::new(
      move |ctx: Arc<Mutex<Http3Ctx>>, stream: Arc<Mutex<Http3Stream>>| -> () {
        let (tx, rx) = mpsc::channel::<u8>();
        let stream_id: usize = HTTP3_STREAM_ID.fetch_add(1, Ordering::Relaxed);

        HTTP3_STREAMS
          .get_or_init(|| Mutex::new(HashMap::new()))
          .lock()
          .unwrap()
          .insert(stream_id, (stream, tx));

        let ctx: Http3Ctx = ctx.lock().unwrap().clone();
        let args: JSON = serde_json::json!([stream_id, path_safe, ctx]);
        let bytes: UnixDomainSocketBytes = Vec::new();

        if let Some(map) = HTTP3_UDS_MAP.get() {
          if let Some(uds) = map.lock().unwrap().get(&py_id) {
            uds.push("http3_on", &args, bytes, true);
          }
        }

        loop {
          match rx.recv_timeout(Duration::from_secs(90)) {
            Ok(val) => {
              if val == 1 {
                break;
              }
            }
            Err(RecvTimeoutError::Timeout) => {
              let args: JSON =
                serde_json::json!(["error", "PYO3 error in http3_on: Stream response timeout."]);
              if let Some(map) = HTTP3_UDS_MAP.get() {
                if let Some(uds) = map.lock().unwrap().get(&py_id) {
                  uds.push("http3_logger", &args, Vec::new(), true);
                }
              }
              break;
            }
            Err(RecvTimeoutError::Disconnected) => {
              let args: JSON = serde_json::json!([
                "error",
                "PYO3 error in http3_on: Stream channel disconnected."
              ]);
              if let Some(map) = HTTP3_UDS_MAP.get() {
                if let Some(uds) = map.lock().unwrap().get(&py_id) {
                  uds.push("http3_logger", &args, Vec::new(), true);
                }
              }

              break;
            }
          }
        }

        if let Some(map) = HTTP3_STREAMS.get() {
          map.lock().unwrap().remove(&stream_id);
        }
      },
    );

    if let Some(map) = HTTP3_MAP.get() {
      if let Some(http3) = map.lock().unwrap().get(&py_id) {
        http3.on(py_path, http3_handler);
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http3_start_ipc(py_id: usize) -> PyResult<()> {
    let (tx, rx) = mpsc::channel::<u8>();

    if let Some(map) = HTTP3_UDS_MAP.get() {
      if let Some(uds) = map.lock().unwrap().get(&py_id) {
        let uds_safe: Arc<UnixDomainSocket> = Arc::clone(uds);
        thread::spawn(move || {
          uds_safe.start(Arc::new(move || {
            let _ = tx.send(1);
          }));
        });
      }
    }

    loop {
      match rx.recv() {
        Ok(1) => break,
        Ok(_) => continue,
        Err(_) => break,
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http3_start(py_id: usize) -> PyResult<()> {
    if let Some(map) = HTTP3_MAP.get() {
      if let Some(http3) = map.lock().unwrap().get(&py_id) {
        let http3_safe: Arc<Http3> = Arc::clone(http3);
        thread::spawn(move || {
          http3_safe.start();
        });
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http3_stop(py_id: usize) -> PyResult<()> {
    if let Some(map) = HTTP3_UDS_MAP.get() {
      if let Some(uds) = map.lock().unwrap().get(&py_id) {
        uds.stop();
      }
    }

    if let Some(map) = HTTP3_MAP.get() {
      if let Some(http3) = map.lock().unwrap().get(&py_id) {
        http3.stop();
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn wt_create(py_opts: &str) -> PyResult<usize> {
    let opts: JSON = {
      match serde_json::from_str(py_opts) {
        Ok(json) => json,
        Err(_) => {
          println!("[Arnelify Server]: Rust PYO3 error in wt_create: Invalid JSON in 'py_opts'.");
          return Ok(0);
        }
      }
    };

    let id: &Mutex<usize> = WT_ID.get_or_init(|| Mutex::new(0));
    let new_id: usize = {
      let mut py: MutexGuard<'_, usize> = id.lock().unwrap();
      *py += 1;
      *py
    };

    let uds_opts: UnixDomainSocketOpts = UnixDomainSocketOpts {
      block_size_kb: get_usize(&opts, "block_size_kb"),
      socket_path: get_str(&opts, "socket_path"),
      thread_limit: get_u64(&opts, "thread_limit"),
    };

    let uds_wt_close: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;

            if let Some(map) = WT_STREAMS.get() {
              let stream: Option<(Arc<Mutex<WebTransportStream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, WebTransportStreams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, tx)) => {
                  let mut stream_lock: MutexGuard<'_, WebTransportStream> =
                    stream_arc.lock().unwrap();
                  stream_lock.close();
                  let _ = tx.send(1);
                }
                None => {
                  let args: JSON =
                    serde_json::json!(["error", "PYO3 error in uds_wt_close: No stream found."]);

                  if let Some(map) = WT_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("wt_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_wt_push: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>, bytes: Arc<Mutex<UnixDomainSocketBytes>>| -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let json: JSON = v[1].clone();
            let bytes: Vec<u8> = bytes.lock().unwrap().clone();

            if let Some(map) = WT_STREAMS.get() {
              let stream: Option<(Arc<Mutex<WebTransportStream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, WebTransportStreams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, WebTransportStream> =
                    stream_arc.lock().unwrap();
                  stream_lock.push(&json, &bytes);
                }
                None => {
                  let args: JSON =
                    serde_json::json!(["error", "PYO3 error in uds_wt_push: No stream found."]);

                  if let Some(map) = WT_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("wt_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_wt_push_bytes: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>, bytes: Arc<Mutex<UnixDomainSocketBytes>>| -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let bytes: Vec<u8> = bytes.lock().unwrap().clone();
            if let Some(map) = WT_STREAMS.get() {
              let stream: Option<(Arc<Mutex<WebTransportStream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, WebTransportStreams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, WebTransportStream> =
                    stream_arc.lock().unwrap();
                  stream_lock.push_bytes(&bytes);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_wt_push_bytes: No stream found."
                  ]);

                  if let Some(map) = WT_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("wt_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_wt_push_json: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let json: JSON = v[1].clone();

            if let Some(map) = WT_STREAMS.get() {
              let stream: Option<(Arc<Mutex<WebTransportStream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, WebTransportStreams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, WebTransportStream> =
                    stream_arc.lock().unwrap();
                  stream_lock.push_json(&json);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_wt_push_json: No stream found."
                  ]);

                  if let Some(map) = WT_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("wt_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let uds_wt_set_compression: Arc<UnixDomainSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<UnixDomainSocketCtx>>,
            _bytes: Arc<Mutex<UnixDomainSocketBytes>>|
            -> () {
        let args: JSON = ctx.lock().unwrap().clone();

        match args.as_array() {
          Some(v) => {
            let stream_id: usize = v[0].as_u64().unwrap_or(0) as usize;
            let compression: &str = v[1].as_str().unwrap_or("");

            if let Some(map) = WT_STREAMS.get() {
              let stream: Option<(Arc<Mutex<WebTransportStream>>, mpsc::Sender<u8>)> = {
                let streams: MutexGuard<'_, WebTransportStreams> = map.lock().unwrap();
                streams.get(&stream_id).cloned()
              };

              match stream {
                Some((stream_arc, _tx)) => {
                  let mut stream_lock: MutexGuard<'_, WebTransportStream> =
                    stream_arc.lock().unwrap();
                  if compression.len() > 0 {
                    stream_lock.set_compression(Some(String::from(compression)));
                    return;
                  }

                  stream_lock.set_compression(None);
                }
                None => {
                  let args: JSON = serde_json::json!([
                    "error",
                    "PYO3 error in uds_wt_set_compression: No stream found."
                  ]);

                  if let Some(map) = WT_UDS_MAP.get() {
                    if let Some(uds) = map.lock().unwrap().get(&new_id) {
                      uds.push("wt_logger", &args, Vec::new(), true);
                    }
                  }
                }
              }
            }
          }
          None => {}
        }
      },
    );

    let mut uds: UnixDomainSocket = UnixDomainSocket::new(uds_opts);
    uds.on("wt_close", uds_wt_close);
    uds.on("wt_push", uds_wt_push);
    uds.on("wt_push_bytes", uds_wt_push_bytes);
    uds.on("wt_push_json", uds_wt_push_json);
    uds.on("wt_set_compression", uds_wt_set_compression);

    let uds_map: &Mutex<HashMap<usize, Arc<UnixDomainSocket>>> =
      WT_UDS_MAP.get_or_init(|| Mutex::new(HashMap::new()));
    {
      uds_map.lock().unwrap().insert(new_id, Arc::new(uds));
    }

    let wt_opts: WebTransportOpts = WebTransportOpts {
      block_size_kb: get_usize(&opts, "block_size_kb"),
      cert_pem: get_str(&opts, "cert_pem"),
      compression: get_bool(&opts, "compression"),
      handshake_timeout: get_u64(&opts, "handshake_timeout"),
      key_pem: get_str(&opts, "key_pem"),
      max_message_size_kb: get_u64(&opts, "max_message_size_kb"),
      ping_timeout: get_u64(&opts, "ping_timeout"),
      port: get_u16(&opts, "port"),
      send_timeout: get_u64(&opts, "send_timeout"),
      thread_limit: get_u64(&opts, "thread_limit"),
    };

    let wt: WebTransport = WebTransport::new(wt_opts);
    let wt_map: &Mutex<HashMap<usize, Arc<WebTransport>>> =
      WT_MAP.get_or_init(|| Mutex::new(HashMap::new()));
    {
      wt_map.lock().unwrap().insert(new_id, Arc::new(wt));
    }

    Ok(new_id)
  }

  #[pyfunction]
  fn wt_destroy(py_id: usize) -> PyResult<()> {
    if let Some(map) = WT_MAP.get() {
      map.lock().unwrap().remove(&py_id);
    }

    Ok(())
  }

  #[pyfunction]
  fn wt_logger(py_id: usize) -> PyResult<()> {
    let wt_logger: Arc<WebTransportLogger> = Arc::new(move |level: &str, message: &str| -> () {
      let args: JSON = serde_json::json!([level, message]);
      let bytes: UnixDomainSocketBytes = Vec::new();

      if let Some(map) = WT_UDS_MAP.get() {
        if let Some(uds) = map.lock().unwrap().get(&py_id) {
          uds.push("wt_logger", &args, bytes, true);
        }
      }
    });

    if let Some(map) = WT_MAP.get() {
      if let Some(wt) = map.lock().unwrap().get(&py_id) {
        wt.logger(wt_logger);
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn wt_on(py_id: usize, py_topic: &str) -> PyResult<()> {
    let topic_safe: String = String::from(py_topic);
    let wt_handler: Arc<WebTransportHandler> = Arc::new(
      move |ctx: Arc<Mutex<WebTransportCtx>>,
            bytes: Arc<Mutex<WebTransportBytes>>,
            stream: Arc<Mutex<WebTransportStream>>|
            -> () {
        let (tx, rx) = mpsc::channel::<u8>();
        let stream_id: usize = WT_STREAM_ID.fetch_add(1, Ordering::Relaxed);

        WT_STREAMS
          .get_or_init(|| Mutex::new(HashMap::new()))
          .lock()
          .unwrap()
          .insert(stream_id, (stream, tx));

        let ctx: WebTransportCtx = ctx.lock().unwrap().clone();
        let args: JSON = serde_json::json!([stream_id, topic_safe, ctx]);
        let bytes: WebTransportBytes = bytes.lock().unwrap().clone();

        if let Some(map) = WT_UDS_MAP.get() {
          if let Some(uds) = map.lock().unwrap().get(&py_id) {
            uds.push("wt_on", &args, bytes, true);
          }
        }

        loop {
          match rx.recv_timeout(Duration::from_secs(90)) {
            Ok(val) => {
              if val == 1 {
                break;
              }
            }
            Err(RecvTimeoutError::Timeout) => {
              let args: JSON =
                serde_json::json!(["error", "PYO3 error in wt_on: Stream response timeout."]);
              if let Some(map) = WT_UDS_MAP.get() {
                if let Some(uds) = map.lock().unwrap().get(&py_id) {
                  uds.push("wt_logger", &args, Vec::new(), true);
                }
              }
              break;
            }
            Err(RecvTimeoutError::Disconnected) => {
              let args: JSON =
                serde_json::json!(["error", "PYO3 error in wt_on: Stream channel disconnected."]);
              if let Some(map) = WT_UDS_MAP.get() {
                if let Some(uds) = map.lock().unwrap().get(&py_id) {
                  uds.push("wt_logger", &args, Vec::new(), true);
                }
              }

              break;
            }
          }
        }

        if let Some(map) = WT_STREAMS.get() {
          map.lock().unwrap().remove(&stream_id);
        }
      },
    );

    if let Some(map) = WT_MAP.get() {
      if let Some(wt) = map.lock().unwrap().get(&py_id) {
        wt.on(py_topic, wt_handler);
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn wt_start_ipc(py_id: usize) -> PyResult<()> {
    let (tx, rx) = mpsc::channel::<u8>();

    if let Some(map) = WT_UDS_MAP.get() {
      if let Some(uds) = map.lock().unwrap().get(&py_id) {
        let uds_safe: Arc<UnixDomainSocket> = Arc::clone(uds);
        thread::spawn(move || {
          uds_safe.start(Arc::new(move || {
            let _ = tx.send(1);
          }));
        });
      }
    }

    loop {
      match rx.recv() {
        Ok(1) => break,
        Ok(_) => continue,
        Err(_) => break,
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn wt_start(py_id: usize) -> PyResult<()> {
    if let Some(map) = WT_MAP.get() {
      if let Some(wt) = map.lock().unwrap().get(&py_id) {
        let wt_safe: Arc<WebTransport> = Arc::clone(wt);
        thread::spawn(move || {
          wt_safe.start();
        });
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn wt_stop(py_id: usize) -> PyResult<()> {
    if let Some(map) = WT_UDS_MAP.get() {
      if let Some(uds) = map.lock().unwrap().get(&py_id) {
        uds.stop();
      }
    }

    if let Some(map) = WT_MAP.get() {
      if let Some(wt) = map.lock().unwrap().get(&py_id) {
        wt.stop();
      }
    }

    Ok(())
  }
}