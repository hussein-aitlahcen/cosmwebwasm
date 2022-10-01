import init, { vm_instantiate, vm_execute, vm_query } from "./pkg/cosmwebwasm.js";

function normalize(state) {
    state.codes = Object.fromEntries(state.codes);
    state.contracts = Object.fromEntries(state.contracts);
    state.storage = Object.fromEntries(state.storage);
    state.storage =
        Object.fromEntries(
            Object.entries(state.storage).map(
                ([k, v]) => [k, {
                    data: Object.fromEntries(v.data),
                    iterators: Object.fromEntries(v.iterators)
                }]
            )
        );
}

function log(x) {
    console.log(JSON.stringify(x, null, 4));
}

async function run() {
    await init();

    const codeId = 0x1337;
    const sender = 0xC0DEC0DE;
    const address = 0xCAFEBABE;
    const code = new Uint8Array(await fetch("./cw20_base.wasm").then(x => x.arrayBuffer()));
    const state = {
        storage: {},
        codes: {
            [codeId]: Array.from(code)
        },
        contracts: {
            [address]: {
                code_id: codeId,
                admin: null,
                label: ""
            }
        },
        next_account_id: address + 1,
        transaction_depth: 0,
        gas: {
            checkpoints: [ 10000000000000 ]
        }
    };

    console.log(state);

    console.log("-- Instantiate --");
    const { state: state1, events: events1 } = vm_instantiate(sender, address, [], JSON.stringify(state), code, JSON.stringify({
        name: "Picasso",
        symbol: "PICA",
        decimals: 12,
        initial_balances: [],
        mint: {
            minter: String(sender),
            cap: null
        },
        marketing: null
    }));

    normalize(state1);

    events1.forEach(log);
    console.log(state1);

    console.log("-- Mint --");
    const { state: state2, events: events2 } = vm_execute(sender, address, [], JSON.stringify(state1), code, JSON.stringify({
        mint: {
            recipient: "10001",
            amount: "5555"
        }
    }));

    events2.forEach(log);
    console.log(state2);

    const tokenInfo = JSON.parse(atob(vm_query(sender, address, [], JSON.stringify(state1), code, {
        wasm: {
            smart: {
                contract_addr: String(address),
                msg: btoa(JSON.stringify({
                    token_info: {}
                }))
            }
        }
    })));
    console.log("-- Token info --");
    log(tokenInfo);
}

export default run;
