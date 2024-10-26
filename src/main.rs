use postgres::{Client, NoTls};
use postgres::Error as PostgresError;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::env;

#[macro_use]
extern crate serde_derive;

// Model: Struct User com id, name e email
#[derive(Serialize, Deserialize)]
struct User {
    id: Option<i32>,
    name: String,
    email: String,
}

// Constantes de resposta HTTP
const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n";
const NOT_FOUND: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
const INTERNAL_SERVER_ERROR: &str = "HTTP/1.1 500 INTERNAL SERVER ERROR\r\n\r\n";

// Função para obter a URL do banco de dados em tempo de execução
fn get_db_url() -> String {
    //env::var("DATABASE_URL").expect("DATABASE_URL não definida")
    let url:String = "postgres://postgres:123456@localhost:5432/api".to_string();
    url
}

// Função principal
fn main() {
    // Configurar o banco de dados
    if let Err(e) = set_database() {
        println!("Erro: {}", e);
        return;
    }

    // Iniciar o servidor na porta 8080
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    println!("Servidor iniciado na porta 8080");

    // Lidar com conexões de cliente
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_client(stream),
            Err(e) => println!("Erro: {}", e),
        }
    }
}

// Função para lidar com o cliente
fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let mut request = String::new();

    if let Ok(size) = stream.read(&mut buffer) {
        request.push_str(String::from_utf8_lossy(&buffer[..size]).as_ref());

        let (status_line, content) = match &*request {
            r if r.starts_with("POST /users") => handle_post_request(r),
            r if r.starts_with("GET /users/") => handle_get_request(r),
            r if r.starts_with("GET /users") => handle_get_all_request(r),
            r if r.starts_with("PUT /users/") => handle_put_request(r),
            r if r.starts_with("DELETE /users/") => handle_delete_request(r),
            _ => (NOT_FOUND.to_string(), "404 Not Found".to_string()),
        };

        stream.write_all(format!("{}{}", status_line, content).as_bytes()).unwrap();
    } else {
        println!("Erro ao ler o buffer");
    }
}

// Função para lidar com requisições POST
fn handle_post_request(request: &str) -> (String, String) {
    match (get_user_request_body(&request), Client::connect(&get_db_url(), NoTls)) {
        (Ok(user), Ok(mut client)) => {
            client
                .execute(
                    "INSERT INTO users (\"name\", email) VALUES ($1, $2)",
                    &[&user.name, &user.email]
                )
                .unwrap();
            (OK_RESPONSE.to_string(), "Usuário criado".to_string())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Erro ao criar usuário".to_string()),
    }
}

// Função para lidar com requisições GET
fn handle_get_request(request: &str) -> (String, String) {
    match (get_id(&request).parse::<i32>(), Client::connect(&get_db_url(), NoTls)) {
        (Ok(id), Ok(mut client)) => match client.query_one("SELECT * FROM users WHERE id = $1", &[&id]) {
            Ok(row) => {
                let user = User {
                    id: row.get(0),
                    name: row.get(1),
                    email: row.get(2),
                };
                (OK_RESPONSE.to_string(), serde_json::to_string(&user).unwrap())
            }
            _ => (NOT_FOUND.to_string(), "Usuário não encontrado".to_string()),
        },
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Erro ao buscar usuário".to_string()),
    }
}

// Função para lidar com requisições GET para todos os usuários
fn handle_get_all_request(_request: &str) -> (String, String) {
    match Client::connect(&get_db_url(), NoTls) {
        Ok(mut client) => {
            let mut users = Vec::new();
            for row in client.query("SELECT * FROM users", &[]).unwrap() {
                users.push(User {
                    id: row.get(0),
                    name: row.get(1),
                    email: row.get(2),
                });
            }
            (OK_RESPONSE.to_string(), serde_json::to_string(&users).unwrap())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Erro ao buscar usuários".to_string()),
    }
}

// Função para lidar com requisições PUT
fn handle_put_request(request: &str) -> (String, String) {
    match (
        get_id(&request).parse::<i32>(),
        get_user_request_body(&request),
        Client::connect(&get_db_url(), NoTls),
    ) {
        (Ok(id), Ok(user), Ok(mut client)) => {
            client
                .execute(
                    "UPDATE users SET name = $1, email = $2 WHERE id = $3",
                    &[&user.name, &user.email, &id]
                )
                .unwrap();
            (OK_RESPONSE.to_string(), "Usuário atualizado".to_string())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Erro ao atualizar usuário".to_string()),
    }
}

// Função para lidar com requisições DELETE
fn handle_delete_request(request: &str) -> (String, String) {
    match (get_id(&request).parse::<i32>(), Client::connect(&get_db_url(), NoTls)) {
        (Ok(id), Ok(mut client)) => {
            let rows_affected = client.execute("DELETE FROM users WHERE id = $1", &[&id]).unwrap();
            if rows_affected == 0 {
                return (NOT_FOUND.to_string(), "Usuário não encontrado".to_string());
            }
            (OK_RESPONSE.to_string(), "Usuário deletado".to_string())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Erro ao deletar usuário".to_string()),
    }
}

// Função para configurar o banco de dados
fn set_database() -> Result<(), PostgresError> {
    let mut client = Client::connect(&get_db_url(), NoTls)?;
    client.batch_execute(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            email VARCHAR NOT NULL
        )"
    )?;
    Ok(())
}

// Função para obter o ID da requisição
fn get_id(request: &str) -> &str {
    request.split("/").nth(2).unwrap_or_default().split_whitespace().next().unwrap_or_default()
}

// Função para desserializar o usuário do corpo da requisição
fn get_user_request_body(request: &str) -> Result<User, serde_json::Error> {
    serde_json::from_str(request.split("\r\n\r\n").last().unwrap_or_default())
}
