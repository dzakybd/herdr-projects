//! The web board's Accounts sheet: the accounts there are with a gateway's fields (the call `accounts`), and an account
//! made or changed (the call `account-save`), both on `agents::account_edit`, which checks every value. A key is never in
//! either: a gateway names the variable its shell exports it as.
use crate::agents::account_edit::{create, list, make_login, save_gateway, Gateway};
use crate::json::*;
use serde_json::{json, Value};

/// Every account: its name, what it is, and a gateway's fields as the page's form has them.
pub fn accounts() -> Value {
    let one = |a: &crate::agents::account_edit::Account| json!({ "name": a.name, "kind": a.kind, "gateway": a.gateway.as_ref().map(|g| json!({
        "baseUrl": g.base_url, "keyVar": g.key_var, "keyLiteral": g.key_literal, "model": g.model, "models": g.models, "context": g.context })) });
    json!({ "accounts": list().iter().map(one).collect::<Vec<_>>() })
}
/// A gateway's fields from the form. `models` is a list or words separated by spaces or new lines.
fn gateway_of(a: &Value) -> Result<Gateway, String> {
    let models = match a.at("models") { Value::Array(l) => l.iter().map(text).collect::<Vec<_>>().join(" "), m => text(m) };
    let context = match a.at("context") {
        Value::Null => None,
        c => match text(c).trim() { "" => None, n => Some(n.parse::<u64>().map_err(|_| "The context tokens are a whole number, or left empty.".to_string())?) },
    };
    Ok(Gateway { base_url: a.s("baseUrl").trim().into(), key_var: a.s("keyVar").trim().into(), model: a.s("model").trim().into(), models: models.split_whitespace().map(String::from).collect(), context, key_literal: false })
}
/// An account made (`new`) or one that is there changed: as a gateway, with the form's fields, or as a Claude login, which
/// for a gateway is its `account.env` removed, only with `confirm`. What was done, in words, or why not.
pub fn save(a: &Value) -> Result<String, String> {
    let (name, gateway) = (a.s("name").trim(), a.s("type") == "gateway");
    if !gateway && a.s("type") != "login" { return Err("Pick what the account is: a Claude login or a gateway.".into()); }
    let fields = if gateway { Some(gateway_of(a)?) } else { None };
    if a.b("new") {
        create(name, fields.as_ref()).map_err(|e| e.msg)?;
        return Ok(format!("Account \"{name}\" made."));
    }
    match &fields { Some(g) => save_gateway(name, g), None => make_login(name, a.b("confirm")) }.map_err(|e| e.msg)?;
    Ok(format!("Account \"{name}\" saved."))
}
