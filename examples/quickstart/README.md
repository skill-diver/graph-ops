# Ofnil Quickstart

This is a quickstart guide for Ofnil. It will help you get started with Ofnil.

## Table of Contents

1. [Environment Setup](#step-1-environment-setup)  

    - [Option 1: Use Docker](#option-1-use-docker)
    - [Option 2: Install Manually](#option-2-install-manually)

2. [Loading Data](#step-2-loading-data)

3. [Run the Demo Code](#step-3-run-the-demo-code)

## Step 1: Environment Setup

> There are two environment setup methods, and we recommend using Docker.
---

### Option 1: Use Docker

#### 1. Install Docker and Docker Compose

Before starting, make sure you have installed

- [Docker & Docker Compose](https://docs.docker.com/engine/install/)

#### 2. Build and start the Docker containers

You will need to clone the Ofnil repository first:

```bash

git clone https://github.com/ofnil/ofnil.git
cd ofnil
tar -xzvf examples/quickstart/neo4j_env.tar.gz -C examples/quickstart/
```

Then you can build the image and start the container via:

```bash

docker-compose --file examples/quickstart/docker-compose.yml build
docker-compose --file examples/quickstart/docker-compose.yml up
```

#### 3. Development options

Once you have launched the container, you can choose one of the following development options:

Option 1: Command-line development (recommended for Vim users)

```bash

docker exec -it ofnil-quickstart /bin/bash
```

Option 2: Visual Studio Code development (recommended for VSCode users)

1. Install the `Remote Development` extension in Visual Studio Code.
2. Attach to the container `ofnil-quickstart` from Remote Explorer > Dev Containers.
3. Open path `/ofnil` in the container from VSCode.

> For Windows users, use the WSL2 backend as recommended when installing Docker.

#### Frequently Asked Questions

1. If you cannot pull the image (Error response from daemon: Get "[https://registry-1.docker.io/v2/](https://registry-1.docker.io/v2/) ": net/http: request canceled (Client.Timeout exceeded while awaiting headers)), check if a proxy setting is needed. If the problem is with the mirror, try adding a mirror by adding "registry-mirrors":["[http://f1361db2.m.daocloud.io](http://f1361db2.m.daocloud.io/) "] in daemon.json. See [here](https://docs.docker.com/config/daemon/)  for the config details. Another workaround is to try adding an access token as suggested by this [post](https://forums.docker.com/t/windows-10-home-error-response-from-daemon-get-https-registry-1-docker-io-v2-net-http-request-canceled-while-waiting-for-connection-client-timeout-exceeded-while-awaiting-headers/104640).
2. If your Docker uses too much memory, or your containers exit with code 137, consider configuring your WSL2 backend by following this [guide](https://learn.microsoft.com/en-us/windows/wsl/wsl-config). For example,

```Bash
[wsl2]
memory=5GB
processors=4
```

---

### Option 2: Install Manually

The procedure for using Ofnil manually is as follows:

1. Set up the infrastructure you want to use, e.g., Neo4j, Redis, etc.
2. Define the graph features you want to use (i.e., `main.rs`), then deploy them (by executing `main.rs`).
3. Enjoy all the features you defined and the subgraph sampler provided by Ofnil (i.e., `example.py`).

#### Prerequisites

To run the quickstart, you need to install the following tools:

1. Install [Rust]([https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install) ) toolchain. We recommend using the latest stable version.
2. Install [Protobuf compiler](https://developers.google.com/protocol-buffers/docs/downloads) . We used version 3.20 during development.
3. Install [Neo4j](https://neo4j.com/download/) . We used the community version 4.4 during development.
4. Install [Etcd](https://etcd.io/docs/latest/install/) . We used version 3.5 during development.
5. Install [Redis](https://redis.io/download) . We used version 7.0 during development.
6. Install Python 3.7 or above. We used version 3.9 during development. You can use [miniconda](https://docs.conda.io/en/latest/miniconda.html)  to manage Python environments. Required Python packages are listed in `pyproject.toml`. Additional required Python packages for development are `maturin`, `pytest`.

#### Build

```bash

# cd to the root directory of the project
cargo build
maturin build
pip install target/wheels/ofnil*.whl  # --force-reinstall if you want to update the package
```

#### Configuration

1. Start Neo4j, Etcd, Redis, and fill in their configurations (listening port and username/password if applicable) in `examples/quickstart/ofnil.toml` and `examples/quickstart/.env`. You can refer to the current `ofnil.toml` and `.env.example` for examples.

Currently supported `infra_type`:

- neo4j
- redis

For each infra, you will need to give it a name (for registration) and its corresponding required connection info. For example, to add a Neo4j instance, append the following to the `ofnil.toml`:

```toml

[[infra]]
name = "neo4j_1"
infra_type = "neo4j"

# properties with `env_` prefix means that the value will be read from the environment variable (or `.env` file)

env_uri = "NEO4J_1_URI"

# properties without `env_` prefix means that the value will be directly read from this config file

username = "neo4j"
env_password = "NEO4J_1_PASSWORD"
```

And provide the corresponding `env_*` environment variable in `.env`:

```bash

NEO4J_1_URI = 'bolt://localhost:7687'
NEO4J_1_PASSWORD = neo4j
```

## Step 2: Loading Data

The scripts for preparing the data are in the [repo](https://github.com/TatianaJin/amazon_product_review_neo4j).

## Step 3: Run the Demo Code

### Graph Features Definition and Transformation Execution

As shown in the example code in `main.rs`, the whole process of graph feature definition can be divided into these steps

1. Register graph schema in the graph database. The concepts of `Graph`, `Entity`, `Field`, and so on can be found in `../../docs/concepts.md`.
2. Define graph feature logic. Refer to `fn graph_feature_engineering()` for details.
3. Determine which features to serve. Refer to `fn graph_feature_serving()` for details.
4. Trigger the feature engineering transformation execution and serving process. Refer to `fn deploy()` of `FeatureStore` for details.

```bash

# cd to the root directory of the project
RUST_LOG=info cargo run --example quickstart
```

### Graph Features Serving

We provide a Python script `examples/quickstart/example.py` to demonstrate how to consume graph features. You can run it with:

```bash

python example.py
```

In this example, we provide a DataLoader to load graph features from Ofnil. For graph topology, it will sample a subgraph from the graph database. For graph features, it will fetch the features defined in Ofnil.
