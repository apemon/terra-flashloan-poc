import {create_wallet, execute, init, query, transfer, upload} from './util'

require('dotenv').config()
const { MNEMONIC } = process.env

const wallet = create_wallet(MNEMONIC);

(async() => {
    const demo_addr = 'terra1gpxmthxtulsrmx6wmp7tcp6vsf6ssnxf9lejqf'

    // set flag for demo contract to not repay debt
    await execute(wallet, demo_addr, {
        set_flag: {
            flag: false
        }
    })
    // perform flashloan -> it should get error and revert transaction
    const response = await execute(wallet, demo_addr, {
        borrow: {}
    })
    console.log(response)
})()