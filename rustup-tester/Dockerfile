FROM ubuntu:17.10

RUN echo cache-clear-1
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update
RUN apt-get install -y --no-install-recommends \
    curl libssl-dev ca-certificates wget less sudo \
    build-essential cmake pkg-config \
    libcairo2-dev libglib2.0-dev libpango* m4 \
    liblzma-dev libnss3-dev git

RUN useradd -ms /bin/bash tester
RUN usermod -aG sudo tester
RUN echo "tester ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers
USER tester
WORKDIR /home/tester

RUN curl https://sh.rustup.rs -sSf -o rustup-installer
RUN sh ./rustup-installer --default-toolchain none -y
ENV PATH="/home/tester/.cargo/bin:$PATH"
RUN export PATH

RUN wget https://gist.githubusercontent.com/nikomatsakis/75e09ed314bea715e192ce226127d3c6/raw/9dcdc359a7a8435677d45ea8a17153a219913090/cargo-curl
RUN chmod u+x cargo-curl

RUN wget https://gist.githubusercontent.com/nikomatsakis/81e50fdf7254da8870c682109c404694/raw/d364e5d69809c54bdd2694f6bd304c0032de1552/bors-curl
RUN chmod u+x bors-curl

RUN echo cache-clear-1

RUN rustup update stable
RUN rustup update beta
RUN rustup update nightly
