use super::VirtualMachine;
use super::Pointer;

pub struct Module {

}

impl Module {
    pub fn make_object(&self, vm: &mut VirtualMachine, names: Vec<(String, String)>) -> Vec<(String, Pointer)> {
        let mut pointers = Vec::new();

        for (name, alias) in names {
            let ptr = vm.object_of_type(&"object".to_owned());
            pointers.push((alias, ptr));
        }

        return pointers;
    }
}