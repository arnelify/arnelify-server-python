# Boot Makefile
# See https://www.gnu.org/software/make/manual/make.html for more about make.

# PATH
PATH_BIN = $(CURDIR)/tests/bin/index.bin
PATH_SRC_HTTP1 = $(CURDIR)/tests/tcp1/http1.py
PATH_SRC_WS = $(CURDIR)/tests/tcp1/ws.py
PATH_SRC_HTTP2 = $(CURDIR)/tests/tcp2/http2.py
PATH_SRC_HTTP3 = $(CURDIR)/tests/tcp2/http3.py
PATH_SRC_WT = $(CURDIR)/tests/tcp2/wt.py

# PYTHON
PYTHON = venv/bin/python
PYTHON_FLAGS = -m
PYPI = venv/bin/pip

# NUITKA
NUITKA = venv/bin/nuitka
NUITKA_FLAGS = --standalone --onefile

install:
	clear && find . -type d -name '__pycache__' -exec rm -r {} +
	rm -rf venv
	python ${PYTHON_FLAGS} venv venv
	${PYPI} install -r requirements.txt

build:
	clear && rm -rf target/* && rm -rf arnelify_server/*.so
	maturin develop
	maturin build --release

test_http1:
	clear && mkdir -p tests/bin && rm -rf tests/bin/*
	${PYTHON} ${PYTHON_FLAGS} tests.tcp1.http1

test_http2:
	clear && mkdir -p tests/bin && rm -rf tests/bin/*
	${PYTHON} ${PYTHON_FLAGS} tests.tcp1.http2

test_ws:
	clear && mkdir -p tests/bin && rm -rf tests/bin/*
	${PYTHON} ${PYTHON_FLAGS} tests.tcp1.ws

test_http3:
	clear && mkdir -p tests/bin && rm -rf tests/bin/*
	${PYTHON} ${PYTHON_FLAGS} tests.tcp2.http3

test_wt:
	clear && mkdir -p tests/bin && rm -rf tests/bin/*
	${PYTHON} ${PYTHON_FLAGS} tests.tcp2.wt

nuitka_http1:
	clear && mkdir -p tests/bin && rm -rf tests/bin/*
	${NUITKA} ${NUITKA_FLAGS} --output-dir=tests/bin ${PATH_SRC_HTTP1} && clear
	${PATH_BIN}

nuitka_ws:
	clear && mkdir -p tests/bin && rm -rf tests/bin/*
	${NUITKA} ${NUITKA_FLAGS} --output-dir=tests/bin ${PATH_SRC_WS} && clear
	${PATH_BIN}

nuitka_http2:
	clear && mkdir -p tests/bin && rm -rf tests/bin/*
	${NUITKA} ${NUITKA_FLAGS} --output-dir=tests/bin ${PATH_SRC_HTTP2} && clear
	${PATH_BIN}

nuitka_http3:
	clear && mkdir -p tests/bin && rm -rf tests/bin/*
	${NUITKA} ${NUITKA_FLAGS} --output-dir=tests/bin ${PATH_SRC_HTTP3} && clear
	${PATH_BIN}

nuitka_wt:
	clear && mkdir -p tests/bin && rm -rf tests/bin/*
	${NUITKA} ${NUITKA_FLAGS} --output-dir=tests/bin ${PATH_SRC_WT} && clear
	${PATH_BIN}

.PHONY: \
  install \
  build \
  test_http1 \
  test_ws \
  test_http2 \
  test_http3 \
  test_wt \
  nuitka_http1 \
  nuitka_ws \
  nuitka_http2 \
  nuitka_http3 \
  nuitka_wt