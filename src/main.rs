use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::BufRead;
use std::io::Read;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::sync::LazyLock;
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use dind_rust_web_repl::{BaseRunner, Runner};

const EXAMPLE_JAVA_SOURCE: &str = r#"
void main(String[] args) {
    IO.println("Hello World from TEMPLATE");   
}
"#;

const JAVA_VERSIONS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut java_versions = HashMap::new();

    java_versions.insert("25", "eclipse-temurin:25-jdk-alpine");

    java_versions
});

const JAVA_RUNNER: BaseRunner = BaseRunner {
    code_language: "Java",
    image_versions: JAVA_VERSIONS,
    entrypoint: None,
};

fn process_request(mut stream: TcpStream) {
    let mut buffer = [0; 16 * 1024];
    let size = stream.read(&mut buffer).expect("Error read stream");

    let mut cursor = io::Cursor::new(&mut buffer);

    let mut request_line: String = String::new();

    cursor
        .read_line(&mut request_line)
        .expect("Error reading request line");

    for line in cursor.lines() {
        println!("Request line {}", line.expect("Error"));
    }

    println!("Request line {}", request_line);

    let handle = thread::spawn(move || {
        JAVA_RUNNER
            .cancellable_computation(Duration::from_secs(15), EXAMPLE_JAVA_SOURCE.to_string())
    });

    match handle.join() {
        Err(_) => println!("Generic error"),
        Ok(result) => match result {
            Ok(value) => {
                print!("{}", value);
                stream
                    .write_all(format!("HTTP/1.1 200 OK\r\n\r\n{}", value).as_bytes())
                    .expect("Error sending response");
            }
            Err(e) => print!("Error {}", e),
        },
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").expect("Error binding");

    for client in listener.incoming() {
        match client {
            Err(e) => println!("Error on incoming request {}", e),
            Ok(stream) => process_request(stream),
        }
    }
}
