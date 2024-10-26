#!/bin/bash

# Verifica se o número de usuários foi passado como argumento
if [ -z "$1" ] || [ "$1" != "-u" ] || [ -z "$2" ]; then
  echo "Uso: $0 -u <quantidade_de_usuarios>"
  exit 1
fi

# Número de usuários a serem criados
NUM_USERS=$2

# Endpoint da API
URL="http://localhost:8080/users"

# Função para criar um usuário
create_user() {
  local name="User$1"
  local email="user$1@example.com"

  # Envia a requisição POST para criar o usuário
  curl -s -X POST "$URL" \
    -H "Content-Type: application/json" \
    -d "{\"name\": \"$name\", \"email\": \"$email\"}"

  echo "Usuário $name criado."
}

# Loop para criar os usuários conforme o número especificado
for i in $(seq 1 $NUM_USERS); do
  create_user "$i"
done

# Requisição GET para listar todos os usuários
echo -e "\nLista de todos os usuários cadastrados:"
curl -s -X GET "$URL" | jq '.'

