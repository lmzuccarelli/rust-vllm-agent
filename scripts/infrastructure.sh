#!/bin/bash

set -x

USER="${REMOTE_USER:-lzuccarelli}"
PK="${PK_ID:?PK_ID environment variable must be set}"
MS="vllm-ai-agent"
DESCRIPTION="A local llama.cpp agent interface written in Rust"
REPO="git@github.com:lmzuccarelli/rust-vllm-agent.git"
REPO_NAME="rust-vllm-agent"
CLEAN=$2

create_configs() {
tee config/${MS}-config.json <<EOF
{
	"name": "${MS}",
	"description": "${DESCRIPTION}",
	"log_level": "debug",
	"certs_dir": "/home/${USER}/certs",
	"cert_mode": "file",
	"base_url": "https://vinnie/vllm/",
	"db_path" :"/home/${USER}/database",
	"test": false
}
EOF
}

clone_build_service() {
  HOSTS=("george")
  for host in "${HOSTS[@]}"; do
    ssh -i "${PK}" "${USER}@${host}" -t "rm -rf /home/${USER}/services/${MS}"
    eval `ssh-agent`
    ssh-add ~/.ssh/id_ed25519-lz
    if [ "${CLEAN}" == "true" ];
    then
      ssh -i "${PK}" "${USER}@${host}" -tA "rm -rf /home/${USER}/Projects/${REPO_NAME} && cd /home/${USER}/Projects && git clone ${REPO} && cd ${REPO_NAME} && make build"
    else 
      ssh -i "${PK}" "${USER}@${host}" -tA "cd /home/lzuccarelli/Projects/${REPO_NAME} && rm -rf target/release/*vllm* && git pull origin main --rebase && make build"
    fi
  done
}

deploy_service() {
  HOSTS=("george")
  for host in "${HOSTS[@]}"; do
    scp -i "${PK}" config/* "${USER}@${host}:/home/${USER}/services"
    ssh -i "${PK}" "${USER}@${host}" -t "cp /home/${USER}/Projects/${REPO_NAME}/target/release/${MS} /home/${USER}/services/${MS}"
  done
}

"$@"

