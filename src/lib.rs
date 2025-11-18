use std::{
    collections::HashMap, fs, io::Read, process::{Command, Stdio}, str::FromStr, sync::LazyLock, thread, time::{Duration, Instant, SystemTime}
};

pub trait Runner {
    fn cancellable_computation(
        &self,
        timeout: Duration,
        source_content: String,
    ) -> Result<String, String> {
        let docker_image = "eclipse-temurin:25-jdk-alpine";

        let now_nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let file_name = format!("J{}", now_nanos);

        fs::write(&file_name, source_content).expect("Error writing file");

        let mut child = Command::new("docker")
            .arg("run")
            .arg("-m")
            .arg("128m")
            .arg("--cpus")
            .arg("0.2")
            .arg("--entrypoint")
            .arg("java")
            .arg("-v")
            .arg(format!("./{}:/home/P.java", &file_name))
            .arg(docker_image)
            .arg("/home/P.java")
            .stdout(Stdio::piped())
            .spawn()
            .expect("Error spawning docker subcommand");

        let init = Instant::now();
        let mut string_result = String::new();
        let mut output: Option<String> = None;
        let mut error: String = String::from("Timeout");

        while init.elapsed() < timeout && output.is_none() {
            match child.try_wait() {
                Ok(None) => {
                    thread::sleep(Duration::from_millis(100));
                }
                Ok(result) => {
                    println!("Exit code {}", result.expect("Error getting exit code"));
                    if let Some(mut stdout) = child.stdout.take() {
                        stdout
                            .read_to_string(&mut string_result)
                            .expect("Error reading stdout");
                    }

                    output = Option::Some(string_result);
                    break;
                }
                Err(e) => {
                    error = e.to_string();
                }
            }
        }

        fs::remove_file(file_name).expect("Error deleting");

        if output.is_none() {
            child.kill().expect("Error killing child");
            Err(error)
        } else {
            Ok(output.take().expect("Error getting output value"))
        }
    }
}

pub struct BaseRunner {
    pub code_language: &'static str,
    pub image_versions: LazyLock<HashMap<&'static str, &'static str>>,
    pub entrypoint: Option<&'static str>,
}

impl Runner for BaseRunner {
    
}
