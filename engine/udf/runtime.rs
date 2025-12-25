use anyhow::Result;

pub struct UdfRuntime;

impl UdfRuntime {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, _function_name: &str, _args: Vec<Vec<u8>>) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }
}
