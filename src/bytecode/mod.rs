mod chunk;
mod opcode;
mod module;
pub mod vm;
pub mod compiler;
pub mod assembler;

pub use chunk::Chunk;
pub use opcode::OpCode;
pub use module::Module;

#[cfg(test)]
mod tests {
    use compiler::Compiler;
    use vm::VM;

    use crate::{Environment, EnvironmentBuilder, Lexer, Parser, Value};

    use super::*;

    #[track_caller]
    fn prepare_module(code: &str) -> (Module, Environment) {
        let tokens = Lexer::new(code).get().unwrap();
        let env = EnvironmentBuilder::new();
        let (program, _) = Parser::new(tokens, &env).parse().unwrap();

        let env = env.build();
        let module = Compiler::new(&program, &env).compile().unwrap();

        (module, env)
    }

    #[test]
    fn test_while() {
        let code = r#"
            entry main() {
                let x: u64 = 0;
                while x < 10 {
                    x = x + 1;
                }
                return x
            }
        "#;

        let (module, environment) = prepare_module(code);

        let mut vm = VM::new(&module, &environment);
        vm.invoke_chunk_id(0).unwrap();
        let value = vm.run().unwrap();
        assert_eq!(value, Value::U64(10));
    }
}
