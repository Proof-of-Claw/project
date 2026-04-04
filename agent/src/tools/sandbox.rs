use anyhow::{anyhow, Result};
use wasmtime::*;

pub struct WasmSandbox {
    engine: Engine,
}

impl WasmSandbox {
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        Ok(Self { engine })
    }

    pub async fn execute(&self, wasm_module: &[u8], _input: &str) -> Result<String> {
        let module = Module::new(&self.engine, wasm_module)?;
        let mut store = Store::new(&self.engine, ());
        let linker = Linker::new(&self.engine);

        let instance = linker.instantiate(&mut store, &module)?;

        // Try "run" export first (returns i32), then "_start" (WASI-style void entry)
        if let Some(run_func) = instance.get_func(&mut store, "run") {
            let func_ty = run_func.ty(&store);
            if func_ty.params().len() == 0 && func_ty.results().len() == 1 {
                let mut results = vec![Val::I32(0)];
                run_func.call(&mut store, &[], &mut results)?;
                match results[0] {
                    Val::I32(v) => return Ok(v.to_string()),
                    Val::I64(v) => return Ok(v.to_string()),
                    _ => return Ok(format!("{:?}", results[0])),
                }
            } else if func_ty.params().len() == 0 && func_ty.results().len() == 0 {
                run_func.call(&mut store, &[], &mut [])?;
                return Ok("ok".to_string());
            }
        }

        if let Some(start_func) = instance.get_func(&mut store, "_start") {
            start_func.call(&mut store, &[], &mut [])?;
            return Ok("ok".to_string());
        }

        Err(anyhow!(
            "No suitable export found: expected 'run' or '_start' function"
        ))
    }
}
