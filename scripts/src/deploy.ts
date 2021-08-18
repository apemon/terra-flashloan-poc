import {create_wallet, init, query, upload} from './util'

require('dotenv').config()
const { MNEMONIC } = process.env

const wallet = create_wallet(MNEMONIC);

(async() => {
    // upload code
    const counter_code_path = '../artifacts/counter.wasm'
    const counter_code_id = await upload(wallet, counter_code_path)
    const bank_code_path = '../artifacts/bank.wasm'
    const bank_code_id = await upload(wallet, bank_code_path)
    const demo_code_path = '../artifacts/demo.wasm'
    const demo_code_id = await upload(wallet, demo_code_path)
    // initialize contract
    const counter_response = await init(wallet, counter_code_id, {
        count: 0
    })
    console.log(counter_response.contract_addr)
    const bank_response = await init(wallet, bank_code_id, {

    })
    console.log(bank_response.contract_addr)
    const demo_response = await init(wallet, demo_code_id, {
        bank_addr: bank_response.contract_addr
    })
    console.log(demo_response.contract_addr)
    //const config_response = await query()
})()