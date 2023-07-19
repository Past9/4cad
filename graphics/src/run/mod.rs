mod program;

use program::*;

pub struct Vulkan {}
impl Vulkan {
    pub fn execute(program: &crate::spec::Program) {
        let program = program::Program::build(program);

        // TODO execute built program
    }
}
