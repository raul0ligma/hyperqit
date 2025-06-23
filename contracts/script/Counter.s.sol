// SPDX-License-Identifier: MIT

pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";

interface ICoreWriter {
    function sendRawAction(bytes memory action) external;
}

/**
 * @dev A script for the sendSpot action on HyperEVM testnet
 * @notice Requires a balance of 0.01 HYPE on the sender's account
 */
contract CoreWriterScript is Script {
    // CoreWriter precompile address (testnet only)
    address public constant CORE_WRITER_ADDRESS =
        0x3333333333333333333333333333333333333333;
    uint8 constant ENCODING_VERSION = 1;
    // HYPE Token ID for spot transfers on testnet (from docs)
    uint64 public constant HYPE_TOKEN_TESTNET = 1105;
    // Action ID for Spot Send
    uint24 public constant ACTION_ID_SPOT_SEND = 6;
    address public constant DEPLOYER =
        0x20D08684a310f1C5ba347620dB2B91289d49620C;
    uint24 constant ACTION_ID_LIMIT_ORDER = 1;

    uint24 constant ACTION_ID_USDC_SEND = 7;

    ICoreWriter public coreWriter;

    function run() public {
        vm.startBroadcast();
        console.log("tx.origin: %s", tx.origin);

        console.log("HYPE Balance", address(DEPLOYER).balance);

        coreWriter = ICoreWriter(CORE_WRITER_ADDRESS);

        // console.log("===Sending 0.01 HYPE Spot===");

        // --- Parameters ---
        // address destination = DEPLOYER;
        // uint64 amountWei = 1 * 10 ** 6; // This is .01 HYPE on spot
        // sendSpotPacked(destination, HYPE_TOKEN_TESTNET, amountWei);

        // console.log("HYPE Balance", address(DEPLOYER).balance);

        // console.log("===SWAPPING USDC TO BTC===");

        // limitOrder(
        //     3, // USDT/USDC asset ID
        //     true, // Buy USDT
        //     uint64(11200000000000), // $0.0026 (1e8 precision)
        //     1500000000, // sz = 38,461.53 units, assuming 1e8 scale
        //     false,
        //     3, // I0C
        //     0
        // );

        //38,461.53
        sendUsdcToSpot(1000000);

        vm.stopBroadcast();
    }

    function limitOrder(
        uint32 asset,
        bool isBuy,
        uint64 limitPx,
        uint64 sz,
        bool reduceOnly,
        uint8 tif,
        uint128 cloid
    ) public {
        bytes memory actionPayload = bytes.concat(
            bytes1(uint8(1)), // Version
            bytes3(uint24(1)), // Action ID = 1 (Limit Order)
            abi.encode(asset, isBuy, limitPx, sz, reduceOnly, tif, cloid)
        );
        coreWriter.sendRawAction(actionPayload);

        console.log("  - Total Payload Size: %s bytes", actionPayload.length);
    }

    function sendSpotPacked(
        address destination,
        uint64 token,
        uint64 weiAmount
    ) public {
        // Construct the action payload according to the encoding specification:
        // 1. First byte: Encoding version (currently only v1 supported)
        // 2. Next 3 bytes: Action ID as big-endian uint24
        // 3. Remaining bytes: Raw ABI encoded action-specific data
        bytes memory fullAction = bytes.concat(
            bytes1(ENCODING_VERSION), // Single version byte
            bytes3(ACTION_ID_SPOT_SEND), // 3 bytes for action ID
            // Action-specific data follows, using standard ABI encoding
            abi.encode(
                destination, // address
                token, // uint64
                weiAmount // uint64
            )
        );

        console.log("Payload being sent to precompile:");
        console.logBytes(fullAction);

        // Send the raw action to HyperCore
        coreWriter.sendRawAction(fullAction);

        console.log("\nPacked Spot Send Action Sent:");
        console.log("  - Destination: %s", destination);
        console.log("  - Token ID: %s", token);
        console.log("  - Amount (Wei): %s", weiAmount);
        console.log("  - Amount (HYPE): 0.01");
        console.log("  - Total Payload Size: %s bytes", fullAction.length);
    }

    function sendUsdcToSpot(uint64 token) public {
        // Construct the action payload according to the encoding specification:
        // 1. First byte: Encoding version (currently only v1 supported)
        // 2. Next 3 bytes: Action ID as big-endian uint24
        // 3. Remaining bytes: Raw ABI encoded action-specific data
        bytes memory fullAction = bytes.concat(
            bytes1(ENCODING_VERSION), // Single version byte
            bytes3(ACTION_ID_USDC_SEND), // 3 bytes for action ID
            // Action-specific data follows, using standard ABI encoding
            abi.encode(
                token, // uint64
                false
            )
        );

        console.log("Payload being sent to precompile:");
        console.logBytes(fullAction);

        // Send the raw action to HyperCore
        coreWriter.sendRawAction(fullAction);

        console.log("\nPacked USDC  Send Action Sent:");
        console.log("  - Token ID: %s", token);
        console.log("  - Amount (HYPE): 0.01");
        console.log("  - Total Payload Size: %s bytes", fullAction.length);
    }
}
