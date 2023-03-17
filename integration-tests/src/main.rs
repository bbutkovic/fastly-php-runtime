use glob::glob;
use serde_derive::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{prelude::*, BufReader},
    ops::{Deref, DerefMut},
    path::PathBuf,
    process::{Child, Command, Stdio},
};

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.0.kill().unwrap();
    }
}

impl Deref for ChildGuard {
    type Target = Child;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ChildGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[tokio::main]
async fn main() {
    let fixtures = env::args().nth(1).expect("fixtures path must be provided");
    let runtime_path = env::args().nth(2).expect("runtime path must be provided");
    println!("Running fixtures: {}", fixtures);

    let search_path = format!("{}/**/test.json", fixtures);
    for test in glob(&search_path).expect("failed to find tests") {
        let test = test.unwrap();
        let mut test_root_path = PathBuf::from(test.clone());
        test_root_path.pop();

        let mut fixture_path = test_root_path.clone();
        fixture_path.push("index.php");

        let mut bundle_wasm_path = test_root_path.clone();
        bundle_wasm_path.push("bundle.wasm");

        let fastly_toml = {
            let mut fastly_toml = test_root_path.clone();
            fastly_toml.push("fastly.toml");

            match fastly_toml.is_file() {
                true => Some(fastly_toml),
                false => None,
            }
        };

        let runtime_path = PathBuf::from(runtime_path.clone());

        bundle_wasm(runtime_path, fixture_path, bundle_wasm_path.clone());

        println!("Starting Viceroy");
        let _server_handle = ChildGuard(run_test_server(bundle_wasm_path, fastly_toml));
        println!("Viceroy started.");

        let test_cases = load_test_cases(test);

        for (test_case_name, test_case) in test_cases {
            println!("Testing: {}", test_case_name);
            run_test_case(&test_case).await;
        }
    }
}

async fn run_test_case(test_case: &TestCase) {
    let url = format!(
        "http://127.0.0.1:7878/{}",
        test_case.downstream_request.pathname
    );

    let client = reqwest::Client::new();

    let client = match test_case.downstream_request.method.as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url),
        _ => unreachable!(),
    };

    let res = client.send().await.unwrap();

    let res_status = res.status().as_u16() as usize;
    let res_headers = res.headers().clone();
    let res_body = res.text().await.unwrap().clone();

    match &test_case.downstream_response.body {
        Some(body) => assert_eq!(res_body.as_str(), body.as_str()),
        None => (),
    }

    match test_case.downstream_response.status {
        Some(status) => assert_eq!(res_status, status),
        None => (),
    }

    match &test_case.downstream_response.headers {
        Some(headers) => {
            for header in headers.iter() {
                let (header_name, header_value) = header
                    .split_once(":")
                    .map(|(k, v)| (k.trim(), v.trim()))
                    .unwrap();

                assert!(res_headers
                    .iter()
                    .find(|&(res_header_name, res_header_value)| {
                        header_name == res_header_name && header_value == res_header_value
                    })
                    .is_some());
            }
        }
        None => (),
    }
}

fn bundle_wasm(runtime_path: PathBuf, fixture_path: PathBuf, out_path: PathBuf) {
    let fixture = File::open(fixture_path.clone()).expect("fixture not found");

    Command::new("wizer")
        .args([
            "--allow-wasi",
            "--wasm-bulk-memory=true",
            "-o",
            out_path.into_os_string().into_string().unwrap().as_str(),
            runtime_path
                .into_os_string()
                .into_string()
                .unwrap()
                .as_str(),
        ])
        .stdin(fixture)
        .output()
        .expect("failed to execute wizer");
}

fn run_test_server(wasm_file: PathBuf, fastly_toml: Option<PathBuf>) -> Child {
    let wasm_file = wasm_file
        .into_os_string()
        .into_string()
        .unwrap()
        .to_string();
    let mut args = vec![wasm_file];

    match fastly_toml {
        Some(fastly_toml) => {
            args.push("-C".to_string());

            let fastly_toml = fastly_toml
                .into_os_string()
                .into_string()
                .unwrap()
                .to_string();
            args.push(fastly_toml);
        }
        None => (),
    }

    let mut child_process = Command::new("viceroy")
        .stdout(Stdio::piped())
        .args(args)
        .spawn()
        .expect("viceroy did not run successfully");

    let stdout = child_process.stdout.take().unwrap();

    let mut reader = BufReader::new(stdout);

    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();

        // wait until Viceroy is ready and listening
        if line.contains("Listening on") {
            break;
        }
    }

    std::thread::spawn(move || {
        std::io::copy(&mut reader, &mut std::io::stdout()).unwrap();
    });

    child_process
}

fn load_test_cases(test_cases_file: PathBuf) -> HashMap<String, TestCase> {
    let test_case_content = std::fs::read_to_string(&test_cases_file).unwrap();
    serde_json::from_str(&test_case_content).unwrap()
}

#[derive(Serialize, Deserialize)]
struct TestCase {
    downstream_request: TestDownstreamRequest,
    downstream_response: TestDownstreamResponse,
}

#[derive(Serialize, Deserialize)]
struct TestDownstreamRequest {
    method: String,
    pathname: String,
}

#[derive(Serialize, Deserialize)]
struct TestDownstreamResponse {
    status: Option<usize>,
    body: Option<String>,
    headers: Option<Vec<String>>,
}
