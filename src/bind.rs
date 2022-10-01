use crate::vm::*;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use cosmwasm_minimal_std::{
    Binary, BlockInfo, Coin, ContractInfo, Empty, Env, Event, MessageInfo,
    Timestamp,
};
use cosmwasm_vm::system::cosmwasm_system_query;
use cosmwasm_vm::{
    executor::{ExecuteInput, InstantiateInput},
    system::cosmwasm_system_entrypoint,
};
use cosmwasm_vm_wasmi::{host_functions, new_wasmi_vm, WasmiImportResolver, WasmiVM};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

pub fn vm_initialize<'a>(
    extension: &'a mut SimpleWasmiVMExtension,
    sender: BankAccount,
    address: BankAccount,
    funds: Vec<Coin>,
    code: &[u8],
) -> WasmiVM<SimpleWasmiVM<'a>> {
    let host_functions_definitions = WasmiImportResolver(host_functions::definitions());
    let module = new_wasmi_vm(&host_functions_definitions, code).unwrap();
    WasmiVM(SimpleWasmiVM {
        host_functions: host_functions_definitions
            .0
            .clone()
            .into_iter()
            .flat_map(|(_, modules)| modules.into_iter().map(|(_, function)| function))
            .collect(),
        executing_module: module,
        env: Env {
            block: BlockInfo {
                height: 0xDEADC0DE,
                time: Timestamp(0),
                chain_id: "abstract-test".into(),
            },
            transaction: None,
            contract: ContractInfo {
                address: address.into(),
            },
        },
        info: MessageInfo {
            sender: sender.into(),
            funds,
        },
        extension,
    })
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Serialize, Deserialize)]
pub struct VMStep {
    state: SimpleWasmiVMExtension,
    events: Vec<Event>,
    data: Option<Binary>,
}

#[wasm_bindgen]
pub fn vm_instantiate(
    sender: BankAccount,
    address: BankAccount,
    funds: JsValue,
    extension: JsValue,
    code: &[u8],
    message: JsValue,
) -> Result<JsValue, String> {
    let mut extension: SimpleWasmiVMExtension =
        serde_json::from_str(&serde_wasm_bindgen::from_value::<String>(extension).map_err(|_| "failed to deserialize state")?)
        .map_err(|_| "failed to deserialize state")?;
    let funds = serde_wasm_bindgen::from_value(funds)
        .map_err(|_| "failed to deserialize funds")?;
    let mut vm = vm_initialize(&mut extension, sender, address, funds, code);
    let message = serde_wasm_bindgen::from_value::<String>(message)
        .map_err(|_| "failed to deserialize message")?;
    let result = cosmwasm_system_entrypoint::<InstantiateInput<Empty>, WasmiVM<SimpleWasmiVM>>(
        &mut vm,
        message.as_bytes(),
    );
    match result {
        Ok((data, events)) => Ok(serde_wasm_bindgen::to_value(&VMStep {
            state: extension,
            events,
            data,
        }).map_err(|_| "failed to serialize state")?),
        Err(e) => Err(format!("{}", e)),
    }
}

#[wasm_bindgen]
pub fn vm_execute(
    sender: BankAccount,
    address: BankAccount,
    funds: JsValue,
    extension: JsValue,
    code: &[u8],
    message: JsValue,
) -> Result<JsValue, String> {
    let mut extension: SimpleWasmiVMExtension =
        serde_json::from_str(&serde_wasm_bindgen::from_value::<String>(extension).map_err(|_| "failed to deserialize state")?)
        .map_err(|_| "failed to deserialize state")?;
    let funds = serde_wasm_bindgen::from_value(funds)
        .map_err(|_| "failed to deserialize funds")?;
    let mut vm = vm_initialize(&mut extension, sender, address, funds, code);
    let message = serde_wasm_bindgen::from_value::<String>(message)
        .map_err(|_| "failed to deserialize message")?;
    let result = cosmwasm_system_entrypoint::<ExecuteInput<Empty>, WasmiVM<SimpleWasmiVM>>(
        &mut vm,
        message.as_bytes(),
    );
    match result {
        Ok((data, events)) => Ok(serde_wasm_bindgen::to_value(&VMStep {
            state: extension,
            events,
            data,
        }).map_err(|_| "failed to serialize state")?),
        Err(e) => Err(format!("{}", e)),
    }
}

#[wasm_bindgen]
pub fn vm_query(
    sender: BankAccount,
    address: BankAccount,
    funds: JsValue,
    extension: JsValue,
    code: &[u8],
    message: JsValue,
) -> Result<JsValue, String> {
    let mut extension: SimpleWasmiVMExtension = serde_json::from_str(
        &serde_wasm_bindgen::from_value::<String>(extension)
            .map_err(|_| "failed to deserialize vm state")?,
    )
    .map_err(|_| "failed to deserialize vm state")?;
    let funds = serde_wasm_bindgen::from_value(funds).map_err(|_| "failed to deserialize funds")?;
    let mut vm = vm_initialize(&mut extension, sender, address, funds, code);
    let query =
        serde_wasm_bindgen::from_value(message).map_err(|_| "failed to deserialize query")?;
    let result = cosmwasm_system_query(&mut vm, query);
    Ok(serde_wasm_bindgen::to_value(&result.unwrap().unwrap().into_result().unwrap()).unwrap())
}
