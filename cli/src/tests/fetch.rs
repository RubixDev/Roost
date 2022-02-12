use serde_json::{Value as JsonValue, Map};
use plist::{Value as PlistValue, Integer};

#[test]
fn fetch() -> Result<(), Box<dyn std::error::Error>> {
    let theme_body = ureq::get("https://raw.githubusercontent.com/Binaryify/OneDark-Pro/master/themes/OneDark-Pro.json").call()?.into_string()?;
    let theme_body: Map<String, JsonValue> = serde_json::from_str(&theme_body)?;
    let mut json_theme = serde_json::Map::new();
    json_theme.insert(String::from("name"), theme_body.get("name").unwrap().clone());
    let colors = theme_body.get("colors").unwrap().as_object().unwrap();
    json_theme.insert(String::from("settings"), JsonValue::Array(vec![JsonValue::Object({
        let mut map = Map::new();
        map.insert(String::from("settings"), JsonValue::Object({
            let mut map = Map::new();
            map.insert(String::from("background"), colors.get("editor.background").unwrap().clone());
            map.insert(String::from("foreground"), colors.get("editor.foreground").unwrap().clone());
            map
        }));
        map
    })]));
    let settings = json_theme.get_mut("settings").unwrap().as_array_mut().unwrap();
    for setting in theme_body.get("tokenColors").unwrap().as_array().unwrap() {
        settings.push(setting.clone());
    }
    let plist_theme = as_plist_val(&JsonValue::Object(json_theme));
    plist::to_file_xml("src/res/one-dark.tmTheme", &plist_theme)?;

    Ok(())
}

fn as_plist_val(value: &JsonValue) -> PlistValue {
    match value {
        JsonValue::Null => PlistValue::String(String::new()),
        JsonValue::Bool(val) => PlistValue::Boolean(*val),
        JsonValue::Number(val) => if val.is_u64() {
            PlistValue::Integer(Integer::from(val.as_u64().unwrap()))
        } else if val.is_i64() {
            PlistValue::Integer(Integer::from(val.as_i64().unwrap()))
        } else {
            PlistValue::Real(val.as_f64().unwrap())
        },
        JsonValue::String(val) => PlistValue::String(val.clone()),
        JsonValue::Array(val) => PlistValue::Array(val.iter().map(|it| as_plist_val(it)).collect()),
        JsonValue::Object(val) => PlistValue::Dictionary(val.iter().map(|(k, v)| (k, as_plist_val(v))).collect()),
    }
}
