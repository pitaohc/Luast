#[cfg(test)]
mod tests {
    use std::fs::File;
    use Luast::{parse, vm};

    #[test]
    fn test_lex() {
        let file = File::open("test_lua/print_more_values.lua").unwrap();
        let proto = parse::ParseProto::load(file);
        let mut vm = vm::ExeState::new();
        vm.execute(&proto);
    }
    #[test]
    fn test_ch2_3_assignment(){
        let file = File::open("test_lua/ch2_3.assignment.lua").unwrap();
        let proto = parse::ParseProto::load(file);
        let mut vm = vm::ExeState::new();
        vm.execute(&proto);
    }
}