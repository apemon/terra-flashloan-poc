import {MsgStoreCode, StdFee, LCDClient, MnemonicKey, MsgInstantiateContract, Coins, MsgExecuteContract, Wallet, MsgSend, Coin} from '@terra-money/terra.js'
import * as fs from 'fs'

const DELAY_TIME = 1000 // this to prevent unauthorization error
const GAS_LIMIT = 10000000

const terra = new LCDClient({
    URL: 'https://bombay-lcd.terra.dev',
    chainID: 'bombay-9',
    gasAdjustment: 1.15
});

export const create_wallet = (mnemonic) => {
    const key = new MnemonicKey({
        mnemonic: mnemonic
    })
    return terra.wallet(key)
}

export const upload = async (
    wallet,
    path,
    fee='1500000uusd'
):Promise<Number> => { 
    const tx = await wallet.createAndSignTx({
        msgs: [
            new MsgStoreCode(
                wallet.key.accAddress,
                fs.readFileSync(path, { encoding: 'base64'})
            )
        ],
        fee: new StdFee(GAS_LIMIT, fee)
    })
    try {
        const response = await terra.tx.broadcast(tx);
        const logs = JSON.parse(response.raw_log)
        let code_id = ''
        logs.forEach( (log) => {
            log.events.forEach( (event) => {
                if(event.type == 'store_code') {
                    code_id = event.attributes.find( (attribute) => attribute.key == 'code_id').value
                }
            })
        })
        await delay(DELAY_TIME)
        return Number(code_id)
    } catch (err) {
        throw err
    }
}

export const init = async (
    wallet,
    code_id,
    init_msg,
    fee='1500000uusd'
) => {
    const tx = await wallet.createAndSignTx({
      msgs: [
        new MsgInstantiateContract(
          wallet.key.accAddress,
          wallet.key.accAddress,
          code_id,
          init_msg
        ),
      ],
      fee: new StdFee(GAS_LIMIT, fee),
    });
    try {
        const response = await terra.tx.broadcast(tx);
        await delay(DELAY_TIME)
        const logs = JSON.parse(response.raw_log)
        let contract_addr = ''
        logs.forEach( (log) => {
            log.events.forEach( (event) => {
                if(event.type == 'instantiate_contract') {
                    contract_addr = event.attributes.find( (attribute) => attribute.key == 'contract_address').value
                }
            })
        })
        return {
            contract_addr: contract_addr,
            logs
        }
    } catch (err) {
        throw err
    }
};

export const execute = async (
    wallet,
    addr,
    execute_msg,
    coins?,
    fee='1500000uusd'
) => {
    let coin = new Coins()
    if(coins)
        coin = Coins.fromString(coins)
    const tx = await wallet.createAndSignTx({
      msgs: [new MsgExecuteContract(wallet.key.accAddress, addr, execute_msg, coin)],
      fee: new StdFee(GAS_LIMIT, fee),
    });
    const response = await terra.tx.broadcast(tx);
    await delay(DELAY_TIME)
    return response;
}

export const transfer = async (wallet:Wallet, addr, coins, fee='1500000uusd') => {
    
    const tx = await wallet.createAndSignTx({
        msgs: [new MsgSend(
            wallet.key.accAddress,
            addr,
            Coins.fromString(coins)
        )],
        fee: new StdFee(GAS_LIMIT, fee),
    })
    const response = await terra.tx.broadcast(tx)
    await delay(DELAY_TIME)
    return response;
}

export const query = async (addr, msg) => {
    const response = await terra.wasm.contractQuery(addr,msg)
    return response
}

const delay = (ms) => {
    return new Promise( resolve => setTimeout(resolve, ms, {}) );
}
