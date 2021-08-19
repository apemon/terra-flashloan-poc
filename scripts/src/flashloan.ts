import {create_wallet, init, query, upload, transfer, execute} from './util'

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
    const counter_addr = counter_response.contract_addr
    console.log(`deploy counter contract: ${counter_addr}`)
    const bank_response = await init(wallet, bank_code_id, {

    })
    const bank_addr = bank_response.contract_addr
    console.log(`deploy bank contract: ${bank_addr}`)
    const demo_response = await init(wallet, demo_code_id, {
        bank_addr: bank_addr,
        counter_addr: counter_addr
    })
    const demo_addr = demo_response.contract_addr
    console.log(`deploy demo contract: ${demo_addr}`)

    const bank_transfer = await transfer(wallet, bank_addr, '100000000uusd', '2000000uusd')
    console.log('transfer fund to bank contract')
    const demo_transfer = await transfer(wallet, demo_addr, '20000000uusd', '2000000uusd')
    console.log('transfer fund to demo contract')
    const response = await execute(wallet, demo_addr, {
        borrow: {}
    })
    console.log(response)
})()