use egui::{ComboBox, Id, Response, RichText, Ui};
use either::Either;
use plist::Value;

#[derive(Debug, PartialEq, Eq)]
pub enum ValueType {
    String,
    Integer,
    Real,
    Boolean,
    Data,
    Array,
    Dictionary,
}

pub fn get_child(k: &str, p: &mut Either<&mut Value, &mut Value>) -> Value {
    match p {
        Either::Left(v) => {
            if let Some(v) = v.as_dictionary_mut() {
                v[k].clone()
            } else if let Some(v) = v.as_array_mut() {
                v[k.parse::<usize>().unwrap()].clone()
            } else {
                unreachable!();
            }
        }
        Either::Right(v) => v.clone(),
    }
}

pub fn set_child(k: &str, p: &mut Either<&mut Value, &mut Value>, val: Value) {
    match p {
        Either::Left(v) => {
            if let Some(v) = v.as_dictionary_mut() {
                v[k] = val;
            } else if let Some(v) = v.as_array_mut() {
                v[k.parse::<usize>().unwrap()] = val;
            } else {
                unreachable!();
            }
        }
        Either::Right(v) => {
            **v = val;
        }
    }
}

pub fn pv<'a>(k: &str, p: &'a mut Either<&mut Value, &mut Value>) -> &'a mut Value {
    match p {
        Either::Left(v) => {
            match v {
                Value::Dictionary(v) => &mut v[k],
                Value::Array(v) => &mut v[k.parse::<usize>().unwrap()],
                _ => unreachable!(),
            }
        }
        Either::Right(v) => v,
    }
}

pub fn value_to_type(k: &str, p: &mut Either<&mut Value, &mut Value>) -> ValueType {
    match get_child(k, p) {
        Value::String(_) => ValueType::String,
        Value::Integer(_) => ValueType::Integer,
        Value::Real(_) => ValueType::Real,
        Value::Boolean(_) => ValueType::Boolean,
        Value::Data(_) => ValueType::Data,
        Value::Array(_) => ValueType::Array,
        Value::Dictionary(_) => ValueType::Dictionary,
        _ => unreachable!(),
    }
}

pub fn render_menu(
    ui: &mut Ui,
    id: Id,
    response: &Response,
    k: &str,
    p: &mut Either<&mut Value, &mut Value>,
) {
    egui::popup::popup_below_widget(ui, id, response, |ui| {
        ui.set_min_width(100.0);
        let ty = value_to_type(k, p);
        match ty {
            ValueType::Dictionary => {
                if ui.button("Add child").clicked() {
                    pv(k, p)
                        .as_dictionary_mut()
                        .unwrap()
                        .insert("New Element".to_string(), Value::String(String::new()));
                }
            }
            ValueType::Array => {
                if ui.button("Add child").clicked() {
                    pv(k, p)
                        .as_array_mut()
                        .unwrap()
                        .push(Value::String(String::new()));
                }
            }
            _ => {}
        }

        if let Either::Left(v) = p {
            if ui.button("Remove").clicked() {
                if let Some(v) = v.as_dictionary_mut() {
                    v.remove(k);
                } else if let Some(v) = v.as_array_mut() {
                    v.remove(k.parse::<usize>().unwrap());
                } else {
                    unreachable!();
                }
            }
        }

        if (ty == ValueType::Dictionary) && ui.button("Sort").clicked() {
            pv(k, p).as_dictionary_mut().unwrap().sort_keys();
        }
    });
}

#[must_use]
pub fn render_key(ui: &mut Ui, k: &str, p: &mut Either<&mut Value, &mut Value>) -> bool {
    let mut ty = value_to_type(k, p);
    let mut changed = false;

    let resp = ui.button("...");
    let id = ui.make_persistent_id(format!("elem_opts_{}", k));
    if resp.secondary_clicked() {
        ui.memory().open_popup(id)
    }
    crate::value::render_menu(ui, id, &resp, k, p);

    ui.label(RichText::new(k).strong());

    let root = p.is_right();

    let mut set = |new_value: Value| {
        set_child(k, p, new_value);
        changed = true;
    };

    ComboBox::new(k, "")
        .selected_text(format!("{:?}", ty))
        .show_ui(ui, |ui| {
            if ui
                .selectable_value(&mut ty, ValueType::Array, "Array")
                .changed()
            {
                set(Value::Array(vec![]));
            }
            if ui
                .selectable_value(&mut ty, ValueType::Dictionary, "Dictionary")
                .changed()
            {
                set(Value::Dictionary(plist::Dictionary::new()));
            }
            if !root {
                if ui
                    .selectable_value(&mut ty, ValueType::Boolean, "Boolean")
                    .changed()
                {
                    set(Value::Boolean(false));
                }

                if ui
                    .selectable_value(&mut ty, ValueType::Data, "Data")
                    .changed()
                {
                    set(Value::Data(vec![]));
                }

                if ui
                    .selectable_value(&mut ty, ValueType::Integer, "Integer")
                    .changed()
                {
                    set(Value::Integer(plist::Integer::from(0)));
                }

                if ui
                    .selectable_value(&mut ty, ValueType::Real, "Real")
                    .changed()
                {
                    set(Value::Real(0.0));
                }

                if ui
                    .selectable_value(&mut ty, ValueType::String, "String")
                    .changed()
                {
                    set(Value::String("".to_string()));
                }
            }
        });

    changed
}
