FROM gcc:15.2.0

WORKDIR /var/www/python

ENV TZ=Europe/Kiev
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:$PATH

RUN apt-get update -y && apt-get upgrade -y
RUN apt-get install sudo ccache python3-venv python3-setuptools -y
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
  | sh -s -- -y --no-modify-path --default-toolchain 1.92.0  
RUN sudo ln -sf $(which python3) /usr/bin/python

EXPOSE 4433