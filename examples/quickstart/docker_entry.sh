#!/bin/bash

cat << EOF > examples/quickstart/.env
NEO4J_1_URI = 'bolt://neo4j:7687'
NEO4J_1_PASSWORD = ofnil
EOF

cat << EOF > examples/quickstart/ofnil.toml
project = "ofnil-quickstart"
registry_endpoints = ["http://etcd:2379"]  # etcd endpoints
[[infra]]
name = "neo4j_1"
infra_type = "neo4j"
env_uri = "NEO4J_1_URI"
username = "neo4j"
env_password = "NEO4J_1_PASSWORD"
[[infra]]
name = "redis"
infra_type = "redis"
uri = "redis://redis:6379"
EOF

tail -f
