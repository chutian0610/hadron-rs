use clap::Parser;
use std::io::Read;
use std::sync::Arc;
use tokio::runtime::Runtime;

#[derive(Parser, Debug)]
#[command(name = "octopus")]
#[command(about = "Distributed MPP query engine", long_about = None)]
struct Cli {
    #[arg(short, long)]
    sql: Option<String>,

    #[arg(long, default_value = "pretty")]
    log_format: String,

    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// Coordinator address for distributed mode
    #[arg(long, default_value = "http://localhost:50051")]
    coordinator: String,
}

/// HTTP-based coordinator client for distributed query execution
#[derive(Clone)]
struct CoordinatorClient {
    base_url: String,
    rt: Arc<Runtime>,
}

impl CoordinatorClient {
    fn new(base_url: &str) -> anyhow::Result<Self> {
        let rt = Runtime::new()?;
        Ok(Self {
            base_url: base_url.to_string(),
            rt: Arc::new(rt),
        })
    }

    fn submit_query(&self, sql: &str) -> anyhow::Result<String> {
        let url = format!("{}/query/submit", self.base_url);
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(&url)
            .json(&serde_json::json!({ "sql": sql }))
            .timeout(std::time::Duration::from_secs(30))
            .send()?;

        if resp.status().is_success() {
            let result: serde_json::Value = resp.json()?;
            Ok(result["query_id"].as_str().unwrap_or("").to_string())
        } else {
            Err(anyhow::anyhow!("Query submission failed: {}", resp.status()))
        }
    }

    fn get_query_state(&self, query_id: &str) -> anyhow::Result<String> {
        let url = format!("{}/query/state/{}", self.base_url, query_id);
        let client = reqwest::blocking::Client::new();
        let resp = client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()?;

        if resp.status().is_success() {
            let result: serde_json::Value = resp.json()?;
            Ok(result["state"].as_str().unwrap_or("UNKNOWN").to_string())
        } else {
            Err(anyhow::anyhow!("Query state lookup failed: {}", resp.status()))
        }
    }

    fn explain_query(&self, sql: &str) -> anyhow::Result<String> {
        let url = format!("{}/query/explain", self.base_url);
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(&url)
            .json(&serde_json::json!({ "sql": sql }))
            .timeout(std::time::Duration::from_secs(30))
            .send()?;

        if resp.status().is_success() {
            let result: serde_json::Value = resp.json()?;
            Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
        } else {
            Err(anyhow::anyhow!("EXPLAIN failed: {}", resp.status()))
        }
    }

    fn poll_for_completion(&self, query_id: &str, max_attempts: u32) -> anyhow::Result<String> {
        for _ in 0..max_attempts {
            let state = self.get_query_state(query_id)?;
            match state.as_str() {
                "Completed" => return Ok(state),
                "Failed" => return Err(anyhow::anyhow!("Query failed")),
                _ => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
        Ok("Still running".to_string())
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    run_repl(cli)
}

fn run_repl(cli: Cli) -> anyhow::Result<()> {
    let client = CoordinatorClient::new(&cli.coordinator)?;

    println!("Octopus Interactive REPL");
    println!("Coordinator: {}", cli.coordinator);
    println!("Type 'exit' or 'quit' to exit");
    println!("Type 'help' for commands");
    println!();

    loop {
        print!("octopus> ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input)? == 0 {
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        match input.to_lowercase().as_str() {
            "exit" | "quit" => break,
            "help" => {
                println!("Commands:");
                println!("  exit, quit - Exit the REPL");
                println!("  help - Show this help");
                println!("  explain <sql> - Show distributed query plan");
                println!("  Any SQL query - Execute the query");
            },
            _ => {
                // Check for EXPLAIN prefix
                if input.to_lowercase().starts_with("explain ") {
                    let sql = &input[8..].trim();
                    match client.explain_query(sql) {
                        Ok(plan) => {
                            println!("Distributed Query Plan:");
                            println!("{}", plan);
                        }
                        Err(e) => {
                            eprintln!("EXPLAIN error: {}", e);
                        }
                    }
                } else {
                    // Regular query execution
                    match client.submit_query(input) {
                        Ok(query_id) => {
                            println!("Query submitted: {}", query_id);
                            match client.poll_for_completion(&query_id, 100) {
                                Ok(state) => println!("Query state: {}", state),
                                Err(e) => eprintln!("Poll error: {}", e),
                            }
                        }
                        Err(e) => {
                            eprintln!("Query error: {}", e);
                        }
                    }
                }
            }
        }
    }

    println!("Goodbye!");
    Ok(())
}