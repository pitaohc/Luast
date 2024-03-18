#[cfg(test)]
mod tests {
    use std::fs::File;
    use Luast::{parse, vm};

    #[test]
    fn test_lex() {
        let file = File::open("test_lua/local_variables.lua").unwrap();
        let proto = parse::ParseProto::load(file);
        let mut vm = vm::ExeState::new();
        vm.execute(&proto);
    }
}