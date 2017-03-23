use super::VirtualMachine;
use super::Pointer;

pub struct Module {

}

impl Module {
    pub fn make_object(&self, vm: &mut VirtualMachine, name: String) -> Pointer {
        return vm.object_of_type(&"object".to_owned());
    }
}