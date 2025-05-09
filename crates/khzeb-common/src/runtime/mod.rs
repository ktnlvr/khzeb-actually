mod bytes;

use std::collections::HashMap;

use bytes::{read_cstring, read_pointer};
use wasmtime::{Engine as WasmEngine, Instance, Memory, Module, Store};

pub struct Engine {
    engine: WasmEngine,
    modules: HashMap<String, Module>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            engine: Default::default(),
            modules: Default::default(),
        }
    }

    pub fn load_wat_subsystem(&mut self, wat: impl ToString) -> SubsystemDescription {
        let wat = wat.to_string();

        let module = Module::new(&self.engine, wat).unwrap();
        let mut store = Store::<()>::new(&self.engine, ());
        let instance = Instance::new(&mut store, &module, &[]).unwrap();

        let get_subsystem = instance
            .get_func(&mut store, "get_subsystem")
            .unwrap()
            .typed::<(), i32>(&mut store)
            .unwrap();

        let subsystem_ptr = get_subsystem.call(&mut store, ()).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let subsystem_description =
            read_subsystem_description(&mut store, &memory, subsystem_ptr as u32);

        subsystem_description
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubsystemDescription {
    name: String,
    brief: String,
}

fn read_subsystem_description(
    mut store: &Store<()>,
    memory: &Memory,
    module_ptr: u32,
) -> SubsystemDescription {
    let memory = memory.data(&mut store);

    let name_ptr = read_pointer(memory, module_ptr);
    let brief_ptr = read_pointer(memory, module_ptr + 4);

    SubsystemDescription {
        name: read_cstring(memory, name_ptr),
        brief: read_cstring(memory, brief_ptr),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_subsystem() {
        let wat = r#"(module
  (type (;0;) (func (result i32)))
  (func (;0;) (type 0) (result i32)
    i32.const 1024)
  (memory (;0;) 2)
  (export "memory" (memory 0))
  (export "get_subsystem" (func 0))
  (data (;0;) (i32.const 1024) "\10\04\00\00\16\04")
  (data (;1;) (i32.const 1040) "Empty\00Does nothing"))
"#;

        let mut engine = Engine::new();
        let subsystem = engine.load_wat_subsystem(wat);
        println!("{:?}", subsystem);
    }
}
