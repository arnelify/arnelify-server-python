// MIT LICENSE

// COPYRIGHT (R) 2025 ARNELIFY. AUTHOR: TARON SARKISYAN

// PERMISSION IS HEREBY GRANTED, FREE OF CHARGE, TO ANY PERSON OBTAINING A COPY
// OF THIS SOFTWARE AND ASSOCIATED DOCUMENTATION FILES (THE "SOFTWARE"), TO DEAL
// IN THE SOFTWARE WITHOUT RESTRICTION, INCLUDING WITHOUT LIMITATION THE RIGHTS
// TO USE, COPY, MODIFY, MERGE, PUBLISH, DISTRIBUTE, SUBLICENSE, AND/OR SELL
// COPIES OF THE SOFTWARE, AND TO PERMIT PERSONS TO WHOM THE SOFTWARE IS
// FURNISHED TO DO SO, SUBJECT TO THE FOLLOWING CONDITIONS:

// THE ABOVE COPYRIGHT NOTICE AND THIS PERMISSION NOTICE SHALL BE INCLUDED IN ALL
// COPIES OR SUBSTANTIAL PORTIONS OF THE SOFTWARE.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use pyo3::prelude::*;

pub mod tcp1;
pub mod tcp2;

#[pymodule]
mod arnelify_server {
  use pyo3::prelude::*;
  use pyo3::types::PyBytes;

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
    },
  };

  use serde_json::Value as JSON;

  type Http1Streams = HashMap<usize, Arc<Mutex<Http1Stream>>>;
  type Http2Streams = HashMap<usize, Arc<Mutex<Http2Stream>>>;
  type WebSocketStreams = HashMap<usize, Arc<Mutex<WebSocketStream>>>;
  type Http3Streams = HashMap<usize, Arc<Mutex<Http3Stream>>>;
  type WebTransportStreams = HashMap<usize, Arc<Mutex<WebTransportStream>>>;

  static HTTP1_MAP: OnceLock<Mutex<HashMap<usize, Arc<Http1>>>> = OnceLock::new();
  static HTTP1_ID: OnceLock<Mutex<usize>> = OnceLock::new();
  static HTTP1_STREAM_ID: AtomicUsize = AtomicUsize::new(1);
  static HTTP1_STREAMS: OnceLock<Mutex<Http1Streams>> = OnceLock::new();

  static HTTP2_MAP: OnceLock<Mutex<HashMap<usize, Arc<Http2>>>> = OnceLock::new();
  static HTTP2_ID: OnceLock<Mutex<usize>> = OnceLock::new();
  static HTTP2_STREAM_ID: AtomicUsize = AtomicUsize::new(1);
  static HTTP2_STREAMS: OnceLock<Mutex<Http2Streams>> = OnceLock::new();

  static WS_MAP: OnceLock<Mutex<HashMap<usize, Arc<WebSocket>>>> = OnceLock::new();
  static WS_ID: OnceLock<Mutex<usize>> = OnceLock::new();
  static WS_STREAM_ID: AtomicUsize = AtomicUsize::new(1);
  static WS_STREAMS: OnceLock<Mutex<WebSocketStreams>> = OnceLock::new();

  static HTTP3_MAP: OnceLock<Mutex<HashMap<usize, Arc<Http3>>>> = OnceLock::new();
  static HTTP3_ID: OnceLock<Mutex<usize>> = OnceLock::new();
  static HTTP3_STREAM_ID: AtomicUsize = AtomicUsize::new(1);
  static HTTP3_STREAMS: OnceLock<Mutex<Http3Streams>> = OnceLock::new();

  static WT_MAP: OnceLock<Mutex<HashMap<usize, Arc<WebTransport>>>> = OnceLock::new();
  static WT_ID: OnceLock<Mutex<usize>> = OnceLock::new();
  static WT_STREAM_ID: AtomicUsize = AtomicUsize::new(1);
  static WT_STREAMS: OnceLock<Mutex<WebTransportStreams>> = OnceLock::new();

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
  fn http1_add_header(py_stream_id: usize, py_key: &str, py_value: &str) -> PyResult<()> {
    if let Some(map) = HTTP1_STREAMS.get() {
      let streams: MutexGuard<'_, Http1Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let mut stream_lock: std::sync::MutexGuard<'_, Http1Stream> = stream.lock().unwrap();
          stream_lock.add_header(py_key, py_value);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http1_add_header: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http1_create(py_opts: &str) -> PyResult<usize> {
    let opts: JSON = {
      match serde_json::from_str(py_opts) {
        Ok(json) => json,
        Err(_) => {
          println!("[Arnelify Server]: Rust PYO3 error in http1_create: Invalid JSON in 'py_opts'.");
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

    Ok(())
  }

  #[pyfunction]
  fn http1_end(py_stream_id: usize) -> PyResult<()> {
    if let Some(map) = HTTP1_STREAMS.get() {
      let streams: MutexGuard<'_, Http1Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let mut stream_lock: std::sync::MutexGuard<'_, Http1Stream> = stream.lock().unwrap();
          stream_lock.end();
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http1_end: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http1_logger(py_id: usize, py_cb: Py<PyAny>) -> PyResult<()> {
    let http1_logger: Arc<Http1Logger> = Arc::new(move |level: &str, message: &str| {
      Python::attach(|py| {
        if let Err(err) = py_cb.call1(py, (level, message)) {
          err.print(py);
        }
      });
    });

    if let Some(map) = HTTP1_MAP.get() {
      if let Some(http1) = map.lock().unwrap().get(&py_id) {
        http1.logger(http1_logger);
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http1_on(py_id: usize, py_path: &str, py_cb: Py<PyAny>) -> PyResult<()> {
    let http1_handler: Arc<Http1Handler> = Arc::new(
      move |ctx: Arc<Mutex<Http1Ctx>>, stream: Arc<Mutex<Http1Stream>>| {
        let stream_id: usize = HTTP1_STREAM_ID.fetch_add(1, Ordering::Relaxed);

        HTTP1_STREAMS
          .get_or_init(|| Mutex::new(HashMap::new()))
          .lock()
          .unwrap()
          .insert(stream_id, stream);

        let json: String = {
          let ctx_lock = ctx.lock().unwrap();
          serde_json::to_string(&*ctx_lock).unwrap()
        };

        Python::attach(|py| {
          if let Err(err) = py_cb.call1(py, (stream_id, json)) {
            err.print(py);
          }
        });

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
  fn http1_push_bytes(
    py_stream_id: usize,
    py_bytes: &[u8],
    py_is_attachment: usize,
  ) -> PyResult<()> {
    if let Some(map) = HTTP1_STREAMS.get() {
      let streams: MutexGuard<'_, Http1Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let is_attachment: bool = py_is_attachment == 1;
          if py_bytes.is_empty() {
            let mut stream_lock: std::sync::MutexGuard<'_, Http1Stream> = stream.lock().unwrap();
            stream_lock.push_bytes(&[], is_attachment);
            return Ok(());
          }

          let mut stream_lock: std::sync::MutexGuard<'_, Http1Stream> = stream.lock().unwrap();
          stream_lock.push_bytes(py_bytes, is_attachment);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http1_push_bytes: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http1_push_file(
    py_stream_id: usize,
    py_file_path: &str,
    py_is_attachment: usize,
  ) -> PyResult<()> {
    if let Some(map) = HTTP1_STREAMS.get() {
      let streams: MutexGuard<'_, Http1Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let is_attachment: bool = py_is_attachment == 1;
          let mut stream_lock: std::sync::MutexGuard<'_, Http1Stream> = stream.lock().unwrap();
          stream_lock.push_file(py_file_path, is_attachment);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http1_push_file: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http1_push_json(py_stream_id: usize, py_json: &str, py_is_attachment: usize) -> PyResult<()> {
    if let Some(map) = HTTP1_STREAMS.get() {
      let streams: MutexGuard<'_, Http1Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let is_attachment: bool = py_is_attachment == 1;
          let json: JSON = match serde_json::from_str(py_json) {
            Ok(json) => json,
            Err(_) => {
              println!(
                "[Arnelify Server]: Rust PYO3 error in http1_push_json: Invalid JSON in 'py_json'."
              );

              return Ok(());
            }
          };

          let mut stream_lock: std::sync::MutexGuard<'_, Http1Stream> = stream.lock().unwrap();
          stream_lock.push_json(&json, is_attachment);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http1_push_json: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http1_set_code(py_stream_id: usize, py_code: usize) -> PyResult<()> {
    if let Some(map) = HTTP1_STREAMS.get() {
      let streams: MutexGuard<'_, Http1Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let mut stream_lock: std::sync::MutexGuard<'_, Http1Stream> = stream.lock().unwrap();
          stream_lock.set_code(py_code as u16);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http1_set_code: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http1_set_compression(py_stream_id: usize, py_compression: &str) -> PyResult<()> {
    if let Some(map) = HTTP1_STREAMS.get() {
      let streams: MutexGuard<'_, Http1Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let mut stream_lock: std::sync::MutexGuard<'_, Http1Stream> = stream.lock().unwrap();
          if py_compression.len() > 0 {
            stream_lock.set_compression(Some(String::from(py_compression)));
            return Ok(());
          }

          stream_lock.set_compression(None);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http1_set_compression: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http1_set_headers(py_stream_id: usize, py_headers: &str) -> PyResult<()> {
    if let Some(map) = HTTP1_STREAMS.get() {
      let streams: MutexGuard<'_, Http1Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let json: Vec<JSON> = match serde_json::from_str(py_headers) {
            Ok(json) => json,
            Err(_) => {
              println!(
                "[Arnelify Server]: Rust PYO3 error in http1_set_headers: Invalid JSON in 'py_headers'."
              );
              return Ok(());
            }
          };

          let mut headers: Vec<(String, String)> = Vec::new();
          for header in json {
            if let JSON::Object(pair) = header {
              for (key, value) in pair {
                let value = match value {
                  JSON::String(s) => s,
                  JSON::Number(n) => n.to_string(),
                  JSON::Bool(b) => b.to_string(),
                  _ => continue,
                };
                headers.push((key, value));
              }
            }
          }

          let mut stream_lock: std::sync::MutexGuard<'_, Http1Stream> = stream.lock().unwrap();
          stream_lock.set_headers(headers);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http1_set_headers: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http1_start(py_id: usize) -> PyResult<()> {
    if let Some(map) = HTTP1_MAP.get() {
      if let Some(http1) = map.lock().unwrap().get(&py_id) {
        let http1 = http1.clone();
        std::thread::spawn(move || {
          http1.start();
        });
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http1_stop(py_id: usize) -> PyResult<()> {
    if let Some(map) = HTTP1_MAP.get() {
      if let Some(http1) = map.lock().unwrap().get(&py_id) {
        http1.stop();
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http2_add_header(py_stream_id: usize, py_key: &str, py_value: &str) -> PyResult<()> {
    if let Some(map) = HTTP2_STREAMS.get() {
      let streams: MutexGuard<'_, Http2Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let mut stream_lock: std::sync::MutexGuard<'_, Http2Stream> = stream.lock().unwrap();
          stream_lock.add_header(py_key, py_value);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http2_add_header: No stream found.");
        }
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
          println!("[Arnelify Server]: Rust PYO3 error in http2_create: Invalid JSON in 'py_opts'.");
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

    Ok(())
  }

  #[pyfunction]
  fn http2_end(py_stream_id: usize) -> PyResult<()> {
    if let Some(map) = HTTP2_STREAMS.get() {
      let streams: MutexGuard<'_, Http2Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let mut stream_lock: std::sync::MutexGuard<'_, Http2Stream> = stream.lock().unwrap();
          stream_lock.end();
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http2_end: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http2_logger(py_id: usize, py_cb: Py<PyAny>) -> PyResult<()> {
    let http2_logger: Arc<Http2Logger> = Arc::new(move |level: &str, message: &str| {
      Python::attach(|py| {
        if let Err(err) = py_cb.call1(py, (level, message)) {
          err.print(py);
        }
      });
    });

    if let Some(map) = HTTP2_MAP.get() {
      if let Some(http2) = map.lock().unwrap().get(&py_id) {
        http2.logger(http2_logger);
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http2_on(py_id: usize, py_path: &str, py_cb: Py<PyAny>) -> PyResult<()> {
    let http2_handler: Arc<Http2Handler> = Arc::new(
      move |ctx: Arc<Mutex<Http2Ctx>>, stream: Arc<Mutex<Http2Stream>>| {
        let stream_id: usize = HTTP2_STREAM_ID.fetch_add(1, Ordering::Relaxed);

        HTTP2_STREAMS
          .get_or_init(|| Mutex::new(HashMap::new()))
          .lock()
          .unwrap()
          .insert(stream_id, stream);

        let json: String = {
          let ctx_lock = ctx.lock().unwrap();
          serde_json::to_string(&*ctx_lock).unwrap()
        };

        Python::attach(|py| {
          if let Err(err) = py_cb.call1(py, (stream_id, json)) {
            err.print(py);
          }
        });

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
  fn http2_push_bytes(
    py_stream_id: usize,
    py_bytes: &[u8],
    py_is_attachment: usize,
  ) -> PyResult<()> {
    if let Some(map) = HTTP2_STREAMS.get() {
      let streams: MutexGuard<'_, Http2Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let is_attachment: bool = py_is_attachment == 1;
          if py_bytes.is_empty() {
            let mut stream_lock: std::sync::MutexGuard<'_, Http2Stream> = stream.lock().unwrap();
            stream_lock.push_bytes(&[], is_attachment);
            return Ok(());
          }

          let mut stream_lock: std::sync::MutexGuard<'_, Http2Stream> = stream.lock().unwrap();
          stream_lock.push_bytes(py_bytes, is_attachment);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http2_push_bytes: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http2_push_file(
    py_stream_id: usize,
    py_file_path: &str,
    py_is_attachment: usize,
  ) -> PyResult<()> {
    if let Some(map) = HTTP2_STREAMS.get() {
      let streams: MutexGuard<'_, Http2Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let is_attachment: bool = py_is_attachment == 1;
          let mut stream_lock: std::sync::MutexGuard<'_, Http2Stream> = stream.lock().unwrap();
          stream_lock.push_file(py_file_path, is_attachment);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http2_push_file: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http2_push_json(py_stream_id: usize, py_json: &str, py_is_attachment: usize) -> PyResult<()> {
    if let Some(map) = HTTP2_STREAMS.get() {
      let streams: MutexGuard<'_, Http2Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let is_attachment: bool = py_is_attachment == 1;
          let json: JSON = match serde_json::from_str(py_json) {
            Ok(json) => json,
            Err(_) => {
              println!(
                "[Arnelify Server]: Rust PYO3 error in http2_push_json: Invalid JSON in 'py_json'."
              );

              return Ok(());
            }
          };

          let mut stream_lock: std::sync::MutexGuard<'_, Http2Stream> = stream.lock().unwrap();
          stream_lock.push_json(&json, is_attachment);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http2_push_json: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http2_set_code(py_stream_id: usize, py_code: usize) -> PyResult<()> {
    if let Some(map) = HTTP2_STREAMS.get() {
      let streams: MutexGuard<'_, Http2Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let mut stream_lock: std::sync::MutexGuard<'_, Http2Stream> = stream.lock().unwrap();
          stream_lock.set_code(py_code as u16);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http2_set_code: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http2_set_compression(py_stream_id: usize, py_compression: &str) -> PyResult<()> {
    if let Some(map) = HTTP2_STREAMS.get() {
      let streams: MutexGuard<'_, Http2Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let mut stream_lock: std::sync::MutexGuard<'_, Http2Stream> = stream.lock().unwrap();
          if py_compression.len() > 0 {
            stream_lock.set_compression(Some(String::from(py_compression)));
            return Ok(());
          }

          stream_lock.set_compression(None);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http2_set_compression: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http2_set_headers(py_stream_id: usize, py_headers: &str) -> PyResult<()> {
    if let Some(map) = HTTP2_STREAMS.get() {
      let streams: MutexGuard<'_, Http2Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let json: Vec<JSON> = match serde_json::from_str(py_headers) {
            Ok(json) => json,
            Err(_) => {
              println!(
                "[Arnelify Server]: Rust PYO3 error in http2_set_headers: Invalid JSON in 'py_headers'."
              );
              return Ok(());
            }
          };

          let mut headers: Vec<(String, String)> = Vec::new();
          for header in json {
            if let JSON::Object(pair) = header {
              for (key, value) in pair {
                let value = match value {
                  JSON::String(s) => s,
                  JSON::Number(n) => n.to_string(),
                  JSON::Bool(b) => b.to_string(),
                  _ => continue,
                };
                headers.push((key, value));
              }
            }
          }

          let mut stream_lock: std::sync::MutexGuard<'_, Http2Stream> = stream.lock().unwrap();
          stream_lock.set_headers(headers);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http2_set_headers: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http2_start(py_id: usize) -> PyResult<()> {
    if let Some(map) = HTTP2_MAP.get() {
      if let Some(http2) = map.lock().unwrap().get(&py_id) {
        let http2 = http2.clone();
        std::thread::spawn(move || {
          http2.start();
        });
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http2_stop(py_id: usize) -> PyResult<()> {
    if let Some(map) = HTTP2_MAP.get() {
      if let Some(http2) = map.lock().unwrap().get(&py_id) {
        http2.stop();
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn ws_close(py_stream_id: usize) -> PyResult<()> {
    if let Some(map) = WS_STREAMS.get() {
      let streams: MutexGuard<'_, WebSocketStreams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let mut stream_lock: std::sync::MutexGuard<'_, WebSocketStream> = stream.lock().unwrap();
          stream_lock.close();
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in ws_close: No stream found.");
        }
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

    Ok(())
  }

  #[pyfunction]
  fn ws_logger(py_id: usize, py_cb: Py<PyAny>) -> PyResult<()> {
    let ws_logger: Arc<WebSocketLogger> = Arc::new(move |level: &str, message: &str| {
      Python::attach(|py| {
        if let Err(err) = py_cb.call1(py, (level, message)) {
          err.print(py);
        }
      });
    });

    if let Some(map) = WS_MAP.get() {
      if let Some(ws) = map.lock().unwrap().get(&py_id) {
        ws.logger(ws_logger);
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn ws_on(py_id: usize, py_topic: &str, py_cb: Py<PyAny>) -> PyResult<()> {
    let ws_handler: Arc<WebSocketHandler> = Arc::new(
      move |ctx: Arc<Mutex<WebSocketCtx>>,
            bytes: Arc<Mutex<WebSocketBytes>>,
            stream: Arc<Mutex<WebSocketStream>>| {
        let stream_id: usize = WS_STREAM_ID.fetch_add(1, Ordering::Relaxed);

        WS_STREAMS
          .get_or_init(|| Mutex::new(HashMap::new()))
          .lock()
          .unwrap()
          .insert(stream_id, stream);

        let json: String = {
          let ctx_lock = ctx.lock().unwrap();
          serde_json::to_string(&*ctx_lock).unwrap()
        };

        let data: Vec<u8> = {
          let locked: MutexGuard<'_, Vec<u8>> = bytes.lock().unwrap();
          locked.clone()
        };

        Python::attach(|py| {
          let py_bytes: Bound<'_, PyBytes> = PyBytes::new(py, &data);
          if let Err(err) = py_cb.call1(py, (stream_id, json, py_bytes)) {
            err.print(py);
          }
        });

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
  fn ws_push(py_stream_id: usize, py_json: &str, py_bytes: &[u8]) -> PyResult<()> {
    if let Some(map) = WS_STREAMS.get() {
      let streams: MutexGuard<'_, WebSocketStreams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let json: JSON = match serde_json::from_str(py_json) {
            Ok(json) => json,
            Err(_) => {
              println!(
                "[Arnelify Server]: Rust PYO3 error in ws_push: Invalid JSON in 'py_json'."
              );

              return Ok(());
            }
          };

          if py_bytes.is_empty() {
            let mut stream_lock: std::sync::MutexGuard<'_, WebSocketStream> =
              stream.lock().unwrap();
            stream_lock.push(&json, &[]);
            return Ok(());
          }

          let mut stream_lock: std::sync::MutexGuard<'_, WebSocketStream> = stream.lock().unwrap();
          stream_lock.push(&json, py_bytes);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in ws_push: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn ws_push_bytes(py_stream_id: usize, py_bytes: &[u8]) -> PyResult<()> {
    if let Some(map) = WS_STREAMS.get() {
      let streams: MutexGuard<'_, WebSocketStreams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          if py_bytes.is_empty() {
            let mut stream_lock: std::sync::MutexGuard<'_, WebSocketStream> =
              stream.lock().unwrap();
            stream_lock.push_bytes(&[]);
            return Ok(());
          }

          let mut stream_lock: std::sync::MutexGuard<'_, WebSocketStream> = stream.lock().unwrap();
          stream_lock.push_bytes(py_bytes);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in ws_push_bytes: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn ws_push_json(py_stream_id: usize, py_json: &str) -> PyResult<()> {
    if let Some(map) = WS_STREAMS.get() {
      let streams: MutexGuard<'_, WebSocketStreams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let json: JSON = match serde_json::from_str(py_json) {
            Ok(json) => json,
            Err(_) => {
              println!(
                "[Arnelify Server]: Rust PYO3 error in ws_push_json: Invalid JSON in 'py_json'."
              );

              return Ok(());
            }
          };

          let mut stream_lock: std::sync::MutexGuard<'_, WebSocketStream> = stream.lock().unwrap();
          stream_lock.push_json(&json);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in ws_push_json: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn ws_set_compression(py_stream_id: usize, py_compression: &str) -> PyResult<()> {
    if let Some(map) = WS_STREAMS.get() {
      let streams: MutexGuard<'_, WebSocketStreams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let mut stream_lock: std::sync::MutexGuard<'_, WebSocketStream> = stream.lock().unwrap();
          if py_compression.len() > 0 {
            stream_lock.set_compression(Some(String::from(py_compression)));
            return Ok(());
          }

          stream_lock.set_compression(None);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in ws_set_compression: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn ws_start(py_id: usize) -> PyResult<()> {
    if let Some(map) = WS_MAP.get() {
      if let Some(ws) = map.lock().unwrap().get(&py_id) {
        let ws = ws.clone();
        std::thread::spawn(move || {
          ws.start();
        });
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn ws_stop(py_id: usize) -> PyResult<()> {
    if let Some(map) = WS_MAP.get() {
      if let Some(ws) = map.lock().unwrap().get(&py_id) {
        ws.stop();
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http3_add_header(py_stream_id: usize, py_key: &str, py_value: &str) -> PyResult<()> {
    if let Some(map) = HTTP3_STREAMS.get() {
      let streams: MutexGuard<'_, Http3Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let mut stream_lock: std::sync::MutexGuard<'_, Http3Stream> = stream.lock().unwrap();
          stream_lock.add_header(py_key, py_value);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http3_add_header: No stream found.");
        }
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
          println!("[Arnelify Server]: Rust PYO3 error in http3_create: Invalid JSON in 'py_opts'.");
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

    Ok(())
  }

  #[pyfunction]
  fn http3_end(py_stream_id: usize) -> PyResult<()> {
    if let Some(map) = HTTP3_STREAMS.get() {
      let streams: MutexGuard<'_, Http3Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let mut stream_lock: std::sync::MutexGuard<'_, Http3Stream> = stream.lock().unwrap();
          stream_lock.end();
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http3_end: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http3_logger(py_id: usize, py_cb: Py<PyAny>) -> PyResult<()> {
    let http3_logger: Arc<Http3Logger> = Arc::new(move |level: &str, message: &str| {
      Python::attach(|py| {
        if let Err(err) = py_cb.call1(py, (level, message)) {
          err.print(py);
        }
      });
    });

    if let Some(map) = HTTP3_MAP.get() {
      if let Some(http3) = map.lock().unwrap().get(&py_id) {
        http3.logger(http3_logger);
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http3_on(py_id: usize, py_path: &str, py_cb: Py<PyAny>) -> PyResult<()> {
    let http3_handler: Arc<Http3Handler> = Arc::new(
      move |ctx: Arc<Mutex<Http3Ctx>>, stream: Arc<Mutex<Http3Stream>>| {
        let stream_id: usize = HTTP3_STREAM_ID.fetch_add(1, Ordering::Relaxed);

        HTTP3_STREAMS
          .get_or_init(|| Mutex::new(HashMap::new()))
          .lock()
          .unwrap()
          .insert(stream_id, stream);

        let json: String = {
          let ctx_lock = ctx.lock().unwrap();
          serde_json::to_string(&*ctx_lock).unwrap()
        };

        Python::attach(|py| {
          if let Err(err) = py_cb.call1(py, (stream_id, json)) {
            err.print(py);
          }
        });

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
  fn http3_push_bytes(
    py_stream_id: usize,
    py_bytes: &[u8],
    py_is_attachment: usize,
  ) -> PyResult<()> {
    if let Some(map) = HTTP3_STREAMS.get() {
      let streams: MutexGuard<'_, Http3Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let is_attachment: bool = py_is_attachment == 1;
          if py_bytes.is_empty() {
            let mut stream_lock: std::sync::MutexGuard<'_, Http3Stream> = stream.lock().unwrap();
            stream_lock.push_bytes(&[], is_attachment);
            return Ok(());
          }

          let mut stream_lock: std::sync::MutexGuard<'_, Http3Stream> = stream.lock().unwrap();
          stream_lock.push_bytes(py_bytes, is_attachment);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http3_push_bytes: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http3_push_file(
    py_stream_id: usize,
    py_file_path: &str,
    py_is_attachment: usize,
  ) -> PyResult<()> {
    if let Some(map) = HTTP3_STREAMS.get() {
      let streams: MutexGuard<'_, Http3Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let is_attachment: bool = py_is_attachment == 1;
          let mut stream_lock: std::sync::MutexGuard<'_, Http3Stream> = stream.lock().unwrap();
          stream_lock.push_file(py_file_path, is_attachment);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http3_push_file: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http3_push_json(py_stream_id: usize, py_json: &str, py_is_attachment: usize) -> PyResult<()> {
    if let Some(map) = HTTP3_STREAMS.get() {
      let streams: MutexGuard<'_, Http3Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let is_attachment: bool = py_is_attachment == 1;
          let json: JSON = match serde_json::from_str(py_json) {
            Ok(json) => json,
            Err(_) => {
              println!(
                "[Arnelify Server]: Rust PYO3 error in http3_push_json: Invalid JSON in 'py_json'."
              );

              return Ok(());
            }
          };

          let mut stream_lock: std::sync::MutexGuard<'_, Http3Stream> = stream.lock().unwrap();
          stream_lock.push_json(&json, is_attachment);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http3_push_json: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http3_set_code(py_stream_id: usize, py_code: usize) -> PyResult<()> {
    if let Some(map) = HTTP3_STREAMS.get() {
      let streams: MutexGuard<'_, Http3Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let mut stream_lock: std::sync::MutexGuard<'_, Http3Stream> = stream.lock().unwrap();
          stream_lock.set_code(py_code as u16);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http3_set_code: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http3_set_compression(py_stream_id: usize, py_compression: &str) -> PyResult<()> {
    if let Some(map) = HTTP3_STREAMS.get() {
      let streams: MutexGuard<'_, Http3Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let mut stream_lock: std::sync::MutexGuard<'_, Http3Stream> = stream.lock().unwrap();
          if py_compression.len() > 0 {
            stream_lock.set_compression(Some(String::from(py_compression)));
            return Ok(());
          }

          stream_lock.set_compression(None);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http3_set_compression: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http3_set_headers(py_stream_id: usize, py_headers: &str) -> PyResult<()> {
    if let Some(map) = HTTP3_STREAMS.get() {
      let streams: MutexGuard<'_, Http3Streams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let json: Vec<JSON> = match serde_json::from_str(py_headers) {
            Ok(json) => json,
            Err(_) => {
              println!(
                "[Arnelify Server]: Rust PYO3 error in http3_set_headers: Invalid JSON in 'py_headers'."
              );
              return Ok(());
            }
          };

          let mut headers: Vec<(String, String)> = Vec::new();
          for header in json {
            if let JSON::Object(pair) = header {
              for (key, value) in pair {
                let value = match value {
                  JSON::String(s) => s,
                  JSON::Number(n) => n.to_string(),
                  JSON::Bool(b) => b.to_string(),
                  _ => continue,
                };
                headers.push((key, value));
              }
            }
          }

          let mut stream_lock: std::sync::MutexGuard<'_, Http3Stream> = stream.lock().unwrap();
          stream_lock.set_headers(headers);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in http3_set_headers: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http3_start(py_id: usize) -> PyResult<()> {
    if let Some(map) = HTTP3_MAP.get() {
      if let Some(http3) = map.lock().unwrap().get(&py_id) {
        let http3 = http3.clone();
        std::thread::spawn(move || {
          http3.start();
        });
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn http3_stop(py_id: usize) -> PyResult<()> {
    if let Some(map) = HTTP3_MAP.get() {
      if let Some(http3) = map.lock().unwrap().get(&py_id) {
        http3.stop();
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn wt_close(py_stream_id: usize) -> PyResult<()> {
    if let Some(map) = WT_STREAMS.get() {
      let streams: MutexGuard<'_, WebTransportStreams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let mut stream_lock: std::sync::MutexGuard<'_, WebTransportStream> =
            stream.lock().unwrap();
          stream_lock.close();
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in wt_close: No stream found.");
        }
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
  fn wt_logger(py_id: usize, py_cb: Py<PyAny>) -> PyResult<()> {
    let wt_logger: Arc<WebTransportLogger> = Arc::new(move |level: &str, message: &str| {
      Python::attach(|py| {
        if let Err(err) = py_cb.call1(py, (level, message)) {
          err.print(py);
        }
      });
    });

    if let Some(map) = WT_MAP.get() {
      if let Some(wt) = map.lock().unwrap().get(&py_id) {
        wt.logger(wt_logger);
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn wt_on(py_id: usize, py_topic: &str, py_cb: Py<PyAny>) -> PyResult<()> {
    let wt_handler: Arc<WebTransportHandler> = Arc::new(
      move |ctx: Arc<Mutex<WebTransportCtx>>,
            bytes: Arc<Mutex<WebTransportBytes>>,
            stream: Arc<Mutex<WebTransportStream>>| {
        let stream_id: usize = WT_STREAM_ID.fetch_add(1, Ordering::Relaxed);

        WT_STREAMS
          .get_or_init(|| Mutex::new(HashMap::new()))
          .lock()
          .unwrap()
          .insert(stream_id, stream);

        let json: String = {
          let ctx_lock = ctx.lock().unwrap();
          serde_json::to_string(&*ctx_lock).unwrap()
        };

        let data: Vec<u8> = {
          let locked: MutexGuard<'_, Vec<u8>> = bytes.lock().unwrap();
          locked.clone()
        };

        Python::attach(|py| {
          let py_bytes: Bound<'_, PyBytes> = PyBytes::new(py, &data);
          if let Err(err) = py_cb.call1(py, (stream_id, json, py_bytes)) {
            err.print(py);
          }
        });

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
  fn wt_push(py_stream_id: usize, py_json: &str, py_bytes: &[u8]) -> PyResult<()> {
    if let Some(map) = WT_STREAMS.get() {
      let streams: MutexGuard<'_, WebTransportStreams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let json: JSON = match serde_json::from_str(py_json) {
            Ok(json) => json,
            Err(_) => {
              println!(
                "[Arnelify Server]: Rust PYO3 error in wt_push: Invalid JSON in 'py_json'."
              );

              return Ok(());
            }
          };

          if py_bytes.is_empty() {
            let mut stream_lock: std::sync::MutexGuard<'_, WebTransportStream> =
              stream.lock().unwrap();
            stream_lock.push(&json, &[]);
            return Ok(());
          }

          let mut stream_lock: std::sync::MutexGuard<'_, WebTransportStream> =
            stream.lock().unwrap();
          stream_lock.push(&json, py_bytes);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in wt_push: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn wt_push_bytes(py_stream_id: usize, py_bytes: &[u8]) -> PyResult<()> {
    if let Some(map) = WT_STREAMS.get() {
      let streams: MutexGuard<'_, WebTransportStreams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          if py_bytes.is_empty() {
            let mut stream_lock: std::sync::MutexGuard<'_, WebTransportStream> =
              stream.lock().unwrap();
            stream_lock.push_bytes(&[]);
            return Ok(());
          }

          let mut stream_lock: std::sync::MutexGuard<'_, WebTransportStream> =
            stream.lock().unwrap();
          stream_lock.push_bytes(py_bytes);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in wt_push_bytes: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn wt_push_json(py_stream_id: usize, py_json: &str) -> PyResult<()> {
    if let Some(map) = WT_STREAMS.get() {
      let streams: MutexGuard<'_, WebTransportStreams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let json: JSON = match serde_json::from_str(py_json) {
            Ok(json) => json,
            Err(_) => {
              println!(
                "[Arnelify Server]: Rust PYO3 error in wt_push_json: Invalid JSON in 'py_json'."
              );

              return Ok(());
            }
          };

          let mut stream_lock: std::sync::MutexGuard<'_, WebTransportStream> =
            stream.lock().unwrap();
          stream_lock.push_json(&json);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in wt_push_json: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn wt_set_compression(py_stream_id: usize, py_compression: &str) -> PyResult<()> {
    if let Some(map) = WT_STREAMS.get() {
      let streams: MutexGuard<'_, WebTransportStreams> = map.lock().unwrap();
      match streams.get(&py_stream_id) {
        Some(stream) => {
          let mut stream_lock: std::sync::MutexGuard<'_, WebTransportStream> =
            stream.lock().unwrap();
          if py_compression.len() > 0 {
            stream_lock.set_compression(Some(String::from(py_compression)));
            return Ok(());
          }

          stream_lock.set_compression(None);
        }
        None => {
          println!("[Arnelify Server]: Rust PYO3 error in wt_set_compression: No stream found.");
        }
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn wt_start(py_id: usize) -> PyResult<()> {
    if let Some(map) = WT_MAP.get() {
      if let Some(wt) = map.lock().unwrap().get(&py_id) {
        let wt = wt.clone();
        std::thread::spawn(move || {
          wt.start();
        });
      }
    }

    Ok(())
  }

  #[pyfunction]
  fn wt_stop(py_id: usize) -> PyResult<()> {
    if let Some(map) = WT_MAP.get() {
      if let Some(wt) = map.lock().unwrap().get(&py_id) {
        wt.stop();
      }
    }

    Ok(())
  }
}
