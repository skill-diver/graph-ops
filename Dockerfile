FROM ubuntu:22.04 as builder

ENV LANG en_US.utf8
ENV DEBIAN_FRONTEND noninteractive

RUN chmod 777 /tmp

RUN apt-get update

RUN apt-get -y install make build-essential cmake protobuf-compiler curl pkg-config bash lld wget vim git htop

SHELL ["/bin/bash", "-c"]

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --no-modify-path --default-toolchain none -y

# Setup Rust
ENV PATH /root/.cargo/bin/:$PATH
RUN rustup self update \
  && rustup set profile minimal \
  && rustup default stable \
  && rustup component add rustfmt

# Install conda
RUN wget --quiet https://repo.anaconda.com/miniconda/Miniconda3-py310_23.1.0-1-Linux-x86_64.sh -O ~/miniconda.sh && \
  /bin/bash ~/miniconda.sh -b -p /opt/conda && \
  rm ~/miniconda.sh && \
  ln -s /opt/conda/etc/profile.d/conda.sh /etc/profile.d/conda.sh && \
  echo ". /opt/conda/etc/profile.d/conda.sh" >> ~/.bashrc && \
  echo "conda activate base" >> ~/.bashrc

SHELL ["/bin/bash","-ic"]
RUN source ~/.bashrc && \
  conda update conda


# Install Pytest
RUN pip install pytest
# Install PyTorch
RUN conda install pytorch==1.13 cpuonly -c pytorch
# Install DGL
RUN conda install -y -c dglteam dgl
# Install PyG
RUN conda install -y pyg -c pyg
# Install Maturin
RUN pip install maturin 


COPY ./ /ofnil

WORKDIR /ofnil
RUN maturin develop
