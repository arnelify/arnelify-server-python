from setuptools import setup, find_packages, Extension

ffi = Extension(
  'arnelify-server-ffi',
  sources=['arnelify_server/src/ffi.cpp'],
  language='c++',
  extra_compile_args=['-std=c++2b', '-w'],
  include_dirs=['arnelify_server/src/include', '/usr/include', '/usr/include/jsoncpp/json'],
  extra_link_args=['-ljsoncpp', '-lz']
)

setup(
  name="arnelify_server",
  version="0.7.5",
  author="Arnelify",
  description="Minimalistic dynamic library which is a powerful server written in C and C++.",
  url='https://github.com/arnelify/arnelify-server-python',
  keywords="arnelify arnelify-server-python arnelify-server",
  packages=find_packages(),
  license="MIT",
  install_requires=["cffi", "setuptools", "wheel"],
  long_description=open("README.md", "r", encoding="utf-8").read(),
  long_description_content_type="text/markdown",
  classifiers=[
        "Programming Language :: Python :: 3"
  ],
  ext_modules=[ffi],
)