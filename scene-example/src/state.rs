use crate::prelude::*;

pub struct PopInput {
    pub inner: bool,
}

pub struct TextResource {
    pub display: String,
    pub input: String,
}

pub struct BaseState {

}

impl State for BaseState {
    fn shadow_update(&mut self, data: &mut StateData, resources: &mut Resources) {
        if let Some(mut res) = resources.get_mut::<TextResource>() {
            let gdstring = res.display.clone().into();
            unsafe {
                // Call shadow update on gdscript
                // This sets the text to current display string
                data.statenode.unwrap().call("_shadow_update".into(), &[Variant::from_godot_string(&gdstring)]);

                // Get the current input string and update res.input if there is one
                let returned = data.statenode.unwrap().call("_get_text_input".into(), &[Variant::new(); 0]);
                if let Some(string) = returned.try_to_godot_string() {
                    res.input = string.to_string();
                }
            }
        }

        // Pop the stack if pop button is pressed
        unsafe {
            let returned = data.statenode.unwrap().call("_get_pop_input".into(), &[Variant::new(); 0]);
            if let Some(b) = returned.try_to_bool() {
                resources.insert(PopInput { inner: b } );
           }
        }
    }

    fn on_push(&mut self, _data: &mut StateData, resources: &mut Resources) {
        // Add a base printer
        let sender = resources.get::<TransResource>().unwrap();
        sender.trans.try_send(Box::from(|| Trans::Push(Box::new(PrintState { output: "Bottom of stack".into() })))).ok();
    }

    fn is_node(&mut self, _data: &mut StateData, resources: &mut Resources) -> Option<usize> {
        // UI instancing
        let renderables = resources.get::<Models<Renderables>>().unwrap();
        let (_, index) = renderables.data_from_t(&Renderables::UI(Ui::Main)).unwrap();
        Some(index)
    }

    fn get_name(&mut self, _: &mut StateData, _: &mut Resources) -> String {
        "Base".into()
    }
}

pub struct PrintState {
    pub output: String,
}

impl State for PrintState {
    fn update(&mut self, _data: &mut StateData, resources: &mut Resources) {
        let sender = resources.get::<TransResource>().unwrap();
        if let Some(res) = resources.get::<TextResource>() {
            if res.input != "" {
                let blah = res.input.clone();
                sender.trans.try_send(Box::from(move || Trans::Push(Box::new(PrintState { output: blah })))).ok();
            }
        }
    }

    fn shadow_update(&mut self, _data: &mut StateData, resources: &mut Resources) {
        if let Some(pop_input) = resources.get::<PopInput>() {
            if pop_input.inner {
                let sender = resources.get::<TransResource>().unwrap();
                sender.trans.try_send(Box::from(|| Trans::Pop)).ok();
            }
        }
        resources.insert(PopInput { inner: false });
    }

    fn on_push(&mut self, _data: &mut StateData, resources: &mut Resources) {
        resources.insert(TextResource { display: self.output.clone(), input: "".into() });
    }

    fn on_uncover(&mut self, _data: &mut StateData, resources: &mut Resources) {
        resources.insert(TextResource { display: self.output.clone(), input: "".into() });
    }

    fn get_name(&mut self, _: &mut StateData, _: &mut Resources) -> String {
        "Printer".into()
    }
}