#[cfg(test)]
mod tests {
    use std::fs::File;
    use Luast::{parse, vm};

    #[test]
    fn test_lex() {
        let mut file = File::open("test_lua/print_more_values.lua").unwrap();
        let proto = parse::load(file);
        let mut vm = vm::ExeState::new();
        vm.execute(&proto);
    }
}