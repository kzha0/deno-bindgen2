use std::collections::HashMap;

use crate::deno::TsMethod;

#[derive(Clone, Debug)]
pub struct TsClass {
    pub methods: Vec<TsMethod>,
}

#[derive(Clone, Debug, Default)]
pub struct Classes {
    inner: HashMap<String, TsClass>,
}

impl Classes {
    pub fn append(&mut self, class_name: &str, mut methods: Vec<TsMethod>) {
        if let Some(class) = self.inner.get_mut(class_name) {
            class.methods.append(&mut methods);
        } else {
            self.inner
                .insert(class_name.to_string(), TsClass { methods });
        }
    }
}
