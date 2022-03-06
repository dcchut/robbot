use anyhow::Result;
use code_sandbox::{CompletedSandbox, SandboxBuilder};

fn format_output(sandbox: CompletedSandbox) -> String {
    let mut reply = sandbox.stdout().trim_end();

    if reply.is_empty() {
        reply = sandbox.stderr().trim_end();
    }

    reply.to_string()
}

async fn _run_python_code(code: String) -> Result<String> {
    let mut builder = SandboxBuilder::new("dcchut/code-sandbox-python")?;
    builder.entry_point(["python3", "/playground/src/main.py"]);
    builder.mount("/playground/src/main.py", code)?;

    let sandbox = builder.build()?;
    Ok(format_output(sandbox.execute().await?))
}

pub async fn run_python_code<S: ToString>(code: S) -> Result<String> {
    _run_python_code(code.to_string()).await
}

async fn _run_rust_code(code: String) -> Result<String> {
    let mut builder = SandboxBuilder::new("dcchut/code-sandbox-rust-stable")?;
    builder.entry_point(["cargo", "run", "--release"]);
    builder.mount("/playground/src/main.rs", code)?;

    let sandbox = builder.build()?;
    Ok(format_output(sandbox.execute().await?))
}

pub async fn run_rust_code<S: ToString>(code: S) -> Result<String> {
    _run_rust_code(code.to_string()).await
}
