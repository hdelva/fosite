mod math;
mod cmath;
mod builtin;
mod string;
mod list;

pub use self::math::*;
pub use self::builtin::*;
pub use self::string::*;
pub use self::cmath::*;
pub use self::list::*;

use core::VirtualMachine;
use core::Mapping;
use core::Message;
use core::ArgInvalid;
use core::CHANNEL;

use std::collections::HashSet;

fn check_arg(vm: &mut VirtualMachine, arg: &Mapping, index: &'static str, permitted: Vec<&'static str>) {
    let permitted_ptr: HashSet<_> = permitted
        .iter()
        .map(|x| *vm.knowledge().get_type(&x.to_string()).unwrap_or(&0))
        .collect();

    let mut problems = Vec::new();

    'outer:
    for &(ref path, ref address) in arg {
        let types = vm.ancestors(address);
        for t in &types {
            if permitted_ptr.contains(t) {
                continue 'outer;
            }
        }

        let last_type = types.first().unwrap(); // there's always one
        let type_name = vm.knowledge().get_type_name(last_type);
        problems.push((path.clone(), type_name.clone()));
    }

    if !problems.is_empty() {
        let content = ArgInvalid::new(index, permitted, problems);
        let message = Message::Output { 
            source: vm.current_node().clone(),
            content: Box::new(content)};
        CHANNEL.publish(message);
    }
}