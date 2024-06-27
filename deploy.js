import 'dotenv/config';
import pkg from 'cosmwasm';
const { SigningCosmWasmClient, Secp256k1HdWallet } = pkg;

import * as fs from "fs";
import { Decimal } from "@cosmjs/math";

// This is your rpc endpoint
const rpcEndpoint = "https://testnet-rpc.orai.io";

const mnemonic = process.env.MNEMONIC;



async function main() {
    const wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic, { prefix: "orai" })
    const client = await SigningCosmWasmClient.connectWithSigner(
        rpcEndpoint,
        wallet,
        {
            gasPrice: {
                denom: "orai",
                //minimum fee per gas
                amount: Decimal.fromUserInput("0.001", 6)
            }
        }
    );
    
    
    
    const account = await wallet.getAccounts()
    console.log(account)
    const address = account[0].address
    console.log(address)
    // get orai balance
     const contractAddress = process.env.CONTRACT_ADDRESS;

    const fee = "auto"

    // //=====================================DEPLOY========================================

    // // Path to your contract's compiled .wasm file
    // const path = "./artifacts/lottery.wasm";
    // const wasmCode = new Uint8Array(fs.readFileSync(path));

    // // Upload code on chain
    // const upload = await client.upload(address, wasmCode, fee);
    // console.log("Upload result:", upload);

    // // Instantiate msg
    // const instantiateMsg = {
    //     admin: address, // Setting the admin to the current address
    //     ticket_price: { denom: "orai", amount: "1" }, // 
    //     round_duration: 600 // 10 min
    // };

    // // Instantiate the contract
    // const res = await client.instantiate(address, upload.codeId, instantiateMsg, "lottery_contract", fee);
    // console.log("Instantiate result:", res);

    // const contractAddress = res.contractAddress;
    // console.log("Contract Address:", contractAddress);

    // //===================================================================================


    // //=====================================EXECUTE=======================================

    // // Example of buying a ticket
    // const buyTicketMsg = {
    //     buy_ticket: {}
    // };

    // const executeBuyTicket = await client.execute(
    //     address, contractAddress, buyTicketMsg, fee, "Buying a ticket", [{ denom: "orai", amount: "1" }]
    // );
    // console.log("Execute Buy Ticket result:", executeBuyTicket);


    //  // Query the ticket ID for the participant
    //  const queryMsg = {
    //     get_ticket_id: { address }
    // };

    // const queryResult = await client.queryContractSmart(
    //     contractAddress,
    //     queryMsg
    // );

    // console.log("Query Ticket ID response:", queryResult);

    // // Example of ending a round
    // const endRoundMsg = {
    //     end_round: {}
    // };

    // const executeEndRound = await client.execute(
    //     address, contractAddress, endRoundMsg, fee
    // );
    // console.log("Execute End Round result:", executeEndRound);

    //===================================================================================

    // //======================================QUERY========================================

    const queryRoundWinners = await client.queryContractSmart(
        contractAddress,
        { get_round_winners: { round_id: 1 } }
    );

    console.log("Query Round Winners response:");
    console.log(queryRoundWinners);

    
    // Close the connection
    await client.disconnect();
}

main();
